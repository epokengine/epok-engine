# Dungeon Generator initiative

Adds seeded procedural level generation to Epok: a **Dungeon Generator** placed in a
map lays out rooms, corridors, doors and gameplay points, then bakes the result into
ordinary Epok content (EditableMesh sections, colliders, navigation and marker actors in
3D; TileMapLayer cells in 2D). Generation runs in the editor and is deterministic: the
same seed and settings always produce the same map, so the baked result is what ships.
Work happens on `feature/dungeon-generator`, one pushed checkpoint per phase.

| Phase | Scope | Status |
| --- | --- | --- |
| 1 | Generator core: seeded random streams (Rust and C++ twins), layout model, spots, rooms-and-corridors layouts, 3D bake (merged mesh sections, compound static collider, navigation volume, spawn/goal markers), 2D bake (TileMapLayer with terrain solving), PSX budget meter, editor workflow, MCP tool | delivered ([phase1.md](phase1.md)) |
| 2 | Dungeon Styles: a reusable asset that maps spots (Floor, Wall, Door, Corner, Spawn, ...) to meshes, mesh variations, Blueprint templates, lights, particle effects and 2D terrains, with probabilities, offsets and first-match rules; a node-graph style editor with live preview | planned |
| 3 | Dungeon Recipes: designer-authored step lists that grow a room graph (main path, branches, loops), place items by path depth, add keys and locks that are always solvable, and promote one-way shortcuts; recipe editor with a room-graph preview | planned |
| 4 | Recipe tilemaps: rasterize the room graph into rooms, corridors and cellular caves with doors, locked doors, walls and item placement, baked through the same 3D and 2D paths and styles | planned |

## Design constraints

These limits of the PSX runtime decide the shape of the output and are checked by the
generator's budget meter before anything is written:

| Limit | Value | Consequence for generated maps |
| --- | --- | --- |
| Actors per map | 64 | Geometry is merged into a few EditableMesh sections; actors are reserved for gameplay (spawn, goal, doors, keys). |
| Resident geometry without streaming | 3,500 quads | The meter counts compiled quads (surfaces are subdivided every 4 units). |
| EditableMesh asset | 3,500 faces, ±128 local units | Large layouts are split into several sections. |
| Collider per actor | one | Phase 1 adds a compound static collider: one actor carries many boxes. |
| Navigation nodes | 512 | The generated bake volume picks a spacing that keeps the floor under budget. |
| Gameplay random numbers | none before this initiative | A shared generator with golden-value tests keeps editor and console results identical. |

## Determinism contract

* Decisions use integers only. Floating point appears only when converting cells to
  world units at bake time.
* `dungeon::random` (Rust) and `runtime/dungeon_random.hpp` (C++) implement the same
  xoshiro128** stream seeded through SplitMix32; both are checked against the same
  golden values.
* Collections that influence results are ordered (`Vec`, `BTreeMap`); hash maps are
  used only for lookups whose iteration order never reaches the output.
* Generated asset and actor identities derive from the generator identity and a role
  name, so regenerating keeps references stable and diffs small.
