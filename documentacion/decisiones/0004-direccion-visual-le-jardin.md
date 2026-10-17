# ADR 0004: Dirección visual "Le Jardin" (rediseño UX/UI)

- Estado: aceptado
- Fecha: 2026-10-16

## Contexto

El frontend de la fase 1 era funcional pero visualmente pobre (tema plano de
91 líneas, tipografía del sistema, colores saturados). Jorge pidió un
rediseño UX/UI completo con una referencia de restaurante parisino
contemporáneo ("Le Jardin"): composición editorial, fotografía, jerarquía
tipográfica y espacios en blanco, sin clichés franceses y sin introducir
frameworks (la decisión vinculante del ADR 0001 sigue vigente). Se optó por
el camino de portar la dirección visual al stack actual (Askama + JS
vanilla) en vez de migrar a React.

## Decisión

Adoptar la dirección "Le Jardin" como evolución de la identidad de Brindis:

- Paleta: marfil cálido `#f5efe5` (fondo), borgoña `#702c3b` (marca —
  reemplaza al vino `#6d1a2d`, casi idéntico), carbón `#26231f` (texto),
  dorado apagado `#a88a59` (acentos). Los estados de mesa conservan su
  semántica (verde libre, terracota reservada, piedra bloqueada, dorado
  seleccionada) con tonos refinados.
- Tipografía: Cormorant Garamond (serif editorial, títulos) + Inter
  (interfaz), self-hosted como woff2 en `static/fuentes/` con CSS generado
  por `descargar_fuentes.py` (Google Fonts, licencia OFL). Sin llamadas a
  CDNs externos: el entregable funciona sin internet.
- Los tokens viven en `static/css/tema.css` (se conservan los nombres
  `--vino`, `--dorado`, `--fondo`, `--texto` para no romper
  `chroma.css`/`counter.css`).
- Composición: cabecera clara con filete inferior, tipografía serif en
  títulos, rotulos `.eyebrow` en versalitas, tablas editoriales con
  hairlines, botones borgoña con microinteracciones y foco visible.

## Consecuencias

- Ninguna dependencia ni build step nuevo: solo CSS, fuentes estáticas y
  los scripts vanilla existentes.
- Las secciones se rediseñan por fases (portada, acceso, reserva con plano
  ilustrado y ficha de mesa, mis reservas, agenda) manteniendo el contrato
  del API y las plantillas Askama.
- `chroma.css` y `counter.css` heredan la paleta vía variables; se ajustan
  en la fase de pulido.
- La sección 6.8 del documento (`brindis_rust.tex`) y el README se
  actualizan en la misma tarea (CACM).
