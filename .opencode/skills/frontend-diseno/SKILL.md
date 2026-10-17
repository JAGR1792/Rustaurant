---
name: frontend-diseno
description: Use when designing Brindis frontend work: Askama templates, CSS tokens, reservation wizard, plano de mesas, toasts and modal. Triggers on frontend, UX, wizard, plano, templates, estilos.
---

# Frontend Diseno (Brindis)

Skill de proyecto: diseno frontend original de Brindis. Todo lo que se genere
aquí es código nuevo del repo; mirar otros repos solo como idea de estructura,
nunca copiar (ver regla de originalidad abajo).

## Mapa de archivos

- `templates/base.html` — layout con skip-link, nav por rol, `#toasts` y `dialog#modal`.
- `templates/login.html`, `templates/registro.html` — auth.
- `templates/reservar.html` — asistente de 4 pasos + `#plano`.
- `templates/mis_reservas.html`, `templates/agenda.html` — cliente y personal.
- `static/css/tema.css` — tokens y estados. `static/js/plano.js`, `static/js/ui.js`.

## Tokens (no cambiar sin actualizar `tema.css` y el doc 6.8)

- Marca vino `--vino: #6d1a2d`, dorado `--dorado: #c9a227`.
- Mesa: libre `--libre` verde, reservada rojo, bloqueada gris, seleccionada dorado.
- `data-estado` en cada mesa: `libre|reservada|bloqueada|seleccionada`.
- Responsive a 700px; `prefers-reduced-motion: reduce` desactiva todo.

## Reglas

1. Askama: solo bloques estilo Jinja (`extends`, `block titulo/contenido/scripts`).
   Sin htmx ni librerías; JS vanilla con `fetch` y `defer`.
2. Contrato API: `GET /api/disponibilidad?fecha=&hora=&zona=` devuelve
   `[{id, zona, capacidad, estado}]`. Reservadas y bloqueadas van `disabled`
   con `aria-label` que diga el estado.
3. Accesibilidad: `aria-current="step"` en el paso activo, `aria-live` en
   avisos de bloqueo y toasts, foco visible en skip-link.
4. Originalidad: reescribir desde cero; si algo se parece a otro repo,
   dejar comentario `Idea vista en X, reescrita` y no pegar código ajeno.
5. Tras tocar vistas, actualizar `documentacion/brindis_rust.tex` 6.8 y
   verificar `tectonic documentacion/brindis_rust.tex`.
