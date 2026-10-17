//! Correos de la fase 1: al confirmar una reserva se redacta el aviso al
//! cliente y a la administración, y queda en un buzón en memoria que el
//! personal ve en la agenda (demostración). En producción se reemplaza
//! por un canal SMTP real sin tocar el servicio de reservas (ADR 0008).

use chrono::{DateTime, Utc};

use crate::dominio::{ModalidadPago, Reserva, UsuarioSesion};
use crate::estado::Datos;
use crate::formato::{cop, fecha_hora_fmt};

/// Un correo listo para enviar; en la demo vive en `Datos::correos`.
#[derive(Clone)]
pub struct Mensaje {
    pub id: i64,
    pub para: String,
    pub asunto: String,
    pub cuerpo: String,
    pub enviado_en: DateTime<Utc>,
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

fn cuerpo_cliente(nombre: &str, r: &Reserva) -> String {
    format!(
        "Hola {nombre}:\n\
         \n\
         Tu cita quedó pedida para el {} (mesas {}, {} comensales).\n\
         {}\n\
         \n\
         Recuerda: puedes cancelar sin retención hasta 24 horas antes del inicio; \
         si cancelas dentro de las últimas 24 horas se retiene el 10% de lo abonado.\n\
         \n\
         Te esperamos en Brindis.",
        fecha_hora_fmt(r.inicio),
        mesas(r),
        r.num_comensales,
        linea_pago(r),
    )
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

/// Deposita en el buzón el correo del cliente y el de cada administrador.
pub fn notificar_reserva(d: &mut Datos, r: &Reserva) {
    let cliente = d.usuarios.iter().find(|u| u.id() == r.id_cliente);
    let nombre = cliente
        .map(|u| u.nombre().to_string())
        .unwrap_or_else(|| format!("Cliente {}", r.id_cliente));
    let correo_cliente = cliente.map(|u| u.correo().to_string()).unwrap_or_default();

    let mut pendientes: Vec<(String, String, String)> = Vec::new();
    if !correo_cliente.is_empty() {
        pendientes.push((
            correo_cliente.clone(),
            format!("Brindis: tu cita quedó pedida ({})", r.codigo),
            cuerpo_cliente(&nombre, r),
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

    for (para, asunto, cuerpo) in pendientes {
        let id = d.correos.iter().map(|m| m.id).max().unwrap_or(0) + 1;
        d.correos.push(Mensaje {
            id,
            para: para.clone(),
            asunto,
            cuerpo,
            enviado_en: Utc::now(),
        });
        tracing::info!(destino = %para, "correo de reserva en el buzón de demostración");
    }
}
