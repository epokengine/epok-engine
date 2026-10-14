# Phase 5: Ready-made keys and doors

Delivered on `feature/dungeon-generator`.

## What shipped

| Area | Delivered |
| --- | --- |
| Scripts | `templates/DungeonKey.hpp`, `DungeonDoor.hpp` and the shared `DungeonKeyRing.hpp` (C++ components for 3D and 2D owners). A key joins the key ring and deactivates the first time something enters its trigger; a door deactivates once its lock number is on the ring. Keys clear their own number when the map starts and turn slowly in 3D. Lock numbers go up to 31. |
| Installation | The first generation that places a Key or Door piece writes the scripts into `assets/scripts` and says so in the report; existing files are never overwritten, so projects can change the behaviour. |
| Style pieces | **Key** and **Door** pieces (both domains) with a colour, or a tint in 2D. Problems are flagged when they sit on other spots. Both count as actors in the budget meter. |
| 3D | Key: a 0.4-unit cube 1.05 above the floor with a trigger (half size 0.5) that stays clear of the floor and walls. Door: a slab as wide as a cell, as high as the door height and 0.3 thick, with a solid collider, on the locked door's edge. Both carry their lock number on their component. |
| 2D | Key: an Area2D with a 12-pixel trigger and the built-in key sprite. Door: a StaticBody2D with a tile-sized collider and the gate sprite, rotated so its bars face the corridor. |
| Built-in style | Keys float above the gold pedestal; locked doors use the Door piece instead of the static gate; 2D keys and doors use the pieces, one-way doors and items keep their marker tiles. |

## Verification

| Check | Result |
| --- | --- |
| Unit tests | Scripts installed once and byte-identical to the templates; one door per lock and one key per key item with matching lock numbers; key triggers clear of the floor; door slabs sized to the doorway; 2D keys and doors on their cells, doors turned to their corridor, key colliders are triggers |
| `--build-psx` | The Third Person fixture (1.77 MiB static RAM) and a 2D project (1.30 MiB) compile both components |
| PCSX-Redux | With the key in front of the player the goal room's door is closed; after one step the key is gone and the doorway is open. Without the key, walking into the door stops the player. |
| Editor | 2D doors facing north and east show their bars towards the corridor |

## Limits and follow-ups

- Keys react to any moving collider entering their trigger, not only the player.
- Navigation is baked with doors closed.
- Doors simply disappear; an opening animation or sound belongs in the project's copy
  of `DungeonDoor`.
