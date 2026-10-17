"""Genera las imagenes de "La carta" de Brindis: platos vistos desde arriba.

Ejecucion: python static/img/carta/generar_carta.py
Salida: PNGs 800x800 en esta misma carpeta (se commitean; la app solo los sirve).
Determinista: mismas semillas -> mismos PNGs. Solo depende de matplotlib.

Diseno: mantel de color + loza clara con aro dorado + composicion de
"blobs" organicos (la comida). Pensado para el efecto ChromaGrid de la
portada: en gris se ve sobrio y con el cursor se encienden los colores.
"""
import math
import os
import random

import matplotlib

matplotlib.use("Agg")

import matplotlib.pyplot as plt  # noqa: E402
from matplotlib.patches import Circle, Ellipse, Rectangle  # noqa: E402

AQUI = os.path.dirname(os.path.abspath(__file__))

# (archivo, mantel, [(color, tamano_rel)], color de detalles, semilla)
PLATOS = [
    (
        "01-ceviche-camaron",
        "#1c4d66",
        [("#ff9d7a", 1.00), ("#b8e04f", 0.85), ("#c86bb1", 0.70), ("#f2d16b", 0.55)],
        "#7ac74f",
        11,
    ),
    (
        "02-picanha-brasa",
        "#58170a",
        [("#7a2e1a", 1.15), ("#d9a066", 0.80), ("#e8c476", 0.70), ("#4f8f3a", 0.55)],
        "#f0e6d2",
        23,
    ),
    (
        "03-tres-leches",
        "#6f4522",
        [("#ead9b8", 1.10), ("#fff6e8", 0.85), ("#cf2e40", 0.55), ("#b97b3c", 0.45)],
        "#fff6e8",
        37,
    ),
    (
        "04-limonada-coco",
        "#17522e",
        [("#d7f07a", 1.15), ("#eefcff", 0.80), ("#5faf4b", 0.55), ("#ffffff", 0.45)],
        "#eefcff",
        41,
    ),
    (
        "05-salmon-hierbas",
        "#362a52",
        [("#ff8f70", 1.10), ("#e8d9ae", 0.75), ("#f2d24b", 0.55), ("#6fae5c", 0.50)],
        "#6fae5c",
        53,
    ),
    (
        "06-coctel-casa",
        "#5c1030",
        [("#ff6f9c", 1.15), ("#ffb26b", 0.70), ("#dfefff", 0.55), ("#d7263d", 0.45)],
        "#dfefff",
        67,
    ),
    ("00-generico", "#3a3340", [], "#c9a227", 7),
]


def blob(ax, cx, cy, radio, color, rng, z=3):
    """Una forma organica cerrada (comida) con borde suave."""
    fases = [rng.uniform(0, 2 * math.pi) for _ in range(3)]
    t = [2 * math.pi * i / 96 for i in range(97)]
    xs, ys = [], []
    for ang in t:
        r = radio * (
            1
            + 0.16 * math.sin(3 * ang + fases[0])
            + 0.09 * math.sin(5 * ang + fases[1])
            + 0.05 * math.sin(8 * ang + fases[2])
        )
        xs.append(cx + r * math.cos(ang))
        ys.append(cy + r * math.sin(ang))
    ax.fill(xs, ys, color=color, zorder=z, linewidth=0)


def plato(archivo, mantel, comidas, detalle, semilla):
    rng = random.Random(semilla)
    fig = plt.figure(figsize=(8, 8), dpi=100)
    ax = fig.add_axes([0, 0, 1, 1])
    ax.set_xlim(0, 1)
    ax.set_ylim(0, 1)
    ax.axis("off")

    ax.add_patch(Rectangle((0, 0), 1, 1, color=mantel, zorder=0))
    # Sombra de la loza y la loza con su aro dorado.
    ax.add_patch(Circle((0.5, 0.475), 0.42, color="#000000", alpha=0.28, zorder=1))
    ax.add_patch(Circle((0.5, 0.5), 0.42, facecolor="#f2eee4", edgecolor="#e2dccc", linewidth=2, zorder=2))
    ax.add_patch(
        Circle((0.5, 0.5), 0.30, facecolor="none", edgecolor="#c9a227", alpha=0.35, linewidth=1.4, zorder=2)
    )

    if comidas:
        n = len(comidas)
        base = rng.uniform(0, 2 * math.pi)
        for k, (color, escala) in enumerate(comidas):
            ang = base + 2 * math.pi * k / n + rng.uniform(-0.35, 0.35)
            dist = rng.uniform(0.07, 0.12) * (1.2 - escala * 0.4)
            cx = 0.5 + dist * math.cos(ang)
            cy = 0.5 + dist * math.sin(ang)
            radio = 0.105 * escala + rng.uniform(0.0, 0.02)
            blob(ax, cx, cy, radio, color, rng)
        # Detalles (hierbas, semillas): puntitos dentro del plato.
        for _ in range(34):
            ang = rng.uniform(0, 2 * math.pi)
            rad = 0.24 * math.sqrt(rng.uniform(0, 1))
            ax.add_patch(
                Circle(
                    (0.5 + rad * math.cos(ang), 0.5 + rad * math.sin(ang)),
                    rng.uniform(0.006, 0.011),
                    color=detalle,
                    alpha=0.85,
                    zorder=4,
                )
            )
        # Brillo de loza: media luna suave arriba a la izquierda.
        ax.add_patch(
            Ellipse((0.42, 0.72), 0.34, 0.10, angle=-18, color="#ffffff", alpha=0.18, zorder=5)
        )

    fig.savefig(os.path.join(AQUI, f"{archivo}.png"), dpi=100, facecolor=mantel)
    plt.close(fig)
    print(f"[carta] {archivo}.png")


def main():
    for archivo, mantel, comidas, detalle, semilla in PLATOS:
        plato(archivo, mantel, comidas, detalle, semilla)
    print(f"[carta] OK {len(PLATOS)} imagenes en {AQUI}")


if __name__ == "__main__":
    main()
