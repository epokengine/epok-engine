# Dungeon Generator

A Dungeon Generator lays out rooms, corridors and doors from a seed and bakes the
result into the map as ordinary Epok content. Rooms are either scattered over a grid
or grown by a [Dungeon Recipe](#dungeon-recipes): a designed route from spawn to goal
with branches, loops, items, keys and locks, and one-way doors. The same seed and settings always
produce the same dungeon, so what you see in the editor is exactly what ships: the
console runs the baked map and does no generation work of its own.

| Output | 3D generator | 2D generator |
| --- | --- | --- |
| Floors and walls | EditableMesh sections under `assets/Dungeons/<name>/` | One TileMapLayer painted with Floor and Wall terrains |
| Collision | One compound collider holding every floor, wall and doorway box | The TileSet's tile collision |
| Navigation | Navigation bake volumes over the floor, spaced for the node budget | — |
| Gameplay points | `Spawn` and `Goal` actors at the centres of the spawn and goal rooms | `Spawn` and `Goal` Node2D actors |
| Keys, locks, items | Spots for the style: blockout markers by default, or your own actors | Spots for the style: marker tiles by default, or your own actors |
| Look | A [Dungeon Style](#dungeon-styles): built-in blockout, or meshes, lights and actors | A Dungeon Style: the built-in tiles, or any TileSet's terrains and tiles |

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

**Layout** chooses where the rooms come from: **Rooms and corridors** scatters rooms
over a grid with the settings below; **Recipe** follows a [Dungeon Recipe](#dungeon-recipes)
and replaces them with a **Recipe** row (built-in recipe or any recipe asset, with
**New** and **Edit** like the style row).

![The generator band with a recipe layout: the Recipe row, the budget and a summary counting keys, one-way doors and items](images/dungeon-recipe-inspector.png)

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

**Tile set** is the TileSet the style's terrains and tiles use when they name none.
*Built-in dungeon tiles* creates `assets/Dungeons/DungeonTiles.epokasset` on first use:
stone floor variants and solid walls with a brick face towards the room. Any TileSet
works when it has a terrain set holding the terrains the style paints (the built-in
look uses **Floor** and **Wall**); the TileSet's own terrain rules choose the tiles,
and its tile collision makes walls solid. One grid cell is one tile.

## Dungeon Recipes

A Dungeon Recipe is a list of steps that grows a dungeon's room graph, then lays it
out on the grid. Where *Rooms and corridors* places rooms at random, a recipe designs
the route: the player starts at the spawn, the goal waits at the end of the main path,
side branches hide treasure, a key opens a locked door further on, and one-way doors
offer shortcuts back.

- Set the generator's **Layout** to **Recipe**. Without a recipe asset it follows the
  built-in recipe; **New** creates a recipe asset from it (under
  `assets/Dungeons/Recipes/`) and opens it, **Edit** opens the chosen one. The Project
  browser's **Add > Dungeon Recipe** creates one too, and double-clicking a recipe
  opens it.
- The editor lists the **steps** on the left, in the order they run; the arrows move a
  step, **Add step** inserts one after the selection. The selected step's settings are
  below the list, each with a tooltip.
- The right side previews the **room graph** for one seed: one square per room slot,
  rooms coloured by path, links between them, the spawn (S) and goal (G), keys and
  items as dots, locked doors red with their key's number and one-way doors as blue
  arrows. Hover a room for its path, depth and contents. **Seed** and the dice button
  preview other seeds without touching any dungeon.
- Every valid edit is saved; with **Live preview** on, the map's dungeons that follow
  the recipe regenerate. A step with a problem (a path no earlier step grows, a range
  upside down) is amber, says why, and the recipe is not saved until it is fixed.

![The Dungeon Recipe editor: steps, the key-and-lock step's settings and the room graph it grows](images/dungeon-recipe-editor.png)

### Steps

| Step | What it does | Settings |
| --- | --- | --- |
| Lattice | The grid of room slots. Each slot holds at most one room; paths walk between neighbouring slots | Room slots (columns, rows), slot size in grid cells |
| Main path | The path from the spawn room to the goal room | Length in rooms |
| Branch | Paths that start next to rooms of other paths. With **Loops back to** empty they are dead ends; otherwise each ends next to a room of those paths and connects to it | Name, starts off, loops back to, count, length |
| Items | Named items in rooms of some paths, between two depths along each path (0% first room, 100% last) | Name, paths, count, depth |
| Key and lock | A key in a room of some paths and the locked door it opens, on a link into a room of other paths | Key in, lock on |
| One-way doors | Links that lead back at least two rooms along the route open from the far side only | Chance |
| Rooms | How much of its slot each room fills, and the corridor width | Fill, corridor width |

A recipe has one Lattice, one Main path and one Rooms step; Branch, Items and Key and
lock steps can repeat, and each step can only name paths that earlier steps grow. The
built-in recipe is a 6 × 5 lattice with a main path of 6–9 rooms, two dead-end branches
with treasure, a loop, a key on a branch for a lock on the main path, and one-way
shortcuts.

### Always solvable

Every key and lock placement is checked by walking from the spawn through open doors,
picking up keys and opening the doors they fit, until nothing new is reached: every
room must be reachable, and the key's own door must really keep its room out of reach
without the key, so a lock is never decorative. One-way doors are kept only when the
same walk still reaches every room. Several keys never deadlock, because each one is
checked with all the others in place.

When a step cannot be satisfied (a path longer than the lattice has room for, a branch
with nowhere to grow), the whole recipe runs again with fresh random numbers, up to 64
times; after that the generator says which step failed and suggests loosening it.

### Layout

Each room sits inside its slot, with empty cells around it. Corridors run straight
between neighbouring rooms, or jog once inside the gap between their slots, always one
cell clear of every other corridor and room. So the grid connects exactly the rooms the
graph links: a locked door cannot be walked around through a neighbouring corridor.
Unused slots are trimmed, keeping the dungeon centred on its generator.

Recipe layouts add spots for the style (see [Spots](#spots)): **Key**, **Locked door**,
**One-way door** and **Item**. The built-in 3D look marks them with blockout: a gold
pedestal for keys, a green one for items, a red gate filling each locked door and a blue
arrow on the floor pointing through each one-way door. The built-in 2D tiles have marker
tiles for them (a key, a gate turned towards its corridor, chevrons and a chest), drawn
on a decoration layer.

![A recipe dungeon in 2D: the gate of a locked door, a key, one-way chevrons and chests](images/dungeon-recipe-2d.png)

### Keys and doors that work in game

The built-in markers show where things are; to make them play, place your own actors
on the Key and Locked door spots with Actor pieces. An actor placed on a Key or Locked
door spot gets the lock number in any editable number property named `lock`, on its
class or on one of its components, so a key actor knows which door it opens:

```cpp
class EPOK_CLASS(Blueprintable, Domain=World3D, Id="your-class-uuid") DungeonGate : public epok::Actor3D {
public:
    EPOK_PROPERTY(EditAnywhere) uint32_t lock = 0;  // set by the Dungeon Generator
};
```

Give the door actor a collider so it blocks the doorway, and remove or deactivate it
when the player holds the key with the same `lock` number. Item pieces can pick items
by name with **Item name**, so a "Treasure" item can become a chest and a "Potion" a
bottle.

![A recipe dungeon on the console: the goal room's locked gate fills the doorway](images/dungeon-recipe-ingame.png)

## Dungeon Styles

A Dungeon Style decides what every spot of a dungeon becomes. It is an asset, so one
style can dress many dungeons, and changing it restyles them all. Without a style a
generator uses the built-in look: blockout geometry in 3D, the built-in tiles in 2D.

- In the generator band, **Style** chooses the style. **New** creates a style asset
  holding the current look (under `assets/Dungeons/Styles/`) and opens it; **Edit**
  opens the chosen one. The Project browser's **Add > Dungeon Style (3D)** and
  **(2D)** create one too, and double-clicking a style opens it.
- The editor shows **spots** on the left and the **pieces** each spot becomes to their
  right. Pieces are tried from left to right, and each piece header shows its place in
  that order: drag a piece left or right to change its priority.
- **Add** (or right-click the canvas) adds a spot or a piece, with a search field.
  Dragging from a spot's dot to empty canvas adds a piece to that spot; dropping a
  piece on another spot moves it there. **Delete** removes the selection; **Ctrl+Z**
  and **Ctrl+Shift+Z** undo and redo; the wheel zooms and **F** or **Frame all** shows
  the whole style.
- Every finished edit is saved. With **Live preview** on, the dungeons of the open map
  that use the style regenerate at once, so the Scene view shows the result; those
  regenerations are one undo step in the map.
- A piece that cannot place anything (no mesh chosen, a 2D piece in a 3D style, a
  terrain on a wall spot) has an amber outline and says why. The details panel counts
  how many times the selected piece is placed in the open map.

![The Dungeon Style editor with spots, ordered pieces and the details of a floor piece](images/dungeon-style-editor.png)

### Spots

| Spot | Where | Facing (+Z) |
| --- | --- | --- |
| Floor | Every floor cell of rooms and corridors | — |
| Wall | Each closed side of a floor cell | into the floor cell |
| Door | Each room side that opens onto a corridor | into the room |
| Doorway | The corridor cell just outside each door | towards the room |
| Solid | Each solid cell of the ring that encloses the floor | — |
| Corner | Each inner corner where two walls of a floor cell meet | away from the first wall |
| Room | The centre of every room, once per room | — |
| Key | Each key a recipe placed, at a room cell | — |
| Locked door | Each door that needs a key, on the guarded room's side | into the guarded room |
| One-way door | Each door passable only into its room | into the room |
| Item | Each item a recipe placed, at a room cell | — |
| Spawn, Goal | The centres of the spawn and goal rooms | — |

Spots sit at floor level. Wall, door, locked-door and one-way spots are on the cell
edge, corner spots on the corner point, the others at the cell centre. Key, locked-door,
one-way and item spots exist only in recipe layouts; a wide corridor's door has one
spot per cell.

### Pieces

| Piece | 3D / 2D | What it places | Cost |
| --- | --- | --- | --- |
| Blockout | 3D | The generator's own geometry for the spot: a floor tile, wall panel, wall top, doorway beam or corner pillar, in a colour or texture | Merged geometry |
| Mesh | 3D | An EditableMesh, merged into the dungeon's sections with its own materials | Merged geometry, no actor |
| Actor | both | An actor of any placeable class (C++, Blueprint or Lua), including Blueprint templates with child actors | One actor per placement |
| Light | 3D | A baked point light | One actor per placement |
| Terrain | 2D | The terrain a Floor or Solid cell is painted with | Tiles |
| Tile | 2D | One tile on a decoration layer above the floor, without collision | Tiles; one layer per TileSet |

Model meshes around the spot: floor level at 0, centred on the cell or the edge, and
facing +Z into the room for walls and doors. Generated collision always comes from the
layout; give props that must block the player an Actor piece with a collider instead.

### Placement

| Field | Meaning | Default |
| --- | --- | ---: |
| Chance | Percentage of eligible spots that get the piece | 100 |
| Stop here | When the piece places, pieces to its right are skipped at that spot. Off, it decorates and the next pieces are tried too | on |
| Where | Anywhere, rooms only or corridors only | Anywhere |
| Limit | Most placements in one dungeon, spread at random; 0 has no limit | 0 |
| Item name | Item spots only: just the items with this name (any case); empty takes every item | empty |
| Offset, Rotation, Scale | Relative to the spot (2D: offset in pixels for actors) | 0, 0, 1 |

The built-in pieces do not stop: a piece added to their right decorates on top of them,
and one added to their left that stops replaces them. For example, a wall mesh with a
15% chance placed left of the blockout wall replaces 15% of the walls; a torch mesh
placed to its right adds torches to 15% of them. Each piece draws its own random
numbers from the seed, so editing one piece never reshuffles another.

![A styled dungeon on the console: a room lit by a baked point light, with a crate merged into the geometry](images/dungeon-style-ingame.png)

## PSX budget

The band shows what the map will hold after generating, so a dungeon that would not
build is visible before it is generated.

| Meter | Limit | Notes |
| --- | ---: | --- |
| Actors | 64 per map | A 3D dungeon uses about eight: geometry sections, collision, navigation volumes, spawn, goal and a light when the map has none. Actor and Light pieces add one each. |
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
  with `create: "3d"` or `"2d"`. `seed` and `settings` change it first (`style` takes a
  Dungeon Style asset id; `source: "Recipe"` with `recipe`, a Dungeon Recipe asset id or
  none for the built-in recipe, switches to a recipe layout); `new_style` creates a style
  from the current look and assigns it, `new_recipe` a recipe from the current one;
  `clear` removes the generated content. The change is one entry in `scene_history`.
  `asset_document` reads and replaces a style's or a recipe's document, refusing
  invalid ones with the step at fault.
- `--generate-dungeon <actor>` regenerates headlessly and saves the map;
  `--generate-dungeon 3d` or `2d` adds a new generator. `--dungeon-seed N` and
  `--dungeon-settings '<json>'` (for example `{"rooms":{"rooms":14},"spawn_actor":"Player"}`)
  apply before generating; `--dungeon-new-style <name>` creates a style from the current
  look and assigns it, `--dungeon-new-recipe <name>` a recipe (and switches the layout
  to it).

## Determinism

Generation uses integers and the dungeon random stream. `runtime/dungeon_random.hpp`
is the C++ twin of the editor's stream; both are tested against the same values, so a
seed chosen in the editor can be reproduced on the console. Generated asset and actor
identities derive from the generator and the role of each piece, so regenerating
rewrites the same files and keeps references to the generated actors valid.

## Verification

- `cargo test dungeon` covers layouts (reachability, spacing, loops, validation),
  recipes (reproducible graphs, solvability with several keys, locks that cannot be
  walked around on the grid, loops, one-way doors, readable errors), the bake into real
  projects (cook, navigation bake, regeneration, clearing, spawn placement, markers,
  face winding) and the random stream's golden values.
- `python tests/runtime/verify_spatial.py compound_collision dungeon_random` runs the
  compound collider and random stream host tests.
- Screenshot a generated dungeon with
  `--screenshot <png> --frame-dungeon` (add `--screenshot-2d` for a 2D generator), the
  recipe editor with `--open-dungeon-recipe <step>`, and gameplay with
  `--screenshot-game --screenshot-input "500:16,2600:0"`.
