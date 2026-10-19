//! Brindis — Sistema de reservas por ocasión (Axum + MVC por capas en Rust).
//! Vista: Askama + plano.js | Controlador: handlers de `vistas.rs` |
//! Modelo: dominio + patrones (factory/builder/proxy/decorator) + DAO/estado.

pub mod controladores;
pub mod modelo;
pub mod vistas;

// Compatibilidad (Etapa 1): re-exportar submódulos del modelo en la raíz
// del crate para no romper referencias internas existentes.
pub use modelo::entidades::dominio;
pub use modelo::entidades::formato;
pub use modelo::patrones::creacionales::builder;
pub use modelo::patrones::creacionales::fabrica as factory;
pub use modelo::patrones::estructurales::decorador as decorator;
pub use modelo::patrones::estructurales::proxy;
pub use modelo::repositorios::dao;
pub use modelo::repositorios::estado;
pub use modelo::servicios::correo;
pub use modelo::servicios::reserva_servicio as servicios;

use axum::{
    Router,
    routing::{get, post},
};
use tower_http::{services::ServeDir, trace::TraceLayer};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    // Configuración local (.env, nunca se commitea): credenciales SMTP, etc.
    // En formato inválido o ausente se sigue con las variables del sistema.
    if let Err(e) = dotenvy::dotenv() {
        tracing::debug!(error = %e, "sin .env utilizable; se usan las variables del sistema");
    }

    let estado = estado::sembrar();

    let app = Router::new()
        .route("/", get(vistas::inicio))
        .route("/login", get(vistas::login_get).post(vistas::login_post))
        .route(
            "/recuperar",
            get(vistas::recuperar_get).post(vistas::recuperar_post),
        )
        .route(
            "/restablecer",
            get(vistas::restablecer_get).post(vistas::restablecer_post),
        )
        .route(
            "/registro",
            get(vistas::registro_get).post(vistas::registro_post),
        )
        .route("/logout", get(vistas::logout))
        .route(
            "/reservar",
            get(vistas::reservar_get).post(vistas::reserva_post),
        )
        .route("/mis-reservas", get(vistas::mis_reservas))
        .route("/reservas/{id}/cancelar", post(vistas::cancelar))
        .route("/dashboard", get(vistas::dashboard))
        .route("/dashboard/hacer-vip", post(vistas::hacer_vip_cliente))
        .route("/agenda", get(vistas::agenda))
        .route("/agenda/dia", get(vistas::agenda_dia))
        .route("/panel-empleado", get(vistas::agenda))
        .route(
            "/empleado/reserva/{id}/estado",
            post(vistas::cambiar_estado_reserva),
        )
        .route(
            "/admin/cliente/{id}/toggle-vip",
            post(vistas::toggle_vip_cliente),
        )
        .route(
            "/admin/cliente/{id}/reset-strikes",
            post(vistas::reset_strikes_cliente),
        )
        .route("/api/disponibilidad", get(vistas::disponibilidad))
        .nest_service("/static", ServeDir::new("static"))
        .layer(TraceLayer::new_for_http())
        .with_state(estado);

    // Dirección de escucha: local por defecto; en contenedor (Render) se
    // define HOST=0.0.0.0 y el PORT lo inyecta la plataforma (ADR 0010).
    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let puerto = std::env::var("PORT").unwrap_or_else(|_| "3000".into());
    let direccion = format!("{host}:{puerto}");
    let listener = tokio::net::TcpListener::bind(&direccion).await.unwrap();
    tracing::info!("Brindis en http://{direccion}");
    axum::serve(listener, app).await.unwrap();
}
