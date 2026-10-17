# ADR 0001: Stack frontend Brindis

- Estado: aceptado
- Fecha: 2026-10-16

## Contexto

El documento debe decir lo mismo que el repo implementa. Se evaluó htmx
contra JS vanilla y librerías de componentes.

## Decisión

Plantillas Askama (`templates/`, bloques estilo Jinja), CSS vanilla
(`static/css/tema.css`) y JS vanilla (`static/js/plano.js`, `ui.js`).
Sin htmx ni librerías.

## Consecuencias

- La Vista no contiene lógica de negocio: `plano.js` solo pinta estados y
  llama `GET /api/disponibilidad`; el total y las validaciones viven en
  servicios (Builder/Decorator) y se verifican en el Proxy.
- Accesibilidad obligatoria: `aria-current`, `aria-live`, `reduced-motion`.
- Si se propone un framework, abrir issue primero (Fractal: sin sorpresas).
