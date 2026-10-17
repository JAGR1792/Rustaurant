"""Genera las fotos de mesa de Brindis, una por zona (ficha de la reserva).

Ejecucion: python static/img/mesas/generar_mesas.py
Salida (en esta carpeta, se commitean; la app solo las sirve):
  - mesa-general.png  (900x675) mesa servida clasica del salon.
  - mesa-terraza.png  (900x675) mesa bajo la luz de la terraza.
  - mesa-vip.png      (900x675) mesa intima con vela.
Determinista y sin dependencias fuera de matplotlib.

Estilo: foto ilustrada cenital (flat-lay), misma familia visual que
generar_salon.py / generar_carta.py (paleta Le Jardin).
"""
import os

import matplotlib

matplotlib.use("Agg")

import matplotlib.pyplot as plt  # noqa: E402
from matplotlib.patches import Circle, Ellipse, FancyBboxPatch, Polygon  # noqa: E402

AQUI = os.path.dirname(os.path.abspath(__file__))

MARFIL = "#f5efe5"
VINO = "#702c3b"
VINO_OSCURO = "#4a1c28"
CARBON = "#26231f"
DORADO = "#a88a59"
DORADO_CLARO = "#cbb48e"
LINEA = "#e4d9c6"
VERDE = "#4e7d57"
VERDE_OSCURO = "#3d6547"
TERRA = "#a35043"
SOMBRA = "#dccfb8"


def lona(fig, ax, w, h, color=MARFIL):
    ax.set_xlim(0, w)
    ax.set_ylim(0, h)
    ax.axis("off")
    fig.patch.set_facecolor(color)
    ax.add_patch(Polygon([(0, 0), (w, 0), (w, h), (0, h)], closed=True, color=color))


def mantel(ax, w, h, color, borde, margen=36):
    ax.add_patch(Polygon([(margen, margen / 2), (w - margen, margen / 2),
                          (w - margen, h - margen / 2), (margen, h - margen / 2)], closed=True,
                         fc=color, ec=borde, lw=2))


def plato(ax, cx, cy, r, borde=DORADO_CLARO):
    ax.add_patch(Circle((cx, cy), r, fc="#ffffff", ec=LINEA, lw=2))
    ax.add_patch(Circle((cx, cy), r * 0.9, fc="#ffffff", ec=borde, lw=3))
    ax.add_patch(Circle((cx, cy), r * 0.58, fc="#f3ecdd", ec=LINEA, lw=1.5))


def comida(ax, cx, cy, piezas):
    for dx, dy, r, ang, color in piezas:
        ax.add_patch(Ellipse((cx + dx, cy + dy), r * 1.85, r, angle=ang, fc=color, alpha=0.95, ec="none"))


def cubiertos(ax, cx, cy, dist, alto=150, color=CARBON, lw=16):
    """Cubiertos a los lados del plato: separados `dist` del centro."""
    for s in (-1, 1):
        ax.add_patch(FancyBboxPatch((cx + s * dist - lw / 2, cy - alto / 2), lw, alto,
                                    boxstyle="round,pad=0,rounding_size=8", fc=color, ec="none", alpha=0.85))


def copa(ax, cx, cy, r=58, licor=VINO, aro=DORADO_CLARO):
    ax.add_patch(Circle((cx, cy), r, fc="none", ec=aro, lw=5))
    ax.add_patch(Circle((cx, cy), r * 0.72, fc=licor, ec="none", alpha=0.9))
    ax.add_patch(Circle((cx, cy), r * 0.72, fc="none", ec=CARBON, lw=1, alpha=0.25))


def ramita(ax, x0, y0, largo=150, hojas=3, angulo=70):
    import math
    x1 = x0 + largo * math.cos(math.radians(angulo))
    y1 = y0 + largo * math.sin(math.radians(angulo))
    ax.plot([x0, x1], [y0, y1], color=VERDE_OSCURO, lw=3)
    for i in range(hojas):
        t = (i + 1) / (hojas + 1)
        px = x0 + (x1 - x0) * t
        py = y0 + (y1 - y0) * t
        ax.add_patch(Ellipse((px, py), 64, 28, angle=angulo - 40 + i * 35,
                             fc=VERDE, alpha=0.9, ec="none"))


# ------------------------------------------------------------------ escenas

def general():
    fig, ax = plt.subplots(figsize=(9, 6.75), dpi=100)
    lona(fig, ax, 900, 675)
    mantel(ax, 900, 675, "#fbf7f0", LINEA)
    plato(ax, 430, 330, 200)
    comida(ax, 430, 330, [(-42, 22, 62, 20, "#dda766"), (36, 34, 54, -30, "#c07a4a"),
                          (-12, -40, 46, 45, "#e8cf9d"), (44, -22, 36, -10, "#b65a44"),
                          (10, 6, 40, 70, VERDE)])
    cubiertos(ax, 430, 330, 250, 150, CARBON)
    copa(ax, 700, 200, 56, VINO, DORADO_CLARO)
    # panera
    ax.add_patch(Circle((712, 500), 72, fc="#e9d9bd", ec=SOMBRA, lw=2))
    for dx, dy in ((-22, 8), (18, 14), (0, -24)):
        ax.add_patch(Ellipse((712 + dx, 500 + dy), 52, 34, angle=dx,
                             fc="#d9a966", ec="#c68f4c", lw=1.2))
    ramita(ax, 150, 560, 130, 3, 65)
    fig.subplots_adjust(left=0, right=1, top=1, bottom=0)
    fig.savefig(os.path.join(AQUI, "mesa-general.png"), facecolor=MARFIL)
    plt.close(fig)
    print("  mesa-general.png")


def terraza():
    fig, ax = plt.subplots(figsize=(9, 6.75), dpi=100)
    lona(fig, ax, 900, 675, "#f7f1e2")
    mantel(ax, 900, 675, "#fdfaf1", "#e9ddc4")
    # luz de tarde: bandas diagonales suaves
    for i in range(5):
        x = 60 + i * 210
        ax.add_patch(Polygon([(x, 0), (x + 90, 0), (x - 60, 675), (x - 150, 675)], closed=True,
                             fc="#f4e4c4", alpha=0.35, ec="none"))
    plato(ax, 420, 340, 195)
    comida(ax, 420, 340, [(-40, 10, 60, 15, "#8fb96c"), (30, 30, 50, -25, VERDE),
                          (-8, -42, 44, 40, "#c3d98d"), (48, -18, 34, -5, "#6fae5c")])
    cubiertos(ax, 420, 340, 245, 150, "#6b6257")
    copa(ax, 690, 210, 54, "#e8d18f", DORADO_CLARO)
    ramita(ax, 690, 470, 140, 4, 245)
    ax.add_patch(Circle((175, 190), 46, fc="none", ec=DORADO_CLARO, lw=3, alpha=0.7))
    ax.add_patch(Circle((175, 190), 30, fc="#f4e4c4", ec="none", alpha=0.8))
    fig.subplots_adjust(left=0, right=1, top=1, bottom=0)
    fig.savefig(os.path.join(AQUI, "mesa-terraza.png"), facecolor=MARFIL)
    plt.close(fig)
    print("  mesa-terraza.png")


def vip():
    fig, ax = plt.subplots(figsize=(9, 6.75), dpi=100)
    lona(fig, ax, 900, 675, VINO_OSCURO)
    mantel(ax, 900, 675, "#5c2331", "#421a24")
    # vela encendida
    ax.add_patch(Circle((690, 210), 34, fc="#3f1822", ec=DORADO, lw=2))
    for r, a in ((70, 0.14), (46, 0.18)):
        ax.add_patch(Circle((690, 210), r, fc="#f4e4c4", alpha=a, ec="none"))
    ax.add_patch(Ellipse((690, 210), 14, 26, fc="#f9efd6", ec="none"))
    ax.add_patch(Ellipse((690, 218), 8, 14, fc="#e8b64a", ec="none"))
    plato(ax, 420, 340, 195, DORADO)
    comida(ax, 420, 340, [(-38, 18, 58, 20, "#e08a6d"), (34, 30, 48, -30, "#c96a4f"),
                          (-10, -40, 42, 45, "#f2d9a0"), (46, -14, 32, -8, "#8f5d43")])
    cubiertos(ax, 420, 340, 245, 150, DORADO, 14)
    # pétalos
    for dx, dy, ang in ((640, 420, 30), (700, 520, -25), (240, 180, 15), (170, 430, -40), (620, 130, 60)):
        ax.add_patch(Ellipse((dx, dy), 44, 24, angle=ang, fc=TERRA, alpha=0.85, ec="none"))
    copa(ax, 210, 210, 54, "#8f2f45", DORADO)
    ramita(ax, 180, 520, 120, 3, 60)
    fig.subplots_adjust(left=0, right=1, top=1, bottom=0)
    fig.savefig(os.path.join(AQUI, "mesa-vip.png"), facecolor=MARFIL)
    plt.close(fig)
    print("  mesa-vip.png")


def main():
    print("Generando fotos de mesa:")
    general()
    terraza()
    vip()


if __name__ == "__main__":
    main()
