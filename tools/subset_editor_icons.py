"""Keep the Codicons glyphs referenced by the Rust editor (requires fonttools 4.60.1)."""
import argparse
import hashlib
import pathlib
import re

from fontTools import subset
from fontTools.ttLib import TTFont


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=pathlib.Path, help="Original Codicons 0.0.46-24 TTF")
    parser.add_argument("--output", type=pathlib.Path)
    args = parser.parse_args()
    source_hash = hashlib.sha256(args.source.read_bytes()).hexdigest()
    if source_hash != "3819e4ae4b87350e7c37a5d8f24e71ada2f1f2ee58f7ce5ebc1f88e3c8c38c80":
        raise SystemExit("Source does not match the original Codicons 0.0.46-24 font")
    root = pathlib.Path(__file__).resolve().parents[1]
    destination = args.output or root / "resources/editor/codicon.ttf"
    points = set()
    for source in (root / "src").rglob("*.rs"):
        text = source.read_text(encoding="utf-8")
        points.update(int(value, 16) for value in re.findall(r"\\u\{([0-9a-fA-F]+)\}", text))
        points.update(ord(character) for character in text)
    points = {point for point in points if 0xEA60 <= point <= 0xEDFF}
    if not points:
        raise SystemExit("No Codicons references found in Rust sources")
    font = TTFont(args.source, recalcTimestamp=False)
    missing = points - font.getBestCmap().keys()
    if missing:
        raise SystemExit(f"Source font is missing glyphs: {sorted(missing)}")
    options = subset.Options()
    options.recalc_timestamp = False
    options.ignore_missing_unicodes = False
    worker = subset.Subsetter(options=options)
    worker.populate(unicodes=points)
    worker.subset(font)
    font.save(destination)
    font.close()
    print(f"Wrote {len(points)} Codicons glyphs ({destination.stat().st_size} bytes)")
    print("Source SHA-256:", source_hash)
    print("Keep the Microsoft attribution and document the subset modification")


if __name__ == "__main__":
    main()
