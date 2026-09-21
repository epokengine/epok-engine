"""Compose terrain tiles into one atlas the PSX texture importer accepts.

A terrain samples one tile per cell out of a single texture, because the
console has no multitexturing and a whole-terrain baked texture would give a
handful of texels per cell. So the ground is a grid of tiles in one image of
at most 256x256, and the cell picks which tile it shows.

Tile order is row-major, matching `terrain_compile::tile_uv`: tile 0 is the
top-left, then left to right, then down. Set the terrain's Atlas columns/rows
to the same grid.
"""
import argparse
import pathlib
import sys

from PIL import Image

MAX_ATLAS = 256


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("sources", nargs="+", type=pathlib.Path, help="Tile images, in tile order")
    parser.add_argument("--output", required=True, type=pathlib.Path)
    parser.add_argument("--cols", type=int, default=0, help="Default: square-ish grid")
    parser.add_argument("--rows", type=int, default=0)
    parser.add_argument(
        "--tile",
        type=int,
        default=0,
        help="Pixels per tile. Default: the largest that keeps the atlas within 256.",
    )
    args = parser.parse_args()

    count = len(args.sources)
    cols = args.cols or min(count, 2 if count <= 4 else 3 if count <= 9 else 4)
    rows = args.rows or -(-count // cols)
    if cols * rows < count:
        raise SystemExit(f"{count} tiles do not fit a {cols}x{rows} grid")

    tile = args.tile or MAX_ATLAS // max(cols, rows)
    if tile < 8:
        raise SystemExit(f"{cols}x{rows} leaves {tile} px per tile; use fewer tiles")
    if cols * tile > MAX_ATLAS or rows * tile > MAX_ATLAS:
        raise SystemExit(
            f"{cols}x{rows} tiles of {tile} px is {cols * tile}x{rows * tile}; "
            f"the importer accepts at most {MAX_ATLAS} per axis"
        )

    atlas = Image.new("RGB", (cols * tile, rows * tile))
    for index, source in enumerate(args.sources):
        with Image.open(source) as image:
            # LANCZOS keeps a hand-painted tile readable at 64-128 px; the
            # engine quantizes to a 256-colour CLUT afterwards.
            patch = image.convert("RGB").resize((tile, tile), Image.LANCZOS)
        atlas.paste(patch, ((index % cols) * tile, (index // cols) * tile))
        print(f"  tile {index}: {source.name} -> {tile}x{tile}")

    args.output.parent.mkdir(parents=True, exist_ok=True)
    atlas.save(args.output, optimize=True)
    colors = len(atlas.getcolors(maxcolors=1 << 24) or [])
    print(
        f"Wrote {args.output} ({atlas.width}x{atlas.height}, {cols}x{rows} tiles, "
        f"{args.output.stat().st_size} bytes, {colors} distinct colours)"
    )
    if colors > 255:
        print(
            "Note: the importer quantizes to a 256-entry CLUT with index 0 reserved,"
            " so expect some colour loss.",
            file=sys.stderr,
        )


if __name__ == "__main__":
    main()
