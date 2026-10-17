"""Descarga las fotos reales de la app desde Wikimedia Commons.

Idea (fotos con licencia verificada por API de Commons) vista en el repo
Parcial-Tellez de CamiloDG07, reescrita desde cero para Brindis.

Para cada foto del MANIFIESTO:
1. consulta la API de Commons (extmetadata),
2. verifica que la licencia esté en la lista permitida,
3. descarga la miniatura al ancho pedido con User-Agent identificable,
4. recomprime a JPEG (si hay Pillow) y la guarda en esta carpeta,
5. escribe `creditos.md` y `creditos.json` con autor, licencia y enlace.

Uso:  python static/img/fotos/descargar_fotos.py
Las CC BY-SA obligan a mantener atribución y a compartir las adaptaciones
con la misma licencia; los créditos quedan en `creditos.md`.

Nota: los PNG de `static/img/{carta,salon,mesas}` generados por los scripts
`generar_*.py` quedan como respaldo offline; la UI usa estas fotos.
"""

from __future__ import annotations

import json
import sys
import time
import urllib.parse
import urllib.request
from pathlib import Path

DESTINO = Path(__file__).resolve().parent
API = "https://commons.wikimedia.org/w/api.php"
UA = "BrindisFotoScript/0.1 (https://github.com/JAGR1792/Rustaurant; proyecto academico)"

# Ancho objetivo de la miniatura descargada.
MANIFIESTO = [
    ("carta-01-ceviche.jpg", "File:Tostada de ceviche de camarón con mango.jpg", 1000),
    ("carta-02-picana.jpg", "File:Churrasco carioca.jpg", 1000),
    ("carta-03-tres-leches.jpg", "File:Tres Leches Cake (4).jpg", 1000),
    ("carta-04-limonada-coco.jpg", "File:Limonada de coco.jpg", 1000),
    ("carta-05-salmon.jpg", "File:Grilled plated salmon fillet (cropped).jpg", 1000),
    ("carta-06-coctel.jpg", "File:Cóctel Negroni Campari.jpg", 1000),
    ("salon-hero.jpg", "File:Interior of a restaurants in Paris (3384656627).jpg", 1600),
    (
        "salon-acceso.jpg",
        "File:Table set for dining in a modern restaurant with a vase of flowers and elegant decorafternoon.jpg",
        1200,
    ),
    (
        "mesa-general.jpg",
        "File:Tables are set in a restaurant with wine bottles on display in the background.jpg",
        1200,
    ),
    (
        "mesa-terraza.jpg",
        "File:DSC 8598 A charming wooden terrace cafe lined with colorful potted flowers vintage lanterns and cozy tables under a clear blue sky.jpg",
        1200,
    ),
    ("mesa-vip.jpg", "File:Table set for dinner in a restaurant.jpg", 1200),
]

LICENCIAS_OK = ("CC0", "Public domain", "CC BY", "CC BY-SA")


def pedir(url: str) -> bytes:
    req = urllib.request.Request(url, headers={"User-Agent": UA})
    with urllib.request.urlopen(req, timeout=60) as r:
        return r.read()


def metadatos(titulo: str, ancho: int) -> dict:
    params = {
        "action": "query",
        "prop": "imageinfo",
        "iiprop": "url|extmetadata",
        "iiurlwidth": str(ancho),
        "format": "json",
        "titles": titulo,
    }
    url = API + "?" + urllib.parse.urlencode(params)
    datos = json.loads(pedir(url))
    paginas = datos["query"]["pages"]
    pagina = next(iter(paginas.values()))
    if "imageinfo" not in pagina:
        raise RuntimeError(f"sin imageinfo para {titulo}")
    return pagina["imageinfo"][0]


def limpiar_html(texto: str) -> str:
    import re

    return re.sub(r"<[^>]+>", "", texto or "").strip()


def recomprimir(ruta: Path) -> None:
    try:
        from PIL import Image
    except ImportError:  # sin Pillow se conserva el archivo tal cual
        return
    with Image.open(ruta) as img:
        img = img.convert("RGB")
        img.save(ruta, "JPEG", quality=82, optimize=True)


def main() -> int:
    creditos = []
    fallos = 0
    for destino, titulo, ancho in MANIFIESTO:
        try:
            info = metadatos(titulo, ancho)
            em = info.get("extmetadata", {})
            licencia = limpiar_html(em.get("LicenseShortName", {}).get("value", ""))
            if not any(licencia.startswith(ok) for ok in LICENCIAS_OK):
                raise RuntimeError(f"licencia no permitida: {licencia}")
            autor = limpiar_html(em.get("Artist", {}).get("value", "")) or "sin autor"
            origen = info.get("descriptionurl", "")
            url = info.get("thumburl") or info["url"]
            ruta = DESTINO / destino
            ruta.write_bytes(pedir(url))
            recomprimir(ruta)
            kb = ruta.stat().st_size // 1024
            creditos.append(
                {
                    "archivo": destino,
                    "titulo": titulo.removeprefix("File:"),
                    "autor": autor,
                    "licencia": licencia,
                    "origen": origen,
                }
            )
            print(f"OK  {destino:28} {licencia:16} {kb} KB")
        except Exception as e:  # noqa: BLE001 - script de datos, se reporta y sigue
            fallos += 1
            print(f"FALLO {destino}: {e}", file=sys.stderr)
        time.sleep(1.5)  # cortesía con la API

    (DESTINO / "creditos.json").write_text(
        json.dumps(creditos, ensure_ascii=False, indent=2), encoding="utf-8"
    )
    lineas = [
        "# Créditos de las fotos",
        "",
        "Fotos descargadas de Wikimedia Commons por `descargar_fotos.py`",
        "(licencias verificadas contra la API; al reutilizarlas se debe",
        "mantener la atribución y, en CC BY-SA, compartir las adaptaciones",
        "bajo la misma licencia).",
        "",
        "| Archivo | Foto en Commons | Autor | Licencia |",
        "|---|---|---|---|",
    ]
    for c in creditos:
        enlace = f"[{c['titulo']}]({c['origen']})"
        lineas.append(f"| `{c['archivo']}` | {enlace} | {c['autor']} | {c['licencia']} |")
    (DESTINO / "creditos.md").write_text("\n".join(lineas) + "\n", encoding="utf-8")
    print(f"\n{len(creditos)} fotos listas, {fallos} fallos, créditos en creditos.md")
    return 1 if fallos else 0


if __name__ == "__main__":
    raise SystemExit(main())
