# Rustaurant — Brindis

Módulo web de gestión de reservas de restaurantes por ocasión
(Universidad Sergio Arboleda — Patrones de diseño de software).

## Cómo correrlo (3 comandos)

```powershell
cargo test
cargo run
tectonic documentacion/brindis_rust.tex
```

La app sirve en `http://127.0.0.1:3000`.
El PDF del documento se genera en `documentacion/brindis_rust.pdf`.

## Estructura

```text
src/                  # Axum + dominio + 4 patrones + DAO
  dominio.rs          # entidades (Figura 4)
  factory.rs          # Factory Method (Figura 7)
  builder.rs          # Builder (Figura 8)
  proxy.rs            # Proxy (Figura 9)
  decorator.rs        # Decorator (Figura 10)
  dao.rs              # GenericDAO + ConexionBD (Figura 6)
diagramas/brindis/    # Fig6-11 PNG/SVG + .drawio (Fig1, Fig4)
documentacion/        # brindis_rust.tex + usa.png + entregables/
```

## Diagramas

Se regeneran sin solapes con `python diagramas/brindis/fig_brindis.py`
(valida 0 incidencias). Los `.drawio` de Fig1 y Fig4 se exportan
a PNG 300 ppp con el mismo nombre para que el `.tex` los tome.

## Variables

Copiar `.env.example` a `.env` (nunca se commitea).
