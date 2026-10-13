# Phase 2: Dungeon Styles

Delivered on `feature/dungeon-generator`.

## What shipped

| Area | Delivered |
| --- | --- |
| Spots | The layout now names nine spot kinds: Floor, Wall, Door, Doorway, Solid, Corner, Room, Spawn and Goal, each with a local frame (floor level, +Z into the floor cell or room). |
| Style model | `src/dungeon_style.rs`: a `DungeonStyle` authored asset holding spot nodes and piece nodes. Pieces are tried left to right; each has chance, stop-here (consume), area filter (anywhere, rooms, corridors), limit, and offset/rotation/scale. Each piece draws from its own seeded stream, so editing one never reshuffles another. Built-in styles reproduce the phase 1 look. |
| Pieces | Blockout (floor tile, wall panel, wall top, doorway beam, corner pillar, in a colour or texture), Mesh (EditableMeshes merged into the sections with their materials and correct winding under mirroring), Actor (any placeable class, Blueprint templates included, placed under the generator with stable identities), Light (baked point light), Terrain and Tile (2D, with one decoration layer per TileSet). |
| Bake | The 3D and 2D bakes are driven entirely by the style's placements; collision, navigation and markers still come from the layout. A changed style marks the output stale through a digest stored with the output. |
| Style editor | `src/dungeon_style_editor.rs`: canvas of spot and piece nodes with links, order numbers in each piece header, drag to reorder, drag from a spot's dot to add, drop on another spot to move, searchable Add menu, zoom and frame, Delete, its own undo/redo, autosave, amber problems on nodes, details panel with asset/class/terrain pickers and a tile atlas picker, placement counts in the open map, and live regeneration of the map's dungeons that use the style (one coalesced undo step). |
| Integration | Content Browser kind, icon, filter, **Add > Dungeon Style (3D)/(2D)**, double-click to open, duplication; generator band **Style** combo with **New** and **Edit**; `place_class` extracted from the Actor picker and shared. |
| Automation | MCP `dungeon_generate` takes `settings.style` and `new_style`; `asset_document` reads and validates style documents; CLI `--dungeon-new-style`; screenshot flag `--open-dungeon-style`; the `dungeon_visual_fixture` writes a styled Third Person project. |

## Verification

| Check | Result |
| --- | --- |
| `cargo test --locked` | 985 passed, 0 failed |
| Style tests | built-in round trip and validation, canvas order and consumption, seeded chance and limit independence, frames facing into the floor, merged corner meshes, three limited lights replacing the default sun, stale detection after a style edit, 3D/2D mismatch refusal, Actor pieces refused without a class model, 2D decoration layer above the tiles, MCP style creation and document editing, editor remove/undo and framing |
| `python tests/runtime/verify_spatial.py` | all host tests pass |
| `cargo clippy --all-targets` | no diagnostic in new files or changed lines |
| `--generate-dungeon Dungeon` on the fixture | 19 actors: geometry, collision, four navigation volumes, spawn, goal, ten room lights and the Actor piece at the goal |
| `--build-psx` | styled Third Person project: 1.68 MiB static RAM |
| PCSX-Redux | a room lit by its baked point light with a merged crate, after walking from the spawn |

## Limits and follow-ups

- RAM is checked by Build, not by the budget meter. A style with many lights and meshes
  on top of a large map can exceed it; the fixture overflowed by 2.4 KiB until the
  template's arena was deleted rather than deactivated (inactive actors still cook).
- Actor pieces rotate around Y only by the spot's facing; their own X and Z rotations
  are added as Euler angles.
- Mesh pieces have no collision of their own; give blocking props an Actor piece.
