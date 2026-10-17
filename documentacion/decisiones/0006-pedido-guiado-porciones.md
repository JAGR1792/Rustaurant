# ADR 0006: Pedido guiado por porciones y revisión viva del paso 4

- Estado: aceptado
- Fecha: 2026-10-16

## Contexto

En la revisión de Jorge: escribir el número de porciones a mano en el paso
3 era mala UX; el pedido debe conversar con los comensales ("4 personas
pueden pedir lo mismo o cosas diferentes"). Además no había forma de
revisar la reserva antes de confirmar: el total aparecía suelto.

## Decisión

- Paso 3 con steppers de restar/sumar por plato (foto, nombre y precio),
  chip "Toda la mesa (N)" que pone N porciones de golpe, y medidor de
  cobertura "cubre X de N comensales" (`aria-live="polite"`).
- Extras con costo vivo ("× N = $ ..." cuando la tarifa es por persona).
- Paso 4 con revisión viva: fecha/hora/mesas/comensales, pedido
  desglosado por línea, extras y monto según la modalidad; el adelanto se
  calcula igual que el dominio (total × 30 / 100, entero).
- Los `<input type="number">` siguen siendo la fuente del POST (nombre
  `item_<id>`): sin JS el paso 3 se degrada a inputs numéricos y el
  servidor recalcula el valor final con precios congelados (RN07).

## Consecuencias

- Todo cálculo del navegador es estimación: la verdad está en
  `servicios.rs` (Decorator + `costo_reserva`).
- Componentes nuevos (`.plato`, `.paso-ctl`, `.cobertura`, `.resumen`) en
  `tema.css`; el resumen se reconstruye al entrar al paso 4 y al cambiar
  la modalidad de pago.
