"""Subset MDI 7.4.47's function icon for ImGui (fonttools 4.60.1)."""
import argparse
import hashlib
from pathlib import Path

from fontTools import subset
from fontTools.ttLib import TTFont

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("source", type=Path, help="Original materialdesignicons-webfont.ttf")
args = parser.parse_args()
expected = "61e8aba5a4e981fe22cf7c8e8bcdbea00476e75c62c37f01bf7ee33361d68428"
if hashlib.sha256(args.source.read_bytes()).hexdigest() != expected:
    raise SystemExit("Expected the original @mdi/font 7.4.47 TTF")
font = TTFont(args.source, recalcTimestamp=False)
glyph = font.getBestCmap()[0xF0295]
for table in font["cmap"].tables:
    if table.isUnicode():
        table.cmap[0xE900] = glyph
options = subset.Options()
options.recalc_timestamp = False
worker = subset.Subsetter(options=options)
worker.populate(unicodes=[0xE900])
worker.subset(font)
destination = Path(__file__).resolve().parents[1] / "resources/editor/blueprint-function-icon.ttf"
font.save(destination)
print(f"Wrote {destination.name}: unchanged MDI function design, mapped to U+E900")
