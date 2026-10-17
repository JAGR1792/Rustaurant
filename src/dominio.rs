//! Entidades del dominio Brindis (Figura 4: Fig4_DiagramaClases.drawio).
//! Adaptación a Rust: `Usuario` abstracta -> `trait Usuario`;
//! herencia `Administrador : Empleado` -> composición + `Deref`
//! (`Administrador(pub Empleado)`); `instanceof Empleado` -> `matches!` / trait.
//! Nota del diagrama: los muchos-a-muchos (Reserva-Mesa, Reserva-ItemMenu,
//! Reserva-Plan) van directos por legibilidad; en la BD y la lógica usan
//! tablas intermedias (ReservaMesa, DetallePedido, ReservaServicioExtra).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ------------------------------------------------------------------ enums

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rol {
    Cliente,
    Empleado,
    Administrador,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TipoCliente {
    General,
    Vip,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EstadoReserva {
    Vigente,
    ClienteEnMesa,
    Completada,
    Cancelada,
    Inasistida,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Ocasion {
    Cumpleanos,
    Aniversario,
    Propuesta,
    Romantica,
    Negocios,
    Casual,
}

impl Ocasion {
    /// Clave estable para formularios HTML (`value` del radio).
    pub fn clave(&self) -> &'static str {
        match self {
            Ocasion::Cumpleanos => "cumpleanos",
            Ocasion::Aniversario => "aniversario",
            Ocasion::Propuesta => "propuesta",
            Ocasion::Romantica => "romantica",
            Ocasion::Negocios => "negocios",
            Ocasion::Casual => "casual",
        }
    }

    /// Nombre legible para la interfaz.
    pub fn nombre(&self) -> &'static str {
        match self {
            Ocasion::Cumpleanos => "Cumpleaños",
            Ocasion::Aniversario => "Aniversario",
            Ocasion::Propuesta => "Propuesta",
            Ocasion::Romantica => "Romántica",
            Ocasion::Negocios => "Negocios",
            Ocasion::Casual => "Casual",
        }
    }

    /// Reverso de `clave()` para los formularios.
    pub fn desde_clave(clave: &str) -> Option<Ocasion> {
        match clave {
            "cumpleanos" => Some(Ocasion::Cumpleanos),
            "aniversario" => Some(Ocasion::Aniversario),
            "propuesta" => Some(Ocasion::Propuesta),
            "romantica" => Some(Ocasion::Romantica),
            "negocios" => Some(Ocasion::Negocios),
            "casual" => Some(Ocasion::Casual),
            _ => None,
        }
    }

    /// Tabla 10 del documento: cada valor conoce su zona y decoración.
    pub fn zona_sugerida(&self) -> &'static str {
        match self {
            Ocasion::Cumpleanos => "General",
            Ocasion::Aniversario => "VIP",
            Ocasion::Propuesta => "VIP",
            Ocasion::Romantica => "Terraza",
            Ocasion::Negocios => "VIP",
            Ocasion::Casual => "Cualquiera",
        }
    }

    pub fn decoracion_ofrecida(&self) -> Option<&'static str> {
        match self {
            Ocasion::Cumpleanos => Some("Cumpleaños"),
            Ocasion::Aniversario => Some("Aniversario"),
            Ocasion::Propuesta => Some("Propuesta"),
            Ocasion::Romantica => Some("Romántica"),
            Ocasion::Negocios | Ocasion::Casual => None,
        }
    }

    pub fn indicacion_personal(&self) -> &'static str {
        match self {
            Ocasion::Cumpleanos => "Preparar la canción al servir el postre.",
            Ocasion::Aniversario => "Luz tenue y música suave.",
            Ocasion::Propuesta => "Coordinar con el cliente el momento de la propuesta.",
            Ocasion::Romantica => "Mesa apartada y servicio discreto.",
            Ocasion::Negocios => "Servicio ágil y sin interrupciones.",
            Ocasion::Casual => "Servicio estándar.",
        }
    }
}

impl EstadoReserva {
    /// Texto para las tablas de la interfaz.
    pub fn etiqueta(&self) -> &'static str {
        match self {
            EstadoReserva::Vigente => "Vigente",
            EstadoReserva::ClienteEnMesa => "Cliente en mesa",
            EstadoReserva::Completada => "Completada",
            EstadoReserva::Cancelada => "Cancelada",
            EstadoReserva::Inasistida => "Inasistida",
        }
    }

    /// Sufijo CSS del badge (clase `.badge.<sufijo>`).
    pub fn clase(&self) -> &'static str {
        match self {
            EstadoReserva::Vigente => "vigente",
            EstadoReserva::ClienteEnMesa => "en-mesa",
            EstadoReserva::Completada => "completada",
            EstadoReserva::Cancelada => "cancelada",
            EstadoReserva::Inasistida => "inasistida",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModalidadPago {
    Adelanto,
    Completo,
    EnRestaurante,
}

// ------------------------------------------------------------------ usuarios

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseUsuario {
    pub id: i64,
    pub correo: String,
    pub hash_pw: String,
    pub tipo_documento: String,
    pub documento: String,
    pub nombre: String,
    pub telefono: String,
    pub activo: bool,
    pub debe_cambiar_pw: bool,
    // rol como columna (CLIENTE/EMPLEADO/ADMINISTRADOR) en single-table;
    // en Rust el rol lo da el enum UsuarioSesion.
}

/// Trait que reemplaza a la clase abstracta `Usuario` (Java) en Rust.
pub trait Usuario {
    fn base(&self) -> &BaseUsuario;
    fn base_mut(&mut self) -> &mut BaseUsuario;
    fn rol(&self) -> Rol;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cliente {
    pub base: BaseUsuario,
    pub tipo: TipoCliente,
    pub strikes: u8,
    pub fecha_ultima_actividad: Option<DateTime<Utc>>, // RN13
}

impl Usuario for Cliente {
    fn base(&self) -> &BaseUsuario {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BaseUsuario {
        &mut self.base
    }
    fn rol(&self) -> Rol {
        Rol::Cliente
    }
}

impl Cliente {
    pub fn es_vip(&self) -> bool {
        self.tipo == TipoCliente::Vip
    }

    /// Diagrama: puedeReservar(). Activo y sin veto duro.
    pub fn puede_reservar(&self) -> bool {
        self.base.activo
    }

    /// Diagrama: incrementarStrike() + RN10 (al 3ro el VIP pasa a general).
    pub fn incrementar_strike(&mut self) {
        self.strikes = self.strikes.saturating_add(1);
        if self.strikes >= 3 {
            self.tipo = TipoCliente::General;
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenReactivacion {
    pub id: i64,
    pub token: String,
    pub expira: DateTime<Utc>,
    pub usado: bool,
    pub id_cliente: i64, // 1 cliente -> 0..1 token vigente
}

impl TokenReactivacion {
    /// Diagrama: validarToken().
    pub fn es_valido(&self, ahora: DateTime<Utc>) -> bool {
        !self.usado && ahora < self.expira
    }

    /// Diagrama: marcarComoUsado().
    pub fn marcar_usado(&mut self) {
        self.usado = true;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Empleado {
    pub base: BaseUsuario,
    pub fecha_contratacion: Option<chrono::NaiveDate>,
}

impl Usuario for Empleado {
    fn base(&self) -> &BaseUsuario {
        &self.base
    }
    fn base_mut(&mut self) -> &mut BaseUsuario {
        &mut self.base
    }
    fn rol(&self) -> Rol {
        Rol::Empleado
    }
}

/// `Administrador hereda de Empleado` (UML) -> en Rust: newtype + Deref.
/// Así `matches!(rol, Empleado|Administrador)` concede permisos del personal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Administrador(pub Empleado);

impl std::ops::Deref for Administrador {
    type Target = Empleado;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Usuario for Administrador {
    fn base(&self) -> &BaseUsuario {
        &self.0.base
    }
    fn base_mut(&mut self) -> &mut BaseUsuario {
        &mut self.0.base
    }
    fn rol(&self) -> Rol {
        Rol::Administrador
    }
}

/// Enum de sesión: lo que `UsuarioDAOImpl` reconstruye según la columna `rol`.
#[derive(Debug, Clone)]
pub enum UsuarioSesion {
    Cliente(Cliente),
    Empleado(Empleado),
    Administrador(Administrador),
}

impl UsuarioSesion {
    pub fn rol(&self) -> Rol {
        match self {
            UsuarioSesion::Cliente(_) => Rol::Cliente,
            UsuarioSesion::Empleado(_) => Rol::Empleado,
            UsuarioSesion::Administrador(_) => Rol::Administrador,
        }
    }

    pub fn correo(&self) -> &str {
        match self {
            UsuarioSesion::Cliente(c) => &c.base.correo,
            UsuarioSesion::Empleado(e) => &e.base.correo,
            UsuarioSesion::Administrador(a) => &a.base.correo,
        }
    }

    /// Equivalente a `instanceof Empleado` (incluye al administrador).
    pub fn es_personal(&self) -> bool {
        matches!(
            self,
            UsuarioSesion::Empleado(_) | UsuarioSesion::Administrador(_)
        )
    }

    pub fn base(&self) -> &BaseUsuario {
        match self {
            UsuarioSesion::Cliente(c) => c.base(),
            UsuarioSesion::Empleado(e) => e.base(),
            UsuarioSesion::Administrador(a) => a.base(),
        }
    }

    pub fn base_mut(&mut self) -> &mut BaseUsuario {
        match self {
            UsuarioSesion::Cliente(c) => c.base_mut(),
            UsuarioSesion::Empleado(e) => e.base_mut(),
            UsuarioSesion::Administrador(a) => a.base_mut(),
        }
    }

    pub fn id(&self) -> i64 {
        self.base().id
    }

    pub fn nombre(&self) -> &str {
        &self.base().nombre
    }

    /// Tipo de cliente (General si no es cliente).
    pub fn tipo_cliente(&self) -> TipoCliente {
        match self {
            UsuarioSesion::Cliente(c) => c.tipo,
            _ => TipoCliente::General,
        }
    }
}

// ------------------------------------------------------------------ mesas, zonas y carta

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Zona {
    pub id: i64,
    pub nombre: String, // VIP | General | Terraza
    pub descripcion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HorarioAtencion {
    pub id: i64,
    pub dia_semana: String,
    pub hora_apertura: String, // Time como String HH:MM por simplicidad
    pub hora_cierre: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mesa {
    pub id: i64,
    pub numero_mesa: u32,
    pub zona_id: i64,  // 1 Zona -> 0..* Mesas
    pub zona: String,  // desnormalizado para el plano
    pub capacidad: u8, // CHECK 1..20
    pub disponible_fisicamente: bool,
}

/// Bloqueo temporal RN03: 1 Mesa -> 0..* Bloqueos en distintas franjas.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BloqueoMesa {
    pub id: i64,
    pub mesa_id: i64,
    pub inicio: DateTime<Utc>,
    pub fin: DateTime<Utc>,
    pub expira: DateTime<Utc>, // +5 min
}

impl BloqueoMesa {
    /// Diagrama: estaExpirado().
    pub fn esta_expirado(&self, ahora: DateTime<Utc>) -> bool {
        ahora >= self.expira
    }

    /// Diagrama: seSolapaCon(). Cruce de franjas RN01.
    pub fn se_solapa(&self, inicio: DateTime<Utc>, fin: DateTime<Utc>) -> bool {
        self.inicio < fin && inicio < self.fin
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemMenu {
    pub id: i64,
    pub nombre: String,
    pub tipo: String, // ENTRADA, PLATO_FUERTE, POSTRE, BEBIDA
    pub precio_vigente: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanServicioExtra {
    pub id: i64,
    pub nombre: String,
    pub tarifa: i64, // precio_base
    pub descripcion: String,
    pub por_persona: bool,
}

/// PlanDecoracion hereda de PlanServicioExtra (motivo + nivel).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanDecoracion {
    pub base: PlanServicioExtra,
    pub motivo: String, // Cumpleaños, Aniversario, Propuesta, Romántica
    pub nivel: String,  // Básico, Medio, Elaborado
}

/// PlanBarraCocteles hereda de PlanServicioExtra.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanBarraCocteles {
    pub base: PlanServicioExtra,
    pub tipo_barra: String, // BASICO, CARTA_COMPLETA, PREMIUM_SHOW
    pub cobra_por_persona: bool,
}

// ------------------------------------------------------------------ reserva

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetallePedido {
    pub item_id: i64,
    pub cantidad: u32,
    pub precio_unitario_congelado: i64, // RN07
}

/// Tabla intermedia Reserva-Mesa (muchos a muchos con franja).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReservaMesa {
    pub reserva_id: i64,
    pub mesa_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReservaServicioExtra {
    pub plan_id: i64,
    pub precio_congelado: i64, // RN07
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetallePago {
    pub modalidad: ModalidadPago,
    pub porcentaje: u8, // 0..100; 100 si COMPLETO, 0 si EN_RESTAURANTE
    pub medio: String,  // TARJETA, TRANSFERENCIA, EFECTIVO
    pub monto_total: i64,
}

/// Comprobante RN14: 1 DetallePago -> 0..* Comprobantes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComprobantePago {
    pub id: i64,
    pub reserva_id: i64,
    pub id_empleado: Option<i64>, // None = autogestión del cliente
    pub concepto: String,         // ABONO, LIQUIDACION, REEMBOLSO_TOTAL, REEMBOLSO_PARCIAL
    pub tipo_movimiento: String,  // INGRESO / EGRESO
    pub medio_pago: String,
    pub monto_subtotal: i64,
    pub monto_retencion: i64, // 10% RN08/RN09 cuando aplica
    pub monto_total: i64,
}

/// `Reserva`: constructor `pub(crate)` — solo `ReservaBuilder` puede crearla.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reserva {
    pub id: i64,
    pub codigo: String,          // 8 alfanuméricos, único (RN15)
    pub id_cliente: i64,         // 1 Cliente -> 1..* Reservas
    pub id_creador: Option<i64>, // 0..1 Empleado crea 1..* Reservas (RN12)
    pub tipo_restaurante_completo: bool,
    pub mesa_ids: Vec<i64>, // vía ReservaMesa; 1..* Mesas <-> 0..* Reservas
    pub inicio: DateTime<Utc>,
    pub duracion_minutos: u32, // RN01: 120 por defecto
    pub num_comensales: u32,
    pub ocasion: Ocasion,
    pub pedido: Vec<DetallePedido>, // 1 Reserva -> 0..* detalles (RN07)
    pub servicios_extra: Vec<ReservaServicioExtra>, // 0..* <-> 0..*
    pub restricciones: String,      // alergias (RF09)
    pub observaciones: String,
    pub pago: DetallePago,
    pub estado: EstadoReserva,
}

impl Reserva {
    /// Solo el Builder la llama (11 piezas = toda la reserva CU04).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        id_cliente: i64,
        id_creador: Option<i64>,
        tipo_restaurante_completo: bool,
        mesa_ids: Vec<i64>,
        inicio: DateTime<Utc>,
        num_comensales: u32,
        ocasion: Ocasion,
        pedido: Vec<DetallePedido>,
        servicios_extra: Vec<ReservaServicioExtra>,
        restricciones: String,
        pago: DetallePago,
    ) -> Self {
        Self {
            id: 0,
            codigo: generar_codigo(),
            id_cliente,
            id_creador,
            tipo_restaurante_completo,
            mesa_ids,
            inicio,
            duracion_minutos: 120, // RN01
            num_comensales,
            ocasion,
            pedido,
            servicios_extra,
            restricciones,
            observaciones: String::new(),
            pago,
            estado: EstadoReserva::Vigente,
        }
    }

    /// Diagrama: cambiarEstado() con transiciones del ciclo de vida.
    pub fn cambiar_estado(&mut self, nuevo: EstadoReserva) -> Result<(), &'static str> {
        use EstadoReserva::*;
        let ok = matches!(
            (&self.estado, nuevo),
            (Vigente, ClienteEnMesa)
                | (Vigente, Cancelada)
                | (Vigente, Inasistida)
                | (ClienteEnMesa, Completada)
        );
        if ok {
            self.estado = nuevo;
            Ok(())
        } else {
            Err("transicion de estado no valida")
        }
    }
}

fn generar_codigo() -> String {
    use uuid::Uuid;
    Uuid::new_v4().simple().to_string()[..8].to_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn cliente_vip() -> Cliente {
        Cliente {
            base: BaseUsuario {
                id: 1,
                correo: "vip@brindis.co".into(),
                hash_pw: "hash".into(),
                tipo_documento: "CC".into(),
                documento: "123".into(),
                nombre: "VIP".into(),
                telefono: "300".into(),
                activo: true,
                debe_cambiar_pw: false,
            },
            tipo: TipoCliente::Vip,
            strikes: 2,
            fecha_ultima_actividad: None,
        }
    }

    #[test]
    fn rn10_tercer_strike_degrada_vip() {
        let mut c = cliente_vip();
        c.incrementar_strike();
        assert_eq!(c.strikes, 3);
        assert_eq!(c.tipo, TipoCliente::General);
    }

    #[test]
    fn ciclo_vida_valido_e_invalido() {
        let mut r = Reserva {
            id: 0,
            codigo: "ABCDEFGH".into(),
            id_cliente: 1,
            id_creador: None,
            tipo_restaurante_completo: false,
            mesa_ids: vec![1],
            inicio: Utc::now(),
            duracion_minutos: 120,
            num_comensales: 2,
            ocasion: Ocasion::Casual,
            pedido: vec![],
            servicios_extra: vec![],
            restricciones: String::new(),
            observaciones: String::new(),
            pago: DetallePago {
                modalidad: ModalidadPago::Adelanto,
                porcentaje: 30,
                medio: "TARJETA".into(),
                monto_total: 100_000,
            },
            estado: EstadoReserva::Vigente,
        };
        assert!(r.cambiar_estado(EstadoReserva::ClienteEnMesa).is_ok());
        assert!(r.cambiar_estado(EstadoReserva::Completada).is_ok());
        assert!(r.cambiar_estado(EstadoReserva::Cancelada).is_err());
    }

    #[test]
    fn bloqueo_detecta_solape() {
        let b = BloqueoMesa {
            id: 1,
            mesa_id: 1,
            inicio: Utc.with_ymd_and_hms(2026, 10, 20, 19, 0, 0).unwrap(),
            fin: Utc.with_ymd_and_hms(2026, 10, 20, 21, 0, 0).unwrap(),
            expira: Utc.with_ymd_and_hms(2026, 10, 20, 18, 5, 0).unwrap(),
        };
        let ini = Utc.with_ymd_and_hms(2026, 10, 20, 20, 0, 0).unwrap();
        let fin = Utc.with_ymd_and_hms(2026, 10, 20, 22, 0, 0).unwrap();
        assert!(b.se_solapa(ini, fin));
        let ini2 = Utc.with_ymd_and_hms(2026, 10, 20, 21, 0, 0).unwrap();
        let fin2 = Utc.with_ymd_and_hms(2026, 10, 20, 23, 0, 0).unwrap();
        assert!(!b.se_solapa(ini2, fin2));
    }
}
