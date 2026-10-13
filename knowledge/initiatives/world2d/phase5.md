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

A review of the 2D workflow as a newcomer meets it, after phase 4 doubled the node list.

| Change | Detail |
| --- | --- |
| Add 2D Node search | The toolbar menu opens with focus in a search field and filters by name, purpose or group as you type; Enter creates the first match. It drops down from its button and is as tall as its results (`world2d_editor::add_node_popup`) |
| Hierarchy warnings | `world2d_warnings::of` lists why a node will not work (body without a solid collider, joint without Body B, picture nobody shows, agent without a region, bone-less skeleton, follower without a path...); the Hierarchy draws an amber sign on the row with the reasons on hover. Cheap checks only: every visible row asks every frame |
| HUD preview fix | The HUD editor's Interact preview failed to build on a new project: its scene script and Blueprint classes went into the native preview runner. `hud_simulation::preview_inputs` leaves both out of the preview, which only needs the HUD scene |
| Scripted capture | `--open-add-2d-node <query>` opens the menu with a query for screenshots and tests |
