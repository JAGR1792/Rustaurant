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
login/registro, asistente de reserva de 4 pasos con plano SVG del salón
por zonas, ficha de mesa con foto real y pedido guiado por porciones
(ADR 0006), mis reservas con cancelación confirmada y reembolso
(RN08/RN09), cuenta regresiva de la próxima reserva (contador
de cifras rodantes, ver ADR 0003) y agenda del personal protegida por el
Proxy. La portada incluye la carta de platos con un efecto de spotlight
que sigue al cursor y revela la foto a color (port vanilla del ChromaGrid
de React Bits, ver ADR 0002).

La interfaz sigue la dirección visual "Le Jardin" (ver ADR 0004): paleta
marfil/borgoña/carbón con dorado apagado, tipografías self-hosted
(Cormorant Garamond + Inter, `static/fuentes/`, licencia OFL) y
componentes en CSS vanilla. Las fotos (carta, salón y mesas) vienen de
Wikimedia Commons con licencia verificada: `static/img/fotos/descargar_fotos.py`
las regenera y `creditos.md` lista autor, licencia y enlace (ADR 0005).
Cada foto viaja en WebP con respaldo JPEG vía `<picture>`
(`convertir_webp.py`, ADR 0007). Los PNG deterministas de `generar_*.py`
quedan como respaldo offline.

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
static/css/           # tema.css + fuentes.css + chroma.css + counter.css
static/js/            # plano.js (plano SVG, ficha y pedido guiado) + ui.js (toasts y confirmacion)
                      # + chroma.js (carta) + counter.js (cuenta regresiva)
static/fuentes/       # Cormorant Garamond + Inter (woff2) + descargar_fuentes.py
static/img/fotos/     # fotos Commons (jpg+webp) + descargar_fotos.py + convertir_webp.py + creditos.md
static/img/carta/     # PNGs de platos + generar_carta.py (respaldo offline)
static/img/salon/     # salon-hero/salon-acceso.png + generar_salon.py
static/img/mesas/     # mesa-general/terraza/vip.png + generar_mesas.py
diagramas/brindis/    # Fig6-11 PNG/SVG + .drawio (Fig1, Fig4)
documentacion/        # brindis_rust.tex + decisiones/ (ADRs) + usa.png + entregables/
```

## Diagramas

Se regeneran sin solapes con `python diagramas/brindis/fig_brindis.py`
(valida 0 incidencias). Los `.drawio` de Fig1 y Fig4 se exportan
a PNG 300 ppp con el mismo nombre para que el `.tex` los tome.

## Variables

Copiar `.env.example` a `.env` (nunca se commitea).
