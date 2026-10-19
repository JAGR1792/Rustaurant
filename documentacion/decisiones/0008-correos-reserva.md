# ADR 0008: Correos de reserva (cliente y administración)

- Estado: aceptado
- Fecha: 2026-10-17

## Contexto

En la revisión de Jorge quedó pedido avisar por correo al cliente y al
administrador cuando una reserva queda pedida, recordando el plazo de
cancelación antes de que se cobre. La app es una demo en memoria: no hay
SMTP ni credenciales (y no pueden commitearse secretos).

## Decisión

- `src/correo.rs` redacta el aviso al confirmar la reserva
  (`ReservaServiceMemoria::confirmar`): un correo al cliente con fecha,
  mesas, comensales, abono y la regla de cancelación vigente — general:
  sin retención hasta 4 h antes, después 10 % de lo abonado (RN08); VIP:
  sin retención hasta 1 h antes, después 10 % y strike (RN09); el texto
  se deriva de `dominio::politica_cancelacion` para no contradecir al
  servicio — y uno a cada usuario con rol Administrador con el detalle
  operativo.
- Los mensajes se depositan en `Datos::correos` (buzón en memoria) y
  `tracing` deja constancia; el personal los ve en la agenda
  ("Correos enviados", últimos 6).
- En producción se reemplaza el buzón por un canal SMTP real alimentado
  por variables de entorno, sin tocar el servicio de reservas: la
  redacción y la entrega están separadas.

## Consecuencias

- La demo muestra el doble envío (cliente + admin) sin credenciales.
- Los avisos se redactan con `formato` (fechas y montos en hora de
  Colombia y pesos), la misma fuente que las vistas.
- Los correos de una reserva cancelada o creada por el personal siguen
  la misma ruta; un SMTP real queda como trabajo futuro.
