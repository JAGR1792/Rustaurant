# ADR 0005: Fotos reales desde Wikimedia Commons

- Estado: aceptado
- Fecha: 2026-10-16

## Contexto

En la revisión de la UI, Jorge pidió "colocar fotos reales" (referencia:
repo Parcial-Tellez de CamiloDG07, que resolvió lo mismo con fotos de
Wikimedia Commons). Las ilustraciones generadas (`generar_carta.py`,
`generar_salon.py`, `generar_mesas.py`) eran deterministas pero planas; la
dirección "Le Jardin" gana con fotografía real. Restricción propia: nada
de imágenes sin licencia clara y el repo debe poder regenerarlas.

## Decisión

- Adoptar 11 fotos de Wikimedia Commons (6 platos de la carta, salón de
  portada y acceso, y una foto por zona para la ficha de mesa),
  descargadas por `static/img/fotos/descargar_fotos.py`:
  1. verifica la licencia por la API de Commons antes de descargar;
  2. recompresa con Pillow (calidad 82, archivos livianos);
  3. escribe `creditos.md` y `creditos.json` con autor, licencia y enlace.
- La idea (fotos con licencia verificada por la API de Commons) se vio en
  el repo Parcial-Tellez de CamiloDG07; el script se reescribió desde cero.
- Los PNG de `generar_*.py` quedan como respaldo offline; un id de plato
  nuevo cae al genérico (`vistas::foto_item`).

## Consecuencias

- Atribución obligatoria según licencia (CC BY y CC BY-SA la exigen;
  CC0/dominio público no). `creditos.md` es la fuente de verdad.
- El repo crece ~2.3 MB en `static/img/fotos/`.
- Regeneración: `python static/img/fotos/descargar_fotos.py` (necesita
  internet; el resto del entregable sigue funcionando offline).
