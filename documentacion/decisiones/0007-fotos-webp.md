# ADR 0007: Fotos en WebP con respaldo JPEG

- Estado: aceptado
- Fecha: 2026-10-17

## Contexto

Las fotos reales de Commons (ADR 0005) pesaban ~2.5 MB en JPEG. En la
revisión de Jorge quedó pedido pasarlas a WebP con `<picture>`, que es
además la recomendación de formato de las pautas web revisadas.

## Decisión

- `static/img/fotos/convertir_webp.py` genera la variante `.webp`
  (calidad 80, método 6) de cada foto y del genérico
  `00-generico.webp`; `descargar_fotos.py` deja el par hecho al bajar
  fotos nuevas.
- La UI sirve `<picture>`: `<source type="image/webp">` primero y el
  JPEG/PNG original como respaldo (`vistas::foto_item` devuelve el par
  jpg/webp de cada plato).
- `picture { display: contents }` en `tema.css` para que el wrapper no
  rompa grid ni selectores existentes (la ficha de `plano.js` actualiza
  el `<source>` y el `<img>` a la vez).

## Consecuencias

- Peso: 2.5 MB -> 1.6 MB (38% menos) manteniendo el respaldo.
- Regenerar variantes: `python static/img/fotos/convertir_webp.py`
  (sin red; requiere Pillow).
