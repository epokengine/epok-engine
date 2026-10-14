# Phase 11: Handmade rooms

Delivered on `features/dungeon-rooms-levels-queries`.

## What shipped

| Area | Delivered |
| --- | --- |
| Asset | `DungeonRoom` (`assets/Dungeons/Rooms/*.epokasset`, editor-only): the captured actor branch in room space (origin at the footprint's corner, +X and +Z, floor at 0), the cell size it was built for, its footprint (size and closed cells), doors (edge cell and side) and spots for keys and items. Captured from a selected root with **GameObject > Save Selection as Dungeon Room...** or the Hierarchy's **Save as Dungeon Room...**; empties named Door become doors. |
| Room editor | Cell size, size, a floor plan with the content by kind (static geometry, colliders, lights, actors), doors toggled outside edge cells, spots by click, closed cells by Shift-click, a summary of merged pieces and copied actors, **Update from selection**, Undo; every edit saves and regenerates the dungeons that use the room. |
| Recipes | A **Handmade rooms** step (paths, at most N rooms, the Dungeon Rooms to use). Each room of the paths takes a room that fits its slot in one of four quarter turns with a door towards every link, the least used first; corridors end at those doors, unused doors are walled by the style, the room's spots take its keys and items first, and a room that nothing fits is generated with a note. The recipe digest folds in the rooms, so editing one makes its dungeons stale. |
| 3D bake | Static pieces (plain meshes, cubes, solid colliders) merge into the geometry sections and the compound collider, exact for quarter turns; cube faces resting on the floor and looking down are left out. Lights and actors with behaviour are copied under the generator with identities derived from the generator, the room and the piece. Rooms built for another cell size are scaled; rooms stand on their level. The budget meter counts the merged quads and copied actors. |
| 2D | Dungeon Rooms hold 3D content: a 2D dungeon draws handmade rooms as generated rooms of their shape and notes it. |
| Template | The Dungeon template builds a Reactor room (an Editable Mesh shell with doorways on four sides, a core, four consoles, a light) in `assets/Dungeons/Rooms/Workbench.epokmap`, saves it as a Dungeon Room and makes one loop room from it in the project's recipe. |

## Verification

| Check | Result |
| --- | --- |
| Tests | Quarter turns of shapes and placements; 12 seeds of the built-in recipe with two rooms on the main path: footprints, doors used or walled, no generated floor, wall or column inside, every walkable cell reachable; a cap of one room holds on 12 seeds; a baked project merges two cubes (ten seen faces) per room, adds one pillar box each, copies one lamp each with stable ids across regenerations, goes stale when the room is edited, and in 2D paints every walkable cell with a note (without the 2D fallback a handmade cell is left unpainted); the template creates its workbench and one Reactor in every flavor. |
| PCSX-Redux | The Dungeon template (C++) on the console: the Reactor's core, consoles and walls, its doorway onto a flight of stairs, the room on level 2. |
| `--build-psx` | Dungeon template: C++ 1.89 MiB (119 KB free), Lua 1.91 MiB (95 KB free), Blueprint 1.92 MiB (83 KB free); 64 KB must stay free. |

## Limits and follow-ups

- Every quad of a map reserves render memory on the console, so a handmade room costs
  RAM even where it replaces a generated room: the Reactor adds about 35 KB, mostly
  render capacity for its 73 extra quads. With three Reactor rooms the Lua and
  Blueprint templates did not leave 64 KB free, hence the step's **At most**.
- Baked lights in a room are copied as actors even though the console only needs their
  baked colours; dropping light-only actors at cook time would save a few KB each.
- Hairline cracks near the camera, already seen on generated corridors, show on the
  room's floor too.
