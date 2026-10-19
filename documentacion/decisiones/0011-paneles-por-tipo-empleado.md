# ADR 0011: Paneles por tipo de empleado, jornada y asignación de mesero

- Estado: aceptado
- Fecha: 2026-10-18

## Contexto

En la revisión de Jorge quedó pedido que el panel de empleado no sea el
mismo para todos: el gerente administra, el mesero atiende sus mesas y la
cocina prepara las restricciones. Antes `/panel-empleado` era un alias de
la agenda y no existía noción de tipo de empleado ni de jornada, así que
tampoco se podía asignar un mesero a una reserva con criterio de turno.

## Decisión

- `TipoEmpleado { Gerente, Mesero, Cocina }` en el dominio con clave
  estable (CSS/BD) y etiqueta legible; `Empleado.tipo` lo guarda
  (`EmpleadoFactory` crea meseros, `AdministradorFactory` gerentes).
- `Jornada { id, id_empleado, inicio, fin, zona }` con `cubre(fecha)`: la
  franja se compara en UTC y "en turno" significa que la jornada de hoy
  cubre el momento actual.
- `/panel-empleado` (personal; otro rol vuelve a `/login`): ficha del
  turno de hoy con rango y zona; el mesero ve sus mesas asignadas
  (`Reserva.id_mesero`) con las acciones de estado, que regresan al panel
  (`FormCambioEstado.volver` en lista blanca); todo el personal ve las
  restricciones por preparar de hoy y mañana; el gerente obtiene el atajo
  al panel de administración.
- `/panel-admin` (solo Administrador; si no, redirige a
  `/panel-empleado`): KPIs globales (reservas activas, comensales,
  meseros en turno y total abonado de comprobantes INGRESO), tabla de
  personal con su tipo y jornada del día y asignación de mesero por
  reserva (`POST /admin/reserva/{id}/mesero`). El selector solo ofrece
  meseros cuya jornada cubre la franja de la cita y la operación delega
  en `Datos::asignar_mesero`, que valida de nuevo tipo MESERO y cobertura
  (valor vacío desasigna); doble defensa UI + servidor.
- `Reserva.id_mesero: Option<i64>` es asignación de servicio, distinta de
  `id_creador`; el seed deja DEMO0001 asignada al mesero demo.
- `db/brindis_db.sql` refleja el modelo: `usuario.tipo_empleado` con
  `CHECK` de valores y coherencia con el rol, tabla `jornada` (FK a
  usuario, `CHECK` de rango y zona) y `reserva.id_mesero` con FK e
  índice, más las semillas de personal y turnos.

## Consecuencias

- La agenda operativa completa sigue en `/agenda`; el enlace "Panel
  Empleado" del menú apunta al panel por tipo.
- La asignación inválida se rechaza en un único punto (estado.rs) y la
  UI no ofrece opciones imposibles; el aviso de error se muestra tanto
  en el panel de admin como en el de empleado.
- El seed suma `cocina@brindis.co` / `Cocina123!` (tipo Cocina) y
  jornadas demo; sin SQL Server la demo funciona igual porque el estado
  vive en memoria y el script queda listo para la entrega.
