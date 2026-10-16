//! Factory Method en Rust (Figura 7).
//! `UsuarioFactory` abstracta -> trait con método plantilla `registrar()`
//! (default impl) + método fábrica `crear_usuario()` que cada subclase decide.

use crate::dao::{DaoError, UsuarioDao};
use crate::dominio::{Administrador, BaseUsuario, Cliente, Empleado, Rol, TipoCliente, UsuarioSesion};

#[derive(Debug, Clone)]
pub struct DatosRegistro {
    pub correo: String,
    pub password_plano: String,
    pub tipo_documento: String,
    pub documento: String,
    pub creado_por_personal: bool,
}

pub trait UsuarioFactory: Send + Sync {
    fn rol(&self) -> Rol;

    /// Método fábrica: decide qué struct concreto instanciar.
    fn crear_usuario(&self, base: BaseUsuario, datos: &DatosRegistro) -> UsuarioSesion;
}

/// Método plantilla (igual para todos los roles): valida, cifra, guarda.
/// `bcrypt::hash` sustituye a `BCrypt.hashpw` de Java (RNF02).
/// Función libre (no método del trait) para mantener `dyn` compatibilidad.
pub async fn registrar_con<D: UsuarioDao + Sync>(
    fabrica: &(dyn UsuarioFactory + Send + Sync),
    dao: &D,
    datos: DatosRegistro,
) -> Result<UsuarioSesion, DaoError> {
        if dao
            .existe_correo_o_documento(&datos.correo, &datos.tipo_documento, &datos.documento)
            .await?
        {
            return Err(DaoError::Duplicado("correo o documento ya existe".into()));
        }
        let hash = bcrypt::hash(&datos.password_plano, bcrypt::DEFAULT_COST)
            .map_err(|e| DaoError::Db(e.to_string()))?;
        let base = BaseUsuario {
            id: 0,
            correo: datos.correo.clone(),
            hash_pw: hash,
            tipo_documento: datos.tipo_documento.clone(),
            documento: datos.documento.clone(),
            nombre: String::new(),
            telefono: String::new(),
            activo: true,
            debe_cambiar_pw: datos.creado_por_personal, // RN12
        };
        let sesion = fabrica.crear_usuario(base, &datos);
        // dao.guardar(...) omitido en esqueleto; la impl real persiste aquí.
        Ok(sesion)
    }

// ---------------- fábricas concretas ----------------

pub struct ClienteFactory;
pub struct EmpleadoFactory;
pub struct AdministradorFactory;

impl UsuarioFactory for ClienteFactory {
    fn rol(&self) -> Rol {
        Rol::Cliente
    }
    fn crear_usuario(&self, base: BaseUsuario, _datos: &DatosRegistro) -> UsuarioSesion {
        UsuarioSesion::Cliente(Cliente {
            base,
            tipo: TipoCliente::General,
            strikes: 0,
            fecha_ultima_actividad: None,
        })
    }
}

impl UsuarioFactory for EmpleadoFactory {
    fn rol(&self) -> Rol {
        Rol::Empleado
    }
    fn crear_usuario(&self, base: BaseUsuario, _datos: &DatosRegistro) -> UsuarioSesion {
        UsuarioSesion::Empleado(Empleado { base, fecha_contratacion: None })
    }
}

impl UsuarioFactory for AdministradorFactory {
    fn rol(&self) -> Rol {
        Rol::Administrador
    }
    fn crear_usuario(&self, base: BaseUsuario, _datos: &DatosRegistro) -> UsuarioSesion {
        UsuarioSesion::Administrador(Administrador(Empleado { base, fecha_contratacion: None }))
    }
}

/// `UsuarioService` elige la fábrica según el rol (HashMap<Rol, Box<dyn Factory>>).
/// Abierto/cerrado (RNF05): nuevo rol "Cajero" = nueva struct + nueva fábrica.
pub struct UsuarioService {
    pub fabricas: std::collections::HashMap<String, Box<dyn UsuarioFactory + Send + Sync>>,
}

impl UsuarioService {
    pub fn fabrica_para(&self, rol: Rol) -> Option<&(dyn UsuarioFactory + Send + Sync)> {
        let k = format!("{:?}", rol);
        self.fabricas.get(&k).map(|b| b.as_ref())
    }
}
