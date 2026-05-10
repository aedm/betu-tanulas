#!/usr/bin/env python3
"""Generate the PWA icon set under assets/icons/.

The app installs to the home screen via a webmanifest + apple-touch-icon
chain (DESIGN.md §24). Both Android (Chrome) and iOS need a clean square
PNG; iOS in particular has no SVG fallback path for the home-screen
icon. We commit the rendered PNGs so the bundle pipeline is a `cp`, not
a build dep on Pillow.

Run from the repo root: `python3 tools/gen_icons.py`. Idempotent (the
output is deterministic given a fixed font choice).

Design: a single uppercase "B" — for "Betűk" — centered on a
rounded-square tile with a subtle border, matching the in-app tile
visuals. No accents or digraphs (matches the v1 dictionary
constraint), reads at 32 px favicon size, and survives iOS's circular
mask on the home screen because the letterform sits well inside an
80%-radius safe zone.

Required: Pillow (`pip3 install --user Pillow`).
"""

from __future__ import annotations

import sys
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

REPO_ROOT = Path(__file__).resolve().parent.parent
OUT = REPO_ROOT / "assets" / "icons"

# Palette: matches assets/tailwind.input.css @theme.
BONE = (251, 250, 247, 255)         # --color-bone   (background)
INK = (31, 41, 55, 255)             # --color-ink    (letterform)
TILE_BORDER = (229, 231, 235, 255)  # --color-tile-border

# Render at 1024 then downsample for crisp small icons.
BASE = 1024
LETTER = "B"

# Output sizes. apple-touch-icon at 180 is the iOS recommendation;
# manifest spec wants at least one 192 + one 512; favicon at 32.
SIZES = [
    ("icon-512.png", 512),
    ("icon-192.png", 192),
    ("apple-touch-icon.png", 180),
    ("favicon-32.png", 32),
]

# Font search order. Prefer a bold system font with predictable metrics;
# fall back to Pillow's default bitmap font (uglier, but the script still
# runs on a fontless CI box).
FONT_CANDIDATES = [
    "/System/Library/Fonts/Supplemental/Arial Bold.ttf",
    "/Library/Fonts/Arial Bold.ttf",
    "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
    "/usr/share/fonts/truetype/liberation/LiberationSans-Bold.ttf",
]


def load_font(size_px: int) -> ImageFont.ImageFont:
    for path in FONT_CANDIDATES:
        if Path(path).exists():
            return ImageFont.truetype(path, size_px)
    print(
        f"warning: no bold TTF found; falling back to PIL default. "
        f"Install Arial / DejaVu / Liberation Sans Bold for a proper render.",
        file=sys.stderr,
    )
    return ImageFont.load_default()


def draw_base() -> Image.Image:
    """Draw the canonical 1024 x 1024 icon."""
    img = Image.new("RGBA", (BASE, BASE), BONE)
    d = ImageDraw.Draw(img)

    # Rounded-square tile border, 16 px stroke at 1024 px → ~1 px at 64 px.
    inset = int(BASE * 0.06)
    radius = int(BASE * 0.18)
    d.rounded_rectangle(
        (inset, inset, BASE - inset, BASE - inset),
        radius=radius,
        outline=TILE_BORDER,
        width=int(BASE * 0.016),
    )

    # Big bold letter, centered. Font size tuned so the letterform sits
    # well inside Android's circular mask + iOS's rounded-square.
    font = load_font(int(BASE * 0.62))
    bbox = d.textbbox((0, 0), LETTER, font=font, anchor="lt")
    w = bbox[2] - bbox[0]
    h = bbox[3] - bbox[1]
    # Glyph metrics include side bearings; recenter from the actual
    # rendered bounds rather than the nominal anchor box.
    x = (BASE - w) // 2 - bbox[0]
    y = (BASE - h) // 2 - bbox[1]
    d.text((x, y), LETTER, fill=INK, font=font)

    return img


def main() -> int:
    OUT.mkdir(parents=True, exist_ok=True)
    base = draw_base()
    for name, size in SIZES:
        out = base.resize((size, size), Image.LANCZOS)
        out.save(OUT / name, format="PNG", optimize=True)
        print(f"wrote {OUT / name} ({size}x{size})")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
