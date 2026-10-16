# Diagramas CR2-T1 (patrones creacionales, SIGTVA)

Contenido
- `png/`  Los 15 diagramas a 300 dpi, tal como están en el Word (fig1 ... fig15).
- `svg/`  Los mismos en vectorial; el texto queda editable (Inkscape, draw.io, Figma).
- `*.py`  Código fuente. `uml.py` es la librería; `fig_*.py` definen cada figura.

Requisitos: Python 3.9+ y matplotlib (`pip install matplotlib`).
Fuente: usa Liberation Sans o Arial si están instaladas; si no, DejaVu Sans
(el ancho de las cajas cambia un poco, se recalcula solo).

Regenerar
    python make_all.py        # las 15 figuras en png/ y svg/
    python make_all.py 5 8    # solo las figuras 5 y 8

Qué archivo editar
| Figuras   | Archivo          | Función / dato                       |
|-----------|------------------|--------------------------------------|
| 1, 2, 3   | fig_factory.py   | fig1, fig2, fig3                     |
| 4, 5, 6   | fig_abstract.py  | SPEC4, SPEC5, SPEC6 (diccionarios)   |
| 7, 8, 9   | fig_builder.py   | fig7, fig8, fig9                     |
| 10, 11, 12| fig_proto.py     | fig10, fig11, fig12                  |
| 13, 14, 15| fig_single.py    | fig13, fig14, fig15                  |

Las figuras 4, 5 y 6 (Abstract Factory) se generan a partir de un diccionario:
para cambiar nombres, métodos, atributos o agregar una familia o un producto,
edite SPEC4 / SPEC5 / SPEC6 y el trazado se recalcula. El resto se dibuja
con posiciones explícitas.

Uso básico de la librería (uml.py)
    c = Canvas(6.4, 4.0, fs=8)                       # ancho y alto en pulgadas, fuente en pt
    a = c.cls("a", 0.1, 0.1, "Taquillero", meths=["+vender() : void"])
    b = c.cls("b", 2.5, 0.1, "Tarifa", kind="interface", meths=["+calcular() : double"])
    c.edge("dep", [c.pt("a", "E"), c.pt("b", "W")], "a", "b", label=("usa", 0, 0.5, 0, -0.1))
    c.validate(); c.fit(); c.render("salida.png")

- Tipos de clase (kind): class, abstract, interface, singleton, utility.
- Tipos de línea (edge): dep (dependencia), real (realización), gen (herencia),
  agg (agregación, rombo en el primer punto), comp (composición), assoc, assocA, note.
- Coordenadas en pulgadas, eje y hacia abajo; las cajas se ajustan al texto.
- `c.tree_v(...)` y `c.spine(...)` dibujan realizaciones con destino compartido.
- `c.validate()` avisa si una línea cruza una caja o una etiqueta pisa algo.

Para cambiar una imagen en el Word: clic derecho sobre la figura > Cambiar imagen,
y elegir el PNG nuevo (conserva tamaño y posición; ajuste el alto si cambió la proporción).
