//! Correos de la fase 1: al confirmar una reserva se redacta el aviso al
//! cliente y a la administración, queda en un buzón en memoria que el
//! personal ve en la agenda (demostración) y, si el entorno trae
//! credenciales SMTP (`SMTP_USUARIO`/`SMTP_CLAVE` en el `.env` local,
//! ADR 0009), se entrega de verdad con `lettre` sin tocar el servicio de
//! reservas.

use std::time::Duration;

use chrono::{DateTime, Utc};
use lettre::message::{Mailbox, MultiPart, header::ContentType};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

use crate::dominio::{
    HORAS_CANCELACION_GENERAL, HORAS_CANCELACION_VIP, ModalidadPago, RETENCION_CANCELACION_PCT,
    Reserva, TipoCliente, UsuarioSesion,
};
use crate::estado::{Datos, Estado};
use crate::formato::{cop, fecha_hora_fmt};

/// Dominio de las cuentas de demostración: no tienen buzón real, así que
/// al entregar quedan solo en el panel (ADR 0009).
const DOMINIO_DEMO: &str = "@brindis.co";

/// Un correo listo para enviar; en la demo vive en `Datos::correos`.
#[derive(Clone)]
pub struct Mensaje {
    pub id: i64,
    pub para: String,
    pub asunto: String,
    pub cuerpo: String,
    /// Versión HTML (multipart/alternative) con el botón de gestión de la
    /// reserva; `None` deja el aviso en texto plano.
    pub html: Option<String>,
    pub enviado_en: DateTime<Utc>,
    /// `true` cuando el SMTP confirmó la entrega real (ADR 0009).
    pub entregado: bool,
}

/// Credenciales SMTP del `.env` local (nunca se commitea).
struct Smtp {
    usuario: String,
    clave: String,
    servidor: String,
    puerto: u16,
}

impl Smtp {
    /// Sin usuario y clave definidos no hay envío real: la app se queda
    /// con el buzón de demostración.
    fn desde_entorno() -> Option<Self> {
        let usuario = std::env::var("SMTP_USUARIO")
            .ok()
            .filter(|v| !v.trim().is_empty())?;
        let clave = std::env::var("SMTP_CLAVE")
            .ok()
            .filter(|v| !v.trim().is_empty())?;
        Some(Self {
            usuario,
            clave,
            servidor: std::env::var("SMTP_SERVIDOR").unwrap_or_else(|_| "smtp.gmail.com".into()),
            puerto: std::env::var("SMTP_PUERTO")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(587),
        })
    }

    fn transporte(
        &self,
    ) -> Result<AsyncSmtpTransport<Tokio1Executor>, lettre::transport::smtp::Error> {
        Ok(
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&self.servidor)?
                .port(self.puerto)
                .timeout(Some(Duration::from_secs(10)))
                .credentials(Credentials::new(self.usuario.clone(), self.clave.clone()))
                .build(),
        )
    }
}

fn mesas(r: &Reserva) -> String {
    r.mesa_ids
        .iter()
        .map(|m| format!("M{m}"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Cuánto se cobró por anticipado y con qué modalidad (RN07/RN14).
fn linea_pago(r: &Reserva) -> String {
    let abonado = r.pago.monto_total * r.pago.porcentaje as i64 / 100;
    if abonado > 0 {
        format!(
            "Abono registrado: {}% ($ {}) de un total de $ {}.",
            r.pago.porcentaje,
            cop(abonado),
            cop(r.pago.monto_total)
        )
    } else {
        format!(
            "Pago en el restaurante: $ {} al finalizar.",
            cop(r.pago.monto_total)
        )
    }
}

fn modalidad_txt(r: &Reserva) -> String {
    match r.pago.modalidad {
        ModalidadPago::Adelanto => format!("abono del {}%", r.pago.porcentaje),
        ModalidadPago::Completo => "pago completo en línea".into(),
        ModalidadPago::EnRestaurante => "pago en el restaurante".into(),
    }
}

fn cuerpo_cliente(nombre: &str, r: &Reserva, tipo: TipoCliente) -> String {
    format!(
        "Hola {nombre}:\n\
         \n\
         Tu cita quedó pedida para el {} (mesas {}, {} comensales).\n\
         {}\n\
         \n\
         {}\n\
         \n\
         Te esperamos en Brindis.",
        fecha_hora_fmt(r.inicio),
        mesas(r),
        r.num_comensales,
        linea_pago(r),
        regla_cancelacion_txt(tipo),
    )
}

/// Recordatorio del plazo de cancelación (RN08/RN09) derivado de las
/// constantes del dominio, para que el correo nunca contradiga al servicio.
fn regla_cancelacion_txt(tipo: TipoCliente) -> String {
    match tipo {
        TipoCliente::Vip => format!(
            "Como socio VIP puedes cancelar sin retención hasta {h} hora antes del inicio; \
             si cancelas dentro de esa última hora se retiene el {r}% de lo abonado y se \
             registra un strike (RN09).",
            h = HORAS_CANCELACION_VIP,
            r = RETENCION_CANCELACION_PCT,
        ),
        TipoCliente::General => format!(
            "Puedes cancelar sin retención hasta {h} horas antes del inicio; si cancelas \
             dentro de esas últimas {h} horas se retiene el {r}% de lo abonado (RN08).",
            h = HORAS_CANCELACION_GENERAL,
            r = RETENCION_CANCELACION_PCT,
        ),
    }
}

fn cuerpo_admin(nombre_cliente: &str, correo_cliente: &str, r: &Reserva) -> String {
    format!(
        "Se registró una reserva nueva.\n\
         \n\
         Cliente: {nombre_cliente} <{correo_cliente}>\n\
         Código: {}\n\
         Fecha: {} · mesas {} · {} comensales\n\
         Ocasión: {} · restricciones: {}\n\
         Total: $ {} ({})\n\
         \n\
         El pedido y los extras quedaron con precios congelados (RN07).",
        r.codigo,
        fecha_hora_fmt(r.inicio),
        mesas(r),
        r.num_comensales,
        r.ocasion.nombre(),
        r.restricciones,
        cop(r.pago.monto_total),
        modalidad_txt(r),
    )
}

// --- versión HTML de los avisos (multipart/alternative) ---------------------

/// Tonos del tema «Le Jardin» incrustados en el correo: los clientes de
/// buzón no cargan `tema.css`, así que el estilo viaja en línea.
const C_VINO: &str = "#4a121a";
const C_VINO_CLARO: &str = "#702c3b";
const C_MARFIL: &str = "#fdf9f1";
const C_FONDO: &str = "#f4ecdc";
const C_LINEA: &str = "#e4d9c4";
const C_CARBON: &str = "#26231f";
const C_SUAVE: &str = "#6b6152";
const C_DORADO: &str = "#a88a59";

/// URL pública base para los enlaces de los correos: `APP_BASE_URL` si está
/// definida; si no, se arma con `HOST`/`PORT`, los mismos del servidor.
fn base_url() -> String {
    std::env::var("APP_BASE_URL")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| {
            let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".into());
            let puerto = std::env::var("PORT").unwrap_or_else(|_| "3000".into());
            format!("http://{host}:{puerto}")
        })
        .trim_end_matches('/')
        .to_string()
}

/// Escapa texto del estado (nombre, restricciones) para el HTML del correo.
fn escapar(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Marco común del correo HTML: antetítulo, tarjeta marfil y pie.
fn marco_html(pie: &str, contenido: &str) -> String {
    format!(
        "<!DOCTYPE html><html lang=\"es\"><body style=\"margin:0;padding:0;background:{fondo};\">\
         <div style=\"max-width:560px;margin:0 auto;padding:28px 18px 34px;\">\
         <p style=\"margin:0 0 16px;font-family:Georgia,'Times New Roman',serif;font-size:12px;letter-spacing:.35em;color:{dorado};text-transform:uppercase;\">Brindis · Salón y Reservas</p>\
         <div style=\"background:{marfil};border:1px solid {linea};border-radius:16px;padding:26px;\">{contenido}</div>\
         <p style=\"margin:18px 0 0;font-family:Helvetica,Arial,sans-serif;font-size:12px;line-height:1.6;color:{dorado};\">{pie}</p>\
         </div></body></html>",
        fondo = C_FONDO,
        dorado = C_DORADO,
        marfil = C_MARFIL,
        linea = C_LINEA,
        contenido = contenido,
        pie = pie,
    )
}

/// Fila de la ficha del correo (etiqueta en versalitas, valor a la derecha).
fn fila_ficha(etiqueta: &str, valor: &str) -> String {
    format!(
        "<tr><td style=\"padding:7px 0;border-bottom:1px solid {linea};font-family:Helvetica,Arial,sans-serif;font-size:11px;letter-spacing:.08em;text-transform:uppercase;color:{suave};\">{etiqueta}</td>\
         <td style=\"padding:7px 0;border-bottom:1px solid {linea};font-family:Georgia,'Times New Roman',serif;font-size:16px;color:{carbon};text-align:right;\">{valor}</td></tr>",
        linea = C_LINEA,
        suave = C_SUAVE,
        carbon = C_CARBON,
    )
}

/// Botón principal del correo (el único enlace con fondo sólido).
fn boton_html(etiqueta: &str, enlace: &str) -> String {
    format!(
        "<a href=\"{enlace}\" style=\"display:inline-block;background:{fondo};color:{texto};text-decoration:none;border-radius:999px;padding:12px 24px;font-family:Helvetica,Arial,sans-serif;font-size:14px;letter-spacing:.04em;\">{etiqueta}</a>",
        fondo = C_VINO_CLARO,
        texto = C_MARFIL,
    )
}

/// Versión HTML del aviso al cliente, con el botón que lleva a gestionar
/// (y cancelar) exactamente esta reserva.
fn html_cliente(nombre: &str, r: &Reserva, tipo: TipoCliente, base: &str) -> String {
    let ficha: String = [
        fila_ficha("Ocasión", r.ocasion.nombre()),
        fila_ficha("Fecha", &fecha_hora_fmt(r.inicio)),
        fila_ficha("Mesas", &mesas(r)),
        fila_ficha("Comensales", &r.num_comensales.to_string()),
    ]
    .concat();
    let contenido = format!(
        "<h1 style=\"margin:0 0 6px;font-family:Georgia,'Times New Roman',serif;font-size:24px;line-height:1.25;font-weight:600;color:{vino};\">Hola, {nombre}</h1>\
         <p style=\"margin:0 0 18px;font-family:Helvetica,Arial,sans-serif;font-size:14px;line-height:1.6;color:{suave};\">Tu cita quedó pedida con el código <strong style=\"color:{vino_claro};\">{codigo}</strong>.</p>\
         <table role=\"presentation\" width=\"100%\" cellpadding=\"0\" cellspacing=\"0\" style=\"border-collapse:collapse;margin:0 0 18px;\">{ficha}</table>\
         <p style=\"margin:0 0 12px;font-family:Helvetica,Arial,sans-serif;font-size:13px;line-height:1.6;color:{suave};\">{pago}</p>\
         <p style=\"margin:0 0 22px;font-family:Helvetica,Arial,sans-serif;font-size:13px;line-height:1.6;color:{suave};\">{regla}</p>\
         {boton}",
        vino = C_VINO,
        vino_claro = C_VINO_CLARO,
        suave = C_SUAVE,
        nombre = escapar(nombre),
        codigo = r.codigo,
        ficha = ficha,
        pago = escapar(&linea_pago(r)),
        regla = escapar(&regla_cancelacion_txt(tipo)),
        boton = boton_html(
            "Gestionar mi reserva",
            &format!("{base}/mis-reservas?reserva={}", r.codigo),
        ),
    );
    marco_html(
        "Recibes este aviso porque hiciste una reserva en Brindis. Responde a este correo si necesitas ayuda.",
        &contenido,
    )
}

/// Versión HTML del aviso interno a la administración, con acceso directo
/// a la agenda del personal.
fn html_admin(nombre_cliente: &str, correo_cliente: &str, r: &Reserva, base: &str) -> String {
    let ficha: String = [
        fila_ficha(
            "Cliente",
            &format!("{} · {}", escapar(nombre_cliente), escapar(correo_cliente)),
        ),
        fila_ficha("Fecha", &fecha_hora_fmt(r.inicio)),
        fila_ficha("Mesas", &mesas(r)),
        fila_ficha("Comensales", &r.num_comensales.to_string()),
        fila_ficha("Ocasión", r.ocasion.nombre()),
        fila_ficha("Restricciones", &escapar(&r.restricciones)),
        fila_ficha(
            "Total",
            &format!("$ {} ({})", cop(r.pago.monto_total), modalidad_txt(r)),
        ),
    ]
    .concat();
    let contenido = format!(
        "<h1 style=\"margin:0 0 6px;font-family:Georgia,'Times New Roman',serif;font-size:24px;line-height:1.25;font-weight:600;color:{vino};\">Nueva reserva {codigo}</h1>\
         <p style=\"margin:0 0 18px;font-family:Helvetica,Arial,sans-serif;font-size:14px;line-height:1.6;color:{suave};\">Reserva pedida desde la web; pendiente de confirmación operativa.</p>\
         <table role=\"presentation\" width=\"100%\" cellpadding=\"0\" cellspacing=\"0\" style=\"border-collapse:collapse;margin:0 0 18px;\">{ficha}</table>\
         <p style=\"margin:0 0 22px;font-family:Helvetica,Arial,sans-serif;font-size:13px;line-height:1.6;color:{suave};\">El pedido y los extras quedaron con precios congelados (RN07).</p>\
         {boton}",
        vino = C_VINO,
        suave = C_SUAVE,
        codigo = r.codigo,
        ficha = ficha,
        boton = boton_html("Abrir la agenda", &format!("{base}/agenda")),
    );
    marco_html(
        "Aviso automático del sistema de reservas de Brindis.",
        &contenido,
    )
}

/// Deposita en el buzón el correo del cliente y el de cada administrador
/// y devuelve los mensajes redactados para la entrega real (ADR 0008/0009).
pub fn notificar_reserva(d: &mut Datos, r: &Reserva) -> Vec<Mensaje> {
    let cliente = d.usuarios.iter().find(|u| u.id() == r.id_cliente);
    let nombre = cliente
        .map(|u| u.nombre().to_string())
        .unwrap_or_else(|| format!("Cliente {}", r.id_cliente));
    let correo_cliente = cliente.map(|u| u.correo().to_string()).unwrap_or_default();
    let tipo_cliente = cliente
        .map(UsuarioSesion::tipo_cliente)
        .unwrap_or(TipoCliente::General);
    let base = base_url();

    let mut pendientes: Vec<(String, String, String, String)> = Vec::new();
    if !correo_cliente.is_empty() {
        pendientes.push((
            correo_cliente.clone(),
            format!("Brindis: tu cita quedó pedida ({})", r.codigo),
            cuerpo_cliente(&nombre, r, tipo_cliente),
            html_cliente(&nombre, r, tipo_cliente, &base),
        ));
    }
    for admin in d
        .usuarios
        .iter()
        .filter(|u| matches!(u, UsuarioSesion::Administrador(_)))
    {
        pendientes.push((
            admin.correo().to_string(),
            format!("Brindis: nueva reserva {} de {}", r.codigo, nombre),
            cuerpo_admin(&nombre, &correo_cliente, r),
            html_admin(&nombre, &correo_cliente, r, &base),
        ));
    }

    let mut redactados = Vec::new();
    for (para, asunto, cuerpo, html) in pendientes {
        let id = d.correos.iter().map(|m| m.id).max().unwrap_or(0) + 1;
        let mensaje = Mensaje {
            id,
            para: para.clone(),
            asunto,
            cuerpo,
            html: Some(html),
            enviado_en: Utc::now(),
            entregado: false,
        };
        tracing::info!(destino = %para, "correo de reserva en el buzón de demostración");
        d.correos.push(mensaje.clone());
        redactados.push(mensaje);
    }
    redactados
}

/// Entrega por SMTP los mensajes recién redactados (ADR 0009). Se llama
/// con el lock del estado ya liberado; un fallo de red no pierde el aviso:
/// se queda en el buzón y solo se registra el error.
pub async fn entregar(estado: &Estado, mensajes: &[Mensaje]) {
    let Some(smtp) = Smtp::desde_entorno() else {
        return;
    };
    let transporte = match smtp.transporte() {
        Ok(t) => t,
        Err(e) => {
            tracing::error!(error = %e, "SMTP mal configurado; los avisos quedan en el buzón");
            return;
        }
    };
    for m in mensajes {
        if m.para.ends_with(DOMINIO_DEMO) {
            tracing::info!(destino = %m.para, "cuenta de demostración: el aviso queda en el panel");
            continue;
        }
        let correo = match a_correo(m, &smtp.usuario) {
            Ok(c) => c,
            Err(e) => {
                tracing::error!(destino = %m.para, error = %e, "correo mal formado; no se envía");
                continue;
            }
        };
        match transporte.send(correo).await {
            Ok(_) => {
                tracing::info!(destino = %m.para, "correo entregado por SMTP");
                marcar_entregado(estado, m.id);
            }
            Err(e) => tracing::error!(
                destino = %m.para,
                error = %e,
                "falló la entrega SMTP; el aviso sigue en el buzón"
            ),
        }
    }
}

/// Arma el correo de `lettre`: multipart texto+HTML cuando hay versión HTML
/// (el buzón del cliente elige la mejor) o solo texto plano si no la hay.
fn a_correo(m: &Mensaje, remitente: &str) -> Result<Message, String> {
    let de: Mailbox = remitente
        .parse()
        .map_err(|e| format!("remitente inválido: {e}"))?;
    let para: Mailbox = m
        .para
        .parse()
        .map_err(|e| format!("destino inválido: {e}"))?;
    let constructor = Message::builder()
        .from(de)
        .to(para)
        .subject(m.asunto.clone());
    match &m.html {
        Some(html) => constructor
            .multipart(MultiPart::alternative_plain_html(
                m.cuerpo.clone(),
                html.clone(),
            ))
            .map_err(|e| format!("correo inválido: {e}")),
        None => constructor
            .header(ContentType::TEXT_PLAIN)
            .body(m.cuerpo.clone())
            .map_err(|e| format!("cuerpo inválido: {e}")),
    }
}

fn marcar_entregado(estado: &Estado, id: i64) {
    let mut d = estado.lock().unwrap();
    if let Some(m) = d.correos.iter_mut().find(|m| m.id == id) {
        m.entregado = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dominio::{DetallePago, EstadoReserva, Ocasion};

    fn reserva_prueba() -> Reserva {
        Reserva {
            id: 1,
            codigo: "TEST0001".into(),
            id_cliente: 3,
            id_creador: None,
            id_mesero: None,
            tipo_restaurante_completo: false,
            mesa_ids: vec![1, 2],
            inicio: Utc::now() + chrono::Duration::days(2),
            duracion_minutos: 120,
            num_comensales: 4,
            ocasion: Ocasion::Casual,
            pedido: vec![],
            servicios_extra: vec![],
            restricciones: "<maní> & \"mostaza\"".into(),
            observaciones: String::new(),
            pago: DetallePago {
                modalidad: ModalidadPago::Adelanto,
                porcentaje: 30,
                medio: "TARJETA".into(),
                monto_total: 100_000,
                propina_pct: 0,
            },
            estado: EstadoReserva::Vigente,
        }
    }

    #[test]
    fn html_cliente_lleva_enlace_de_gestion_y_regla_vip() {
        let html = html_cliente(
            "Ana",
            &reserva_prueba(),
            TipoCliente::Vip,
            "http://demo.test",
        );
        assert!(
            html.contains("http://demo.test/mis-reservas?reserva=TEST0001"),
            "sin enlace de gestión"
        );
        assert!(html.contains("1 hora"), "sin ventana de cancelación VIP");
        assert!(html.contains("$ 30.000"), "sin abono registrado");
    }

    #[test]
    fn html_admin_escapa_datos_del_cliente() {
        let html = html_admin(
            "Ana",
            "ana@ejemplo.com",
            &reserva_prueba(),
            "http://demo.test",
        );
        assert!(html.contains("TEST0001"));
        assert!(
            html.contains("&lt;maní&gt;"),
            "las restricciones deben ir escapadas"
        );
        assert!(!html.contains("<maní>"));
        assert!(
            html.contains("http://demo.test/agenda"),
            "sin acceso a la agenda"
        );
    }

    #[test]
    fn a_correo_arma_alternativa_texto_y_html() {
        let m = Mensaje {
            id: 1,
            para: "ana@ejemplo.com".into(),
            asunto: "Brindis: prueba".into(),
            cuerpo: "texto plano".into(),
            html: Some(html_cliente(
                "Ana",
                &reserva_prueba(),
                TipoCliente::General,
                "http://demo.test",
            )),
            enviado_en: Utc::now(),
            entregado: false,
        };
        let crudo = String::from_utf8(a_correo(&m, "no-reply@brindis.co").unwrap().formatted())
            .expect("correo en UTF-8");
        assert!(crudo.contains("multipart/alternative"), "sin alternativa");
        assert!(
            crudo.contains("text/plain") && crudo.contains("text/html"),
            "faltan las partes"
        );
        let simple = Mensaje {
            html: None,
            ..m.clone()
        };
        let crudo_simple = String::from_utf8(
            a_correo(&simple, "no-reply@brindis.co")
                .unwrap()
                .formatted(),
        )
        .expect("correo en UTF-8");
        assert!(crudo_simple.contains("text/plain"));
        assert!(!crudo_simple.contains("multipart/alternative"));
    }
}
