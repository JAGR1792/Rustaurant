//! Capa Control del MVC: recibe HTTP, orquesta dominio/estado y responde
//! HTML (Askama) o JSON. No contiene reglas de negocio (viven en builder,
//! decorator y servicios) ni control de acceso (vive en el Proxy).

use std::collections::HashMap;

use askama::Template;
use axum::{
    Form, Json,
    extract::{Path, Query, State},
    http::{StatusCode, header},
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use chrono::{DateTime, Duration, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use serde::{Deserialize, Serialize};

use crate::builder::{ReservaBuilder, validar_modalidad};
use crate::dao::DaoError;
use crate::dominio::{
    DetallePago, DetallePedido, EstadoReserva, ModalidadPago, Ocasion, Reserva,
    ReservaServicioExtra, TipoCliente, UsuarioSesion,
};
use crate::estado::{self, Datos, Estado};
use crate::factory::{ClienteFactory, DatosRegistro, registrar_con};
use crate::proxy::{AccesoError, ReservaService, ReservaServiceProxy};
use crate::servicios::ReservaServiceMemoria;

// ------------------------------------------------------------------ utilidades

/// Respuesta HTML de una plantilla Askama (integración mínima y propia).
struct Html<T: Template>(T);

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

/// Fase 1 asume la hora de Colombia (UTC-5, sin horario de verano).
const UTC_A_LOCAL: i64 = 5; // UTC = local + 5 horas

fn local_a_utc(n: NaiveDateTime) -> DateTime<Utc> {
    DateTime::from_naive_utc_and_offset(n + Duration::hours(UTC_A_LOCAL), Utc)
}

fn utc_a_local(dt: DateTime<Utc>) -> NaiveDateTime {
    dt.naive_utc() - Duration::hours(UTC_A_LOCAL)
}

fn fecha_hora_fmt(dt: DateTime<Utc>) -> String {
    utc_a_local(dt).format("%Y-%m-%d %H:%M").to_string()
}

/// ISO 8601 con el desfase local (UTC-5) para que el navegador calcule la
/// cuenta regresiva con la hora correcta (p. ej. 2026-10-19T19:30:00-05:00).
fn fecha_iso_local(dt: DateTime<Utc>) -> String {
    let offset = FixedOffset::west_opt((UTC_A_LOCAL * 3600) as i32).expect("desfase UTC-5 válido");
    dt.with_timezone(&offset)
        .format("%Y-%m-%dT%H:%M:%S%:z")
        .to_string()
}

/// Pesos colombianos con separador de miles: 440000 -> "440.000".
fn cop(n: i64) -> String {
    let s = n.abs().to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push('.');
        }
        out.push(c);
    }
    if n < 0 { format!("-{out}") } else { out }
}

/// La reserva futura más cercana (solo vigentes): alimenta la cuenta
/// regresiva que aparece tras reservar.
fn proxima_reserva(reservas: &[Reserva], ahora: DateTime<Utc>) -> Option<&Reserva> {
    reservas
        .iter()
        .filter(|r| r.estado == EstadoReserva::Vigente && r.inicio > ahora)
        .min_by_key(|r| r.inicio)
}

fn sesion_actual(jar: &CookieJar, e: &Estado) -> Option<UsuarioSesion> {
    let token = jar.get("sesion")?.value().to_string();
    let d = e.lock().unwrap();
    let id = *d.sesiones.get(&token)?;
    d.usuarios.iter().find(|u| u.id() == id).cloned()
}

/// Nombre y bandera de personal para la barra de navegación.
fn barra(jar: &CookieJar, e: &Estado) -> (String, bool) {
    match sesion_actual(jar, e) {
        Some(u) => (u.nombre().to_string(), u.es_personal()),
        None => (String::new(), false),
    }
}

fn servicio(e: &Estado) -> ReservaServiceProxy<ReservaServiceMemoria> {
    ReservaServiceProxy::new(ReservaServiceMemoria::new(e.clone()))
}

fn poner_sesion(e: &Estado, jar: CookieJar, id: i64) -> CookieJar {
    let token = uuid::Uuid::new_v4().to_string();
    e.lock().unwrap().sesiones.insert(token.clone(), id);
    let cookie = Cookie::build(("sesion", token))
        .http_only(true)
        .same_site(SameSite::Lax)
        .path("/")
        .build();
    jar.add(cookie)
}

fn parsear_modalidad(clave: &str) -> (ModalidadPago, u8) {
    match clave {
        "completo" => (ModalidadPago::Completo, 100),
        "restaurante" => (ModalidadPago::EnRestaurante, 0),
        _ => (ModalidadPago::Adelanto, 30),
    }
}

// ------------------------------------------------------------------ vistas (structs Askama)

struct ZonaVista {
    nombre: String,
    mesas: usize,
    puestos: u32,
}

struct OcasionVista {
    clave: String,
    nombre: String,
    zona: String,
    decoracion: Option<String>,
    activa: bool,
}

/// Opción del select de zonas con su marca de selección ya resuelta
/// (Askama 0.13 no compara cómodamente `&String` contra `String`).
struct ZonaOpcion {
    nombre: String,
    seleccionada: bool,
}

struct ItemVista {
    id: i64,
    nombre: String,
    precio: i64,
    precio_fmt: String,
}

/// Tarjeta de "La carta" en la portada (efecto chroma del frontend).
/// Imagen y acento son presentación, por eso el mapeo vive aquí y no
/// en el dominio.
struct ItemCartaVista {
    titulo: String,
    subtitulo: String,
    imagen: String,
    accent: String,
    url: String,
}

struct PlanVista {
    id: i64,
    nombre: String,
    tarifa: i64,
    tarifa_fmt: String,
    por_persona: bool,
}

struct ReservaFila {
    id: i64,
    codigo: String,
    fecha: String,
    mesas: String,
    comensales: u32,
    ocasion: String,
    total: String,
    movimiento: String,
    estado: String,
    estado_clase: String,
    puede_cancelar: bool,
}

/// Panel de cuenta regresiva para la próxima reserva del cliente
/// (la dibuja `counter.js` a partir de `iso`).
struct CuentaRegresiva {
    codigo: String,
    detalle: String,
    iso: String,
}

struct AgendaFila {
    hora: String,
    codigo: String,
    cliente: String,
    mesas: String,
    comensales: u32,
    ocasion: String,
    indicacion: String,
    restricciones: String,
    alerta: bool,
    estado: String,
    estado_clase: String,
}

#[derive(Template)]
#[template(path = "inicio.html")]
struct Inicio {
    usuario: String,
    es_personal: bool,
    zonas: Vec<ZonaVista>,
    carta: Vec<ItemCartaVista>,
}

#[derive(Template)]
#[template(path = "login.html")]
struct Login {
    usuario: String,
    es_personal: bool,
    error: Option<String>,
    reactivar: bool,
    correo: String,
}

#[derive(Template)]
#[template(path = "registro.html")]
struct Registro {
    usuario: String,
    es_personal: bool,
    error: Option<String>,
}

#[derive(Template)]
#[template(path = "reservar.html")]
struct Reservar {
    usuario: String,
    es_personal: bool,
    error: Option<String>,
    es_vip: bool,
    fecha: String,
    fecha_hoy: String,
    fecha_max: String,
    hora: String,
    comensales: u32,
    zonas: Vec<ZonaOpcion>,
    ocasiones: Vec<OcasionVista>,
    items: Vec<ItemVista>,
    planes: Vec<PlanVista>,
}

#[derive(Template)]
#[template(path = "mis_reservas.html")]
struct MisReservas {
    usuario: String,
    es_personal: bool,
    aviso: Option<String>,
    filas: Vec<ReservaFila>,
    vacio: bool,
    cuenta: Option<CuentaRegresiva>,
}

#[derive(Template)]
#[template(path = "agenda.html")]
struct Agenda {
    usuario: String,
    es_personal: bool,
    denegado: bool,
    fecha: String,
    dia_ant: String,
    dia_sig: String,
    filas: Vec<AgendaFila>,
    vacio: bool,
}

#[derive(Template)]
#[template(path = "error.html")]
struct ErrorPagina {
    usuario: String,
    es_personal: bool,
    mensaje: String,
}

fn lista_ocasiones(seleccionada: &str) -> Vec<OcasionVista> {
    use Ocasion::*;
    [
        Cumpleanos,
        Aniversario,
        Propuesta,
        Romantica,
        Negocios,
        Casual,
    ]
    .iter()
    .map(|o| OcasionVista {
        clave: o.clave().into(),
        nombre: o.nombre().into(),
        zona: o.zona_sugerida().into(),
        decoracion: o.decoracion_ofrecida().map(str::to_string),
        activa: o.clave() == seleccionada,
    })
    .collect()
}

fn zonas_disponibles(seleccionada: &str) -> Vec<ZonaOpcion> {
    ["Cualquiera", "VIP", "General", "Terraza"]
        .iter()
        .map(|z| ZonaOpcion {
            nombre: (*z).into(),
            seleccionada: *z == seleccionada,
        })
        .collect()
}

/// Nombre legible de la categoría de un plato para la carta.
fn tipo_legible(tipo: &str) -> &'static str {
    match tipo {
        "ENTRADA" => "Entrada",
        "PLATO_FUERTE" => "Plato fuerte",
        "POSTRE" => "Postre",
        "BEBIDA" => "Bebida",
        _ => "De la casa",
    }
}

/// Carta de la portada: platos del menú con su imagen y acento de color.
/// El id del item mapea al PNG generado en `static/img/carta` (el mismo
/// id que usa `generar_carta.py`); si aparece un id nuevo hay respaldo.
fn carta(d: &Datos) -> Vec<ItemCartaVista> {
    d.items
        .iter()
        .map(|i| {
            let (archivo, accent) = match i.id {
                1 => ("01-ceviche-camaron.png", "#7cc4ff"),
                2 => ("02-picanha-brasa.png", "#ff5a36"),
                3 => ("03-tres-leches.png", "#ffb27a"),
                4 => ("04-limonada-coco.png", "#ffd23f"),
                5 => ("05-salmon-hierbas.png", "#a98bff"),
                6 => ("06-coctel-casa.png", "#ff7eb6"),
                _ => ("00-generico.png", "#a88a59"),
            };
            ItemCartaVista {
                titulo: i.nombre.clone(),
                subtitulo: format!("{} · {}", tipo_legible(&i.tipo), cop(i.precio_vigente)),
                imagen: format!("/static/img/carta/{archivo}"),
                accent: accent.into(),
                url: "/reservar".into(),
            }
        })
        .collect()
}

/// Campos del asistente que se conservan al re-renderizar con error.
#[derive(Clone, Default)]
struct CamposReserva {
    ocasion: String,
    fecha: String,
    hora: String,
    comensales: String,
    zona: String,
}

fn render_reservar(
    e: &Estado,
    u: &UsuarioSesion,
    campos: &CamposReserva,
    error: Option<String>,
) -> Response {
    let hoy = utc_a_local(Utc::now()).date();
    let fecha_hoy = hoy.format("%Y-%m-%d").to_string();
    let fecha_max = (hoy + Duration::days(60)).format("%Y-%m-%d").to_string();
    let fecha = if campos.fecha.is_empty() {
        (hoy + Duration::days(1)).format("%Y-%m-%d").to_string()
    } else {
        campos.fecha.clone()
    };
    let hora = if campos.hora.is_empty() {
        "19:00".into()
    } else {
        campos.hora.clone()
    };
    let zona = if campos.zona.is_empty() {
        "Cualquiera".into()
    } else {
        campos.zona.clone()
    };
    let (items, planes) = {
        let d = e.lock().unwrap();
        (
            d.items
                .iter()
                .map(|i| ItemVista {
                    id: i.id,
                    nombre: i.nombre.clone(),
                    precio: i.precio_vigente,
                    precio_fmt: cop(i.precio_vigente),
                })
                .collect::<Vec<_>>(),
            d.planes
                .iter()
                .map(|p| PlanVista {
                    id: p.id,
                    nombre: p.nombre.clone(),
                    tarifa: p.tarifa,
                    tarifa_fmt: cop(p.tarifa),
                    por_persona: p.por_persona,
                })
                .collect::<Vec<_>>(),
        )
    };
    Html(Reservar {
        usuario: u.nombre().into(),
        es_personal: u.es_personal(),
        error,
        es_vip: matches!(u, UsuarioSesion::Cliente(c) if c.tipo == TipoCliente::Vip)
            || u.es_personal(),
        fecha,
        fecha_hoy,
        fecha_max,
        hora,
        comensales: campos.comensales.parse().unwrap_or(2),
        zonas: zonas_disponibles(&zona),
        ocasiones: lista_ocasiones(&campos.ocasion),
        items,
        planes,
    })
    .into_response()
}

// ------------------------------------------------------------------ handlers

pub async fn inicio(State(e): State<Estado>, jar: CookieJar) -> Response {
    let (usuario, es_personal) = barra(&jar, &e);
    let (zonas, carta_vista) = {
        let d = e.lock().unwrap();
        let zonas = ["VIP", "General", "Terraza"]
            .iter()
            .filter_map(|z| {
                let mesas: Vec<_> = d.mesas.iter().filter(|m| m.zona == *z).collect();
                if mesas.is_empty() {
                    return None;
                }
                Some(ZonaVista {
                    nombre: (*z).to_string(),
                    mesas: mesas.len(),
                    puestos: mesas.iter().map(|m| m.capacidad as u32).sum(),
                })
            })
            .collect();
        (zonas, carta(&d))
    };
    Html(Inicio {
        usuario,
        es_personal,
        zonas,
        carta: carta_vista,
    })
    .into_response()
}

pub async fn login_get(State(e): State<Estado>, jar: CookieJar) -> Response {
    if sesion_actual(&jar, &e).is_some() {
        return Redirect::to("/").into_response();
    }
    Html(Login {
        usuario: String::new(),
        es_personal: false,
        error: None,
        reactivar: false,
        correo: String::new(),
    })
    .into_response()
}

#[derive(Deserialize)]
pub struct FormLogin {
    correo: String,
    password: String,
}

pub async fn login_post(
    State(e): State<Estado>,
    jar: CookieJar,
    Form(f): Form<FormLogin>,
) -> Response {
    let encontrado = {
        let d = e.lock().unwrap();
        estado::buscar_por_correo(&d, f.correo.trim())
    };
    let credenciales_ok = encontrado
        .as_ref()
        .map(|u| bcrypt::verify(&f.password, &u.base().hash_pw).unwrap_or(false))
        .unwrap_or(false);
    let Some(u) = encontrado.filter(|_| credenciales_ok) else {
        return Html(Login {
            usuario: String::new(),
            es_personal: false,
            // CU01 2a: error genérico, sin revelar cuál dato falló.
            error: Some("Correo o contraseña incorrectos.".into()),
            reactivar: false,
            correo: f.correo,
        })
        .into_response();
    };
    if !u.base().activo {
        // CU01 2b: cuenta inactiva -> se ofrece reactivación por token.
        return Html(Login {
            usuario: String::new(),
            es_personal: false,
            error: Some("Tu cuenta está inactiva.".into()),
            reactivar: true,
            correo: f.correo,
        })
        .into_response();
    }
    let jar = poner_sesion(&e, jar, u.id());
    let destino = if u.es_personal() {
        "/agenda"
    } else {
        "/mis-reservas"
    };
    (jar, Redirect::to(destino)).into_response()
}

pub async fn registro_get(State(e): State<Estado>, jar: CookieJar) -> Response {
    if sesion_actual(&jar, &e).is_some() {
        return Redirect::to("/").into_response();
    }
    Html(Registro {
        usuario: String::new(),
        es_personal: false,
        error: None,
    })
    .into_response()
}

#[derive(Deserialize)]
pub struct FormRegistro {
    nombre: String,
    correo: String,
    password: String,
}

pub async fn registro_post(
    State(e): State<Estado>,
    jar: CookieJar,
    Form(f): Form<FormRegistro>,
) -> Response {
    let correo = f.correo.trim().to_lowercase();
    let fallo = |msg: &str| {
        Html(Registro {
            usuario: String::new(),
            es_personal: false,
            error: Some(msg.into()),
        })
        .into_response()
    };
    if !correo.contains('@') || !correo.contains('.') {
        return fallo("Escribe un correo válido.");
    }
    if f.password.len() < 8 {
        return fallo("La contraseña debe tener al menos 8 caracteres.");
    }
    if f.nombre.trim().is_empty() {
        return fallo("Escribe tu nombre.");
    }
    // Factory Method (Figura 7): el registro público siempre crea un cliente
    // general con 0 strikes; el método plantilla valida, cifra y persiste.
    let datos = DatosRegistro {
        correo,
        password_plano: f.password,
        tipo_documento: "CC".into(),
        documento: String::new(),
        nombre: f.nombre.trim().to_string(),
        telefono: String::new(),
        creado_por_personal: false,
    };
    let dao = estado::DaoMemoria { estado: e.clone() };
    let sesion = match registrar_con(&ClienteFactory, &dao, datos).await {
        Ok(s) => s,
        Err(DaoError::Duplicado(_)) => return fallo("Ese correo ya está registrado."),
        Err(err) => {
            tracing::error!("no se pudo registrar: {err}");
            return fallo("No se pudo crear la cuenta; intenta de nuevo.");
        }
    };
    let jar = poner_sesion(&e, jar, sesion.id());
    (jar, Redirect::to("/mis-reservas?ok=bienvenida")).into_response()
}

pub async fn logout(State(e): State<Estado>, jar: CookieJar) -> Response {
    if let Some(c) = jar.get("sesion") {
        e.lock().unwrap().sesiones.remove(c.value());
    }
    (jar.remove(Cookie::from("sesion")), Redirect::to("/")).into_response()
}

pub async fn reservar_get(State(e): State<Estado>, jar: CookieJar) -> Response {
    let Some(u) = sesion_actual(&jar, &e) else {
        return Redirect::to("/login").into_response();
    };
    render_reservar(&e, &u, &CamposReserva::default(), None)
}

pub async fn reserva_post(
    State(e): State<Estado>,
    jar: CookieJar,
    Form(campos): Form<HashMap<String, String>>,
) -> Response {
    let Some(u) = sesion_actual(&jar, &e) else {
        return Redirect::to("/login").into_response();
    };
    let vista = CamposReserva {
        ocasion: campos.get("ocasion").cloned().unwrap_or_default(),
        fecha: campos.get("fecha").cloned().unwrap_or_default(),
        hora: campos.get("hora").cloned().unwrap_or_default(),
        comensales: campos.get("comensales").cloned().unwrap_or_default(),
        zona: campos.get("zona").cloned().unwrap_or_default(),
    };
    let fallo = |msg: String| render_reservar(&e, &u, &vista, Some(msg));

    let Some(ocasion) = Ocasion::desde_clave(vista.ocasion.trim()) else {
        return fallo("Elige una ocasión válida.".into());
    };
    let Ok(fecha) = NaiveDate::parse_from_str(vista.fecha.trim(), "%Y-%m-%d") else {
        return fallo("La fecha no es válida.".into());
    };
    let Ok(hora) = NaiveTime::parse_from_str(vista.hora.trim(), "%H:%M") else {
        return fallo("La hora no es válida.".into());
    };
    let Ok(comensales) = vista.comensales.trim().parse::<u32>() else {
        return fallo("Número de comensales inválido.".into());
    };
    if !(1..=20).contains(&comensales) {
        return fallo("Los comensales deben estar entre 1 y 20.".into());
    }
    let mesas: Vec<i64> = campos
        .get("mesas")
        .map(|s| s.split(',').filter_map(|x| x.trim().parse().ok()).collect())
        .unwrap_or_default();
    if mesas.is_empty() {
        return fallo("Selecciona al menos una mesa en el paso 2.".into());
    }
    let extras_ids: Vec<i64> = campos
        .get("extras")
        .map(|s| s.split(',').filter_map(|x| x.trim().parse().ok()).collect())
        .unwrap_or_default();
    let pedir: Vec<(i64, u32)> = campos
        .iter()
        .filter_map(|(k, v)| {
            let id = k.strip_prefix("item_")?.parse::<i64>().ok()?;
            let n = v.trim().parse::<u32>().ok()?;
            (n > 0).then_some((id, n))
        })
        .collect();

    // Resolver capacidades, precios congelados (RN07) y planes de una sola vez.
    let (capacidades, pedido, extras) = {
        let d = e.lock().unwrap();
        let mut capacidades = Vec::with_capacity(mesas.len());
        for id in &mesas {
            match d.mesas.iter().find(|m| m.id == *id) {
                Some(m) => capacidades.push(m.capacidad),
                None => return fallo(format!("La mesa {id} no existe.")),
            }
        }
        let pedido: Vec<DetallePedido> = pedir
            .iter()
            .filter_map(|(id, n)| {
                d.items.iter().find(|i| i.id == *id).map(|i| DetallePedido {
                    item_id: i.id,
                    cantidad: *n,
                    precio_unitario_congelado: i.precio_vigente,
                })
            })
            .collect();
        let extras: Vec<ReservaServicioExtra> = extras_ids
            .iter()
            .filter_map(|id| {
                d.planes
                    .iter()
                    .find(|p| p.id == *id)
                    .map(|p| ReservaServicioExtra {
                        plan_id: p.id,
                        precio_congelado: p.tarifa,
                    })
            })
            .collect();
        (capacidades, pedido, extras)
    };

    let (modalidad, porcentaje) = parsear_modalidad(
        campos
            .get("modalidad")
            .map(String::as_str)
            .unwrap_or("adelanto"),
    );
    if let Err(err) = validar_modalidad(modalidad, u.tipo_cliente(), u.es_personal()) {
        return fallo(format!("{err}"));
    }
    let restricciones = campos
        .get("restricciones")
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    let construida = ReservaBuilder::nuevo()
        .cliente(u.id())
        .creador(if u.es_personal() { Some(u.id()) } else { None })
        .tipo_completo(false)
        .mesas(mesas, capacidades)
        .fecha(local_a_utc(fecha.and_time(hora)))
        .comensales(comensales)
        .ocasion(ocasion)
        .pedido(pedido)
        .extras(extras)
        .restricciones(if restricciones.is_empty() {
            "Sin alergias".into()
        } else {
            restricciones
        })
        .pago(DetallePago {
            modalidad,
            porcentaje,
            medio: "TARJETA".into(),
            monto_total: 0, // el servicio lo recalcula con los decoradores
        })
        .build();
    let reserva = match construida {
        Ok(r) => r,
        Err(err) => return fallo(format!("{err}")),
    };
    match servicio(&e).confirmar(&u, reserva).await {
        Ok(_) => Redirect::to("/mis-reservas?ok=creada").into_response(),
        Err(AccesoError::Denegado(m) | AccesoError::Negocio(m)) => fallo(m),
    }
}

pub async fn mis_reservas(
    State(e): State<Estado>,
    jar: CookieJar,
    Query(q): Query<HashMap<String, String>>,
) -> Response {
    let Some(u) = sesion_actual(&jar, &e) else {
        return Redirect::to("/login").into_response();
    };
    let aviso = q.get("ok").map(|ok| {
        match ok.as_str() {
            "creada" => "Reserva registrada con éxito.",
            "cancelada" => "Reserva cancelada.",
            "bienvenida" => "¡Cuenta creada! Bienvenido a Brindis.",
            _ => "Listo.",
        }
        .to_string()
    });
    let (filas, vacio, cuenta) = {
        let d = e.lock().unwrap();
        let mut reservas: Vec<_> = d
            .reservas
            .iter()
            .filter(|r| match &u {
                UsuarioSesion::Cliente(c) => r.id_cliente == c.base.id,
                _ => true,
            })
            .cloned()
            .collect();
        reservas.sort_by_key(|r| r.inicio);
        let filas: Vec<ReservaFila> = reservas
            .iter()
            .map(|r| ReservaFila {
                id: r.id,
                codigo: r.codigo.clone(),
                fecha: fecha_hora_fmt(r.inicio),
                mesas: r
                    .mesa_ids
                    .iter()
                    .map(|m| format!("M{m}"))
                    .collect::<Vec<_>>()
                    .join(", "),
                comensales: r.num_comensales,
                ocasion: r.ocasion.nombre().into(),
                total: if r.pago.monto_total > 0 {
                    format!("$ {}", cop(r.pago.monto_total))
                } else {
                    "—".into()
                },
                movimiento: match d.comprobantes.iter().rev().find(|c| c.reserva_id == r.id) {
                    Some(c) if c.tipo_movimiento == "INGRESO" => {
                        format!("Pagado $ {}", cop(c.monto_total))
                    }
                    Some(c) => format!(
                        "Reembolsado $ {} (retención $ {})",
                        cop(c.monto_total),
                        cop(c.monto_retencion)
                    ),
                    None => "—".into(),
                },
                estado: r.estado.etiqueta().into(),
                estado_clase: r.estado.clase().into(),
                puede_cancelar: r.estado == EstadoReserva::Vigente,
            })
            .collect();
        let vacio = filas.is_empty();
        // Cuenta regresiva solo para clientes: es "su" próxima mesa.
        let cuenta = if matches!(u, UsuarioSesion::Cliente(_)) {
            proxima_reserva(&reservas, Utc::now()).map(|r| CuentaRegresiva {
                codigo: r.codigo.clone(),
                detalle: format!(
                    "{} · {} comensales · {}",
                    r.mesa_ids
                        .iter()
                        .map(|m| format!("M{m}"))
                        .collect::<Vec<_>>()
                        .join(", "),
                    r.num_comensales,
                    r.ocasion.nombre()
                ),
                iso: fecha_iso_local(r.inicio),
            })
        } else {
            None
        };
        (filas, vacio, cuenta)
    };
    Html(MisReservas {
        usuario: u.nombre().into(),
        es_personal: u.es_personal(),
        aviso,
        filas,
        vacio,
        cuenta,
    })
    .into_response()
}

pub async fn cancelar(State(e): State<Estado>, jar: CookieJar, Path(id): Path<i64>) -> Response {
    let Some(u) = sesion_actual(&jar, &e) else {
        return Redirect::to("/login").into_response();
    };
    match servicio(&e).cancelar(&u, id).await {
        Ok(()) => Redirect::to("/mis-reservas?ok=cancelada").into_response(),
        Err(AccesoError::Denegado(m) | AccesoError::Negocio(m)) => Html(ErrorPagina {
            usuario: u.nombre().into(),
            es_personal: u.es_personal(),
            mensaje: m,
        })
        .into_response(),
    }
}

pub async fn agenda(
    State(e): State<Estado>,
    jar: CookieJar,
    Query(q): Query<HashMap<String, String>>,
) -> Response {
    let Some(u) = sesion_actual(&jar, &e) else {
        return Redirect::to("/login").into_response();
    };
    let hoy = utc_a_local(Utc::now()).date();
    let dia = q
        .get("dia")
        .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
        .unwrap_or(hoy);
    // El Proxy (Figura 9) verifica el rol antes de entregar la agenda (RF15).
    let mut denegado = false;
    let mut filas: Vec<AgendaFila> = vec![];
    match servicio(&e).agenda(&u).await {
        Err(_) => denegado = true,
        Ok(reservas) => {
            let d = e.lock().unwrap();
            let mut del_dia: Vec<_> = reservas
                .iter()
                .filter(|r| utc_a_local(r.inicio).date() == dia)
                .collect();
            del_dia.sort_by_key(|r| r.inicio);
            filas = del_dia
                .iter()
                .map(|r| {
                    let cliente = d
                        .usuarios
                        .iter()
                        .find(|x| x.id() == r.id_cliente)
                        .map(|x| x.nombre().to_string())
                        .unwrap_or_else(|| format!("Cliente {}", r.id_cliente));
                    let restricciones = r.restricciones.clone();
                    AgendaFila {
                        hora: utc_a_local(r.inicio).format("%H:%M").to_string(),
                        codigo: r.codigo.clone(),
                        cliente,
                        mesas: r
                            .mesa_ids
                            .iter()
                            .map(|m| format!("M{m}"))
                            .collect::<Vec<_>>()
                            .join(", "),
                        comensales: r.num_comensales,
                        ocasion: r.ocasion.nombre().into(),
                        indicacion: r.ocasion.indicacion_personal().into(),
                        alerta: !restricciones.is_empty()
                            && !restricciones.eq_ignore_ascii_case("sin alergias"),
                        restricciones,
                        estado: r.estado.etiqueta().into(),
                        estado_clase: r.estado.clase().into(),
                    }
                })
                .collect();
        }
    }
    let vacio = filas.is_empty();
    Html(Agenda {
        usuario: u.nombre().into(),
        es_personal: u.es_personal(),
        denegado,
        fecha: dia.format("%Y-%m-%d").to_string(),
        dia_ant: (dia - Duration::days(1)).format("%Y-%m-%d").to_string(),
        dia_sig: (dia + Duration::days(1)).format("%Y-%m-%d").to_string(),
        filas,
        vacio,
    })
    .into_response()
}

#[derive(Deserialize)]
pub struct ParamDisponibilidad {
    fecha: Option<String>,
    hora: Option<String>,
    zona: Option<String>,
}

#[derive(Serialize)]
struct MesaDto {
    id: i64,
    zona: String,
    capacidad: u8,
    estado: &'static str,
}

/// API del plano: GET /api/disponibilidad?fecha=&hora=&zona=
pub async fn disponibilidad(
    State(e): State<Estado>,
    Query(q): Query<ParamDisponibilidad>,
) -> Response {
    let ahora_local = utc_a_local(Utc::now());
    let fecha = q
        .fecha
        .unwrap_or_else(|| ahora_local.format("%Y-%m-%d").to_string());
    let hora = q.hora.unwrap_or_else(|| "19:00".to_string());
    let Ok(fecha) = NaiveDate::parse_from_str(&fecha, "%Y-%m-%d") else {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "fecha inválida" })),
        )
            .into_response();
    };
    let Ok(hora) = NaiveTime::parse_from_str(&hora, "%H:%M") else {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "hora inválida" })),
        )
            .into_response();
    };
    let inicio = local_a_utc(fecha.and_time(hora));
    let fin = inicio + Duration::minutes(120); // RN01: franja estándar
    let zona = q.zona.unwrap_or_else(|| "Cualquiera".into());
    let d = e.lock().unwrap();
    let mesas: Vec<MesaDto> = d
        .mesas
        .iter()
        .filter(|m| zona == "Cualquiera" || m.zona == zona)
        .map(|m| MesaDto {
            id: m.id,
            zona: m.zona.clone(),
            capacidad: m.capacidad,
            estado: if m.disponible_fisicamente {
                estado::estado_de_mesa(&d, m.id, inicio, fin)
            } else {
                "bloqueada"
            },
        })
        .collect();
    Json(mesas).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn reserva_prueba(id: i64, inicio: DateTime<Utc>, estado: EstadoReserva) -> Reserva {
        Reserva {
            id,
            codigo: format!("TEST{id:04}"),
            id_cliente: 3,
            id_creador: None,
            tipo_restaurante_completo: false,
            mesa_ids: vec![1],
            inicio,
            duracion_minutos: 120,
            num_comensales: 2,
            ocasion: Ocasion::Casual,
            pedido: vec![],
            servicios_extra: vec![],
            restricciones: String::new(),
            observaciones: String::new(),
            pago: DetallePago {
                modalidad: ModalidadPago::Adelanto,
                porcentaje: 30,
                medio: "TARJETA".into(),
                monto_total: 0,
            },
            estado,
        }
    }

    #[test]
    fn carta_mapea_todos_los_platos_del_seed() {
        let e = estado::sembrar();
        let d = e.lock().unwrap();
        let platos = carta(&d);
        assert_eq!(platos.len(), d.items.len());
        for p in &platos {
            assert!(!p.titulo.is_empty());
            assert!(p.imagen.starts_with("/static/img/carta/"));
            assert!(p.accent.starts_with('#'));
            assert!(!p.subtitulo.is_empty(), "sin subtitulo: {}", p.titulo);
            assert_eq!(p.url, "/reservar");
        }
        assert_eq!(platos[0].titulo, "Ceviche de camarón");
        assert!(platos[0].imagen.ends_with("01-ceviche-camaron.png"));
        assert!(platos[0].subtitulo.contains("Entrada"));
        assert!(
            platos[0].subtitulo.contains("15.000"),
            "con precio: {}",
            platos[0].subtitulo
        );
    }

    #[test]
    fn carta_sin_items_es_vacia() {
        let e = estado::nuevo();
        let d = e.lock().unwrap();
        assert!(carta(&d).is_empty());
    }

    #[test]
    fn proxima_reserva_ignora_pasadas_canceladas_y_completadas() {
        let ahora = Utc::now();
        let reservas = vec![
            reserva_prueba(1, ahora + Duration::days(5), EstadoReserva::Vigente),
            reserva_prueba(2, ahora + Duration::days(2), EstadoReserva::Cancelada),
            reserva_prueba(3, ahora - Duration::hours(3), EstadoReserva::Vigente),
            reserva_prueba(4, ahora + Duration::days(8), EstadoReserva::Completada),
            reserva_prueba(5, ahora + Duration::days(3), EstadoReserva::Vigente),
        ];
        let p = proxima_reserva(&reservas, ahora).expect("hay una vigente futura");
        assert_eq!(p.id, 5, "debe elegir la vigente más cercana");
        assert!(proxima_reserva(&[], ahora).is_none());
    }

    #[test]
    fn fecha_iso_local_lleva_el_desfase_de_colombia() {
        let dt = Utc.with_ymd_and_hms(2026, 10, 20, 0, 30, 0).unwrap();
        assert_eq!(fecha_iso_local(dt), "2026-10-19T19:30:00-05:00");
    }

    #[test]
    fn el_seed_deja_una_reserva_futura_para_la_cuenta() {
        let e = estado::sembrar();
        let d = e.lock().unwrap();
        let p = proxima_reserva(&d.reservas, Utc::now());
        assert_eq!(p.map(|r| r.codigo.as_str()), Some("DEMO0001"));
    }
}
