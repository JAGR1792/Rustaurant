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
(ADR 0006), notificación por correo al cliente y a la administración al
confirmar (buzón de demostración visible en la agenda, ADR 0008, con
entrega real por SMTP si el entorno trae credenciales, ADR 0009), mis
reservas con cancelación confirmada y reembolso (RN08/RN09), cuenta
regresiva de la próxima reserva (contador de cifras rodantes, ver ADR
0003) y agenda del personal protegida por el Proxy. El personal opera
con paneles por tipo de empleado (RF15): panel propio con el turno del
día, mesas asignadas y restricciones por preparar, y panel de
administración con KPIs, jornadas y asignación de meseros (ADR 0011).
La portada incluye
la carta de platos con un efecto de spotlight que sigue al cursor y
revela la foto a color (port vanilla del ChromaGrid de React Bits, ver
ADR 0002).

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
| Cocina | `cocina@brindis.co` | `Cocina123!` |
| Admin | `brindis.rust@gmail.com` | `Admin123!` |

La fase 1 guarda todo en memoria (`src/estado.rs` + seed de demo);
los DAO de `src/dao.rs` son la ruta de conexión a SQL Server y el
esquema completo vive en `db/brindis_db.sql` (ver Base de datos).

## Base de datos `brindis_db`

El modelo relacional que documenta la Figura 5 vive en
`db/brindis_db.sql`: 14 tablas 3FN (usuarios, jornadas, mesa y zona, agenda,
reservas, pagos y tokens) con restricciones, índices y datos de prueba.
Se carga en SQL Server 2025 con SSMS abriendo el script, o por línea
de comandos:

```powershell
sqlcmd -S .\SQLEXPRESS -E -i db/brindis_db.sql
```

El diagrama de la Figura 5 se genera desde esa misma base con la
herramienta de diagramas de SQL Server Management Studio (exportado en
`diagramas/brindis/Fig5_DiagramaBD.png`). La app de la fase 1 no
depende de la base (corre en memoria); el script es el entregable del
modelo relacional.

## Estructura

```text
Dockerfile            # imagen multietapa (build Rust -> runtime Debian)
render.yaml           # servicio web en Render (ADR 0010)
db/                   # brindis_db.sql (14 tablas 3FN: esquema, indices, datos de prueba)
src/                  # Axum + dominio + 4 patrones + DAO
  dominio.rs          # entidades (Figura 4)
  factory.rs          # Factory Method (Figura 7)
  builder.rs          # Builder (Figura 8)
  proxy.rs            # Proxy (Figura 9)
  decorator.rs        # Decorator (Figura 10)
  dao.rs              # GenericDAO + pool (Figura 6)
  estado.rs           # almacen demo en memoria + seed
  servicios.rs        # sujeto real del Proxy sobre el almacen
  correo.rs           # avisos al cliente/admin (buzon demo + SMTP, ADR 0008/0009)
  formato.rs          # fechas UTC-5 y pesos para vistas y correos
  vistas.rs           # controladores Axum + structs Askama
templates/            # base + inicio/login/registro/reservar/mis-reservas/agenda/panel_empleado/panel_admin/error
static/css/           # tema.css + fuentes.css + chroma.css + counter.css
static/js/            # plano.js (plano SVG, ficha y pedido guiado) + ui.js (toasts y confirmacion)
                      # + chroma.js (carta) + counter.js (cuenta regresiva)
static/fuentes/       # Cormorant Garamond + Inter (woff2) + descargar_fuentes.py
static/img/fotos/     # fotos Commons (jpg+webp) + descargar_fotos.py + convertir_webp.py + creditos.md
static/img/carta/     # PNGs de platos + generar_carta.py (respaldo offline)
static/img/salon/     # salon-hero/salon-acceso.png + generar_salon.py
static/img/mesas/     # mesa-general/terraza/vip.png + generar_mesas.py
diagramas/brindis/    # Fig1/4/5 y Fig6-16 (PNG/SVG) + .drawio (Fig1, Fig4)
documentacion/        # brindis_rust.tex + decisiones/ (ADRs) + usa.png + entregables/
```

## Arquitectura MVC por capas

El código se organiza por capas como en la Figura 3 del documento:

| Capa | Rol | Archivos |
|---|---|---|
| Vista | Pinta la interfaz; no conoce al modelo. | `templates/` + `static/` |
| Controlador | Atiende las peticiones HTTP y delega en el modelo. | `src/vistas.rs` + rutas de `src/main.rs` |
| Modelo: servicios y patrones | Reglas de negocio detrás de las interfaces de servicio. | `src/servicios.rs` + `factory.rs`, `builder.rs`, `proxy.rs`, `decorator.rs`, `correo.rs` |
| Modelo: acceso a datos | DAO genéricos y almacenamiento (fase 1 en memoria). | `src/dao.rs` + `src/estado.rs` |
| Modelo: entidades | Structs del dominio y utilidades de formato. | `src/dominio.rs` + `src/formato.rs` |

Regla de dependencias: la Vista solo habla con el Controlador; el
Controlador solo conoce las interfaces de servicio (los Proxy); los
servicios usan los DAO y las entidades; las entidades no conocen a nadie.
Ninguna capa salta a otra que no le corresponde (Figuras 3 y 11 del
documento).

## Diagramas

Se regeneran sin solapes con `python diagramas/brindis/fig_brindis.py`
(valida 0 incidencias). Los `.drawio` de Fig1 y Fig4 se exportan
a PNG 300 ppp con el mismo nombre para que el `.tex` los tome.

## Variables

Copiar `.env.example` a `.env` (nunca se commitea). Para que los avisos
salgan de verdad por Gmail, definir `SMTP_USUARIO` y `SMTP_CLAVE` (con
verificación en dos pasos, la "contraseña de aplicación"); sin ellas la
app mantiene el buzón de demostración (ADR 0009).

## Despliegue

`Dockerfile` + `render.yaml` publican la demo en Render (ADR 0010):
web service con runtime Docker, health check en `/` y plan free. En el
panel se configuran `SMTP_USUARIO` y `SMTP_CLAVE` (secreto); sin clave
la demo sigue con el buzón interno.
