# Phase 9: Gameplay queries

Delivered on `features/dungeon-rooms-levels-queries`.

## What shipped

| Area | Delivered |
| --- | --- |
| Baked data | Every generation stores what game code can ask (`dungeon::Output.queries`, `src/dungeon_query.rs`): each room's bounds, centre, level, recipe path, roles (spawn, goal, key, item, locked), rooms from the spawn and to the goal, its links and up to eight free spots chosen by farthest-point sampling away from doors, keys and items; every key and item; one doorway per lock; the seed. Positions stay in the generator root's space. |
| Cook | `dungeon_query::cpp` writes constant tables for the map's first generated dungeon in world space (through the root's transform, 2D roots included) and a `reset` in `initialize_components`, or an empty reset, so a previous map's rooms never leak. |
| Runtime | `runtime/dungeon_map.hpp` holds the active map and the room walk (breadth-first over the links, at most 64 rooms) and a stateless random stream tied to the seed. |
| API | `epok::DungeonLibrary` in `runtime/gameplay_api.hpp` (category Dungeon): `room_count`, `room_at`, `room_of`, `spawn_room`, `goal_room`, `room` (a `DungeonRoom` record), `neighbour`, `next_room`, `distance`, `item_count`, `item`, `lock_count`, `lock`, `spot` (`DungeonPoint` records), `seed`, `random`. Being an annotated library, it reaches Blueprint (Gameplay API > Dungeon), Lua (`epok.dungeon`, VM and native) and C++ through the shared operation lowering; the parity inventory lists the sixteen functions as public gameplay with nothing pending. |
| Editor | The generator band's **Gameplay queries** list numbers the rooms as game code does, with path and roles; pointing at a row picks the room out in the Scene view and its tooltip gives level, depth, links and spots. A dungeon generated before the queries warns with **Regenerate**. `--dungeon-query-room N` captures the list. |

## Verification

| Check | Result |
| --- | --- |
| Tests | For four seeds of the built-in recipe: links both ways, depth changing by at most one per link, spots distinct inside their room and off items, keys and locks in flagged rooms, rooms off the main path; the cook moves the tables with the generator and resets maps without a dungeon; a Lua class calling all sixteen functions and a Blueprint graph calling three compile through the real native catalog, and the graph lowers to `epok::DungeonLibrary`. |
| Host runtime | `verify_spatial.py gameplay_api`: empty map, three rooms in a row (rooms on the plane whatever the height, corridors are no room, ways both directions, same room, unreachable), records, items, locks, spots, the random stream against `Random::stream`, the 2D plane. |
| Parity | API reference regenerated (`--check` clean); `gameplay_api_parity.py` 589 public gameplay rows, 0 pending. |
| PCSX-Redux, C++ | A probe in the Dungeon template prints on the console: 15 rooms, spawn 7, goal 5, the player in room 7, next room 12, distance 7, two spots, seed 1207, roll 61; the editor's baked data and the host stream give the same numbers. |
| PCSX-Redux, Lua VM | A probe fades the screen only if the player's room is the spawn, its record's flags and distances agree with the next room's, its spot round-trips through `room_at`, and the seed and roll match: the screen fades; with a wrong expected roll it does not. |
| `--build-psx` | Dungeon template: C++ 1.81 MiB, Lua 1.83 MiB static RAM (the tables add about 2 KB). |

## Limits and follow-ups

- One dungeon per map answers: the first generated one.
- `room_of` reads 3D actors; 2D code passes its actor's position to `room_at`.
- Room numbers change when the dungeon is generated again.
