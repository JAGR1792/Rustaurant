//! Estado en memoria del servidor (demo sin SQL Server) + seed.
//! La capa de datos real son los traits de `dao.rs`; aquí hay un almacén
//! demostrativo para ejecutar la app y verificar la funcionalidad.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use chrono::{Duration, Utc};

use crate::dao::{DaoError, GenericDao, UsuarioDao};
use crate::dominio::{
    Administrador, BaseUsuario, BloqueoMesa, Cliente, ComprobantePago, DetallePago, Empleado,
    EstadoReserva, ItemMenu, Mesa, ModalidadPago, Ocasion, PlanServicioExtra, Reserva, TipoCliente,
    UsuarioSesion,
};

#[derive(Default)]
pub struct Datos {
    pub usuarios: Vec<UsuarioSesion>,
    pub mesas: Vec<Mesa>,
    pub reservas: Vec<Reserva>,
    pub bloqueos: Vec<BloqueoMesa>,
    pub items: Vec<ItemMenu>,
    pub planes: Vec<PlanServicioExtra>,
    pub comprobantes: Vec<ComprobantePago>,
    /// token de sesión -> id de usuario
    pub sesiones: HashMap<String, i64>,
}

pub type Estado = Arc<Mutex<Datos>>;

pub fn nuevo() -> Estado {
    let d = Datos::default();
    Arc::new(Mutex::new(d))
}

pub fn sembrar() -> Estado {
    let e = nuevo();
    {
        let mut d = e.lock().unwrap();
        // Usuarios demo: hash de bcrypt de las contraseñas de prueba.
        let admin = BaseUsuario {
            id: 1,
            correo: "admin@brindis.co".into(),
            hash_pw: bcrypt::hash("Admin123!", bcrypt::DEFAULT_COST).unwrap(),
            tipo_documento: "CC".into(),
            documento: "1".into(),
            nombre: "Admin".into(),
            telefono: "3000000001".into(),
            activo: true,
            debe_cambiar_pw: false,
        };
        let empleado = BaseUsuario {
            id: 2,
            correo: "empleado@brindis.co".into(),
            hash_pw: bcrypt::hash("Empleado123!", bcrypt::DEFAULT_COST).unwrap(),
            tipo_documento: "CC".into(),
            documento: "2".into(),
            nombre: "Empleado".into(),
            telefono: "3000000002".into(),
            activo: true,
            debe_cambiar_pw: false,
        };
        let cliente = BaseUsuario {
            id: 3,
            correo: "cliente@brindis.co".into(),
            hash_pw: bcrypt::hash("Cliente123!", bcrypt::DEFAULT_COST).unwrap(),
            tipo_documento: "CC".into(),
            documento: "3".into(),
            nombre: "Cliente".into(),
            telefono: "3000000003".into(),
            activo: true,
            debe_cambiar_pw: false,
        };
        let vip = BaseUsuario {
            id: 4,
            correo: "vip@brindis.co".into(),
            hash_pw: bcrypt::hash("Vip123!", bcrypt::DEFAULT_COST).unwrap(),
            tipo_documento: "CC".into(),
            documento: "4".into(),
            nombre: "VIP".into(),
            telefono: "3000000004".into(),
            activo: true,
            debe_cambiar_pw: false,
        };
        d.usuarios
            .push(UsuarioSesion::Administrador(Administrador(Empleado {
                base: admin,
                fecha_contratacion: None,
            })));
        d.usuarios.push(UsuarioSesion::Empleado(Empleado {
            base: empleado,
            fecha_contratacion: None,
        }));
        d.usuarios.push(UsuarioSesion::Cliente(Cliente {
            base: cliente,
            tipo: TipoCliente::General,
            strikes: 0,
            fecha_ultima_actividad: None,
        }));
        d.usuarios.push(UsuarioSesion::Cliente(Cliente {
            base: vip,
            tipo: TipoCliente::Vip,
            strikes: 0,
            fecha_ultima_actividad: None,
        }));

        // Mesas: 8 en 3 zonas.
        let zonas = [("VIP", 2), ("General", 4), ("Terraza", 2)];
        let mut id = 1;
        for (z, n) in zonas {
            for k in 0..n {
                d.mesas.push(Mesa {
                    id,
                    numero_mesa: id as u32,
                    zona_id: 0,
                    zona: z.into(),
                    capacidad: if k % 2 == 0 { 4 } else { 2 },
                    disponible_fisicamente: true,
                });
                id += 1;
            }
        }

        // Carta y planes. Las imagenes de `static/img/carta` siguen el mismo
        // id (ver `vistas::carta`); el generador esta en esa carpeta.
        d.items.push(ItemMenu {
            id: 1,
            nombre: "Ceviche de camarón".into(),
            tipo: "ENTRADA".into(),
            precio_vigente: 15_000,
        });
        d.items.push(ItemMenu {
            id: 2,
            nombre: "Picanha a la brasa".into(),
            tipo: "PLATO_FUERTE".into(),
            precio_vigente: 30_000,
        });
        d.items.push(ItemMenu {
            id: 3,
            nombre: "Tres leches de la casa".into(),
            tipo: "POSTRE".into(),
            precio_vigente: 12_000,
        });
        d.items.push(ItemMenu {
            id: 4,
            nombre: "Limonada de coco".into(),
            tipo: "BEBIDA".into(),
            precio_vigente: 8_000,
        });
        d.items.push(ItemMenu {
            id: 5,
            nombre: "Salmón en costra de hierbas".into(),
            tipo: "PLATO_FUERTE".into(),
            precio_vigente: 38_000,
        });
        d.items.push(ItemMenu {
            id: 6,
            nombre: "Cóctel de la casa".into(),
            tipo: "BEBIDA".into(),
            precio_vigente: 22_000,
        });
        d.planes.push(PlanServicioExtra {
            id: 1,
            nombre: "Decoración de ocasión".into(),
            tarifa: 80_000,
            descripcion: "Montaje 20 min antes".into(),
            por_persona: false,
        });
        d.planes.push(PlanServicioExtra {
            id: 2,
            nombre: "Barra de cócteles".into(),
            tarifa: 45_000,
            descripcion: "Por persona".into(),
            por_persona: true,
        });

        // Una reserva vigente de ejemplo para el cliente general.
        let inicio = Utc::now() + Duration::days(2);
        d.reservas.push(Reserva {
            id: 1,
            codigo: "DEMO0001".into(),
            id_cliente: 3,
            id_creador: None,
            tipo_restaurante_completo: false,
            mesa_ids: vec![5],
            inicio,
            duracion_minutos: 120,
            num_comensales: 4,
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
        });
    }
    e
}

/// Busca por correo exacto.
pub fn buscar_por_correo(d: &Datos, correo: &str) -> Option<UsuarioSesion> {
    d.usuarios
        .iter()
        .find(|u| u.correo().eq_ignore_ascii_case(correo))
        .cloned()
}

// ------------------------------------------------------------------ DAO en memoria

/// Implementación de `UsuarioDao` sobre el almacén demo: la Factory Method
/// (`registrar_con`) funciona igual que contra sqlx, solo que aquí sobre
/// `Datos`. En producción se reemplaza por el DAO sqlx sin tocar el control.
pub struct DaoMemoria {
    pub estado: Estado,
}

impl GenericDao<UsuarioSesion, i64> for DaoMemoria {
    async fn obtener(&self, id: i64) -> Result<Option<UsuarioSesion>, DaoError> {
        let d = self.estado.lock().unwrap();
        Ok(d.usuarios.iter().find(|u| u.id() == id).cloned())
    }

    async fn listar(&self) -> Result<Vec<UsuarioSesion>, DaoError> {
        let d = self.estado.lock().unwrap();
        Ok(d.usuarios.clone())
    }

    async fn guardar(&self, entidad: &UsuarioSesion) -> Result<i64, DaoError> {
        let mut d = self.estado.lock().unwrap();
        let id = d.usuarios.iter().map(|u| u.id()).max().unwrap_or(0) + 1;
        let mut copia = entidad.clone();
        copia.base_mut().id = id;
        d.usuarios.push(copia);
        Ok(id)
    }

    async fn eliminar(&self, id: i64) -> Result<(), DaoError> {
        let mut d = self.estado.lock().unwrap();
        d.usuarios.retain(|u| u.id() != id);
        Ok(())
    }
}

impl UsuarioDao for DaoMemoria {
    async fn buscar_por_correo(&self, correo: &str) -> Result<Option<UsuarioSesion>, DaoError> {
        let d = self.estado.lock().unwrap();
        Ok(buscar_por_correo(&d, correo))
    }

    async fn existe_correo_o_documento(
        &self,
        correo: &str,
        tipo_doc: &str,
        doc: &str,
    ) -> Result<bool, DaoError> {
        let d = self.estado.lock().unwrap();
        Ok(d.usuarios.iter().any(|u| {
            u.correo().eq_ignore_ascii_case(correo)
                || (!doc.is_empty()
                    && u.base().tipo_documento == tipo_doc
                    && u.base().documento == doc)
        }))
    }
}

/// Estado de una mesa en una franja: libre, reservada o bloqueada.
pub fn estado_de_mesa(
    d: &Datos,
    mesa_id: i64,
    inicio: chrono::DateTime<Utc>,
    fin: chrono::DateTime<Utc>,
) -> &'static str {
    if d.bloqueos
        .iter()
        .any(|b| b.mesa_id == mesa_id && b.se_solapa(inicio, fin) && !b.esta_expirado(Utc::now()))
    {
        return "bloqueada";
    }
    let cruzada = d.reservas.iter().any(|r| {
        r.mesa_ids.contains(&mesa_id)
            && matches!(
                r.estado,
                EstadoReserva::Vigente | EstadoReserva::ClienteEnMesa
            )
            && r.inicio < fin
            && inicio < r.inicio + Duration::minutes(r.duracion_minutos as i64)
    });
    if cruzada { "reservada" } else { "libre" }
}
