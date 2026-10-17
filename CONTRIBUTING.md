# Guía de Contribución — Rustaurant (Brindis)

¡Gracias por tu interés en contribuir! Aquí está cómo organizamos el proyecto,
cómo escribir código que aceptemos y dónde está cada cosa.

## Índice

1. [Estructura del repositorio](#estructura-del-repositorio)
2. [Cómo empezar](#cómo-empezar)
3. [Estándares de código](#estándares-de-código)
4. [Documentación obligatoria](#documentación-obligatoria)
5. [Tests](#tests)
6. [Convención de commits y ramas](#convención-de-commits-y-ramas)
7. [Proceso de revisión](#proceso-de-revisión)

---

## Estructura del repositorio

```text
Rustaurant/
├── src/                    ← Backend Axum + dominio (era plan inicial y sigue)
│   ├── dominio.rs          ← Entidades Fig4 + RN10/ciclo/solape
│   ├── factory.rs          ← Factory Method Fig7
│   ├── builder.rs          ← Builder Fig8 (director: servicio real)
│   ├── proxy.rs            ← Proxy Fig9
│   ├── decorator.rs        ← Decorator Fig10 (+ test CP09)
│   ├── dao.rs              ← GenericDAO + ConexionBD Fig6
│   └── main.rs             ← Rutas Axum
├── templates/              ← Vistas Askama (base, login, registro, reservar)
├── static/css/tema.css     ← Tokens vino + estados de mesa
├── static/js/              ← plano.js (fetch disponibilidad), ui.js
├── diagramas/brindis/      ← Fig6-11 PNG/SVG + fig_brindis.py + .drawio Fig1/Fig4
├── diagramas/creacionales/ ← Patrones genéricos de referencia
├── documentacion/
│   ├── brindis_rust.tex    ← Documento (12 tablas, TikZ Fig6-11)
│   ├── decisiones/         ← ADRs (ej: 0001-stack-frontend)
│   ├── prompts/diseno-bd/  ← PROMPT + contexto para el SQL Server
│   └── entregables/        ← Zips y PDFs de entrega
└── .opencode/skills/frontend-diseno/ ← Skill de diseño Brindis
```

**Regla de oro:** el dominio (`dominio.rs`: entidades, RN10, ciclo de vida,
solape) NO puede depender de Axum ni Tokio. Debe compilar y testearse solo.

---

## Cómo empezar

```powershell
git clone https://github.com/JAGR1792/Rustaurant.git
git checkout -b agent/<tu-nombre>/<tu-funcionalidad>
cargo test
cargo run  # http://127.0.0.1:3000
```

Verificación de calidad — los mismos comandos que el CI, sin sorpresas:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
python diagramas/brindis/fig_brindis.py
tectonic documentacion/brindis_rust.tex
```

## Estándares de código

### Rust

| Herramienta | Comando | Qué verifica |
|-------------|---------|--------------|
| `rustfmt` | `cargo fmt --check` | Formato consistente |
| `clippy` | `cargo clippy -- -D warnings` | Buenas prácticas |
| `cargo test` | `cargo test` | Unitarios (van junto al módulo) |

**Reglas adicionales:**

1. **Español:** dominio y comentarios en español (`mesa_ids`, `cambiar_estado`).
2. **Errores:** `Result`/`Option`, nunca `unwrap()` en rutas de producción.
3. **Sin `panic!`:** en biblioteca, retornar error descriptivo.
4. **Tipos explícitos** en funciones públicas.

### Frontend (Askama + vanilla)

- Plantillas solo con bloques (`extends`, `block titulo/contenido/scripts`).
- Sin htmx ni librerías; JS con `fetch` y `defer` (ver ADR 0001).
- Mesa siempre con `data-estado` y `aria-label` con el estado.
- `prefers-reduced-motion` respetado en CSS.

## Documentación obligatoria

Todo cambio de lógica o diseño trae:

1. **Doc comment** o comentario que cite la regla (ej: `// RN10`).
2. **Entrada en `documentacion/`**: si es decisión, ADR en `decisiones/`;
   si toca diseño, el `.tex` en la misma tarea (nada de "después").
3. **Test** que demuestre el comportamiento (ej: degradación VIP, solape).

## Tests

Viven junto al módulo (`src/dominio.rs::tests`, `src/decorator.rs::tests`).
Hoy: CP09 440.000, RN10 tercer strike, ciclo válido/inválido y solape de
bloqueo. Todo cambio de RN agrega o ajusta su test.

## Convención de commits y ramas

- NUNCA directo a `main` ni `dev`; rama `agent/<nombre>/<tema>`.
- Convencional en español: `tipo(alcance): descripción`
  (`feat`, `fix`, `docs`, `test`, `refactor`, `chore`).
- Un commit = un cambio lógico. Nada de `wip`.

## Proceso de revisión

1. PR con descripción clara hacia `dev`.
2. CI en verde (fmt, clippy, tests, diagramas, documento).
3. Revisión del equipo y merge.
