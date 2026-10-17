//! Servicio de reservas de la fase 1: sujeto real del Proxy (Figura 9).
//! `ReservaServiceMemoria` concentra la lógica de negocio sobre el almacén
//! de `estado.rs`: disponibilidad, costo con Decorator y comprobantes RN14.
//! Los controladores solo conocen el trait `ReservaService`; al migrar a
//! sqlx se reemplaza esta struct sin tocar la capa Control.

use chrono::{Duration, Utc};

use crate::decorator::{
    BarraCoctelesDecorator, ComponenteReserva, DecoracionDecorator, ReservaBase,
};
use crate::dominio::{ComprobantePago, EstadoReserva, Reserva, UsuarioSesion};
use crate::estado::{self, Datos, Estado};
use crate::proxy::{AccesoError, ReservaService};

pub struct ReservaServiceMemoria {
    pub estado: Estado,
}

impl ReservaServiceMemoria {
    pub fn new(estado: Estado) -> Self {
        Self { estado }
    }
}

/// Costo autoritativo con Decorator (Figura 10): pedido con precios
/// congelados (RN07) + cada servicio extra (decoración fija, barra por
/// persona). Reproduce el ejemplo CP09: 180.000 + 80.000 + 4 x 45.000.
fn costo_reserva(d: &Datos, r: &Reserva) -> i64 {
    let pedido_total: i64 = r
        .pedido
        .iter()
        .map(|p| p.precio_unitario_congelado * p.cantidad as i64)
        .sum();
    let mut componente: Box<dyn ComponenteReserva> = Box::new(ReservaBase {
        pedido_total,
        num_comensales: r.num_comensales,
    });
    for extra in &r.servicios_extra {
        if let Some(plan) = d.planes.iter().find(|p| p.id == extra.plan_id) {
            componente = if plan.por_persona {
                Box::new(BarraCoctelesDecorator {
                    envuelto: componente,
                    tarifa: extra.precio_congelado,
                    por_persona: true,
                    num_comensales: r.num_comensales,
                })
            } else {
                Box::new(DecoracionDecorator {
                    envuelto: componente,
                    tarifa: extra.precio_congelado,
                    motivo: plan.nombre.clone(),
                })
            };
        }
    }
    componente.costo()
}

/// Cuánto lleva pagado la reserva según su modalidad (0 si paga en sitio).
fn pagado(r: &Reserva) -> i64 {
    r.pago.monto_total * r.pago.porcentaje as i64 / 100
}

fn siguiente_id(ids: impl Iterator<Item = i64>) -> i64 {
    ids.max().unwrap_or(0) + 1
}

impl ReservaService for ReservaServiceMemoria {
    async fn confirmar(
        &self,
        sesion: &UsuarioSesion,
        mut reserva: Reserva,
    ) -> Result<i64, AccesoError> {
        // El lock se libera antes del await de la entrega: el guard no es
        // `Send` y la red no debe retener el estado (ADR 0009).
        let (id, mensajes) = {
            let mut d = self.estado.lock().unwrap();
            // RN01: ninguna mesa puede cruzarse con reserva vigente o bloqueo.
            let fin = reserva.inicio + Duration::minutes(reserva.duracion_minutos as i64);
            for mesa in &reserva.mesa_ids {
                if estado::estado_de_mesa(&d, *mesa, reserva.inicio, fin) != "libre" {
                    return Err(AccesoError::Negocio(format!(
                        "la mesa {mesa} acaba de ocuparse; elige otra"
                    )));
                }
            }
            // RN07 + Figura 10: el servidor recalcula el total, nunca el cliente.
            reserva.pago.monto_total = costo_reserva(&d, &reserva);
            reserva.id = siguiente_id(d.reservas.iter().map(|r| r.id));
            // RN14: comprobante del abono si hubo pago anticipado.
            let abonado = pagado(&reserva);
            if abonado > 0 {
                let id_comprobante = siguiente_id(d.comprobantes.iter().map(|c| c.id));
                d.comprobantes.push(ComprobantePago {
                    id: id_comprobante,
                    reserva_id: reserva.id,
                    id_empleado: if sesion.es_personal() {
                        Some(sesion.id())
                    } else {
                        None
                    },
                    concepto: "ABONO".into(),
                    tipo_movimiento: "INGRESO".into(),
                    medio_pago: reserva.pago.medio.clone(),
                    monto_subtotal: reserva.pago.monto_total,
                    monto_retencion: 0,
                    monto_total: abonado,
                });
            }
            // Notificación por correo al cliente y a los administradores:
            // siempre al buzón de demostración (ADR 0008).
            let mensajes = crate::correo::notificar_reserva(&mut d, &reserva);
            let id = reserva.id;
            d.reservas.push(reserva);
            (id, mensajes)
        };
        // Entrega real por SMTP, si está configurado, ya sin el lock (ADR 0009).
        crate::correo::entregar(&self.estado, &mensajes).await;
        Ok(id)
    }

    async fn cancelar(&self, sesion: &UsuarioSesion, reserva_id: i64) -> Result<(), AccesoError> {
        let mut d = self.estado.lock().unwrap();
        let id_sesion = sesion.id();
        let mut reembolso: Option<ComprobantePago> = None;
        {
            let r = d
                .reservas
                .iter_mut()
                .find(|r| r.id == reserva_id)
                .ok_or_else(|| AccesoError::Negocio("la reserva no existe".into()))?;
            // CP05: un cliente solo cancela lo suyo; el personal, cualquiera.
            if let UsuarioSesion::Cliente(_) = sesion
                && r.id_cliente != id_sesion
            {
                return Err(AccesoError::Denegado("esa reserva no es tuya".into()));
            }
            r.cambiar_estado(EstadoReserva::Cancelada)
                .map_err(|e| AccesoError::Negocio(e.into()))?;
            // RN08/RN09: con 24 h o más, reembolso total; si no, retención 10%.
            let abonado = pagado(r);
            if abonado > 0 {
                let horas = (r.inicio - Utc::now()).num_hours();
                let retencion = if horas >= 24 { 0 } else { abonado / 10 };
                reembolso = Some(ComprobantePago {
                    id: 0, // se asigna justo abajo
                    reserva_id: r.id,
                    id_empleado: if sesion.es_personal() {
                        Some(id_sesion)
                    } else {
                        None
                    },
                    concepto: if retencion == 0 {
                        "REEMBOLSO_TOTAL"
                    } else {
                        "REEMBOLSO_PARCIAL"
                    }
                    .into(),
                    tipo_movimiento: "EGRESO".into(),
                    medio_pago: r.pago.medio.clone(),
                    monto_subtotal: abonado,
                    monto_retencion: retencion,
                    monto_total: abonado - retencion,
                });
            }
        }
        if let Some(mut c) = reembolso {
            c.id = siguiente_id(d.comprobantes.iter().map(|x| x.id));
            d.comprobantes.push(c);
        }
        Ok(())
    }

    async fn agenda(&self, _sesion: &UsuarioSesion) -> Result<Vec<Reserva>, AccesoError> {
        let d = self.estado.lock().unwrap();
        Ok(d.reservas.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dominio::{DetallePago, DetallePedido, ModalidadPago, Ocasion};
    use crate::estado::sembrar;
    use crate::proxy::ReservaServiceProxy;
    use chrono::DateTime;

    fn usuario(e: &Estado, id: i64) -> UsuarioSesion {
        e.lock()
            .unwrap()
            .usuarios
            .iter()
            .find(|u| u.id() == id)
            .cloned()
            .unwrap()
    }

    fn reserva_prueba(mesa: i64, inicio: DateTime<Utc>, id_cliente: i64) -> Reserva {
        Reserva {
            id: 0,
            codigo: format!("TEST{mesa:04}"),
            id_cliente,
            id_creador: None,
            tipo_restaurante_completo: false,
            mesa_ids: vec![mesa],
            inicio,
            duracion_minutos: 120,
            num_comensales: 2,
            ocasion: Ocasion::Casual,
            pedido: vec![],
            servicios_extra: vec![],
            restricciones: "Sin alergias".into(),
            observaciones: String::new(),
            pago: DetallePago {
                modalidad: ModalidadPago::Adelanto,
                porcentaje: 30,
                medio: "TARJETA".into(),
                monto_total: 0,
            },
            estado: EstadoReserva::Vigente,
        }
    }

    #[tokio::test]
    async fn confirmar_rechaza_cruce_con_reserva_existente() {
        let e = sembrar();
        let s = ReservaServiceMemoria::new(e.clone());
        // DEMO0001 ocupa la mesa 5 en +2 días: solapar 30 min debe fallar.
        let inicio_demo = e.lock().unwrap().reservas[0].inicio;
        let r = reserva_prueba(5, inicio_demo + Duration::minutes(30), 3);
        let err = s.confirmar(&usuario(&e, 3), r).await;
        assert!(matches!(err, Err(AccesoError::Negocio(_))));
    }

    #[tokio::test]
    async fn cancelar_dentro_de_24h_reembolsa_con_10_pct() {
        let e = sembrar();
        let s = ReservaServiceMemoria::new(e.clone());
        let mut r = reserva_prueba(1, Utc::now() + Duration::hours(2), 3);
        r.pedido.push(DetallePedido {
            item_id: 1,
            cantidad: 1,
            precio_unitario_congelado: 100_000,
        });
        r.pago.modalidad = ModalidadPago::Completo;
        r.pago.porcentaje = 100;
        let id = s.confirmar(&usuario(&e, 3), r).await.unwrap();
        s.cancelar(&usuario(&e, 3), id).await.unwrap();
        let d = e.lock().unwrap();
        let reembolso = d
            .comprobantes
            .iter()
            .rev()
            .find(|c| c.reserva_id == id && c.tipo_movimiento == "EGRESO")
            .unwrap();
        assert_eq!(reembolso.monto_retencion, 10_000);
        assert_eq!(reembolso.monto_total, 90_000);
        assert_eq!(reembolso.concepto, "REEMBOLSO_PARCIAL");
        assert_eq!(
            d.reservas.iter().find(|x| x.id == id).unwrap().estado,
            EstadoReserva::Cancelada
        );
    }

    #[tokio::test]
    async fn confirmar_notifica_al_cliente_y_al_admin() {
        let e = sembrar();
        let s = ReservaServiceMemoria::new(e.clone());
        let r = reserva_prueba(1, Utc::now() + Duration::days(5), 3);
        s.confirmar(&usuario(&e, 3), r).await.unwrap();
        let d = e.lock().unwrap();
        let al_cliente = d
            .correos
            .iter()
            .find(|m| m.para == "cliente@brindis.co")
            .expect("correo al cliente");
        assert!(al_cliente.asunto.contains("TEST0001"));
        assert!(al_cliente.cuerpo.contains("24 horas"));
        let al_admin = d
            .correos
            .iter()
            .find(|m| m.para == "brindis.rust@gmail.com")
            .expect("correo al admin");
        assert!(al_admin.asunto.contains("TEST0001"));
        assert!(al_admin.cuerpo.contains("M1"));
        assert!(
            !al_admin.entregado,
            "sin SMTP en pruebas no hay entrega real"
        );
    }

    #[tokio::test]
    async fn proxy_deniega_agenda_al_cliente_y_la_permite_al_personal() {
        let e = sembrar();
        let proxy = ReservaServiceProxy::new(ReservaServiceMemoria::new(e.clone()));
        let cliente = usuario(&e, 3);
        assert!(matches!(
            proxy.agenda(&cliente).await,
            Err(AccesoError::Denegado(_))
        ));
        let personal = usuario(&e, 2);
        assert!(proxy.agenda(&personal).await.is_ok());
    }
}
