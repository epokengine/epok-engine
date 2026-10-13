# Phase 3: Dungeon Recipes

Delivered on `feature/dungeon-generator`.

## What shipped

| Area | Delivered |
| --- | --- |
| Recipe model | `src/dungeon_recipe.rs`: a `DungeonRecipe` authored asset holding ordered steps: Lattice, Main path, Branch (dead ends, or loops back onto named paths), Items (by name and depth along paths), Key and lock, One-way doors and Rooms. Validation names the step at fault and only accepts paths grown by earlier steps. The built-in recipe has a main path, two dead-end branches with treasure, a loop, a key and lock and one-way shortcuts. |
| Room graph | Paths grow as self-avoiding walks over the lattice. Every key and lock placement is checked by a walk from the spawn that collects keys and opens the doors they fit: all rooms must stay reachable and the locked room must really be out of reach without its key. One-way doors turn links that lead back at least two rooms along the route into shortcuts, kept only when everything stays reachable. Failed steps rerun the whole recipe with a fresh stream, up to 64 times, then report the step. |
| Layout | Rooms grow inside their slots; corridors run straight or jog once inside the gap between slots, one cell clear of everything else, so the grid connects exactly the graph's links and a locked door cannot be walked around. Unused slots are trimmed. The layout carries locks, one-way doors, items and each room's path. |
| Spots and styles | New spots Key, Locked door, One-way door and Item (locked and one-way doors are framed like doors). Item pieces can filter by item name. Actors placed on key and locked-door spots receive the lock number in an editable number property named `lock`, on their class or components. Built-in markers: blockout pedestals, a red gate and a floor arrow in 3D; marker tiles in the built-in 2D tiles (directional ones turned with tile flags), with existing projects' built-in tiles upgraded in place. |
| Generator | `source` (Rooms or Recipe) and `recipe` settings; the recipe's digest is stored with the output and an edited recipe marks the dungeon stale. Layout summaries count keys, one-way doors and items. |
| Recipe editor | `src/dungeon_recipe_editor.rs`: step list with order, summaries and amber errors; Add step (unique steps disabled once present); move up/down; per-step settings with path checkboxes and tooltips; branch renames follow into later steps; a room-graph preview coloured by path with spawn, goal, keys, items, locks with their numbers and one-way arrows, hover details and a preview seed; its own undo/redo, autosave of valid recipes and live regeneration of the map's dungeons that follow the recipe. |
| Integration | Content Browser kind, icon, filter, **Add > Dungeon Recipe**, double-click to open; generator band **Layout** combo and **Recipe** row with **New** and **Edit**; recipe markers in the Scene view's layout lines. |
| Automation | MCP `dungeon_generate` takes `settings.source`, `settings.recipe` and `new_recipe`; `asset_document` reads and validates recipe documents; CLI `--dungeon-new-recipe`; screenshot flag `--open-dungeon-recipe N`; the visual fixture now uses a recipe with a gate actor class and glowing keys. |
| Fixes found on the way | Corner pillar side faces were wound towards the corner and culled on the console; a winding test now checks walls, pillars, doorway beams and marker pedestals. UI ranges use plain hyphens (the editor font has no en dash). |

## Verification

| Check | Result |
| --- | --- |
| `cargo test --locked` | 999 passed, 0 failed |
| Recipe tests | built-in round trip; 80 seeds reproducible, connected and solvable with one effective lock; three keys never deadlock; loops add one extra link; 40 seeds of layouts carrying locks, one-way doors and items to spots with every floor cell reachable; a locked door cannot be walked around on the grid; validation messages name the step |
| Bake and editor tests | 3D recipe dungeons within budget with marker materials and a cooking map; recipe edits mark the output stale; a missing recipe is reported; 2D marker tiles on the right cells with flags matching their doors and no false "style changed" after the built-in tiles are pinned; MCP recipe creation, document editing, refusal and regeneration; recipe editor path scoping, renames and error placement |
| `cargo clippy --all-targets` | no diagnostic in new files or changed lines |
| Fixture | `--generate-dungeon Dungeon`: 15 rooms, 1 key and lock, 1 one-way door, 2 items; 25 generated actors, 1,592 quads, 149 boxes, 379 navigation nodes; the `DungeonGate` actor received `lock: 1` |
| `--build-psx` | 1.79 MiB static RAM |
| PCSX-Redux | the goal room's red gate fills its doorway; the blue one-way arrow, the gold key pedestal and the green treasure pedestal render |

## Limits and follow-ups

- Keys and doors have no built-in gameplay: the built-in markers are static geometry
  and do not block. Projects make them play with their own actor classes and the
  `lock` property; ready-made key and door classes are a candidate for a later phase.
- The lattice is up to 16 × 16 slots; rooms are rectangles inside their slots and
  corridors jog at most once. Phase 4 adds room shapes, caves and corridor variety.
- A recipe whose locks guard a loop's only entrance may need several attempts; when
  all 64 fail the generator names the step to loosen.
