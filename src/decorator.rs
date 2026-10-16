//! Decorator en Rust (Figura 10).
//! `ComponenteReserva` (trait objeto) + `ReservaBase` + `ReservaDecorator`.
//! Ejemplo del doc: 180.000 + 80.000 + 4 x 45.000 = 440.000 (CP09).

pub trait ComponenteReserva {
    fn costo(&self) -> i64;
    fn descripcion(&self) -> String;
    fn indicaciones(&self) -> Vec<String>;
}

/// Reserva base: costo del pedido anticipado (suma cantidad x precio congelado).
pub struct ReservaBase {
    pub pedido_total: i64,
    pub num_comensales: u32,
}

impl ComponenteReserva for ReservaBase {
    fn costo(&self) -> i64 {
        self.pedido_total
    }
    fn descripcion(&self) -> String {
        format!("Pedido anticipado: {}", self.pedido_total)
    }
    fn indicaciones(&self) -> Vec<String> {
        vec![]
    }
}

/// Decorador abstracto: envuelve otro `Box<dyn ComponenteReserva>`.
pub struct DecoracionDecorator {
    pub envuelto: Box<dyn ComponenteReserva>,
    pub tarifa: i64,
    pub motivo: String,
}

impl ComponenteReserva for DecoracionDecorator {
    fn costo(&self) -> i64 {
        self.envuelto.costo() + self.tarifa
    }
    fn descripcion(&self) -> String {
        format!("{} + Decoración {} ({})", self.envuelto.descripcion(), self.motivo, self.tarifa)
    }
    fn indicaciones(&self) -> Vec<String> {
        let mut v = self.envuelto.indicaciones();
        v.push("Montar decoración 20 min antes de la llegada.".into());
        v
    }
}

pub struct BarraCoctelesDecorator {
    pub envuelto: Box<dyn ComponenteReserva>,
    pub tarifa: i64,
    pub por_persona: bool,
    pub num_comensales: u32,
}

impl ComponenteReserva for BarraCoctelesDecorator {
    fn costo(&self) -> i64 {
        let extra = if self.por_persona {
            self.tarifa * self.num_comensales as i64
        } else {
            self.tarifa
        };
        self.envuelto.costo() + extra
    }
    fn descripcion(&self) -> String {
        format!("{} + Barra ({})", self.envuelto.descripcion(), self.costo() - self.envuelto.costo())
    }
    fn indicaciones(&self) -> Vec<String> {
        let mut v = self.envuelto.indicaciones();
        v.push("Preparar barra y cristalería según plan.".into());
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cp09_total_440mil() {
        // CP09: pedido 180.000 + decoración 80.000 + barra 45.000 x 4 = 440.000
        let base = ReservaBase { pedido_total: 180_000, num_comensales: 4 };
        let con_deco = DecoracionDecorator {
            envuelto: Box::new(base),
            tarifa: 80_000,
            motivo: "Aniversario".into(),
        };
        let total = BarraCoctelesDecorator {
            envuelto: Box::new(con_deco),
            tarifa: 45_000,
            por_persona: true,
            num_comensales: 4,
        };
        assert_eq!(total.costo(), 440_000);
    }
}
