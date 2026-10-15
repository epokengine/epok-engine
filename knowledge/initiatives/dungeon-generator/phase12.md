# Phase 12: Minimap and fog of war

Delivered on `features/dungeon-minimap-seeds-volumes`.

## What shipped

| Area | Delivered |
| --- | --- |
| Bake | Each generation bakes the minimap with the gameplay queries: the bounding box of the walkable cells, each room's cells and the corridors' cells merged into rectangles (a room's carry its index, a corridor's the up to two rooms it opens into through a doorway), and the grid's corner and step in the generator's space. Keys and locked doors now name the generated actors standing for them (built-in Key and Door pieces, and actor pieces on their spots). |
| Settings | `Generator.minimap` (outside the generation fingerprint, so changing it needs no new generation): on/off, corner, size and margin in pixels, follow the player at N pixels a cell or fit the dungeon, as explored or everything, shown at start, eight colours. |
| Cook | When the minimap is on, the map cooks the rectangles, the grid moved to world space, the player's and the key and door actors' object slots, the box settings and an explored bitset, and resets it on load; the scene bank sizes the renderer from the largest map's rectangles (`dungeon_minimap_capacity`, 0 compiles it out). |
| Runtime | `dungeon_map.hpp` keeps the minimap data and the exploring: the player's cell, the rectangle under it, a room explored in full the first time. `dungeon_minimap.hpp` draws a translucent backdrop, the explored rectangles (corridor stubs of explored rooms dim), the goal, keys still there, locked doors still closed and the blinking player, clipped to the box, before the HUD; following the player pins keys and the goal to the box's edge. The full map fills the screen. The player's point comes from its world matrix in 3D and its 2D root in 2D. |
| Gameplay API | Dungeon library: `show_minimap`, `minimap_shown`, `show_full_map`, `full_map_shown`, `explore_room`, `room_explored`, `explore_everything`, `forget_explored` in C++, Blueprint and Lua. |
| Editor | Minimap section in the generator band with every setting, the pixels a cell when fitting, the rectangle count and console RAM, warnings for a missing Player or an old generation, and a picture of the console's screen with the map. |

## Verification

| Check | Result |
| --- | --- |
| Tests | Rectangles cover every walkable cell once with the right room on 8 seeds and corridors name the rooms they open into; settings flags; the cook writes the rectangles, the moved grid, the player's slot and the box, or an empty reset and capacity 0; Lua and a Blueprint exec chain reach the new functions; the host runtime test explores a corridor rectangle, a whole room on entering, a room from game code, everything and nothing, and the visibility flags. |
| PCSX-Redux | Dungeon template (C++) with the minimap: the spawn room and its corridor stubs at the start, the corridor and the next room filling in as the character walks down the stairs, the full map opened from game code with the goal room revealed and its locked door; everything from the start and following the player at 3 pixels a cell; a 2D dungeon in the platformer project with its player marked. |
| `--build-psx` | The template's minimap: 50 rectangles, 9 KB of RAM (6 KB code, 2.9 KB packets). |

## Limits and follow-ups

- The template keeps its minimap off: on, the Blueprint flavor would keep about 74 KB
  free, close to the 64 KB the console needs.
- Exploration is per map load; a save keeps it through `explore_room`.
- No line-of-sight reveal: a room shows when entered.
