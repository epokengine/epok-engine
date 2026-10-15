# Phase 4: Recipe tilemaps

Delivered on `feature/dungeon-generator`.

## What shipped

| Area | Delivered |
| --- | --- |
| Caves | A repeatable **Caves** step turns the rooms of named paths into caves. A cave fills its whole lattice slot; rock is scattered with a chance that grows towards the slot's edges (the step's **Rock** percentage), then **Smoothing** passes of a cellular automaton (outside the slot counts as rock) round it into chambers. |
| Cut corners | The **Rooms** step's **Cut corners** cuts one cell from each corner, or a three-cell triangle in rooms at least eight cells across. |
| Shape guarantees | Every shaped room keeps its centre, every door cell and a way from each door straight in and across to the centre, then drops floor the centre cannot reach. Shapes only remove cells inside the room's rectangle, so corridor separation, locks that cannot be walked around and the solvability check are unchanged. |
| Item placement | Items steps choose **In the room**: anywhere (a free cell away from the doors, preferring the middle), centre (the nearest free cell) or against a wall. Keys go to the centre. Two items never share a cell. |
| Path styling | Style pieces gain **Path**: a piece places only at spots in rooms of that path, so caves can get their own floors and walls, banners can keep to the main path, and so on. |
| Bake | Caves and cut corners need nothing new from the 3D and 2D bakes: walls, wall tops, corner pillars, compound collision, navigation and 2D terrain solving follow any floor shape. |
| Editors | Recipe editor: Caves step (path checkboxes, Rock slider, Smoothing), Cut corners toggle, In the room combo, summaries for each; cave rooms are drawn round in the room-graph preview. Style editor: Path field and badge. |
| Built-in recipe | Its two dead-end branches are caves and its rooms have cut corners. |

## Verification

| Check | Result |
| --- | --- |
| `cargo test --locked` | 1,002 passed, 0 failed (a file-watcher timing test in `native_metadata` failed once under load and passed on rerun and alone) |
| New tests | 30 seeds: every room keeps its centre, caves are never rectangles, cut-corner rooms lose their corners unless a door needs one; 20 seeds of centred and wall-side items on the right cells, never sharing one; a path-filtered piece places only in that path's rooms and an item filter that matches nothing places nothing. All phase 3 recipe properties (reachability of every floor cell, locks that cannot be walked around, solvability) hold with the built-in caves. |
| `cargo clippy --all-targets` | no diagnostic in the dungeon modules |
| Fixture | `--generate-dungeon Dungeon`: 15 rooms, 1 key and lock, 1 one-way door, 2 items; 25 generated actors, 1,512 quads, 177 boxes, 379 navigation nodes; cave branches with mossy floors and rock walls from path-filtered pieces |
| `--build-psx` | 1.76 MiB static RAM |
| PCSX-Redux | a cave branch with its mossy floor, irregular rock walls, the treasure pedestal and the key |

## Limits and follow-ups

- Caves live inside one lattice slot (slot size minus its margins); larger caves need
  larger slots. Corridors between caves stay straight.
- Corridors belong to no path, so **Path** pieces never place on corridor spots.
- Cave walls are one face per cell side like rooms, so a cave costs more quads and
  collision boxes than a rectangle of the same area; the budget meter shows it.
