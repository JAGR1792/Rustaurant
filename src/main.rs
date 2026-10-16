//! Brindis — Sistema de reservas por ocasión (Axum + MVC por capas en Rust).
//! Vista: Askama/Tera + plano.js | Controlador: handlers Axum | Modelo: servicios+patrones+DAO+entidades.

pub mod builder;
pub mod dao;
pub mod decorator;
pub mod dominio;
pub mod factory;
pub mod proxy;

use axum::{routing::get, Router};

async fn salud() -> &'static str {
    "Brindis Rustaurant OK"
}

#[tokio::main]
async fn main() {
    let app: Router = Router::new()
        .route("/", get(salud))
        // Fase 1: AuthController, ReservaController, PersonalController (handlers)
        // .route("/login", post(auth::login))
        // .route("/reservas", post(reserva::confirmar))
        ;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("Brindis en http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}
