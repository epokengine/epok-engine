# Phase 10: Levels and stairs

Delivered on `features/dungeon-rooms-levels-queries`.

## What shipped

| Area | Delivered |
| --- | --- |
| Layout | `Layout.levels` (floor level by walkable cell) and `Layout.stairs` (stair cells and the side they climb towards). A level is 1.6 units, a flight four steps of 0.4 in one cell (`STEP_RISE`, `STEPS_PER_LEVEL`, `LEVEL_HEIGHT`). Spots carry their level; ceilings the level of their cell's highest point, solid cells their highest walkable neighbour's. |
| Recipes | A **Levels** step (climb chance, highest level 1-4). Corridors report their cross-sections in order; flights fit only on straight stretches, never at a door or next to a bend. Levels grow from the spawn (level 0) over the links, each link bounded by the flights its corridor has room for; a dungeon whose levels cannot all be joined stays flat. The built-in recipe climbs (45%, up to level 2). |
| 3D bake | Flights drawn as treads and risers with the floor texture laid from above; walls from each cell's floor to the ceiling over its highest point, as a ladder of faces whose edges match the walls beside them (no T-junctions, which crack on the console); ceilings over each cell with risers where a neighbour's is lower; every piece, light and actor of the style on its level. Collision: solid but for the air over each walkable cell (floors to the bottom, one box per step, walls and ceilings to the top), so no wall or ceiling top is a floor for the character or navigation; navigation volumes reach the highest floor. |
| Controllers | The Third Person controller (C++, Lua, Blueprint) keeps its feet on the floor going down: when it was grounded, did not jump and lost the ground, it probes the ground below (at most the step height) and moves onto it with the ground bias, so stairs are walked down instead of falling and landing on every step. |
| Editors | Recipe editor: Levels step with Climb chance and Highest level, an arrow and the level on each room above the spawn and in its tooltip. Generator band: the summary counts levels and flights. Scene view: outlines at each room's and corridor's level. |

## Verification

| Check | Result |
| --- | --- |
| Tests | 24 seeds of the built-in recipe: floors meet across every open edge (a stair low on one side, high on the other), lanes of a flight climb together, rooms flat, stairs only in corridors, the spawn on level 0, half or more of the dungeons climb; in a baked dungeon every flight has four step boxes a step apart, the spawn stands on its floor and navigation from the lowest floor reaches a higher one; the ceiling test follows each cell's height; the three controller flavors compile and share their tuning and operations. |
| PCSX-Redux | The Dungeon template's first corridor climbs from the spawn room: the character walks up the flight (state log Idle, Walk, Run, Walk at 1.6). Down the same flight the old controller falls and lands twice (JumpDown, Land); the new C++ one runs down (Run, then Walk at 0), and the Lua and Blueprint ones show the same running pose halfway down where the old one falls. |
| `--build-psx` | Dungeon template with levels, C++: 1.85 MiB static RAM. |

## Limits and follow-ups

- Levels belong to recipe layouts in 3D; 2D dungeons and rooms-and-corridors layouts stay on one plane.
- A camera pressed against a corridor wall can show a hairline crack along a face near
  the near plane; the same happens on flat corridors.
- Projects made from the Third Person template before this phase keep their controller,
  which falls down each step; recreate the controller or add the same ground probe.
