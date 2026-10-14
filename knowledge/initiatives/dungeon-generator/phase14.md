# Phase 14: Generation volumes

Delivered on `features/dungeon-minimap-seeds-volumes`.

## What shipped

| Area | Delivered |
| --- | --- |
| Data | `Generator.volumes`: up to 32 rectangles of grid cells, each Keep out, Only here, Room here, Mirror left-right or Mirror front-back (`dungeon_volumes.rs`). They are generator settings: part of its fingerprint, never cooked. `blocked()` closes the cells inside Keep out volumes and, when any Only here volume exists, those outside all of them; Room here cells stay open. |
| Rooms and corridors | Room here volumes become rooms first (one cell off the grid's edge); scattered rooms and their ring stay off closed cells; corridors never route through them; each Mirror volume drops the scattered rooms reaching its second half and repeats those of its first half there when they fit. With no volume the random stream and the layout are unchanged. |
| Recipes | A lattice slot that touches a closed cell is blocked: paths and branches grow around it (`Graph.blocked`, filtered in `neighbours`, the main path starting on an open slot), so no room or corridor uses it. With volumes the layout keeps the recipe's whole grid (`Recipe::grid_size`, `layout_on_grid`) instead of cropping, so volumes stay where they were drawn. Room here and Mirror shape rooms-and-corridors layouts only; the band says so. |
| Editor | Volumes section in the band (count, kinds editable, remove, **Draw volumes...**) and the Volumes window: the grid from above with the layout, recipe lattice slots (closed ones shaded), the volumes in their colours (Keep out crossed, mirror lines), drawing by dragging with the selected kind, moving, resizing by the corner, right-click to remove, the layout following each change, Generate. The Scene view outlines volumes on the grid. |
| Orientation | In 3D the Volumes window, the Dungeon Room editor's floor plan and the console minimap now put +Z at the top, as the Scene view from above and the camera at the start do (the minimap of phase 12 had it at the bottom, so walking forward moved the player down the map). 2D keeps the screen's own up. |

## Verification

| Check | Result |
| --- | --- |
| Tests | Blocking rules and validation; on 10 seeds of a rooms-and-corridors grid nothing walks into a Keep out volume, everything stays inside an Only here one, a Room here volume holds its room, and with a whole-grid mirror every room of the second half has its first-half twin; on 12 seeds of the built-in recipe the slots touching two Keep out volumes stay empty on the uncropped grid, and with no volume the layout equals the plain one; the minimap's rows turn over in 3D with a negative step. |
| Editor | The template's recipe with a Keep out volume in the middle: four closed slots shaded, the paths around them, the same layout generated in the Scene view with the volume outlined. A 48x40 rooms-and-corridors grid mirrored left to right around a forced middle room with two corners kept out: a symmetric dungeon. |
| PCSX-Redux | The minimap again after the turn: walking forward from the spawn, the next room appears above it. |

## Limits and follow-ups

- Volumes are drawn in the Volumes window, not with gizmos in the Scene view.
- Mirroring repeats rooms; corridors are routed afterwards and are usually, not always,
  symmetric.
- A recipe dungeon with volumes covers its whole lattice, which may reach further from the
  generator than a cropped one.
