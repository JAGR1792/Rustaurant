"""Descarga las fuentes de Brindis (Cormorant Garamond + Inter) desde Google Fonts.

Ejecucion: python static/fuentes/descargar_fuentes.py
Salida:
  - .woff2 en esta misma carpeta (se commitean; la app solo los sirve).
  - static/css/fuentes.css con los @font-face apuntando a rutas locales.

Determinista: Google Fonts sirve los mismos archivos por URL; el script es
re-ejecutable y no depende de nada fuera de la libreria estandar (urllib).

Direccion visual "Le Jardin": serif editorial (Cormorant Garamond) para
titulos y sans neutra (Inter) para interfaz.
"""
import os
import re
import sys
import urllib.request

AQUI = os.path.dirname(os.path.abspath(__file__))
CSS_SALIDA = os.path.join(AQUI, "..", "css", "fuentes.css")

# Chrome moderno: la API de Google Fonts responde woff2 con este UA.
UA = (
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 "
    "(KHTML, like Gecko) Chrome/124.0 Safari/537.36"
)

FAMILIAS = (
    "family=Cormorant+Garamond:ital,wght@0,400;0,500;0,600;1,400"
    "&family=Inter:wght@400;500;600;700"
)
URL_CSS = f"https://fonts.googleapis.com/css2?{FAMILIAS}&display=swap"

SUBSETS = ("latin", "latin-ext")

# (comentario del subset, cuerpo del bloque @font-face)
BLOQUE = re.compile(r"/\*\s*([a-z-]+)\s*\*/\s*@font-face\s*\{(.*?)\}", re.S)


def pedir(url: str, binario: bool = False):
    req = urllib.request.Request(url, headers={"User-Agent": UA})
    with urllib.request.urlopen(req, timeout=30) as r:
        datos = r.read()
    return datos if binario else datos.decode("utf-8")


def slug(texto: str) -> str:
    return re.sub(r"[^a-z0-9]+", "-", texto.lower()).strip("-")


def main() -> int:
    print(f"GET {URL_CSS}")
    try:
        css = pedir(URL_CSS)
    except Exception as e:  # noqa: BLE001 - se reporta y se corta
        print(f"ERROR descargando el CSS: {e}")
        return 1

    salida = [
        "/* Fuentes self-hosted de Brindis (Le Jardin).",
        "   Generado por static/fuentes/descargar_fuentes.py — no editar a mano.",
        "   Cormorant Garamond (display) + Inter (interfaz), subsets latin/latin-ext. */",
    ]
    n = 0
    for subset, cuerpo in BLOQUE.findall(css):
        if subset not in SUBSETS:
            continue
        familia = re.search(r"font-family:\s*'([^']+)'", cuerpo).group(1)
        estilo = re.search(r"font-style:\s*(\w+)", cuerpo).group(1)
        peso = re.search(r"font-weight:\s*(\d+)", cuerpo).group(1)
        url = re.search(r"url\((https://[^)]+\.woff2)\)", cuerpo).group(1)
        rango = re.search(r"unicode-range:\s*([^;]+);", cuerpo)

        nombre = f"{slug(familia)}-{peso}"
        if estilo == "italic":
            nombre += "-italic"
        nombre += f"-{subset}.woff2"

        destino = os.path.join(AQUI, nombre)
        with open(destino, "wb") as f:
            f.write(pedir(url, binario=True))
        n += 1
        print(f"  {nombre}")

        salida.append("")
        salida.append("@font-face {")
        salida.append(f"  font-family: '{familia}';")
        salida.append(f"  font-style: {estilo};")
        salida.append(f"  font-weight: {peso};")
        salida.append("  font-display: swap;")
        salida.append(f"  src: url('/static/fuentes/{nombre}') format('woff2');")
        if rango:
            salida.append(f"  unicode-range: {rango.group(1)};")
        salida.append("}")

    if n == 0:
        print("ERROR: la API no devolvio bloques esperados.")
        return 1

    with open(os.path.normpath(CSS_SALIDA), "w", encoding="utf-8") as f:
        f.write("\n".join(salida) + "\n")
    print(f"OK: {n} archivos woff2 + {os.path.normpath(CSS_SALIDA)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
