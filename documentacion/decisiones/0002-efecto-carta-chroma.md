# ADR 0002: Efecto de la carta (port vanilla del ChromaGrid)

- Estado: aceptado
- Fecha: 2026-10-16

## Contexto

Jorge pidió que la sección de platos de la portada se viera "más fina,
más especial" y propuso el componente ChromaGrid de React Bits
(reactbits.dev): tarjetas con una capa a color revelada por un spotlight
que sigue al cursor. El repo es Askama + JS vanilla y el ADR 0001 prohíbe
frameworks y copiar código ajeno.

## Decisión

Portar la mecánica del efecto a código propio vanilla: `static/js/chroma.js`
(spotlight con amortiguación en `requestAnimationFrame`, máscara radial por
CSS custom properties y auto-init de `[data-chroma-grid]`) y
`static/css/chroma.css` (imagen duplicada gris/color, gradiente de caída,
variantes `spotlight`/`card` y respeto de `prefers-reduced-motion`). El
diseño original se acredita a React Bits (MIT) en el encabezado de ambos
archivos. Las fotos de los platos son PNG deterministas generados por
`static/img/carta/generar_carta.py`.

## Consecuencias

- Sin dependencias nuevas ni build step: la página sigue sirviéndose desde
  `/static` y el efecto se degrada a la imagen gris si no hay JS.
- El mapeo plato → imagen/acento vive en la Vista (`vistas.rs::carta`),
  no en el dominio.
- Las tarjetas son enlaces reales (`/reservar`), con foco visible y textos
  accesibles; el hint de interacción no depende del color.
- Es un port del comportamiento, no del código: si React Bits cambia su
  licencia o implementación, el repo no se ve afectado.
