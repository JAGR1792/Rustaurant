//! Correos de la fase 1: al confirmar una reserva se redacta el aviso al
//! cliente y a la administración, queda en un buzón en memoria que el
//! personal ve en la agenda (demostración) y, si el entorno trae
//! credenciales SMTP (`SMTP_USUARIO`/`SMTP_CLAVE` en el `.env` local,
//! ADR 0009), se entrega de verdad con `lettre` sin tocar el servicio de
//! reservas.

use std::time::Duration;

use chrono::{DateTime, Utc};
use lettre::message::{Mailbox, header::ContentType};
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

    let mut pendientes: Vec<(String, String, String)> = Vec::new();
    if !correo_cliente.is_empty() {
        pendientes.push((
            correo_cliente.clone(),
            format!("Brindis: tu cita quedó pedida ({})", r.codigo),
            cuerpo_cliente(&nombre, r, tipo_cliente),
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
        ));
    }

    let mut redactados = Vec::new();
    for (para, asunto, cuerpo) in pendientes {
        let id = d.correos.iter().map(|m| m.id).max().unwrap_or(0) + 1;
        let mensaje = Mensaje {
            id,
            para: para.clone(),
            asunto,
            cuerpo,
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

/// Arma el correo de `lettre` con cabeceras y cuerpo en texto plano.
fn a_correo(m: &Mensaje, remitente: &str) -> Result<Message, String> {
    let de: Mailbox = remitente
        .parse()
        .map_err(|e| format!("remitente inválido: {e}"))?;
    let para: Mailbox = m
        .para
        .parse()
        .map_err(|e| format!("destino inválido: {e}"))?;
    Message::builder()
        .from(de)
        .to(para)
        .subject(m.asunto.clone())
        .header(ContentType::TEXT_PLAIN)
        .body(m.cuerpo.clone())
        .map_err(|e| format!("cuerpo inválido: {e}"))
}

fn marcar_entregado(estado: &Estado, id: i64) {
    let mut d = estado.lock().unwrap();
    if let Some(m) = d.correos.iter_mut().find(|m| m.id == id) {
        m.entregado = true;
    }
}
