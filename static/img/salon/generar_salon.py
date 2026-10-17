"""Genera las escenas ilustradas del salon de Brindis (direccion "Le Jardin").

Ejecucion: python static/img/salon/generar_salon.py
Salida (en esta carpeta, se commitean; la app solo las sirve):
  - salon-hero.png    (1600x1000) hero de la portada.
  - salon-acceso.png  (1200x1600) panel lateral del acceso (mesa servida).
Determinista y sin dependencias fuera de matplotlib.

Estilo: ilustracion plana editorial (nada de fotos), paleta marfil /
borgoña / carbon / dorado apagado, misma familia que generar_carta.py.
"""
import math
import os

import matplotlib

matplotlib.use("Agg")

import matplotlib.pyplot as plt  # noqa: E402
from matplotlib.patches import Circle, Ellipse, FancyBboxPatch, Polygon, Wedge  # noqa: E402

AQUI = os.path.dirname(os.path.abspath(__file__))

MARFIL = "#f5efe5"
MARFIL_CLARO = "#fbf7f0"
LUZ = "#f4e4c4"
LUZ_FUERTE = "#f9efd6"
VINO = "#702c3b"
CARBON = "#26231f"
DORADO = "#a88a59"
DORADO_CLARO = "#cbb48e"
LINEA = "#e4d9c6"
SOMBRA = "#dccfb8"
VERDE = "#4e7d57"
VERDE_OSCURO = "#3d6547"
TERRA = "#a35043"
PISO = "#ece0ca"


# ------------------------------------------------------------------ utilidades

def lona(fig, ax, w, h):
    ax.set_xlim(0, w)
    ax.set_ylim(0, h)
    ax.axis("off")
    fig.patch.set_facecolor(MARFIL)
    ax.add_patch(Polygon([(0, 0), (w, 0), (w, h), (0, h)], closed=True, color=MARFIL))


def ventana(ax, cx, base, ancho, alto):
    """Ventanuco arqueado con luz calida interior."""
    radio = ancho / 2
    cima = base + alto
    # marco exterior (carbon) y luz interior
    ax.add_patch(FancyBboxPatch((cx - radio, base), ancho, alto,
                                boxstyle="round,pad=0,rounding_size=6",
                                fc=CARBON, ec="none"))
    ax.add_patch(Wedge((cx, cima), radio, 0, 180, fc=CARBON, ec="none"))
    m = 12
    ax.add_patch(FancyBboxPatch((cx - radio + m, base + m), ancho - 2 * m, alto - m,
                                boxstyle="round,pad=0,rounding_size=4",
                                fc=LUZ, ec="none"))
    ax.add_patch(Wedge((cx, cima), radio - m, 0, 180, fc=LUZ, ec="none"))
    # resplandor interior
    for r, a in ((radio * 0.62, 0.35), (radio * 0.34, 0.4)):
        ax.add_patch(Wedge((cx, base + alto * 0.45), r, 0, 180, fc=LUZ_FUERTE, alpha=a, ec="none"))
    # parteluces
    ax.plot([cx, cx], [base + 6, cima + radio - m - 4], color=CARBON, lw=5)
    ax.plot([cx - radio + m, cx + radio - m], [base + alto * 0.52, base + alto * 0.52],
            color=CARBON, lw=5)


def lampara(ax, cx, y0, altura, escala=1.0):
    """Lampara colgante: varilla, pantalla dorada y halo."""
    y1 = y0 - altura
    ax.plot([cx, cx], [y0, y1], color=CARBON, lw=2.5)
    ancho = 46 * escala
    alto = 26 * escala
    ax.add_patch(Polygon([(cx - ancho / 2, y1), (cx + ancho / 2, y1),
                          (cx + ancho * 0.30, y1 - alto), (cx - ancho * 0.30, y1 - alto)],
                         closed=True, fc=DORADO, ec=CARBON, lw=1.5))
    ax.add_patch(Circle((cx, y1 - alto - 4), 7 * escala, fc=LUZ_FUERTE, ec="none"))
    for r, a in ((78 * escala, 0.10), (46 * escala, 0.14)):
        ax.add_patch(Circle((cx, y1 - alto - 6), r, fc=LUZ, alpha=a, ec="none"))


def planta(ax, cx, base, escala=1.0):
    """Matera con hojas simples."""
    ancho = 56 * escala
    alto = 46 * escala
    ax.add_patch(Polygon([(cx - ancho / 2, base), (cx + ancho / 2, base),
                          (cx + ancho * 0.38, base + alto), (cx - ancho * 0.38, base + alto)],
                         closed=True, fc=TERRA, ec="none"))
    ax.add_patch(FancyBboxPatch((cx - ancho * 0.42, base + alto - 6), ancho * 0.84, 10,
                                boxstyle="round,pad=0,rounding_size=3", fc=TERRA, ec="none"))
    hojas = [(0, 120, 26, -8, VERDE), (-34, 96, 22, 30, VERDE_OSCURO),
             (36, 100, 22, -32, VERDE_OSCURO), (-16, 74, 18, 52, VERDE),
             (18, 78, 18, -52, VERDE), (0, 92, 24, 0, VERDE_OSCURO)]
    for dx, dy, r, ang, color in hojas:
        ax.add_patch(Ellipse((cx + dx, base + alto + dy * escala * 0.6), r * escala * 1.9, r * escala,
                             angle=ang, fc=color, alpha=0.92, ec="none"))


def mesa_servida(ax, cx, base, escala=1.0):
    """Mesa bistró con botella y copas (vista frontal)."""
    alto_total = 104 * escala
    tablero_y = base + alto_total
    # base, fuste y tablero
    ax.add_patch(Ellipse((cx, base + 7 * escala), 168 * escala, 20 * escala, fc=SOMBRA, alpha=0.6, ec="none"))
    ax.add_patch(Ellipse((cx, base + 12 * escala), 104 * escala, 16 * escala, fc=CARBON, alpha=0.9, ec="none"))
    ax.add_patch(FancyBboxPatch((cx - 8 * escala, base + 12 * escala), 16 * escala, alto_total - 24 * escala,
                                boxstyle="round,pad=0,rounding_size=4", fc=CARBON, alpha=0.9, ec="none"))
    ax.add_patch(Ellipse((cx, tablero_y), 216 * escala, 28 * escala, fc=MARFIL_CLARO, ec=LINEA, lw=1.5))
    ax.add_patch(Ellipse((cx, tablero_y + 3 * escala), 190 * escala, 20 * escala, fc="#fffdf9", alpha=0.85, ec="none"))
    # botella al centro
    bx = cx
    ax.add_patch(FancyBboxPatch((bx - 8 * escala, tablero_y), 16 * escala, 56 * escala,
                                boxstyle="round,pad=0,rounding_size=4", fc=VERDE_OSCURO, ec="none"))
    ax.add_patch(FancyBboxPatch((bx - 4 * escala, tablero_y + 50 * escala), 8 * escala, 26 * escala,
                                boxstyle="round,pad=0,rounding_size=3", fc=VERDE_OSCURO, ec="none"))
    ax.add_patch(Circle((bx, tablero_y + 78 * escala), 4.5 * escala, fc=CARBON, ec="none"))
    # copas
    for s in (-1, 1):
        gx = cx + s * 54 * escala
        gy = tablero_y + 2 * escala
        ax.add_patch(Polygon([(gx - 9 * escala, gy + 24 * escala), (gx + 9 * escala, gy + 24 * escala),
                              (gx + 3 * escala, gy + 3 * escala), (gx - 3 * escala, gy + 3 * escala)],
                             closed=True, fc="none", ec=DORADO, lw=1.8))
        ax.plot([gx, gx], [gy + 3 * escala, gy], color=DORADO, lw=1.8)
        ax.plot([gx - 7 * escala, gx + 7 * escala], [gy, gy], color=DORADO, lw=1.8)


def silla(ax, cx, base, escala=1.0):
    """Silla bistró de perfil con respaldo alto (carbon)."""
    for dx in (-15 * escala, 15 * escala):
        ax.plot([cx + dx, cx + dx], [base, base + 44 * escala],
                color=CARBON, lw=4.2, alpha=0.9, solid_capstyle="round")
    ax.add_patch(FancyBboxPatch((cx - 23 * escala, base + 44 * escala), 46 * escala, 9 * escala,
                                boxstyle="round,pad=0,rounding_size=3.5", fc=CARBON, alpha=0.9, ec="none"))
    ax.add_patch(FancyBboxPatch((cx + 13 * escala, base + 50 * escala), 10 * escala, 68 * escala,
                                boxstyle="round,pad=0,rounding_size=4", fc=CARBON, alpha=0.9, ec="none"))
    ax.add_patch(FancyBboxPatch((cx - 20 * escala, base + 62 * escala), 34 * escala, 7 * escala,
                                boxstyle="round,pad=0,rounding_size=3", fc=CARBON, alpha=0.55, ec="none"))


# ------------------------------------------------------------------ escenas

def hero():
    fig, ax = plt.subplots(figsize=(16, 10), dpi=100)
    lona(fig, ax, 1600, 1000)

    # piso y zócalo
    ax.add_patch(Polygon([(0, 0), (1600, 0), (1600, 175), (0, 175)], closed=True, fc=PISO))
    ax.plot([0, 1600], [175, 175], color=SOMBRA, lw=2.5)
    for x in range(90, 1600, 175):
        ax.plot([x, x - 60], [175, 0], color=SOMBRA, lw=0.8, alpha=0.5)
    # moldura de pared
    ax.plot([110, 1490], [905, 905], color=LINEA, lw=2)

    # ventanales
    ventana(ax, 340, 235, 300, 560)
    ventana(ax, 760, 235, 300, 560)

    # lámparas
    lampara(ax, 1120, 1000, 230, 1.3)
    lampara(ax, 1340, 1000, 180, 1.0)
    lampara(ax, 560, 1000, 130, 0.8)

    # rincón de mesa (derecha)
    mesa_servida(ax, 1105, 175, 1.5)
    silla(ax, 1345, 175, 1.4)

    # plantas
    planta(ax, 125, 175, 1.5)
    planta(ax, 1520, 175, 1.2)

    fig.subplots_adjust(left=0, right=1, top=1, bottom=0)
    fig.savefig(os.path.join(AQUI, "salon-hero.png"), facecolor=MARFIL)
    plt.close(fig)
    print("  salon-hero.png")


def acceso():
    """Mesa servida vista desde arriba (panel del acceso)."""
    fig, ax = plt.subplots(figsize=(12, 16), dpi=100)
    lona(fig, ax, 1200, 1600)

    # mantel: lienzo completo con caída sutil
    ax.add_patch(Polygon([(60, 40), (1140, 40), (1140, 1560), (60, 1560)], closed=True,
                         fc=MARFIL_CLARO, ec=LINEA, lw=2))
    # servilleta borgoña (arriba izquierda, bajo el plato)
    ax.add_patch(FancyBboxPatch((150, 205), 190, 150, boxstyle="round,pad=0,rounding_size=10",
                                fc=VINO, ec="none", alpha=0.92))
    ax.add_patch(FancyBboxPatch((150, 205), 190, 28, boxstyle="round,pad=0,rounding_size=6",
                                fc="#5c2331", ec="none", alpha=0.9))
    # plato principal
    cx, cy = 600, 620
    ax.add_patch(Circle((cx, cy), 330, fc="#ffffff", ec=LINEA, lw=2))
    ax.add_patch(Circle((cx, cy), 300, fc="#ffffff", ec=DORADO_CLARO, lw=3))
    ax.add_patch(Circle((cx, cy), 190, fc="#f3ecdd", ec=LINEA, lw=1.5))
    # composición del plato (blobs elegantes)
    for dx, dy, r, ang, color in ((-70, 40, 88, 20, "#dda766"), (60, 60, 74, -30, "#c07a4a"),
                                  (-20, -70, 66, 45, VERDE), (80, -40, 52, -10, "#e8cf9d"),
                                  (-95, -20, 40, 70, "#b65a44")):
        ax.add_patch(Ellipse((cx + dx, cy + dy), r * 1.85, r, angle=ang, fc=color, alpha=0.95, ec="none"))
    # cubiertos
    ax.add_patch(FancyBboxPatch((150, 520), 16, 200, boxstyle="round,pad=0,rounding_size=8",
                                fc=CARBON, ec="none", alpha=0.85))
    ax.add_patch(FancyBboxPatch((1034, 520), 16, 200, boxstyle="round,pad=0,rounding_size=8",
                                fc=CARBON, ec="none", alpha=0.85))
    # copa de vino (arriba derecha, vista cenital: círculo con vino)
    ax.add_patch(Circle((905, 300), 74, fc="none", ec=DORADO_CLARO, lw=6))
    ax.add_patch(Circle((905, 300), 52, fc=VINO, ec="none", alpha=0.9))
    ax.add_patch(Circle((905, 300), 52, fc="none", ec=CARBON, lw=1, alpha=0.25))
    # panera
    ax.add_patch(Circle((950, 1080), 95, fc="#e9d9bd", ec=SOMBRA, lw=2))
    for dx, dy in ((-30, 10), (25, 20), (0, -35), (45, -12)):
        ax.add_patch(Ellipse((950 + dx, 1080 + dy), 70, 46, angle=dx / 2,
                             fc="#d9a966", ec="#c68f4c", lw=1.2))
    # ramita verde
    ax.plot([430, 470, 445], [1420, 1350, 1290], color=VERDE_OSCURO, lw=3)
    for i, (px, py) in enumerate(((452, 1345), (438, 1385), (462, 1305))):
        ax.add_patch(Ellipse((px, py), 66, 30, angle=35 - i * 30, fc=VERDE, alpha=0.9, ec="none"))

    fig.subplots_adjust(left=0, right=1, top=1, bottom=0)
    fig.savefig(os.path.join(AQUI, "salon-acceso.png"), facecolor=MARFIL)
    plt.close(fig)
    print("  salon-acceso.png")


def main():
    print("Generando escenas del salon:")
    hero()
    acceso()


if __name__ == "__main__":
    main()
