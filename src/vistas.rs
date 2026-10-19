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
use chrono::{Datelike, DateTime, Duration, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};

use crate::builder::{ReservaBuilder, validar_modalidad};
use crate::dao::DaoError;
use crate::dominio::{
    DetallePago, DetallePedido, EstadoReserva, ModalidadPago, Ocasion, Reserva,
    ReservaServicioExtra, UsuarioSesion,
};
use crate::estado::{self, Datos, Estado};
use crate::factory::{ClienteFactory, DatosRegistro, registrar_con};
use crate::formato::{cop, fecha_hora_fmt, fecha_iso_local, local_a_utc, utc_a_local};
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

/// Nombre, bandera de personal y bandera de VIP para la barra de navegación.
fn barra(jar: &CookieJar, e: &Estado) -> (String, bool, bool) {
    match sesion_actual(jar, e) {
        Some(u) => (u.nombre().to_string(), u.es_personal(), u.es_vip()),
        None => (String::new(), false, false),
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
    imagen: String,
    imagen_webp: String,
}

/// Tarjeta de "La carta" en la portada (efecto chroma del frontend).
/// Imagen y acento son presentación, por eso el mapeo vive aquí y no
/// en el dominio.
struct ItemCartaVista {
    titulo: String,
    subtitulo: String,
    imagen: String,
    imagen_webp: String,
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
    id: i64,
    hora: String,
    fecha: String,
    codigo: String,
    cliente: String,
    es_vip: bool,
    mesas: String,
    comensales: u32,
    ocasion: String,
    indicacion: String,
    restricciones: String,
    alerta: bool,
    estado: String,
    estado_clase: String,
    puede_sentar: bool,
    puede_completar: bool,
    puede_inasistir: bool,
}

struct DiaCalendarioVista {
    dia: u32,
    fecha_iso: String,
    es_hoy: bool,
    es_seleccionado: bool,
    num_reservas: usize,
}

struct ClienteGestionVista {
    id: i64,
    nombre: String,
    correo: String,
    documento: String,
    telefono: String,
    es_vip: bool,
    strikes: u8,
}

/// Fila del buzón de demostración que ve el personal en la agenda.
struct CorreoVista {
    para: String,
    asunto: String,
    cuerpo: String,
    cuando: String,
    /// `true` si el SMTP confirmó la entrega real (ADR 0009).
    entregado: bool,
}

#[derive(Template)]
#[template(path = "inicio.html")]
struct Inicio {
    usuario: String,
    es_personal: bool,
    es_vip: bool,
    zonas: Vec<ZonaVista>,
    carta: Vec<ItemCartaVista>,
}

#[derive(Template)]
#[template(path = "login.html")]
struct Login {
    usuario: String,
    es_personal: bool,
    es_vip: bool,
    error: Option<String>,
    reactivar: bool,
    correo: String,
}

#[derive(Template)]
#[template(path = "registro.html")]
struct Registro {
    usuario: String,
    es_personal: bool,
    es_vip: bool,
    error: Option<String>,
}

#[derive(Template)]
#[template(path = "reservar.html")]
struct Reservar {
    usuario: String,
    es_personal: bool,
    es_vip: bool,
    error: Option<String>,
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
    es_vip: bool,
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
    es_vip: bool,
    denegado: bool,
    fecha: String,
    dia_ant: String,
    dia_sig: String,
    mes_nombre: String,
    mes_anterior: String,
    mes_siguiente: String,
    calendario: Vec<DiaCalendarioVista>,
    filas: Vec<AgendaFila>,
    proximas: Vec<AgendaFila>,
    clientes: Vec<ClienteGestionVista>,
    total_hoy: usize,
    comensales_hoy: u32,
    ocupacion_mesas_hoy: usize,
    total_mesas: usize,
    vacio: bool,
    correos: Vec<CorreoVista>,
    aviso: Option<String>,
}

#[derive(Template)]
#[template(path = "dashboard.html")]
struct Dashboard {
    usuario: String,
    es_personal: bool,
    es_vip: bool,
    nombre: String,
    correo: String,
    documento: String,
    telefono: String,
    tipo_cliente: String,
    strikes: u8,
    total_reservas: usize,
    reservas_completadas: usize,
    progreso_vip: usize,
    progreso_meta: usize,
    progreso_pct: u32,
    puede_reclamar_vip: bool,
    proxima: Option<ReservaFila>,
    cuenta: Option<CuentaRegresiva>,
    aviso: Option<String>,
}

#[derive(Template)]
#[template(path = "error.html")]
struct ErrorPagina {
    usuario: String,
    es_personal: bool,
    es_vip: bool,
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

/// Par de la foto real de un plato: WebP para el `<picture>` de la UI y
/// el JPEG/PNG original como respaldo (ver
/// `static/img/fotos/descargar_fotos.py` y `creditos.md`). Un id nuevo
/// cae al PNG genérico de `static/img/carta`.
struct Foto {
    jpg: &'static str,
    webp: &'static str,
}

fn foto_item(id: i64) -> Foto {
    match id {
        1 => Foto {
            jpg: "/static/img/fotos/carta-01-ceviche.jpg",
            webp: "/static/img/fotos/carta-01-ceviche.webp",
        },
        2 => Foto {
            jpg: "/static/img/fotos/carta-02-picana.jpg",
            webp: "/static/img/fotos/carta-02-picana.webp",
        },
        3 => Foto {
            jpg: "/static/img/fotos/carta-03-tres-leches.jpg",
            webp: "/static/img/fotos/carta-03-tres-leches.webp",
        },
        4 => Foto {
            jpg: "/static/img/fotos/carta-04-limonada-coco.jpg",
            webp: "/static/img/fotos/carta-04-limonada-coco.webp",
        },
        5 => Foto {
            jpg: "/static/img/fotos/carta-05-salmon.jpg",
            webp: "/static/img/fotos/carta-05-salmon.webp",
        },
        6 => Foto {
            jpg: "/static/img/fotos/carta-06-coctel.jpg",
            webp: "/static/img/fotos/carta-06-coctel.webp",
        },
        _ => Foto {
            jpg: "/static/img/carta/00-generico.png",
            webp: "/static/img/carta/00-generico.webp",
        },
    }
}

/// Carta de la portada: platos del menú con su foto y acento de color.
fn carta(d: &Datos) -> Vec<ItemCartaVista> {
    d.items
        .iter()
        .map(|i| {
            let accent = match i.id {
                1 => "#7cc4ff",
                2 => "#ff5a36",
                3 => "#ffb27a",
                4 => "#ffd23f",
                5 => "#a98bff",
                6 => "#ff7eb6",
                _ => "#a88a59",
            };
            let foto = foto_item(i.id);
            ItemCartaVista {
                titulo: i.nombre.clone(),
                subtitulo: format!("{} · {}", tipo_legible(&i.tipo), cop(i.precio_vigente)),
                imagen: foto.jpg.into(),
                imagen_webp: foto.webp.into(),
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
                .map(|i| {
                    let foto = foto_item(i.id);
                    ItemVista {
                        id: i.id,
                        nombre: i.nombre.clone(),
                        precio: i.precio_vigente,
                        precio_fmt: cop(i.precio_vigente),
                        imagen: foto.jpg.into(),
                        imagen_webp: foto.webp.into(),
                    }
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
        es_vip: u.es_vip() || u.es_personal(),
        error,
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
    let (usuario, es_personal, es_vip) = barra(&jar, &e);
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
        es_vip,
        zonas,
        carta: carta_vista,
    })
    .into_response()
}

pub async fn login_get(State(e): State<Estado>, jar: CookieJar) -> Response {
    if let Some(u) = sesion_actual(&jar, &e) {
        if u.es_personal() {
            return Redirect::to("/agenda").into_response();
        } else {
            return Redirect::to("/dashboard").into_response();
        }
    }
    Html(Login {
        usuario: String::new(),
        es_personal: false,
        es_vip: false,
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
            es_vip: false,
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
            es_vip: false,
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
        "/dashboard"
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
        es_vip: false,
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
            es_vip: false,
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
    (jar, Redirect::to("/dashboard?ok=bienvenida")).into_response()
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
    let propina_pct = campos
        .get("propina_pct")
        .and_then(|s| s.parse::<u8>().ok())
        .unwrap_or(0);

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
            propina_pct,
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
        es_vip: u.es_vip(),
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
            es_vip: u.es_vip(),
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
    let aviso = q.get("ok").map(|ok| {
        match ok.as_str() {
            "cliente_actualizado" => "Estatus de cliente actualizado correctamente.",
            "estado_actualizado" => "Estado de la reserva actualizado en el sistema.",
            "strikes_reiniciados" => "Strikes del cliente restablecidos a 0.",
            _ => "Operación completada.",
        }
        .to_string()
    });

    let hoy = utc_a_local(Utc::now()).date();
    let dia = q
        .get("dia")
        .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
        .unwrap_or(hoy);
    let anio = dia.year();
    let mes = dia.month();

    let mut denegado = false;
    let mut filas: Vec<AgendaFila> = vec![];
    let mut proximas: Vec<AgendaFila> = vec![];
    let mut clientes: Vec<ClienteGestionVista> = vec![];
    let mut correos: Vec<CorreoVista> = vec![];
    let mut calendario: Vec<DiaCalendarioVista> = vec![];
    let mut total_hoy = 0;
    let mut comensales_hoy = 0;
    let mut ocupacion_mesas_hoy = 0;
    let mut total_mesas = 0;

    let mes_nombre = match mes {
        1 => "Enero",
        2 => "Febrero",
        3 => "Marzo",
        4 => "Abril",
        5 => "Mayo",
        6 => "Junio",
        7 => "Julio",
        8 => "Agosto",
        9 => "Septiembre",
        10 => "Octubre",
        11 => "Noviembre",
        12 => "Diciembre",
        _ => "",
    }
    .to_string();

    let mes_anterior = if mes == 1 {
        NaiveDate::from_ymd_opt(anio - 1, 12, 1)
            .unwrap()
            .format("%Y-%m-%d")
            .to_string()
    } else {
        NaiveDate::from_ymd_opt(anio, mes - 1, 1)
            .unwrap()
            .format("%Y-%m-%d")
            .to_string()
    };

    let mes_siguiente = if mes == 12 {
        NaiveDate::from_ymd_opt(anio + 1, 1, 1)
            .unwrap()
            .format("%Y-%m-%d")
            .to_string()
    } else {
        NaiveDate::from_ymd_opt(anio, mes + 1, 1)
            .unwrap()
            .format("%Y-%m-%d")
            .to_string()
    };

    match servicio(&e).agenda(&u).await {
        Err(_) => denegado = true,
        Ok(reservas) => {
            let d = e.lock().unwrap();
            total_mesas = d.mesas.len();

            // Calendario del mes
            let dias_en_mes = match mes {
                1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
                4 | 6 | 9 | 11 => 30,
                2 => {
                    if (anio % 4 == 0 && anio % 100 != 0) || (anio % 400 == 0) {
                        29
                    } else {
                        28
                    }
                }
                _ => 30,
            };
            for d_num in 1..=dias_en_mes {
                if let Some(fecha_dia) = NaiveDate::from_ymd_opt(anio, mes, d_num) {
                    let num_reservas = reservas
                        .iter()
                        .filter(|r| utc_a_local(r.inicio).date() == fecha_dia)
                        .count();
                    calendario.push(DiaCalendarioVista {
                        dia: d_num,
                        fecha_iso: fecha_dia.format("%Y-%m-%d").to_string(),
                        es_hoy: fecha_dia == hoy,
                        es_seleccionado: fecha_dia == dia,
                        num_reservas,
                    });
                }
            }

            // Próximas reservas inmediatas (activas en mesa o vigentes futuras)
            let mut proximas_raw: Vec<_> = reservas
                .iter()
                .filter(|r| {
                    matches!(
                        r.estado,
                        EstadoReserva::Vigente | EstadoReserva::ClienteEnMesa
                    )
                })
                .cloned()
                .collect();
            proximas_raw.sort_by_key(|r| r.inicio);
            proximas = proximas_raw
                .iter()
                .take(4)
                .map(|r| {
                    let cliente_obj = d.usuarios.iter().find(|x| x.id() == r.id_cliente);
                    let cliente_nom = cliente_obj
                        .map(|x| x.nombre().to_string())
                        .unwrap_or_else(|| format!("Cliente {}", r.id_cliente));
                    let es_vip = cliente_obj.map(|x| x.es_vip()).unwrap_or(false);
                    let restricciones = r.restricciones.clone();
                    AgendaFila {
                        id: r.id,
                        hora: utc_a_local(r.inicio).format("%H:%M").to_string(),
                        fecha: utc_a_local(r.inicio).format("%Y-%m-%d").to_string(),
                        codigo: r.codigo.clone(),
                        cliente: cliente_nom,
                        es_vip,
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
                        puede_sentar: r.estado == EstadoReserva::Vigente,
                        puede_completar: r.estado == EstadoReserva::ClienteEnMesa,
                        puede_inasistir: r.estado == EstadoReserva::Vigente,
                    }
                })
                .collect();

            // Reservas del día seleccionado
            let mut del_dia: Vec<_> = reservas
                .iter()
                .filter(|r| utc_a_local(r.inicio).date() == dia)
                .collect();
            del_dia.sort_by_key(|r| r.inicio);
            total_hoy = del_dia.len();
            comensales_hoy = del_dia.iter().map(|r| r.num_comensales).sum();
            let mut mesas_ocupadas_set = std::collections::HashSet::new();
            for r in &del_dia {
                if matches!(
                    r.estado,
                    EstadoReserva::Vigente | EstadoReserva::ClienteEnMesa
                ) {
                    for m in &r.mesa_ids {
                        mesas_ocupadas_set.insert(*m);
                    }
                }
            }
            ocupacion_mesas_hoy = mesas_ocupadas_set.len();

            filas = del_dia
                .iter()
                .map(|r| {
                    let cliente_obj = d.usuarios.iter().find(|x| x.id() == r.id_cliente);
                    let cliente = cliente_obj
                        .map(|x| x.nombre().to_string())
                        .unwrap_or_else(|| format!("Cliente {}", r.id_cliente));
                    let es_vip = cliente_obj.map(|x| x.es_vip()).unwrap_or(false);
                    let restricciones = r.restricciones.clone();
                    AgendaFila {
                        id: r.id,
                        hora: utc_a_local(r.inicio).format("%H:%M").to_string(),
                        fecha: utc_a_local(r.inicio).format("%Y-%m-%d").to_string(),
                        codigo: r.codigo.clone(),
                        cliente,
                        es_vip,
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
                        puede_sentar: r.estado == EstadoReserva::Vigente,
                        puede_completar: r.estado == EstadoReserva::ClienteEnMesa,
                        puede_inasistir: r.estado == EstadoReserva::Vigente,
                    }
                })
                .collect();

            // Lista de clientes registrados para gestión VIP
            clientes = d
                .usuarios
                .iter()
                .filter_map(|usr| {
                    if let UsuarioSesion::Cliente(c) = usr {
                        Some(ClienteGestionVista {
                            id: c.base.id,
                            nombre: c.base.nombre.clone(),
                            correo: c.base.correo.clone(),
                            documento: format!("{} {}", c.base.tipo_documento, c.base.documento),
                            telefono: c.base.telefono.clone(),
                            es_vip: c.es_vip(),
                            strikes: c.strikes,
                        })
                    } else {
                        None
                    }
                })
                .collect();

            // Buzón de demostración: los últimos correos "enviados" (ADR 0008).
            correos = d
                .correos
                .iter()
                .rev()
                .take(6)
                .map(|m| CorreoVista {
                    para: m.para.clone(),
                    asunto: m.asunto.clone(),
                    cuerpo: m.cuerpo.clone(),
                    cuando: fecha_hora_fmt(m.enviado_en),
                    entregado: m.entregado,
                })
                .collect();
        }
    }
    let vacio = filas.is_empty();
    Html(Agenda {
        usuario: u.nombre().into(),
        es_personal: u.es_personal(),
        es_vip: u.es_vip(),
        denegado,
        fecha: dia.format("%Y-%m-%d").to_string(),
        dia_ant: (dia - Duration::days(1)).format("%Y-%m-%d").to_string(),
        dia_sig: (dia + Duration::days(1)).format("%Y-%m-%d").to_string(),
        mes_nombre,
        mes_anterior,
        mes_siguiente,
        calendario,
        filas,
        proximas,
        clientes,
        total_hoy,
        comensales_hoy,
        ocupacion_mesas_hoy,
        total_mesas,
        vacio,
        correos,
        aviso,
    })
    .into_response()
}

pub async fn dashboard(
    State(e): State<Estado>,
    jar: CookieJar,
    Query(q): Query<HashMap<String, String>>,
) -> Response {
    let Some(u) = sesion_actual(&jar, &e) else {
        return Redirect::to("/login").into_response();
    };
    if u.es_personal() {
        return Redirect::to("/agenda").into_response();
    }
    let aviso = q.get("ok").map(|ok| {
        match ok.as_str() {
            "vip_activado" => "¡Felicidades! Tu membresía VIP Brindis Club ha sido activada con éxito.",
            "bienvenida" => "¡Bienvenido a Brindis! Tu cuenta ha sido creada exitosamente.",
            _ => "Operación completada.",
        }
        .to_string()
    });
    let d = e.lock().unwrap();
    let cliente_id = u.id();
    let cliente_reservas: Vec<_> = d
        .reservas
        .iter()
        .filter(|r| r.id_cliente == cliente_id)
        .cloned()
        .collect();
    let total_reservas = cliente_reservas.len();
    let reservas_completadas = cliente_reservas
        .iter()
        .filter(|r| r.estado == EstadoReserva::Completada)
        .count();
    let es_vip = u.es_vip();
    let progreso_meta = 3;
    let progreso_vip = reservas_completadas.min(progreso_meta);
    let progreso_pct = ((progreso_vip as f32 / progreso_meta as f32) * 100.0) as u32;
    let puede_reclamar_vip = !es_vip && reservas_completadas >= progreso_meta;

    // Próxima reserva
    let mut reservas_futuras = cliente_reservas.clone();
    reservas_futuras.sort_by_key(|r| r.inicio);
    let prox_reserva = proxima_reserva(&reservas_futuras, Utc::now()).cloned();
    let cuenta = prox_reserva.as_ref().map(|r| CuentaRegresiva {
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
    });
    let proxima = prox_reserva.map(|r| ReservaFila {
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
        movimiento: String::new(),
        estado: r.estado.etiqueta().into(),
        estado_clase: r.estado.clase().into(),
        puede_cancelar: r.estado == EstadoReserva::Vigente,
    });

    let (documento, telefono, strikes) = match &u {
        UsuarioSesion::Cliente(c) => (
            format!("{} {}", c.base.tipo_documento, c.base.documento),
            c.base.telefono.clone(),
            c.strikes,
        ),
        _ => (String::new(), String::new(), 0),
    };

    Html(Dashboard {
        usuario: u.nombre().into(),
        es_personal: false,
        es_vip,
        nombre: u.nombre().into(),
        correo: u.correo().into(),
        documento,
        telefono,
        tipo_cliente: if es_vip {
            "Socio VIP Club"
        } else {
            "Cliente General"
        }
        .into(),
        strikes,
        total_reservas,
        reservas_completadas,
        progreso_vip,
        progreso_meta,
        progreso_pct,
        puede_reclamar_vip,
        proxima,
        cuenta,
        aviso,
    })
    .into_response()
}

#[derive(Deserialize)]
pub struct FormHacerVip {
    #[allow(dead_code)]
    motivo: Option<String>,
}

pub async fn hacer_vip_cliente(
    State(e): State<Estado>,
    jar: CookieJar,
    Form(_f): Form<FormHacerVip>,
) -> Response {
    let Some(u) = sesion_actual(&jar, &e) else {
        return Redirect::to("/login").into_response();
    };
    if let UsuarioSesion::Cliente(c) = u {
        let mut d = e.lock().unwrap();
        d.actualizar_vip_cliente(c.base.id, true);
    }
    Redirect::to("/dashboard?ok=vip_activado").into_response()
}

pub async fn toggle_vip_cliente(
    State(e): State<Estado>,
    jar: CookieJar,
    Path(id): Path<i64>,
) -> Response {
    let Some(u) = sesion_actual(&jar, &e) else {
        return Redirect::to("/login").into_response();
    };
    if !u.es_personal() {
        return Redirect::to("/login").into_response();
    }
    {
        let mut d = e.lock().unwrap();
        let es_vip_actual = d.usuarios.iter().any(|usr| {
            if let UsuarioSesion::Cliente(c) = usr {
                c.base.id == id && c.es_vip()
            } else {
                false
            }
        });
        d.actualizar_vip_cliente(id, !es_vip_actual);
    }
    Redirect::to("/agenda?ok=cliente_actualizado").into_response()
}

#[derive(Deserialize)]
pub struct FormCambioEstado {
    nuevo: String,
}

pub async fn cambiar_estado_reserva(
    State(e): State<Estado>,
    jar: CookieJar,
    Path(id): Path<i64>,
    Form(f): Form<FormCambioEstado>,
) -> Response {
    let Some(u) = sesion_actual(&jar, &e) else {
        return Redirect::to("/login").into_response();
    };
    if !u.es_personal() {
        return Redirect::to("/login").into_response();
    }
    let nuevo_estado = match f.nuevo.as_str() {
        "en-mesa" => EstadoReserva::ClienteEnMesa,
        "completada" => EstadoReserva::Completada,
        "inasistida" => EstadoReserva::Inasistida,
        "cancelada" => EstadoReserva::Cancelada,
        _ => return Redirect::to("/agenda").into_response(),
    };
    {
        let mut d = e.lock().unwrap();
        let _ = d.cambiar_estado_reserva(id, nuevo_estado);
    }
    Redirect::to("/agenda?ok=estado_actualizado").into_response()
}

pub async fn reset_strikes_cliente(
    State(e): State<Estado>,
    jar: CookieJar,
    Path(id): Path<i64>,
) -> Response {
    let Some(u) = sesion_actual(&jar, &e) else {
        return Redirect::to("/login").into_response();
    };
    if !u.es_personal() {
        return Redirect::to("/login").into_response();
    }
    {
        let mut d = e.lock().unwrap();
        d.limpiar_strikes_cliente(id);
    }
    Redirect::to("/agenda?ok=strikes_reiniciados").into_response()
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
                propina_pct: 0,
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
            assert!(p.imagen.starts_with("/static/img/"));
            assert!(p.imagen_webp.ends_with(".webp"));
            assert!(p.accent.starts_with('#'));
            assert!(!p.subtitulo.is_empty(), "sin subtitulo: {}", p.titulo);
            assert_eq!(p.url, "/reservar");
        }
        assert_eq!(platos[0].titulo, "Ceviche de camarón");
        assert!(platos[0].imagen.ends_with("carta-01-ceviche.jpg"));
        assert!(platos[0].imagen_webp.ends_with("carta-01-ceviche.webp"));
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
    fn el_seed_deja_una_reserva_futura_para_la_cuenta() {
        let e = estado::sembrar();
        let d = e.lock().unwrap();
        let p = proxima_reserva(&d.reservas, Utc::now());
        assert_eq!(p.map(|r| r.codigo.as_str()), Some("DEMO0001"));
    }
}
