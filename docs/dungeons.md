# Dungeon Generator

A Dungeon Generator lays out rooms, corridors and doors from a seed and bakes the
result into the map as ordinary Epok content. The same seed and settings always
produce the same dungeon, so what you see in the editor is exactly what ships: the
console runs the baked map and does no generation work of its own.

| Output | 3D generator | 2D generator |
| --- | --- | --- |
| Floors and walls | EditableMesh sections under `assets/Dungeons/<name>/` | One TileMapLayer painted with Floor and Wall terrains |
| Collision | One compound collider holding every floor, wall and doorway box | The TileSet's tile collision |
| Navigation | Navigation bake volumes over the floor, spaced for the node budget | — |
| Gameplay points | `Spawn` and `Goal` actors at the centres of the two rooms furthest apart | `Spawn` and `Goal` Node2D actors |

## Create a dungeon

- **3D:** choose **GameObject > Dungeon Generator**, or the same entry in the
  Hierarchy's create menu.
- **2D:** choose **Add 2D Node** and search for `DungeonGenerator` (group *Level*).

The generator appears at the map origin (3D) or at the centre of the 2D view, generates
a first dungeon straight away and frames the Scene view on it. The generated actors
are children of the generator, so moving the generator moves the whole dungeon.

![The Dungeon Generator Inspector band with the generated 3D dungeon](images/dungeon-generator-3d.png)

## The Dungeon Generator band

Select the generator to edit it in the Inspector.

- **Generate** rebuilds the dungeon from the current settings. **New seed** picks
  another seed and generates. Both are one undo step: **Ctrl+Z** brings back the
  previous dungeon.
- While the generator is selected in 3D, the Scene view draws its layout: the grid
  outline, rooms (spawn green, goal red), corridor walls and doors. Before the first
  generation, or when the settings changed, the outline lies on the floor; afterwards
  it floats just above the walls.
- An amber box lists anything that needs attention, with a button where there is an
  obvious fix: nothing generated yet, settings changed since the last generation, a
  rotated or scaled generator, rooms that did not fit, or a budget overrun.
- The three-dot menu holds **Remove generated content** (the generator keeps its
  settings) and **Reset settings**.

### Layout settings

| Setting | Meaning | Default |
| --- | --- | ---: |
| Seed | Chooses the dungeon; same seed, same dungeon | random on creation |
| Grid size | Cells across and deep | 40 × 40 |
| Rooms | Rooms to place; a crowded grid fits fewer | 10 |
| Room size | Smallest and largest room side, in cells | 4 – 8 |
| Spacing | Empty cells kept between rooms; a corridor needs three to pass between two rooms | 3 |
| Corridor width | Corridor width in cells | 1 |
| Loops | Chance, in percent, that each extra connection becomes a loop; 0 gives a tree | 25 |

Rooms are placed at random positions that keep the spacing. Rooms whose centres see
each other across an empty circle are candidate connections; a spanning tree of them
connects every room, and each remaining candidate becomes a loop with the **Loops**
chance. Corridors are routed around other rooms, prefer straight runs and merge into
corridors that already exist. Spawn and goal are the two rooms the most
connections apart.

### 3D settings

| Setting | Meaning | Default |
| --- | --- | ---: |
| Cell size | World units per grid cell | 2 |
| Wall height | Height of walls and wall tops | 3 |
| Door height | Doorways are open below this height; a beam fills the rest | 2.25 |
| Navigation | Add navigation bake volumes over the floor | on |
| Move to spawn | An actor placed at the spawn room's centre after each generation, usually the player | none |

Walls are the solid cells around every floor, so the dungeon reads from above as well
as from inside. Floors and walls are one face per cell: every vertex then lies on the
grid, which avoids the one-pixel cracks the PSX shows where an edge ends in the middle
of another face, and gives baked lighting a vertex at every cell corner. Run
**Build > Build Lighting** after generating, or let Build and Play bake it.

### 2D settings

![A generated 2D dungeon painted with the built-in dungeon tiles](images/dungeon-generator-2d.png)

**Tile set** chooses the TileSet the layer is painted with. *Built-in dungeon tiles*
creates `assets/Dungeons/DungeonTiles.epokasset` on first use: stone floor variants and
solid walls with a brick face towards the room. Any TileSet works when it has a
terrain set holding terrains named **Floor** and **Wall**; the TileSet's own terrain
rules choose the tiles, and its tile collision makes walls solid. One grid cell is one
tile.

## PSX budget

The band shows what the map will hold after generating, so a dungeon that would not
build is visible before it is generated.

| Meter | Limit | Notes |
| --- | ---: | --- |
| Actors | 64 per map | A 3D dungeon uses about eight: geometry sections, collision, navigation volumes, spawn, goal and a light when the map has none. |
| Geometry | 3,500 quads per map | Counts the whole map. Surfaces are split every 4 units when compiled. |
| Collision | 2,048 boxes | One compound collider per dungeon. |
| Navigation | 512 nodes | Spacing grows with the floor area; very large floors skip navigation. |
| Tiles (2D) | 65,536 cells per layer | |

A 3D dungeon must stay within 254 units across (grid size × cell size), because mesh
vertices and collider boxes are stored within ±128 units of the generator. The
generator must be moved only, not rotated or scaled; the amber box offers a fix.

## Compound colliders

The generated collision is one `Collider3DComponent` carrying a list of boxes in the
actor's local space. The collider's own box encloses them and is only the broad phase:
movement, raycasts, overlaps, ground probes, triggers and the navigation bake all test
the individual boxes. An actor with a compound collider, and its parents, must not be
rotated or scaled.

## Automation

- MCP `dungeon_generate` regenerates a generator named by its actor, or creates one
  with `create: "3d"` or `"2d"`. `seed` and `settings` change it first; `clear` removes
  the generated content. The change is one entry in `scene_history`.
- `--generate-dungeon <actor>` regenerates headlessly and saves the map;
  `--generate-dungeon 3d` or `2d` adds a new generator. `--dungeon-seed N` and
  `--dungeon-settings '<json>'` (for example `{"rooms":{"rooms":14},"spawn_actor":"Player"}`)
  apply before generating.

## Determinism

Generation uses integers and the dungeon random stream. `runtime/dungeon_random.hpp`
is the C++ twin of the editor's stream; both are tested against the same values, so a
seed chosen in the editor can be reproduced on the console. Generated asset and actor
identities derive from the generator and the role of each piece, so regenerating
rewrites the same files and keeps references to the generated actors valid.

## Verification

- `cargo test dungeon` covers layouts (reachability, spacing, loops, validation),
  the bake into real projects (cook, navigation bake, regeneration, clearing, spawn
  placement) and the random stream's golden values.
- `python tests/runtime/verify_spatial.py compound_collision dungeon_random` runs the
  compound collider and random stream host tests.
- Screenshot a generated dungeon with
  `--screenshot <png> --frame-dungeon` (add `--screenshot-2d` for a 2D generator), and
  gameplay with `--screenshot-game --screenshot-input "500:16,2600:0"`.
