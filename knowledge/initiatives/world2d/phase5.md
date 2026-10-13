# Phase 5 — Showcase template (2026-10-13)

User guide: [docs/2d.md](../../../docs/2d.md) (The Platformer 2D template).

The Platformer 2D template becomes a showcase that stays a template: more depth and
life on screen, every part an ordinary node, grouped by purpose so a game can start by
deleting what it does not need. No gameplay script changed: the three flavors (C++,
Lua, Blueprint) still share one scene.

| Change | Detail |
| --- | --- |
| Hierarchy | Groups Scenery, Level, Characters, Props, Coins, Effects; HUD canvases at the end |
| Sky | `SkyLayer` (CanvasLayer2D at -20, screen space): a 16 x 64 gradient stretched x40 / x7.5 (covers up to 640 x 480) and an additive sun with its glow baked in |
| Depth | `Treeline` Parallax2D at 0.7 between the hills and the level; `Foreground` bushes at 1.25, z 20, in front of everything |
| Ambience | `Petals`: a Particles2D on its own screen canvas (layer 2), drifting down across the view |
| HUD | `HelpCanvas` / `ControlsHint`: a centred line of help at the top, built-in font |
| Fix | The built-in font's space no longer draws a glyph: its cell holds the atlas's CLUT by design, and every space on the console showed a few stray texels. One primitive fewer per space |

## Validation

| Check | Result |
| --- | --- |
| `cargo test` | every flavor creates a valid template scene |
| `tests/runtime/hud_layout.cpp` | the built-in font advances over spaces without emitting them |
| PSX | Platformer 2D in C++, Lua and Blueprint in PCSX-Redux; the help line renders clean |
| Static RAM | 1.49 MiB of 2 MiB (from 1.41 MiB): the new nodes' pools, scene data and four small textures |
