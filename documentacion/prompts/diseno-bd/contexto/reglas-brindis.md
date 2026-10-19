# Contexto Brindis para el diseño de la BD (SQL Server)

Fuente: `documentacion/brindis_rust.tex` (Tablas 2, 5, 10 y sec. 6.3).

## 1. Las 13 tablas (3FN) y para qué sirve cada una

1. `usuario` — las 3 subclases en una sola tabla (`rol`: CLIENTE/EMPLEADO/ADMINISTRADOR).
   Cliente agrega tipo (VIP/GENERAL), strikes y última actividad; empleado, fecha de
   contratación. Únicos: `correo` y `(tipo_documento, documento)`.
2. `token_reactivacion` — reactivar clientes inactivos (RN13): token, expiración, usado.
3. `zona` — VIP, General, Terraza.
4. `mesa` — SOLO estado físico (`DISPONIBLE`/`FUERA_DE_SERVICIO`); `capacidad` 1..20;
   `numero_mesa`; `zona_id`. Lo de reservada/bloqueada es por franja (abajo).
5. `bloqueo_mesa` — bloqueo temporal RN03 (5 min): inicio, fin, expiración.
6. `horario_atencion` — día semana + apertura/cierre (RN05).
7. `reserva` — código único 8 alfanumérico (RN15); `id_cliente` (1 cliente→1..* reservas);
   `id_creador` NULABLE (0..1 empleado→1..* reservas; NULL = autogestión); flag de
   restaurante completo; inicio + duración 120 (RN01); comensales; ocasión; estado
   (VIGENTE/CLIENTE_EN_MESA/COMPLETADA/CANCELADA/INASISTIDA).
8. `reserva_mesa` — intermedia Reserva↔Mesa (1..* ↔ 0..*).
9. `item_menu` — carta: nombre, tipo (ENTRADA/PLATO_FUERTE/POSTRE/BEBIDA), precio vigente.
10. `reserva_detalle_pedido` — intermedia con `cantidad` + `precio_unitario` CONGELADO (RN07).
11. `plan_servicio_extra` — base de decoración/barra: nombre, tarifa, descripción,
    por_persona. Subtipos: decoración (motivo, nivel) y barra (tipo, cobra_por_persona).
12. `reserva_servicio_extra` — intermedia con `precio` CONGELADO (RN07).
13. `detalle_pago` + `comprobante_pago` — modalidad (ADELANTO/COMPLETO/EN_RESTAURANTE),
    porcentaje 0..100 (100/0 según modalidad), medio, montos y retención 10% (RN08/RN09);
    `id_empleado` NULABLE. Cada movimiento genera comprobante (RN14).

## 2. Reglas que la BD debe hacer cumplir (RN01–RN15, resumen operativo)

- RN01: 120 min; sin dos reservas activas (vigente/en-mesa) cruzadas en la misma mesa.
- RN02: suma(capacidades) ≥ comensales.
- RN03: bloqueo 5 min, se libera al confirmar/cancelar/vencer.
- RN04: restaurante completo exige cero reservas/bloqueos en la franja.
- RN05: dentro del horario; anticipación 1h–60 días.
- RN06: adelanto 30%, completo 100%, en-restaurante 0% solo VIP o personal.
- RN07: precios congelados al agregar; en modificación rige el vigente.
- RN08: general cancela ≥4h → 100%, si no → retiene 10%.
- RN09: VIP cancela ≥1h → 100%, si no → 10% + strike.
- RN10: 3er strike degrada VIP→general.
- RN11: VIP por admin o 10 completadas o $2.000.000 sin 3 strikes.
- RN12: quién crea a quién + genérica con cambio obligatorio.
- RN13: inactivo 180 días + token; admin no se auto-desactiva; siempre ≥1 admin.
- RN14/RN15: comprobante por movimiento; código único de 8.

## 3. Índices y unicidades mínimas

- UNIQUE: `usuario.correo`, `(usuario.tipo_documento, usuario.documento)`, `reserva.codigo`.
- Índices: `reserva_mesa(id_mesa)`, `reserva(fecha_hora_inicio)`, `bloqueo_mesa(mesa_id, inicio, fin)`.
- FK con `ON DELETE` explícito y documentado (pedido/comprobantes en cascada con la reserva).
