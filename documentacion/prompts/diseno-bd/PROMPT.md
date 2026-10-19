# PROMPT — Diseño de la BD Brindis en Microsoft SQL Server

Copia y pega esto en la IA, adjuntando TODA la carpeta `contexto/`:

---

Eres un diseñador de bases de datos SQL Server. Genera UN solo script T-SQL
(`brindis_db.sql`) para el sistema de reservas **Brindis**, siguiendo al pie
de la letra el estilo del archivo `contexto/ejemplo-estilo-ViaAndinaDB.sql`
(batches con `GO`, `IF DB_ID(...) IS NULL CREATE DATABASE`, tipos `VARCHAR`,
`BIT`, `DATETIME`, `IDENTITY`, etc.). NADA de sintaxis Postgres/MySQL.

Entradas (léelas todas antes de escribir):
- `contexto/Fig4_DiagramaClases.drawio` — clases, herencia y multiplicidades
  (`1`, `0..1`, `1..*`, `0..*`).
- `contexto/dominio.rs` — entidades y lógica ya implementada en Rust
  (strikes RN10, ciclo de vida, solape de bloqueos).
- `contexto/reglas-brindis.md` — las 14 tablas, RN01–RN15 e índices.

Requisitos del script:
1. `CREATE DATABASE brindis_db` + `USE`, en orden de dependencias.
2. Las 14 tablas con PK, FK (`ON DELETE` explícito), `UNIQUE`, `CHECK`
   (capacidad 1..20, porcentaje 0..100, modalidad, estados, rol, ocasión),
   `DEFAULT` (duración 120, strikes 0, activo 1) y los índices pedidos.
3. Intermedias con precio congelado: `reserva_mesa`, `reserva_detalle_pedido`
   (cantidad + precio_unitario), `reserva_servicio_extra` (precio).
4. `reserva.id_creador` y `comprobante_pago.id_empleado` NULABLES.
5. Seed mínimo coherente: 3 roles (1 admin, 1 empleado, 2 clientes uno VIP),
   3 zonas, 1 horario, 6 mesas, 2 planes (1 decoración + 1 barra), 3 ítems,
   1 reserva vigente de ejemplo con su pago y comprobante.
6. Al final, 5 consultas de verificación comentadas: cruce RN01, capacidad RN02,
   bloqueo vigente RN03, reembolso RN08/RN09 y degradación RN10.
7. Todo en español, comentado por bloques, sin `PRINT` ruidoso. Si algo de
   `reglas-brindis.md` contradice al `.drawio`, manda la regla escrita y avísalo.

Devuélveme SOLO el `.sql` completo en un bloque de código, más una lista de
3–5 decisiones que tomaste (tipo de `ON DELETE`, índices extra, etc.).
