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
        .route("/", get(controladores::inicio))
        .route(
            "/login",
            get(controladores::login_get).post(controladores::login_post),
        )
        .route(
            "/recuperar",
            get(controladores::recuperar_get).post(controladores::recuperar_post),
        )
        .route(
            "/restablecer",
            get(controladores::restablecer_get).post(controladores::restablecer_post),
        )
        .route(
            "/registro",
            get(controladores::registro_get).post(controladores::registro_post),
        )
        .route("/logout", get(controladores::logout))
        .route(
            "/reservar",
            get(controladores::reservar_get).post(controladores::reserva_post),
        )
        .route("/mis-reservas", get(controladores::mis_reservas))
        .route("/reservas/{id}/cancelar", post(controladores::cancelar))
        .route("/dashboard", get(controladores::dashboard))
        .route(
            "/dashboard/hacer-vip",
            post(controladores::hacer_vip_cliente),
        )
        .route("/agenda", get(controladores::agenda))
        .route("/agenda/dia", get(controladores::agenda_dia))
        .route("/panel-empleado", get(controladores::agenda))
        .route(
            "/empleado/reserva/{id}/estado",
            post(controladores::cambiar_estado_reserva),
        )
        .route(
            "/admin/cliente/{id}/toggle-vip",
            post(controladores::toggle_vip_cliente),
        )
        .route(
            "/admin/cliente/{id}/reset-strikes",
            post(controladores::reset_strikes_cliente),
        )
        .route("/api/disponibilidad", get(controladores::disponibilidad))
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
