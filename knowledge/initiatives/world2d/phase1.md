# Phase 1 — Playable 2D (2026-10-13)

User guide: [docs/2d.md](../../../docs/2d.md).

## 1. Contract changes

- **Pixels, Y down.** `pixels_per_unit_2d` is 1 and the 2D world shares the screen's
  axes; `world2d_position_limit` grew from 8192 to 65536 pixels (squared distances stay
  below 2^59). No 2D content existed, so no document migrates.
- **One-way colliders** (`Collider2D::one_way`, `one_way_margin`) and per-axis blockers
  in `MoveResult2D` so a body can tell floors, walls and ceilings apart.
- **`ActorComponent::level_attached()`**, a non-reflected hook the Level calls when it
  binds a component to its owner. The 2D components register there (intrusive lists, no
  storage) and unlink in their destructor, so a Blueprint or Lua subclass that overrides
  `begin_play` without calling its parent still renders and collides. Reflection rejects
  custom constructors, which is why registration is not done at construction.
- **`Actor::overlap_2d(ObjectId other, TriggerPhase)`**, a Blueprint event every actor
  can override; component `trigger_event` receives the same deliveries.

## 2. Runtime

| Header | Contents |
| --- | --- |
| `world2d_components.hpp` | `Sprite2DComponent` (regions, sheet cells, cooked frame clips, flips, modulate, blend, z/y-sort), `Camera2DComponent` (drag margins, limits, smoothing, offset, zoom, rotation, shake, background), `Collider2DComponent`, `CharacterBody2DComponent` (`move_and_slide`, `move_and_collide`, floor/wall/ceiling, floor snap, floating mode), `world2d::find<T>` |
| `world2d_service.hpp` | GPU-free service: per-step collider sync into `CollisionWorld2D`, trigger dispatch, camera following, animation advance, the hooks behind `move_and_slide` / `raycast_2d` / `overlap_box_2d`, and `collect_sprites` (projection, culling, draw keys, stable radix sort) |
| `world2d_render.hpp` | PSX renderer: one byte arena per parity for `Sprite` rectangles, `TexturedQuad`s and `TPage` changes |

Cooking emits `EPOK_WORLD2D` and the capacities (`_COLLIDERS`, `_TRIGGER_PAIRS`,
`_DRAW_ITEMS`, `_ARENA_BYTES`) only when a bank or Blueprint template has 2D content;
`main.cpp` and the Native PC runner compile the service only then. A project without
2D content links exactly as before (the sample's static RAM is unchanged).

## 3. Editor

`world2d.rs` holds the document records (`sprite2d`, `camera2d`, `collider2d`,
`body2d`), the node presets, the texture references a bank must cook and the C++
setters the actor tables emit. `world2d_editor.rs` is the 2D view (tools, snapping,
rulers, previews, gizmos, drag-and-drop, context menu, empty state, automatic framing
of the game camera), the 2D Inspector and the region / sprite-sheet dialogs.
`platformer_template.rs` generates the Platformer 2D template, its artwork and its
C++, Lua and Blueprint gameplay.

The Lua frontend now resolves a typed receiver by class identity as well as by name,
so `epok.cast(ref, "epok::Actor2D"):set_active(false)` compiles.

## 4. Validation

| Check | Result |
| --- | --- |
| `tests/runtime/world2d.cpp` | units, projection (Y down), hierarchy, colliders, rays, movement, one-way platforms, triggers, picking |
| `tests/runtime/world2d_service.cpp` | sprite clips and events, camera limits/smoothing/make_current/shake, `move_and_slide` landing, walls, one-way from below and above, floating bodies, trigger Enter/Stay/Exit to `overlap_2d` and `trigger_event`, inactive owners, draw sort, sprite culling / y-sort / rectangle fast path |
| `cargo test` | world2d, platformer template (all flavors), editor camera |
| PSX builds | Platformer 2D in C++, Lua and Blueprint |
| `tests/integration/verify_world2d.py` | each flavor played in PCSX-Redux over MCP: run right, jump, run left; the three flavors produce identical frames |
| Native PC Play | draws the same 2D frame as the console |
