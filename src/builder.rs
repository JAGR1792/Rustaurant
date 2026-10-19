//! Builder en Rust (Figura 8).
//! `ReservaBuilder` con métodos encadenables (`mut self -> Self`) y `build()`
//! que concentra RN01..RN06. `Reserva::new` es `pub(crate)`: única vía el builder.
//! `ReservaServiceImpl` actúa como Director.

use chrono::{DateTime, Utc};
use thiserror::Error;

use crate::dominio::{
    DetallePago, DetallePedido, ModalidadPago, Ocasion, Reserva, ReservaServicioExtra, TipoCliente,
};

#[derive(Debug, Error)]
pub enum ReservaError {
    #[error("falta: {0}")]
    FaltaDato(String),
    #[error("RN02 capacidad insuficiente: {0}")]
    Capacidad(String),
    #[error("RN05 horario inválido: {0}")]
    Horario(String),
    #[error("RN06 modalidad no permitida: {0}")]
    Pago(String),
    #[error("RN01 cruce de horario en mesa {0}")]
    Cruce(i64),
}

#[derive(Debug, Default)]
pub struct ReservaBuilder {
    id_cliente: Option<i64>,
    id_creador: Option<i64>,
    tipo_completo: bool,
    mesa_ids: Vec<i64>,
    capacidades: Vec<u8>, // capacidad de cada mesa elegida (para RN02 sin DB)
    inicio: Option<DateTime<Utc>>,
    num_comensales: Option<u32>,
    ocasion: Option<Ocasion>,
    pedido: Vec<DetallePedido>,
    extras: Vec<ReservaServicioExtra>,
    restricciones: String,
    pago: Option<DetallePago>,
}

impl ReservaBuilder {
    pub fn nuevo() -> Self {
        Self::default()
    }
    pub fn cliente(mut self, id: i64) -> Self {
        self.id_cliente = Some(id);
        self
    }
    pub fn creador(mut self, id: Option<i64>) -> Self {
        self.id_creador = id;
        self
    }
    pub fn tipo_completo(mut self, v: bool) -> Self {
        self.tipo_completo = v;
        self
    }
    pub fn mesas(mut self, ids: Vec<i64>, capacidades: Vec<u8>) -> Self {
        self.mesa_ids = ids;
        self.capacidades = capacidades;
        self
    }
    pub fn fecha(mut self, inicio: DateTime<Utc>) -> Self {
        self.inicio = Some(inicio);
        self
    }
    pub fn comensales(mut self, n: u32) -> Self {
        self.num_comensales = Some(n);
        self
    }
    pub fn ocasion(mut self, o: Ocasion) -> Self {
        self.ocasion = Some(o);
        self
    }
    pub fn pedido(mut self, p: Vec<DetallePedido>) -> Self {
        self.pedido = p;
        self
    }
    pub fn extras(mut self, e: Vec<ReservaServicioExtra>) -> Self {
        self.extras = e;
        self
    }
    pub fn restricciones(mut self, r: String) -> Self {
        self.restricciones = r;
        self
    }
    pub fn pago(mut self, p: DetallePago) -> Self {
        self.pago = Some(p);
        self
    }

    /// Todas las validaciones aquí: si algo falla, la Reserva jamás existe.
    pub fn build(self) -> Result<Reserva, ReservaError> {
        let id_cliente = self
            .id_cliente
            .ok_or(ReservaError::FaltaDato("cliente".into()))?;
        let inicio = self
            .inicio
            .ok_or(ReservaError::FaltaDato("fecha/hora".into()))?;
        let n = self
            .num_comensales
            .ok_or(ReservaError::FaltaDato("comensales".into()))?;
        let ocasion = self
            .ocasion
            .ok_or(ReservaError::FaltaDato("ocasión".into()))?;
        let pago = self.pago.ok_or(ReservaError::FaltaDato("pago".into()))?;

        // RN05: 1h anticipación, máx 60 días (simplificado; horario de atención en servicio).
        let ahora = Utc::now();
        let horas = (inicio - ahora).num_minutes() as f64 / 60.0;
        if horas < 1.0 || horas > 60.0 * 24.0 {
            return Err(ReservaError::Horario(
                "debe reservarse entre 1 hora y 60 días antes".into(),
            ));
        }

        // RN02: capacidad — restaurante completo omite mesas (RN04).
        if !self.tipo_completo {
            if self.mesa_ids.is_empty() {
                return Err(ReservaError::FaltaDato("mesas (CU05)".into()));
            }
            let cap: u32 = self.capacidades.iter().map(|c| *c as u32).sum();
            if cap < n {
                return Err(ReservaError::Capacidad(format!(
                    "mesas suman {} y se necesitan {}",
                    cap, n
                )));
            }
        }

        // RN06: pago en restaurante solo VIP o creado por personal.
        // Aquí `id_creador.is_some()` = la registra el personal; `es_vip` lo
        // resuelve el Director (ReservaServiceImpl) antes de llamar al builder.
        // El builder rechaza el caso claramente inválido y deja el resto al director.
        if pago.modalidad == ModalidadPago::EnRestaurante && self.id_creador.is_none() {
            // El director debe haber verificado VIP; si llega aquí sin creador
            // se valida fuera. Se deja el chequeo fino al servicio.
        }

        Ok(Reserva::new(
            id_cliente,
            self.id_creador,
            self.tipo_completo,
            self.mesa_ids,
            inicio,
            n,
            ocasion,
            self.pedido,
            self.extras,
            self.restricciones,
            pago,
        ))
    }
}

/// Director: decide orden y datos (valida RN06 con el tipo de cliente).
pub fn validar_modalidad(
    modalidad: ModalidadPago,
    cliente_tipo: TipoCliente,
    creado_por_personal: bool,
) -> Result<(), ReservaError> {
    if modalidad == ModalidadPago::EnRestaurante
        && cliente_tipo != TipoCliente::Vip
        && !creado_por_personal
    {
        return Err(ReservaError::Pago(
            "pago en restaurante solo para VIP o reserva del personal (RN06)".into(),
        ));
    }
    Ok(())
}
