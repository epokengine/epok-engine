# Phase 5 — Showcase template (2026-10-13)

User guide: [docs/2d.md](../../../docs/2d.md) (The Platformer 2D template).

The Platformer 2D template becomes a showcase that stays a template: more depth and
life on screen, every part an ordinary node, grouped by purpose so a game can start by
deleting what it does not need. No gameplay script changed: the three flavors (C++,
Lua, Blueprint) still share one scene.

| Change | Detail |
| --- | --- |
| Hierarchy | Groups Scenery, Level, Characters, Props, Coins, Effects; HUD canvases at the end |
| Sky | `SkyLayer` (CanvasLayer2D at -20, screen space): a 16 x 64 gradient stretched x40 / x7.5 (covers up to 640 x 480) and an additive sun with its glow baked in |
| Depth | `Treeline` Parallax2D at 0.7 between the hills and the level; `Foreground` bushes at 1.25, z 20, in front of everything |
| Ambience | `Petals`: a Particles2D on its own screen canvas (layer 2), drifting down across the view |
| HUD | `HelpCanvas` / `ControlsHint`: a centred line of help at the top, built-in font |
| Fix | The built-in font's space no longer draws a glyph: its cell holds the atlas's CLUT by design, and every space on the console showed a few stray texels. One primitive fewer per space |

## Validation

| Check | Result |
| --- | --- |
| `cargo test` | every flavor creates a valid template scene |
| `tests/runtime/hud_layout.cpp` | the built-in font advances over spaces without emitting them |
| PSX | Platformer 2D in C++, Lua and Blueprint in PCSX-Redux; the help line renders clean |
| Static RAM | 1.49 MiB of 2 MiB (from 1.41 MiB): the new nodes' pools, scene data and four small textures |

## Editor usability pass

A review of the 2D workflow as a newcomer meets it, after phase 4 doubled the node list,
checked by scripted usability scenarios (below).

| Change | Detail |
| --- | --- |
| Add 2D Node search | The toolbar menu opens with focus in a search field and the project's recent nodes on top (`UserSettings/Recent2DNodes.epokprefs`); results are ranked name start, name, group, description (`world2d_editor::search_rank`) with the group on the right; Enter creates the first. It drops down from its button and is as tall as its results |
| Add Component | Lists every 2D component (derived from the node presets, so a new component appears with its icon, group and description) with the same search; it used to offer eight, the rest only as bare reflected names |
| Long pickers | Node and texture combos with more than eight choices open with a focused search field; Enter picks the first match (`world2d_editor::combo_filter`) |
| Warnings | `world2d_warnings::of` lists why a node will not work (body without a solid collider unless a joint holds it, joint without Body B, picture nobody shows, agent without a region, bone-less skeleton, follower without a path...). The Hierarchy draws an amber sign with the reasons on hover; the Inspector shows them in an amber box with one-click fixes: add a fitted Collider2D, show a picture on the HUD, add a NavigationRegion2D over the ground layer. Each fix is one undo step. Templates must open without warnings (`every_flavor_creates_a_valid_platformer_with_its_gameplay`) |
| Capability fix | The PSX target declared only `physics2d` of the World2D capabilities, so validating a map with any other 2D component against the editor's object model failed: creating a 2D node from the menus was rejected. `TargetCapabilities::psx` now provides every World2D token when `world2d.hpp` is cooked, and a test checks every `Capability=` of a cooked class against it |
| HUD preview fix | The HUD editor's Interact preview failed to build on a new project: its scene script and Blueprint classes went into the native preview runner. `hud_simulation::preview_inputs` leaves both out |
| Scripted capture | `--open-add-2d-node <query>` and `--open-add-component <query>` open the menus for screenshots |

## Usability scenarios

`src/world2d_usability_tests.rs` performs common tasks in a new Platformer 2D project
through the real 2D view toolbar, Hierarchy and Inspector with simulated mouse and
keyboard input, counting interactions (a click, a typed word or a key press; placing
nodes in the view is not counted). Each checks the result works without a script and
leaves no warning.

| Task | Interactions |
| --- | --- |
| A crate the player can push (with its texture) | 6 |
| A lamp hanging on a rope, fixed from its warning | 9 |
| A picture-in-picture on the HUD | 4 |
| An enemy that chases the player | 6 |
| Physics on an existing sprite | 7 |
| A split screen for two players | 10 |

Writing these found the capability bug, the description-first search ranking
("agent" created a NavigationRegion2D), the eight-component Add Component list and the
unsearchable 60-node pickers, all fixed above.
