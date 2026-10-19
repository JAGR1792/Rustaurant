# ADR 0009: Entrega real de correos por SMTP (Gmail)

- Estado: aceptado
- Fecha: 2026-10-17

## Contexto

El buzón de ADR 0008 permite demostrar los avisos, pero Jorge necesita que
el correo llegue de verdad para validar el flujo. La app corre en local,
sin servidor de correo propio, y el repositorio no puede contener secretos.

## Decisión

- `src/correo.rs` entrega los mensajes por SMTP con `lettre` (transporte
  async y rustls, sin OpenSSL) cuando el entorno define `SMTP_USUARIO` y
  `SMTP_CLAVE`; el `.env` local (ignorado por git) se carga con `dotenvy`.
  Opcionales: `SMTP_SERVIDOR` (por defecto `smtp.gmail.com`) y
  `SMTP_PUERTO` (587, STARTTLS).
- Gmail exige que el remitente coincida con la cuenta autenticada, así que
  `SMTP_USUARIO` es remitente y credencial a la vez. Con verificación en
  dos pasos se usa una "contraseña de aplicación".
- La redacción sigue en `notificar_reserva`; la entrega (`entregar`) corre
  fuera del lock del estado, con timeout de 10 s por envío. Un fallo SMTP
  no cancela la reserva: el aviso permanece en el buzón y se registra el
  error.
- Los destinatarios `@brindis.co` son cuentas demo sin buzón real: se
  omiten en SMTP y quedan solo en el panel hasta definir la autenticación
  de clientes con Google.
- `Mensaje` agrega `entregado`; la agenda muestra la etiqueta "enviado"
  cuando el SMTP confirmó la entrega y "buzón demo" cuando no.

## Consecuencias

- Sin credenciales la app se comporta como antes (buzón demo, sin red):
  la demo y los tests no dependen del correo.
- Los secretos viven solo en `.env`; `.env.example` documenta las
  variables sin valores.
- En producción se reemplaza la contraseña de aplicación por OAuth2
  (XOAUTH2) o un proveedor transaccional, sin tocar el servicio de
  reservas.
