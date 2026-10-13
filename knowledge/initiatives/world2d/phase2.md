# Phase 2 — Tile worlds (2026-10-13)

User guide: [docs/2d.md](../../../docs/2d.md) (Tile maps, Parallax2D, Canvas layers).

## 1. Assets and documents

- **TileSet** is an authored asset (`assets::Kind::TileSet`, `Settings::Authored`, JSON
  payload; `src/tileset.rs`). One atlas texture cut by tile size, margin and separation;
  tiles are addressed by atlas column and row and carry collision (none, full or a
  rectangle), one-way, probability, animation (frames to the right, optional wrap,
  fps), terrain set, centre terrain and eight peering bits, and custom data values.
  Terrain sets, custom data layers and patterns live in the TileSet.
- **TileMapLayer** stores its TileSet and its cells as one base64 string (8 bytes per
  cell: x, y as i16, atlas column, row, flip flags) so a large map stays one line of the
  map document. `src/tilemap.rs` encodes, paints (lines, rectangles, flood fill, stamps,
  flips and quarter turns) and cooks.
- Maps resolve the TileSets they use into `Scene::tilesets` inside `texture::resolve`
  (a TileSet names the atlas texture the bank must load), so every place that resolves
  textures — editor, builds, Blueprint templates, native play — sees them.
- **Flip semantics** (shared by the editor preview, the PSX renderer, the PC runner and
  tile collision): a cell maps screen to texels by transposing first, then mirroring
  the texel axes. A quarter turn clockwise maps (t, h, v) to (true, h, !v) for an
  upright cell and (false, !h, v) for a transposed one.

## 2. Terrains

`tileset::solve` re-solves the painted cells and their 8-neighbourhood. Each terrain
cell wants a side bit equal to the neighbour's terrain and a corner bit equal to its own
terrain only when both sides and the diagonal share it (otherwise the breaking
terrain). The tile of the same terrain with the most matching bits (only the bits the
set's mode uses: corners and sides, sides or corners) wins; ties pick by tile
probability with deterministic per-cell noise, so painting is reproducible. Tests cover
the 47-tile blob (every canonical neighbourhood has an exact match), sides mode and
neighbour updates on erase.

## 3. Runtime (`runtime/world2d_tiles.hpp`)

| Piece | Contract |
| --- | --- |
| `TileSetData` / `TileInfo` | Cooked once per bank per TileSet: texels of frame 0, collision kind and rectangle, one-way flag, animation frames, wrap and stride, frame time; custom data tile-major |
| `TileMapLayerComponent` | 16 x 16-cell chunks of `uint16_t` (0 empty, tile + 1 in 13 bits, flip_h, flip_v, transpose) plus a grid of chunk indices; only used chunks are cooked. Grid anchored at the owner's world translation |
| Runtime edits | `set_cell` / `erase_cell` write a sorted table of `EPOK_WORLD2D_TILE_EDITS` entries in the service (binary search per lookup); restoring the cooked tile frees the entry; a destroyed layer forgets its edits; loading a map clears the table |
| `Parallax2DComponent` | After the cameras: root = base + (camera centre - reference) * (1 - scroll scale) + drift; `reference` is the editor's starting view (`world2d::start_view`), so the authored layout is what the game shows at start; drift wraps by the repeat size |
| `CanvasLayer2DComponent` | Nearest ancestor (through attachments) decides the canvas; screen canvases project through an identity camera centred on the display |
| `CanvasModulate2DComponent` | Tints its canvas; resolved once per frame |

The service (`world2d_service.hpp`) gathers solid tiles into temporary collision
entries after the colliders (`EPOK_WORLD2D_TILE_PROBES`): the cells under a movement's
sweep, or the cells a ray crosses in grid-traversal order, so tiles and colliders
resolve through the one axis-separated solver (no seam snagging; one-way tiles behave
like one-way platforms). `overlap_box_2d` reports each tile layer once.
`CharacterBody2D::last_collision_is_tile / last_tile_x / last_tile_y` name the cell.

Drawing: one draw item per tile layer (plus parallax copies), sorted by canvas layer
(a fifth radix pass) then the z/y key. `each_tile` expands a layer over the visible cell
range only; the PSX renderer emits `Sprite16x16` / `Sprite8x8` / sized `Sprite`
rectangles for unflipped tiles at identity projection and textured quads otherwise; the
PC runner draws the same tiles as quads. The cooked arena adds a screen of tiles per
layer (20 bytes each).

Builds without tile layers define one probe and one edit, so their RAM is unchanged.

## 4. Editor

- `tilemap_editor.rs`: tile tools (select, paint, line, rectangle, bucket, picker,
  eraser; random scatter; flips and turns; single-key shortcuts that only
  apply while painting), right-drag erasing, ghost preview of the brush, cell grid,
  the resizable palette (Tiles / Patterns / Terrains tabs), the New TileSet dialog
  (tiles auto-created for cells with visible pixels) and the TileMapLayer Inspector band.
  Painting starts when a layer is selected; Q, Esc or Done return to the node tools.
  Other tile layers are dimmed while one is painted.
- `tileset_editor.rs`: the TileSet window — texture, tile size, margin and separation;
  modes Tiles, Collision (shape brushes and one-way), Terrains (3 x 3 zones per tile,
  Shift fills, 3x3 block assignment), Animation (live preview) and Custom data; local
  undo/redo; every edit saved at once and pushed to the open map.
- `world2d_editor.rs`: tile layers, parallax repeats (faded copies), canvas tints and
  the screen-canvas frame in the 2D view; tile layers pick by painted cells only;
  presets and Inspector bands for Parallax2D, CanvasLayer2D and CanvasModulate2D.
- Content Browser: Create > TileSet and double-click opens the TileSet editor.

## 5. Validation

| Check | Result |
| --- | --- |
| `tests/runtime/world2d_service.cpp` | cells, atlas lookups, custom data, map conversion, runtime edits (bounded, freed, forgotten); landing, walls, seams, one-way tiles, disabled collision; rays (nearest tile, one-way from below), overlaps; tile drawing, culling, animation, flips; parallax follow, drift and repeats; canvas order, screen space and modulate |
| `cargo test` | TileSet geometry, auto-created tiles, round trips, blob and sides terrains, painting (stamps, random, erase, terrain connect/erase), cooking (shared tables, chunks), capacities, start view |
| PSX builds | Platformer 2D (tile level, parallax, HUD canvas) in C++, Lua and Blueprint |
| `tests/integration/verify_world2d.py` | all three flavors played in PCSX-Redux |
| Native PC Play | draws the same frame |
