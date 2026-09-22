# Controller artwork

`psx-controller.svg` is original Epok artwork, distributed under the repository's
MIT license. `psx-controller.png` is its embedded, transparent 1500 x 975 render.
This is a hand-authored vector illustration, not an AI-generated image.

The source uses a 1000 x 650 viewBox. Button hit regions in `src/controls_ui.rs`
refer to these coordinates and scale with the image. Face symbols and arrows
are vector paths; they do not rely on the editor's font glyph coverage.

To regenerate the PNG with Python and `resvg-py`:

```python
from pathlib import Path
import resvg_py

source = Path("resources/editor/psx-controller.svg")
source.with_suffix(".png").write_bytes(
    resvg_py.svg_to_bytes(svg_string=source.read_text(encoding="utf-8"))
)
```

The SVG's small wordmarks use Arial (with a sans-serif fallback). Runtime needs
only the embedded PNG and has no SVG renderer or extra font dependency.
