//! Brindis — Sistema de reservas por ocasión (Axum + MVC por capas en Rust).
//! Vista: Askama + plano.js | Controlador: handlers de `vistas.rs` |
//! Modelo: dominio + patrones (factory/builder/proxy/decorator) + DAO/estado.

pub mod builder;
pub mod dao;
pub mod decorator;
pub mod dominio;
pub mod estado;
pub mod factory;
pub mod proxy;
pub mod servicios;
pub mod vistas;

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

    let estado = estado::sembrar();

    let app = Router::new()
        .route("/", get(vistas::inicio))
        .route("/login", get(vistas::login_get).post(vistas::login_post))
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
        .route("/agenda", get(vistas::agenda))
        .route("/api/disponibilidad", get(vistas::disponibilidad))
        .nest_service("/static", ServeDir::new("static"))
        .layer(TraceLayer::new_for_http())
        .with_state(estado);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    tracing::info!("Brindis en http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}
