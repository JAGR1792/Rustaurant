//! Proxy de protección en Rust (Figura 9).
//! `IReservaService` (Java) -> `trait ReservaService` (Rust).
//! Los handlers Axum solo conocen el trait; en runtime reciben el Proxy.

use crate::dominio::{Reserva, UsuarioSesion};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AccesoError {
    #[error("denegado: {0}")]
    Denegado(String),
    #[error("negocio: {0}")]
    Negocio(String),
}

// Interfaz común (sujeto real + proxy).
#[allow(async_fn_in_trait)]
pub trait ReservaService: Send + Sync {
    async fn confirmar(&self, sesion: &UsuarioSesion, reserva: Reserva)
    -> Result<i64, AccesoError>;
    async fn cancelar(&self, sesion: &UsuarioSesion, reserva_id: i64) -> Result<(), AccesoError>;
    async fn agenda(&self, sesion: &UsuarioSesion) -> Result<Vec<Reserva>, AccesoError>;
}

// Sujeto real: lógica de negocio pura (usa Builder + Decorator + DAO en transacción).
pub struct ReservaServiceImpl;

impl ReservaService for ReservaServiceImpl {
    async fn confirmar(
        &self,
        _sesion: &UsuarioSesion,
        _reserva: Reserva,
    ) -> Result<i64, AccesoError> {
        // Aquí: builder.build() + decoradores + reserva_dao.guardar en transacción sqlx.
        Ok(1)
    }
    async fn cancelar(&self, _sesion: &UsuarioSesion, _reserva_id: i64) -> Result<(), AccesoError> {
        Ok(())
    }
    async fn agenda(&self, _sesion: &UsuarioSesion) -> Result<Vec<Reserva>, AccesoError> {
        Ok(vec![])
    }
}

// Proxy: verifica rol/propiedad antes de delegar (no mezcla con negocio).
pub struct ReservaServiceProxy<R: ReservaService> {
    pub real: R,
}

impl<R: ReservaService> ReservaServiceProxy<R> {
    pub fn new(real: R) -> Self {
        Self { real }
    }

    fn exigir_personal(&self, s: &UsuarioSesion) -> Result<(), AccesoError> {
        if s.es_personal() {
            Ok(())
        } else {
            Err(AccesoError::Denegado(
                "solo el personal ve la agenda (RF15)".into(),
            ))
        }
    }
}

impl<R: ReservaService + Send + Sync> ReservaService for ReservaServiceProxy<R> {
    async fn confirmar(
        &self,
        sesion: &UsuarioSesion,
        reserva: Reserva,
    ) -> Result<i64, AccesoError> {
        // Cliente solo a su nombre; personal puede crear a nombre de otro (RF12).
        if let UsuarioSesion::Cliente(c) = sesion
            && reserva.id_cliente != c.base.id
        {
            return Err(AccesoError::Denegado(
                "un cliente solo reserva a su nombre".into(),
            ));
        }
        self.real.confirmar(sesion, reserva).await
    }

    async fn cancelar(&self, sesion: &UsuarioSesion, reserva_id: i64) -> Result<(), AccesoError> {
        // La propiedad se verifica cargando la reserva; simplificado aquí.
        // Cliente ajeno -> denegado (CP05); personal -> permitido.
        match sesion {
            UsuarioSesion::Cliente(_) => self.real.cancelar(sesion, reserva_id).await,
            UsuarioSesion::Empleado(_) | UsuarioSesion::Administrador(_) => {
                self.real.cancelar(sesion, reserva_id).await
            }
        }
    }

    async fn agenda(&self, sesion: &UsuarioSesion) -> Result<Vec<Reserva>, AccesoError> {
        self.exigir_personal(sesion)?;
        self.real.agenda(sesion).await
    }
}

// Nota: `async fn` en traits es estable desde Rust 1.75; no se necesita async-trait.
