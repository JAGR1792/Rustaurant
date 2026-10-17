# ADR 0010: Despliegue de la demo en Render (Docker)

- Estado: aceptado
- Fecha: 2026-10-17

## Contexto

Hace falta una URL pública para mostrar el sistema sin depender de un PC
local encendido (el túnel efímero queda descartado). El estado es en
memoria, las plantillas Askama se embeben al compilar y `static/` se
sirve desde disco; los secretos van en `.env`, que no se commitea.

## Decisión

- `Dockerfile` multietapa: compila en `rust:1.99-slim` (con `gcc` para
  `ring`) y publica en `debian:trixie-slim` solo el binario y `static/`.
  El truco de caché (compilar un main vacío y luego copiar `src/`) va
  seguido de `touch` sobre los `.rs`: `COPY` conserva los mtime del
  contexto y sin eso Cargo reutiliza el binario vacío.
- `main.rs` lee `HOST` y `PORT` del entorno (local por defecto sigue
  siendo `127.0.0.1:3000`); dentro del contenedor se usa `HOST=0.0.0.0`
  y el `PORT` que inyecta Render, con health check en `/`.
- `render.yaml` describe el servicio (runtime Docker, plan free,
  `autoDeploy` y health check). `SMTP_USUARIO` viaja como variable
  normal; `SMTP_CLAVE` se define una sola vez en el panel como secreto.
- `.dockerignore` recorta `target/`, `.git/`, documentación, diagramas y
  el `.env` para que la imagen no cargue peso ni credenciales.

## Consecuencias

- Cada push a la rama conectada redepliega sola; el plan free se
  suspende sin tráfico y reinicia el seed al despertar (demo estable).
- El mismo Dockerfile sirve para cualquier host con contenedores.
- Sin `SMTP_CLAVE` en el panel, la demo sigue con el buzón interno.
