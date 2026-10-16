# AGENTS.md — Rustaurant

## Prioridad de reglas

1. Este archivo manda en este repo.
2. Las reglas globales del agente solo complementan.

## Ramas

- NUNCA commit directo a `main` ni a `dev`.
- Una rama por tarea: `agent/<nombre>/<tema>`
  (ej: `agent/alya/buenas-practicas`).
- `dev` integra, `main` solo entregas.

## Commits atómicos

- Un commit = un cambio lógico. Nada de `wip`.
- Convencional: `tipo(scope): descripción`
  (`docs`, `feat`, `fix`, `test`, `chore`).

## Docs sincronizadas (CACM)

- Al tocar código, actualizar en la misma tarea:
  `documentacion/brindis_rust.tex` si cambia diseño,
  docstrings de lo tocado y este archivo si cambia el flujo.
- No dejar "se actualiza después".

## Guardrails

1. NO commitear secretos (`.env`, keys) ni `target/`, PDFs generados ni `temp/`.
2. NO `print!` como logging en producción.
3. Sí validar que compila (`cargo check`) y los tests pasan (`cargo test`).

## Verificación antes de cerrar

1. `cargo test` en verde.
2. `tectonic documentacion/brindis_rust.tex` compila.
3. `python diagramas/brindis/fig_brindis.py` valida 0 incidencias.
