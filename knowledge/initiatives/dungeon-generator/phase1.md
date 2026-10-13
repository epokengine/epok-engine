# Phase 1: generator core and rooms-and-corridors layouts

Delivered on `feature/dungeon-generator`.

## What shipped

| Area | Delivered |
| --- | --- |
| Random stream | `src/dungeon_random.rs` and `runtime/dungeon_random.hpp`: xoshiro128** seeded through SplitMix32, unbiased bounded draws, independent per-stage streams. Both are checked against the same golden values. |
| Layout model | `src/dungeon.rs`: grid of cells (empty, room, corridor), rooms, links, spawn and goal, and spots (Floor, Wall, Door, Spawn, Goal) naming where content belongs. Settings are validated with actionable messages and fingerprinted to detect stale output. |
| Rooms and corridors | `src/dungeon_rooms.rs`: spaced rectangular rooms, Gabriel-graph candidates, a spanning tree plus loops by percentage, A* corridors that keep a wall cell from rooms they do not join, prefer straight runs and merge, repair of blocked links, spawn and goal at the largest hop distance. |
| 3D bake | `src/dungeon_bake.rs`: walls are the solid cells around floors; one face per floor cell and wall side (no T-junction cracks), merged wall tops, doorway beams; EditableMesh sections split to stay within 3,400 faces and ±120 units; one compound collider; navigation volumes tiled within the ±64 transform limit and spaced for 420 nodes; Spawn, Goal and a light when the map has none; optional move of an actor to the spawn. |
| 2D bake | Built-in dungeon TileSet (generated pixel art, Floor and Wall terrains, solid walls) or any TileSet with those terrains; one TileMapLayer solved with the TileSet's terrain rules; Spawn and Goal Node2D actors. |
| Compound collider | `Collider.boxes` (editor) and `ColliderT::boxes` (runtime): overlap, raycast, ground, move-and-slide and trigger pairs test the parts; the navigation bake and Scene view treat them as separate boxes. Translation-only owners, up to 2,048 boxes. |
| Editor | GameObject / Hierarchy **Dungeon Generator** and the searchable 2D node **DungeonGenerator**; the generator generates and frames the view on creation. Inspector band with Generate, New seed, settings with tooltips, amber warnings with one-click fixes, PSX budget meters and a summary; 3D layout overlay; dungeon icon in the Hierarchy; one undo step per generation. |
| Automation | MCP `dungeon_generate`; CLI `--generate-dungeon`, `--dungeon-seed`, `--dungeon-settings`; evidence flags `--frame-dungeon` and `--screenshot-input`. |
| Docs | `docs/dungeons.md`, plus the feature catalog, formats, MCP guide, README and testing guide. |

## Verification

| Check | Result |
| --- | --- |
| `cargo test --locked` | 975 passed, 0 failed |
| `cargo test --locked dungeon` | layouts over 120 seeds (reachability, doors, wall cells), cook of a generated 3D map with its compound collider, navigation bake under 512 nodes, regeneration determinism, clearing, spawn placement, refusals, 2D terrain painting and tile cooking, MCP create/regenerate/undo |
| `python tests/runtime/verify_spatial.py` | all host tests pass, including `compound_collision` and `dungeon_random` |
| `cargo fmt --all -- --check` | clean |
| `cargo clippy --all-targets -D warnings` | no diagnostic in new files or changed lines; the macOS toolchain reports pre-existing diagnostics elsewhere |
| `--build-psx` | Basic project with a 2D dungeon: 1.23 MiB static RAM. Third Person project with a 3D dungeon: 1.60 MiB static RAM. |
| PCSX-Redux | The Third Person player spawns in the spawn room, stands on the generated floor and stops against a generated wall when walking into it (`--screenshot-input "500:16,2600:0"`). |

## Limits and follow-ups

- The 2D view does not draw the layout overlay; the generated tiles show it instead.
- Generated meshes are rewritten on every generation; manual edits to them are lost.
  After an undo the scene returns but the mesh files keep the newer geometry, and the
  band reports the output as stale with a Regenerate button.
- Styles (phase 2) replace the built-in blockout materials and tiles.
