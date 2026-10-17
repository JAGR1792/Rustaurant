"""Genera las variantes WebP de las fotos reales de la app.

Convierte cada JPEG de `static/img/fotos/` y el genérico
`static/img/carta/00-generico.png` a `.webp` (calidad 80, método 6).
La UI las sirve con `<picture>`: WebP primero y el archivo original como
respaldo, así un navegador viejo sigue viendo la foto.

Uso:  python static/img/fotos/convertir_webp.py
Corre sin red; requiere Pillow (el mismo que usa descargar_fotos.py).
Al bajar fotos nuevas, `descargar_fotos.py` ya deja el par listo.
"""

from __future__ import annotations

import sys
from pathlib import Path

FOTOS = Path(__file__).resolve().parent
RAIZ = FOTOS.parents[2]  # ...\static\img\fotos -> raíz del repo
CALIDAD = 80


def convertir(origen: Path) -> Path:
    from PIL import Image

    destino = origen.with_suffix(".webp")
    with Image.open(origen) as img:
        img.convert("RGB").save(destino, "WEBP", quality=CALIDAD, method=6)
    return destino


def main() -> int:
    try:
        import PIL  # noqa: F401 - solo se valida que exista
    except ImportError:
        print("Falta Pillow: pip install pillow", file=sys.stderr)
        return 1

    origenes = sorted(FOTOS.glob("*.jpg"))
    generico = RAIZ / "static" / "img" / "carta" / "00-generico.png"
    if generico.exists():
        origenes.append(generico)

    total_antes = 0
    total_despues = 0
    for origen in origenes:
        destino = convertir(origen)
        antes = origen.stat().st_size // 1024
        despues = destino.stat().st_size // 1024
        total_antes += antes
        total_despues += despues
        print(f"OK  {destino.name:34} {antes:5} KB -> {despues:5} KB")

    ahorro = (1 - total_despues / total_antes) * 100 if total_antes else 0
    print(
        f"\n{len(origenes)} variantes webp; "
        f"{total_antes} KB -> {total_despues} KB ({ahorro:.0f}% menos)"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
