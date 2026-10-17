# Rustaurant — Brindis

Módulo web de gestión de reservas de restaurantes por ocasión
(Universidad Sergio Arboleda — Patrones de diseño de software).

## Cómo correrlo (3 comandos)

```powershell
cargo test
cargo run
tectonic documentacion/brindis_rust.tex
```

La app sirve en `http://127.0.0.1:3000` con la fase 1 funcional:
login/registro, asistente de reserva de 4 pasos con plano de mesas,
mis reservas con cancelación (reembolso RN08/RN09) y agenda del
personal protegida por el Proxy.

El PDF del documento se genera en `documentacion/brindis_rust.pdf`.

### Cuentas demo

| Rol | Correo | Contraseña |
|---|---|---|
| Cliente | `cliente@brindis.co` | `Cliente123!` |
| VIP | `vip@brindis.co` | `Vip123!` |
| Empleado | `empleado@brindis.co` | `Empleado123!` |
| Admin | `admin@brindis.co` | `Admin123!` |

La fase 1 guarda todo en memoria (`src/estado.rs` + seed de demo);
los DAO de `src/dao.rs` son la ruta de conexión a SQL Server.

## Estructura

```text
src/                  # Axum + dominio + 4 patrones + DAO
  dominio.rs          # entidades (Figura 4)
  factory.rs          # Factory Method (Figura 7)
  builder.rs          # Builder (Figura 8)
  proxy.rs            # Proxy (Figura 9)
  decorator.rs        # Decorator (Figura 10)
  dao.rs              # GenericDAO + pool (Figura 6)
  estado.rs           # almacen demo en memoria + seed
  servicios.rs        # sujeto real del Proxy sobre el almacen
  vistas.rs           # controladores Axum + structs Askama
templates/            # base + inicio/login/registro/reservar/mis-reservas/agenda/error
static/               # tema.css + plano.js (asistente) + ui.js (toasts)
diagramas/brindis/    # Fig6-11 PNG/SVG + .drawio (Fig1, Fig4)
documentacion/        # brindis_rust.tex + usa.png + entregables/
```

## Diagramas

Se regeneran sin solapes con `python diagramas/brindis/fig_brindis.py`
(valida 0 incidencias). Los `.drawio` de Fig1 y Fig4 se exportan
a PNG 300 ppp con el mismo nombre para que el `.tex` los tome.

## Variables

Copiar `.env.example` a `.env` (nunca se commitea).
