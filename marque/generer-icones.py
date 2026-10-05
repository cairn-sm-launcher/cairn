#!/usr/bin/env python3
"""Genere le jeu d'icones de Cairn a partir des SVG de marque/.

    pip install cairosvg pillow
    python3 marque/generer-icones.py

Ecrit dans src-tauri/icons/. Les PNG viennent tous de cairn.svg. Le .ico prend
cairn-16.svg pour ses entrees de 24 px et moins : en dessous de cette taille le
double contour exterieur n'a plus assez de pixels, et la version 16 px dessinee
par Xirdrak a des contours proportionnellement plus epais qui tiennent mieux.
"""

from pathlib import Path

import cairosvg
from PIL import Image

RACINE = Path(__file__).resolve().parent.parent
MARQUE = RACINE / "marque"
ICONES = RACINE / "src-tauri" / "icons"

GRAND = MARQUE / "cairn.svg"
PETIT = MARQUE / "cairn-16.svg"

# Les quatre premiers sont declares dans tauri.conf.json. Les autres servent
# aux magasins d'applications et a la documentation.
PNG = [
    ("32x32.png", 32),
    ("128x128.png", 128),
    ("128x128@2x.png", 256),
    ("256x256.png", 256),
    ("512x512.png", 512),
    ("icon.png", 512),
]

ICO = [16, 20, 24, 32, 48, 64, 128, 256]
SEUIL_PETIT = 24


def rendre(source: Path, cible: Path, taille: int) -> None:
    cairosvg.svg2png(
        url=str(source),
        write_to=str(cible),
        output_width=taille,
        output_height=taille,
        background_color=None,
    )


def main() -> None:
    ICONES.mkdir(parents=True, exist_ok=True)
    temporaire = ICONES / ".ico"
    temporaire.mkdir(exist_ok=True)

    for nom, taille in PNG:
        rendre(GRAND, ICONES / nom, taille)
        print(f"  {nom:18} {taille} px")

    images = []
    for taille in ICO:
        source = PETIT if taille <= SEUIL_PETIT else GRAND
        chemin = temporaire / f"{taille}.png"
        rendre(source, chemin, taille)
        images.append(Image.open(chemin).convert("RGBA"))

    images[-1].save(
        ICONES / "icon.ico",
        format="ICO",
        sizes=[(t, t) for t in ICO],
        append_images=images[:-1],
    )
    print(f"  {'icon.ico':18} {', '.join(str(t) for t in ICO)} px")

    for image in images:
        image.close()
    for fichier in temporaire.iterdir():
        fichier.unlink()
    temporaire.rmdir()


if __name__ == "__main__":
    main()
