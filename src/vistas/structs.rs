//! Capa Vista del MVC: structs de plantilla (Askama) y DTOs de presentacion.
//! Los extractores de formulario viajan tambien aqui porque son parte del
//! contrato HTTP de la vista; el controlador los consume via `use crate::vistas::*;`.

use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};

use askama::Template;
use serde::{Deserialize, Serialize};

/// Respuesta HTML de una plantilla Askama (integración mínima y propia).
pub struct Html<T: Template>(pub T);
impl<T: Template> IntoResponse for Html<T> {
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(cuerpo) => {
                ([(header::CONTENT_TYPE, "text/html; charset=utf-8")], cuerpo).into_response()
            }
            Err(e) => {
                tracing::error!("no se pudo renderizar la plantilla: {e}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Error interno al renderizar la vista.",
                )
                    .into_response()
            }
        }
    }
}
pub struct ZonaVista {
    pub nombre: String,
    pub mesas: usize,
    pub puestos: u32,
}
pub struct OcasionVista {
    pub clave: String,
    pub nombre: String,
    pub zona: String,
    pub decoracion: Option<String>,
    pub activa: bool,
}
/// Opción del select de zonas con su marca de selección ya resuelta
/// (Askama 0.13 no compara cómodamente `&String` contra `String`).
pub struct ZonaOpcion {
    pub nombre: String,
    pub seleccionada: bool,
}
pub struct ItemVista {
    pub id: i64,
    pub nombre: String,
    pub precio: i64,
    pub precio_fmt: String,
    pub imagen: String,
    pub imagen_webp: String,
}
/// Tarjeta de "La carta" en la portada (efecto chroma del frontend).
/// Imagen y acento son presentación, por eso el mapeo vive aquí y no
/// en el dominio.
pub struct ItemCartaVista {
    pub titulo: String,
    pub subtitulo: String,
    pub imagen: String,
    pub imagen_webp: String,
    pub accent: String,
    pub url: String,
}
pub struct PlanVista {
    pub id: i64,
    pub nombre: String,
    pub tarifa: i64,
    pub tarifa_fmt: String,
    pub por_persona: bool,
}
pub struct ReservaFila {
    pub id: i64,
    pub codigo: String,
    pub fecha: String,
    pub mesas: String,
    pub comensales: u32,
    pub ocasion: String,
    pub total: String,
    pub movimiento: String,
    pub estado: String,
    pub estado_clase: String,
    pub puede_cancelar: bool,
    /// Consecuencia exacta de cancelar ahora, para el modal (RN08/RN09).
    pub aviso_cancelacion: String,
    /// `true` cuando se llegó desde el enlace del correo
    /// (`/mis-reservas?reserva=CÓDIGO`): la fila se resalta.
    pub resaltada: bool,
}
/// Panel de cuenta regresiva para la próxima reserva del cliente
/// (la dibuja `counter.js` a partir de `iso`).
pub struct CuentaRegresiva {
    pub codigo: String,
    pub detalle: String,
    pub iso: String,
}
pub struct AgendaFila {
    pub id: i64,
    pub hora: String,
    pub fecha: String,
    pub codigo: String,
    pub cliente: String,
    pub es_vip: bool,
    pub mesas: String,
    pub comensales: u32,
    pub ocasion: String,
    pub indicacion: String,
    pub restricciones: String,
    pub alerta: bool,
    pub estado: String,
    pub estado_clase: String,
    pub puede_sentar: bool,
    pub puede_completar: bool,
    pub puede_inasistir: bool,
}
pub struct DiaCalendarioVista {
    pub dia: u32,
    pub fecha_iso: String,
    pub es_hoy: bool,
    pub es_seleccionado: bool,
    pub num_reservas: usize,
}
pub struct ClienteGestionVista {
    pub id: i64,
    pub nombre: String,
    pub correo: String,
    pub documento: String,
    pub telefono: String,
    pub es_vip: bool,
    pub strikes: u8,
}
/// Fila del buzón de demostración que ve el personal en la agenda.
pub struct CorreoVista {
    pub para: String,
    pub asunto: String,
    pub cuerpo: String,
    /// Versión HTML del aviso (multipart/alternative) para la vista previa.
    pub html: Option<String>,
    pub cuando: String,
    /// `true` si el SMTP confirmó la entrega real (ADR 0009).
    pub entregado: bool,
}
#[derive(Template)]
#[template(path = "inicio.html")]
pub struct Inicio {
    pub usuario: String,
    pub es_personal: bool,
    pub es_vip: bool,
    pub zonas: Vec<ZonaVista>,
    pub carta: Vec<ItemCartaVista>,
}
#[derive(Template)]
#[template(path = "login.html")]
pub struct Login {
    pub usuario: String,
    pub es_personal: bool,
    pub es_vip: bool,
    pub error: Option<String>,
    pub reactivar: bool,
    pub correo: String,
}
#[derive(Template)]
#[template(path = "registro.html")]
pub struct Registro {
    pub usuario: String,
    pub es_personal: bool,
    pub es_vip: bool,
    pub error: Option<String>,
}
#[derive(Template)]
#[template(path = "recuperar.html")]
pub struct Recuperar {
    pub usuario: String,
    pub es_personal: bool,
    pub es_vip: bool,
    pub correo: String,
    pub enviado: bool,
    pub error: Option<String>,
}
#[derive(Template)]
#[template(path = "restablecer.html")]
pub struct Restablecer {
    pub usuario: String,
    pub es_personal: bool,
    pub es_vip: bool,
    pub token: String,
    pub error: Option<String>,
    pub exito: bool,
}
#[derive(Template)]
#[template(path = "reservar.html")]
pub struct Reservar {
    pub usuario: String,
    pub es_personal: bool,
    pub es_vip: bool,
    pub error: Option<String>,
    pub fecha: String,
    pub fecha_hoy: String,
    pub fecha_max: String,
    pub hora: String,
    pub comensales: u32,
    pub zonas: Vec<ZonaOpcion>,
    pub ocasiones: Vec<OcasionVista>,
    pub items: Vec<ItemVista>,
    pub planes: Vec<PlanVista>,
}
#[derive(Template)]
#[template(path = "mis_reservas.html")]
pub struct MisReservas {
    pub usuario: String,
    pub es_personal: bool,
    pub es_vip: bool,
    pub aviso: Option<String>,
    pub filas: Vec<ReservaFila>,
    pub vacio: bool,
    pub cuenta: Option<CuentaRegresiva>,
    /// Strikes acumulados del cliente (RN10); solo aplica a clientes.
    pub strikes: u8,
}
#[derive(Template)]
#[template(path = "agenda.html")]
pub struct Agenda {
    pub usuario: String,
    pub es_personal: bool,
    pub es_vip: bool,
    pub denegado: bool,
    pub fecha: String,
    pub dia_ant: String,
    pub dia_sig: String,
    pub mes_nombre: String,
    pub mes_anterior: String,
    pub mes_siguiente: String,
    pub calendario: Vec<DiaCalendarioVista>,
    pub espacios_mes: usize,
    pub filas: Vec<AgendaFila>,
    pub proximas: Vec<AgendaFila>,
    pub clientes: Vec<ClienteGestionVista>,
    pub total_hoy: usize,
    pub comensales_hoy: u32,
    pub ocupacion_mesas_hoy: usize,
    pub total_mesas: usize,
    pub vacio: bool,
    pub correos: Vec<CorreoVista>,
    pub aviso: Option<String>,
}
/// Fragmento del panel de agenda: lo sirve GET /agenda/dia para navegar por
/// días y meses sin recargar la página completa.
#[derive(Template)]
#[template(path = "agenda_dia.html")]
pub struct AgendaDia {
    pub fecha: String,
    pub dia_ant: String,
    pub dia_sig: String,
    pub mes_nombre: String,
    pub mes_anterior: String,
    pub mes_siguiente: String,
    pub calendario: Vec<DiaCalendarioVista>,
    pub espacios_mes: usize,
    pub filas: Vec<AgendaFila>,
    pub total_hoy: usize,
    pub comensales_hoy: u32,
    pub ocupacion_mesas_hoy: usize,
    pub total_mesas: usize,
    pub vacio: bool,
}
#[derive(Template)]
#[template(path = "dashboard.html")]
pub struct Dashboard {
    pub usuario: String,
    pub es_personal: bool,
    pub es_vip: bool,
    pub nombre: String,
    pub correo: String,
    pub documento: String,
    pub telefono: String,
    pub tipo_cliente: String,
    pub strikes: u8,
    pub total_reservas: usize,
    pub reservas_completadas: usize,
    pub progreso_vip: usize,
    pub progreso_meta: usize,
    pub progreso_pct: u32,
    pub puede_reclamar_vip: bool,
    pub proxima: Option<ReservaFila>,
    pub cuenta: Option<CuentaRegresiva>,
    pub aviso: Option<String>,
}
#[derive(Template)]
#[template(path = "error.html")]
pub struct ErrorPagina {
    pub usuario: String,
    pub es_personal: bool,
    pub es_vip: bool,
    pub mensaje: String,
}
/// Par de la foto real de un plato: WebP para el `<picture>` de la UI y
/// el JPEG/PNG original como respaldo (ver
/// `static/img/fotos/descargar_fotos.py` y `creditos.md`). Un id nuevo
/// cae al PNG genérico de `static/img/carta`.
pub struct Foto {
    pub jpg: &'static str,
    pub webp: &'static str,
}
/// Campos del asistente que se conservan al re-renderizar con error.
#[derive(Clone, Default)]
pub struct CamposReserva {
    pub ocasion: String,
    pub fecha: String,
    pub hora: String,
    pub comensales: String,
    pub zona: String,
}
#[derive(Deserialize)]
pub struct FormLogin {
    pub correo: String,
    pub password: String,
}
#[derive(Deserialize)]
pub struct FormRegistro {
    pub nombre: String,
    pub correo: String,
    pub password: String,
}
/// Contadores y filas del día seleccionado.
pub struct ResumenDia {
    pub filas: Vec<AgendaFila>,
    pub total_hoy: usize,
    pub comensales_hoy: u32,
    pub ocupacion_mesas_hoy: usize,
    pub vacio: bool,
}
#[derive(Deserialize)]
pub struct FormHacerVip {
    #[allow(dead_code)]
    pub motivo: Option<String>,
}
#[derive(Deserialize)]
pub struct FormCambioEstado {
    pub nuevo: String,
}
#[derive(Deserialize)]
pub struct ParamDisponibilidad {
    pub fecha: Option<String>,
    pub hora: Option<String>,
    pub zona: Option<String>,
}
#[derive(Serialize)]
pub struct MesaDto {
    pub id: i64,
    pub zona: String,
    pub capacidad: u8,
    pub estado: &'static str,
}
#[derive(Deserialize)]
pub struct FormRecuperar {
    pub correo: String,
}
#[derive(Deserialize)]
pub struct FormRestablecer {
    pub token: String,
    pub password: String,
    pub password2: String,
}
