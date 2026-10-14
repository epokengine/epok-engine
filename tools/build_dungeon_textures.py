"""Draw the Dungeon Generator's built-in sci-fi surface textures.

Each texture covers one face of the generated geometry exactly once (the console
clamps UVs, so nothing repeats inside a face): a floor or ceiling cell, a wall panel
from the floor to the ceiling, a doorway beam. They are drawn at their final PSX size
from a few flat colours, so the importer's palette keeps them crisp. Everything is
deterministic; rerun the tool after changing it and commit the PNGs:

    python tools/build_dungeon_textures.py resources/dungeon
"""
import pathlib
import sys

from PIL import Image

STEEL = (112, 124, 140)
STEEL_LIGHT = (168, 180, 194)
STEEL_DARK = (66, 74, 88)
SHADOW = (36, 40, 52)
CYAN = (70, 214, 230)
CYAN_DARK = (34, 118, 136)
AMBER = (232, 172, 52)
BLACK = (24, 24, 30)


def noise(x, y, seed):
    h = (x * 374761393 + y * 668265263 + seed * 2246822519) & 0xFFFFFFFF
    h = ((h ^ (h >> 13)) * 1274126177) & 0xFFFFFFFF
    return (h ^ (h >> 16)) & 0xFFFFFFFF


def shade(colour, amount):
    return tuple(max(0, min(255, c + amount)) for c in colour)


def grain(img, seed, strength=6):
    """A little wear so flat panels do not band."""
    px = img.load()
    for y in range(img.height):
        for x in range(img.width):
            n = noise(x, y, seed) % (2 * strength + 1) - strength
            px[x, y] = shade(px[x, y], n)


def rect(img, x0, y0, x1, y1, colour):
    for y in range(max(0, y0), min(img.height, y1)):
        for x in range(max(0, x0), min(img.width, x1)):
            img.putpixel((x, y), colour)


def bevel(img, x0, y0, x1, y1, face, light, dark):
    """A raised panel: lit top and left edges, shadowed bottom and right."""
    rect(img, x0, y0, x1, y1, face)
    rect(img, x0, y0, x1, y0 + 1, light)
    rect(img, x0, y0, x0 + 1, y1, light)
    rect(img, x0, y1 - 1, x1, y1, dark)
    rect(img, x1 - 1, y0, x1, y1, dark)


def bolt(img, x, y):
    img.putpixel((x, y), STEEL_LIGHT)
    img.putpixel((x + 1, y + 1), SHADOW)


def floor():
    img = Image.new("RGB", (64, 64), SHADOW)
    # Four plates separated by dark seams, each with a tread of raised diamonds.
    for py in (0, 32):
        for px in (0, 32):
            bevel(img, px + 1, py + 1, px + 31, py + 31, STEEL, STEEL_LIGHT, STEEL_DARK)
            for y in range(py + 5, py + 27, 4):
                for x in range(px + 5 + ((y - py) // 4 % 2) * 2, px + 27, 4):
                    img.putpixel((x, y), STEEL_LIGHT)
                    img.putpixel((x + 1, y + 1), STEEL_DARK)
            for bx, by in ((px + 3, py + 3), (px + 27, py + 3), (px + 3, py + 27), (px + 27, py + 27)):
                bolt(img, bx, by)
    # A small lit hatch where the four plates meet.
    rect(img, 29, 29, 35, 35, SHADOW)
    rect(img, 30, 30, 34, 34, CYAN_DARK)
    rect(img, 31, 31, 33, 33, CYAN)
    grain(img, 1, 4)
    return img


def corridor():
    img = Image.new("RGB", (64, 64), BLACK)
    # A grating between two rails with hazard edges.
    for y in range(4, 60, 6):
        bevel(img, 10, y, 54, y + 4, STEEL_DARK, STEEL, SHADOW)
    for x0 in (0, 54):
        bevel(img, x0, 0, x0 + 10, 64, STEEL, STEEL_LIGHT, STEEL_DARK)
        for y in range(64):
            for x in range(x0 + 2, x0 + 8):
                img.putpixel((x, y), AMBER if (x + y) % 8 < 4 else BLACK)
    grain(img, 2, 4)
    return img


def wall():
    img = Image.new("RGB", (64, 128), SHADOW)
    # Skirting with vents at the bottom, a lit band near the top, and two tall
    # recessed panels framed by a pillar on each side.
    bevel(img, 0, 112, 64, 128, STEEL_DARK, STEEL, SHADOW)
    for x in range(6, 58, 6):
        rect(img, x, 116, x + 3, 124, BLACK)
    bevel(img, 0, 0, 64, 10, STEEL_DARK, STEEL, SHADOW)
    for x in range(2, 64, 8):
        rect(img, x, 3, x + 4, 7, STEEL)
    rect(img, 0, 10, 64, 16, BLACK)
    rect(img, 0, 11, 64, 15, CYAN_DARK)
    rect(img, 0, 12, 64, 14, CYAN)
    for x0 in (0, 56):
        bevel(img, x0, 16, x0 + 8, 112, STEEL, STEEL_LIGHT, STEEL_DARK)
        for y in range(22, 110, 12):
            bolt(img, x0 + 3, y)
    for x0 in (8, 32):
        bevel(img, x0, 16, x0 + 24, 112, STEEL_DARK, SHADOW, STEEL)
        bevel(img, x0 + 3, 20, x0 + 21, 108, STEEL, STEEL_LIGHT, STEEL_DARK)
        rect(img, x0 + 7, 44, x0 + 17, 46, STEEL_DARK)
        rect(img, x0 + 7, 80, x0 + 17, 82, STEEL_DARK)
        # A small status light on each panel.
        rect(img, x0 + 10, 26, x0 + 14, 29, CYAN if x0 == 8 else AMBER)
    grain(img, 3, 5)
    return img


def ceiling():
    img = Image.new("RGB", (64, 64), BLACK)
    dark = shade(STEEL_DARK, -14)
    for py in (0, 32):
        for px in (0, 32):
            bevel(img, px + 2, py + 2, px + 30, py + 30, dark, STEEL_DARK, BLACK)
    rect(img, 28, 28, 36, 36, CYAN_DARK)
    rect(img, 30, 30, 34, 34, CYAN)
    grain(img, 4, 3)
    return img


def doorway():
    img = Image.new("RGB", (64, 64), STEEL_DARK)
    # A frame beam: hazard stripes between two steel rails.
    bevel(img, 0, 0, 64, 12, STEEL, STEEL_LIGHT, STEEL_DARK)
    bevel(img, 0, 52, 64, 64, STEEL, STEEL_LIGHT, STEEL_DARK)
    for y in range(12, 52):
        for x in range(64):
            img.putpixel((x, y), AMBER if (x + y) % 16 < 8 else BLACK)
    grain(img, 5, 4)
    return img


RED = (200, 56, 48)
TEAL = (64, 120, 128)
TEAL_LIGHT = (104, 168, 172)
TEAL_DARK = (36, 70, 78)


def floor_grid():
    """Room variant: a grid of small teal-tinted panels."""
    img = Image.new("RGB", (64, 64), SHADOW)
    for py in range(0, 64, 16):
        for px in range(0, 64, 16):
            bevel(img, px + 1, py + 1, px + 15, py + 15, TEAL, TEAL_LIGHT, TEAL_DARK)
            rect(img, px + 6, py + 6, px + 10, py + 10, TEAL_DARK)
    rect(img, 30, 30, 34, 34, CYAN)
    grain(img, 6, 4)
    return img


def floor_hazard():
    """Room variant: a dark plate framed in amber with a central grate."""
    img = Image.new("RGB", (64, 64), STEEL_DARK)
    bevel(img, 1, 1, 63, 63, STEEL_DARK, STEEL, SHADOW)
    for k in range(4):
        rect(img, 4 + k, 4 + k, 60 - k, 5 + k, AMBER if k % 2 == 0 else BLACK)
        rect(img, 4 + k, 59 - k, 60 - k, 60 - k, AMBER if k % 2 == 0 else BLACK)
        rect(img, 4 + k, 4 + k, 5 + k, 60 - k, AMBER if k % 2 == 0 else BLACK)
        rect(img, 59 - k, 4 + k, 60 - k, 60 - k, AMBER if k % 2 == 0 else BLACK)
    rect(img, 16, 16, 48, 48, BLACK)
    for y in range(18, 46, 4):
        bevel(img, 18, y, 46, y + 2, STEEL, STEEL_LIGHT, STEEL_DARK)
    for bx, by in ((10, 10), (52, 10), (10, 52), (52, 52)):
        bolt(img, bx, by)
    grain(img, 7, 4)
    return img


def wall_ribbed():
    """Room variant: narrow vertical ribs and an amber light band."""
    img = Image.new("RGB", (64, 128), SHADOW)
    bevel(img, 0, 112, 64, 128, STEEL_DARK, STEEL, SHADOW)
    for x in range(4, 60, 8):
        rect(img, x, 117, x + 4, 123, BLACK)
    bevel(img, 0, 0, 64, 12, STEEL, STEEL_LIGHT, STEEL_DARK)
    rect(img, 0, 12, 64, 18, BLACK)
    rect(img, 0, 13, 64, 17, (150, 104, 30))
    rect(img, 0, 14, 64, 16, AMBER)
    for x in range(0, 64, 16):
        bevel(img, x + 1, 18, x + 15, 112, STEEL, STEEL_LIGHT, STEEL_DARK)
        rect(img, x + 6, 30, x + 10, 100, STEEL_DARK)
        rect(img, x + 7, 31, x + 9, 99, SHADOW)
    grain(img, 8, 3)
    return img


def wall_pipes():
    """Room variant: plain panels above a run of pipes and a red warning band."""
    img = Image.new("RGB", (64, 128), SHADOW)
    bevel(img, 0, 112, 64, 128, STEEL_DARK, STEEL, SHADOW)
    bevel(img, 0, 0, 64, 10, STEEL_DARK, STEEL, SHADOW)
    for x0 in (0, 32):
        bevel(img, x0 + 1, 10, x0 + 31, 62, STEEL, STEEL_LIGHT, STEEL_DARK)
        rect(img, x0 + 8, 22, x0 + 24, 24, STEEL_DARK)
    for i, y in enumerate((64, 72, 80)):
        rect(img, 0, y, 64, y + 6, STEEL_DARK)
        rect(img, 0, y + 1, 64, y + 2, STEEL_LIGHT)
        rect(img, 0, y + 5, 64, y + 6, BLACK)
        for x in range(6 + i * 7, 64, 21):
            rect(img, x, y, x + 2, y + 6, STEEL)
    rect(img, 0, 88, 64, 92, BLACK)
    for x in range(64):
        for y in range(92, 110):
            img.putpixel((x, y), RED if (x + y) % 12 < 6 else SHADOW)
    rect(img, 0, 110, 64, 112, BLACK)
    grain(img, 9, 5)
    return img


def column():
    """A pilaster: a footing, a grooved shaft and a lit capital."""
    img = Image.new("RGB", (32, 128), SHADOW)
    bevel(img, 0, 110, 32, 128, STEEL, STEEL_LIGHT, STEEL_DARK)
    bevel(img, 2, 18, 30, 110, STEEL, STEEL_LIGHT, STEEL_DARK)
    for x in (8, 15, 22):
        rect(img, x, 24, x + 2, 104, STEEL_DARK)
    bevel(img, 0, 0, 32, 18, STEEL_DARK, STEEL, SHADOW)
    rect(img, 6, 6, 26, 12, CYAN_DARK)
    rect(img, 8, 8, 24, 10, CYAN)
    grain(img, 10, 4)
    return img


TEXTURES = {
    "floor": floor,
    "floor_grid": floor_grid,
    "floor_hazard": floor_hazard,
    "corridor": corridor,
    "wall": wall,
    "wall_ribbed": wall_ribbed,
    "wall_pipes": wall_pipes,
    "column": column,
    "ceiling": ceiling,
    "doorway": doorway,
}


def main():
    out = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else "resources/dungeon")
    out.mkdir(parents=True, exist_ok=True)
    for name, draw in TEXTURES.items():
        path = out / f"{name}.png"
        draw().save(path, optimize=True)
        print(path)


if __name__ == "__main__":
    main()
