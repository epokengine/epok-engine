# Phase 3 — Content and polish (2026-10-13)

User guide: [docs/2d.md](../../../docs/2d.md) (Paths and moving platforms, Lines and
polygons, Particles2D, PointLight2D, AudioPlayer2D, Notifiers and remote transforms,
Colliders from sprites).

## 1. Runtime (`runtime/world2d_content.hpp`)

| Component | Contract |
| --- | --- |
| `Path2DComponent` | Cooked polyline (curves sampled by the editor, 12 steps per curved segment) with cumulative lengths; closed paths repeat their first point. `sample_x/y(distance)` in world pixels |
| `PathFollow2DComponent` | Follows the Path2D of `path` (an actor reference cooked as `actors[slot]`) or of its attach parent; progress in pixels, speed, loop / ping-pong / stop, offset along and across, optional rotation (rational arctangent). Advanced before the collision sync each step; bodies whose floor collider belongs to the moved actor are displaced by the same delta (moving platforms) |
| `Line2DComponent` | Up to 32 inline, editable points; width, start/end colour, closed; drawn as Gouraud quads with square caps at inner joints |
| `Polygon2DComponent` | Editor-triangulated (ear clipping, crossing outlines rejected); flat or textured triangles |
| `Particles2DComponent` | CPU emitter over a pool of `EPOK_WORLD2D_PARTICLES` particles shared by the map (cooked from the sum of amounts); steady or burst, one-shot with `take_finished`, point / rectangle / circle shapes, gravity, damping, scale and colour over life, flipbook frames, local or world coordinates. Emitters release their particles when destroyed |
| `AudioPlayer2DComponent` | Owns an `AudioSource`; per step its volume follows the distance to the camera centre (linear to `max_distance`, shaped by `attenuation`) and `AudioSource::pan` (new, -127..127, in existing padding) places it left or right; `audio_parameters` lowers the far channel |
| `VisibleOnScreenNotifier2DComponent` | Rectangle tested against the current view each step; entered / exited edges; optional owner deactivation off screen |
| `RemoteTransform2DComponent` | Pushes world position, rotation and/or scale onto a target actor each step |

The draw list gained entry kinds (`DrawKind`: sprite, tiles, line, polygon, particles)
expanded by `each_line_quad`, `each_polygon_triangle` and `each_particle`. Untextured
primitives use their colour as is (textured ones treat 128 as 1.0); lines and polygons
repeat under a repeating Parallax2D from their points' screen bounds. Native PC draws
lines as flat quads per segment and triangles as quads with a repeated corner.

## 2. Editor

- `shape2d.rs`: Bézier sampling and lengths, Catmull-Rom style smoothing, ear clipping
  with a crossing check, and a stateless particle preview.
- `world2d_shapes_editor.rs`: on-canvas point editing (drag, midpoint insert, Ctrl+click
  append, right-click / Delete remove, mirrored curve handles with Shift to break them),
  the Points row (edit / add / delete, closed, smooth, straighten), drawing of every
  content node (path direction arrow, follower start ring, gradient lines, textured
  polygons, live particle preview with emission outline, notifier rectangle, remote
  target link, audio radius), Inspector bands with particle presets, and the generated
  PointLight2D texture.
- Presets are grouped by purpose in every Add 2D Node menu; nodes that draw shapes no
  longer show the empty-node cross; parallax repeats include shapes.
- Collider2D: **Fit to Visible Pixels** (tight box of the frame's opaque texels).

## 3. UI widgets and localization

User guide: [docs/hud.md](../../../docs/hud.md) (Pressing, check boxes and sliders;
Colour tags; Localization).

| Piece | Contract |
| --- | --- |
| `Focusable` | `kind` (`FocusKind::Button` / `Toggle`), `checked`, `events` (`focus_pressed`, `focus_changed`, `focus_cancelled`); with `ProgressBar::step`, `ActorData` grows 8 bytes per slot |
| `hud_focus_update` | Cross sets pressed (a Toggle flips and sets changed), Circle sets cancelled; Left/Right on a focused Progress Bar with `step > 0` move the value by the step (clamped) instead of the focus |
| `ProgressBar::step` | Slider step; a focusable bar with a step draws a handle (one rectangle, tinted by the highlight) |
| `hud_core::check_box` | Frame, well and mark at the right edge of a Toggle's rect (3 rectangles, counted by the editor's budget check) |
| `hud_core::markup` | `[color=#rrggbb]` / `[/color]` recolour glyphs; tags draw nothing and take no room; malformed tags are text |
| `Localization` / `LocalizationLibrary` | Language-major `const char*` table; `get` falls back to the first language; `set_language` keeps the choice across banks; `language`, `language_count`, `key_count` |
| `Text::key`, `TextComponent::set_text_key` | Keyed texts; the bank's `apply` rewrites them after a switch |
| Cooking | A bank with keyed texts declares its own table in its namespace and installs it with `apply`; any other bank clears both, so nothing points into an unloaded bank |

Editor: Check Box and Slider recipes (amber highlight by default, also on Button);
Inspector Kind / Checked / Slider step / String key with "Shows:" preview and
Insert colour tag; **Window > Strings (Localization)** with languages, start language,
preview language, CSV import/export, untranslated counts, search and per-key grid
(`src/localization.rs`, `src/localization_editor.rs`, asset kind `StringTable`).
Native preview protocol 6 carries toggle, checked and step.

Deferred from the phase 3 UI scope, documented in docs/hud.md: scroll containers,
tab bars and shared themes. Also deferred from phase 3: interactive XA music
(segment switching), Timeline tracks driving 2D components, and an importer for
scenes from other engines.

## 4. Validation

| Check | Result |
| --- | --- |
| `tests/runtime/world2d_service.cpp` | paths (sampling, speed, wrap, ping-pong, rotation), moving-platform carry, line quads and gradients, polygon triangles and culling, particle bursts / streams / pool limit / release, notifier edges and deactivation, remote transforms, 2D audio gain and pan |
| `cargo test` | path sampling and smoothing, triangulation (concave, both windings, crossing rejection), particle preview, cooking of every content node and capacities, opaque bounds |
| PSX builds | Platformer 2D with a path lift, vine, fireflies and parallax mountains, in C++, Lua and Blueprint, played in PCSX-Redux |
| Native PC Play | draws lines, polygons and particles |
| `tests/runtime/hud_layout.cpp` | pressed / changed / cancelled events, toggles, slider steps and clamping, check box and slider handle geometry and tint, colour tags, string table fallback |
| `cargo test` | string table validation, CSV round trip, cooking of keyed texts and per-bank tables, recipes, budget counts |
| `tests/integration/verify_hud_widgets.py` | PCSX-Redux: a keyed label switched to the second language by a C++ component is pixel-identical to a plain label; Cross clears the check box; Down then Right raises the slider |
| Sample game | 2D content: static RAM unchanged (973.7 KiB). UI widgets: 976.0 KiB (+8 bytes per actor slot, the rest is code) |
