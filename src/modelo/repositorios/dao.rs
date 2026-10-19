//! DAO genérico + interfaces (Figura 6). En Rust: traits async.
//! Impl concreta con `sqlx::MssqlPool` + queries preparadas (RNF02/RNF03).

use crate::dominio::{Mesa, Reserva, UsuarioSesion};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DaoError {
    #[error("no encontrado")]
    NoEncontrado,
    #[error("duplicado: {0}")]
    Duplicado(String),
    #[error("db: {0}")]
    Db(String),
}

// GenericDAO<T, ID> -> trait genérico. Equivale a la interfaz Java.
#[allow(async_fn_in_trait)]
pub trait GenericDao<T, Id> {
    async fn obtener(&self, id: Id) -> Result<Option<T>, DaoError>;
    async fn listar(&self) -> Result<Vec<T>, DaoError>;
    async fn guardar(&self, entidad: &T) -> Result<Id, DaoError>;
    async fn eliminar(&self, id: Id) -> Result<(), DaoError>;
}

// --- interfaces por agregado ---
#[allow(async_fn_in_trait)]
pub trait UsuarioDao: GenericDao<UsuarioSesion, i64> {
    async fn buscar_por_correo(&self, correo: &str) -> Result<Option<UsuarioSesion>, DaoError>;
    async fn existe_correo_o_documento(
        &self,
        correo: &str,
        tipo_doc: &str,
        doc: &str,
    ) -> Result<bool, DaoError>;
}

#[allow(async_fn_in_trait)]
pub trait MesaDao: GenericDao<Mesa, i64> {
    async fn mesas_en_servicio(&self) -> Result<Vec<Mesa>, DaoError>;
}

#[allow(async_fn_in_trait)]
pub trait ReservaDao: GenericDao<Reserva, i64> {
    async fn agenda_del_dia(&self, dia: chrono::NaiveDate) -> Result<Vec<Reserva>, DaoError>;
    async fn hay_cruce(
        &self,
        mesa_id: i64,
        inicio: chrono::DateTime<chrono::Utc>,
        fin: chrono::DateTime<chrono::Utc>,
    ) -> Result<bool, DaoError>;
}

// CatalogoDAO y PagoDAO agrupan carta/planes y comprobantes.
#[allow(async_fn_in_trait)]
pub trait CatalogoDao {
    async fn planes_para_ocasion(
        &self,
        ocasion: crate::dominio::Ocasion,
    ) -> Result<Vec<crate::dominio::PlanServicioExtra>, DaoError>;
}

#[allow(async_fn_in_trait)]
pub trait PagoDao {
    async fn emitir_comprobante(&self, reserva_id: i64, valor: i64) -> Result<i64, DaoError>;
}

/// Pool de conexiones (antes `ConexionBD` singleton Java).
/// En Rust no hace falta Singleton: se clona `DbPool` (Arc interno de sqlx).
#[derive(Debug, Clone)]
pub struct DbPool {
    pub url: String,
    // En la impl real: `pool: sqlx::MssqlPool`
}
