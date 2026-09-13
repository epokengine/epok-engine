# P11 — Editor: actor and component authoring (2026-09-13)

Closes the open issues P7 left behind (design.md sections 5, 7 and 8): actors gain a full
set of Hierarchy commands, the Inspector's **Add Component** button is no longer disabled,
component and actor properties are editable with override marking, a continuous drag is one
undo step instead of one per frame, `scene_set_actor` reaches parity with the UI, and the
`TwoD` placeholder becomes a real orthographic World2D view.

Everything an author can do here goes through `Editor::changed()` (or
`Editor::changed_coalesced`), so the map's dirty flag, the P7 undo history and Save remain
the only bookkeeping. Every mutation validates a *candidate* document with
`Scene::validate_with_model` before it is published; a refusal leaves the open document
byte-identical and reports the reason in `Editor::last_error` and the log.

## 1. Inspector — Actor section

### Components

Each component is a collapsible tree node inside the **Components** heading. The header row
is `name`, the class `cpp_name` in grey, a green `root` marker on the root, and a grey
`(class default)` marker on an `inherited` component. Expanding it shows:

| Row | Meaning |
| --- | --- |
| `Identity` | the component's UUID, read only |
| the property table | reflected properties of the component's class chain (section 3) |
| `Remove` | removes this component |

**Remove** is disabled with a tooltip in exactly two cases, and `Editor::remove_actor_component`
refuses the same two cases as a second line of defence:

| Case | Tooltip |
| --- | --- |
| the component is the actor's root | `the root component cannot be removed` |
| `ComponentInstance::inherited` is true | `inherited from the class; disable instead` |

Removing a component also clears any `ActorInstance::attach` that named it and any sibling
`attach_parent` that pointed at it, so nothing is left dangling.

### Add Component

A **Add Component** button opens the `add-component` popup. Its entries are
`Editor::addable_component_classes(actor)`: every class in the `Model` that has a
`ComponentContract` and that `Model::validate_component(owner_class, component_class)`
accepts. They are grouped under two grey headings, in this order:

1. **Shared** — components whose `owners` set is not a single domain, that is the domain-less
   and multi-domain ones (`epok::AudioComponent`, `epok::LegacyBehaviourComponent`, …).
   They are offered for what they do rather than for where the actor lives.
2. **Domain** — components owned by exactly one domain, which for an eligible class is
   always the owner actor's own domain.

Each group is sorted by the unqualified class name; hovering an entry shows the qualified
`cpp_name`. The button itself is disabled when the model offers nothing, with the tooltip
`No component class of this project may be attached to this actor.` — which is also what an
unresolved reflection model produces.

Choosing a class appends a `ComponentInstance` with:

- a fresh `Uuid::new_v4()`;
- `name` = the unqualified class name, suffixed `.001`, `.002`, … until it is unique **inside
  that actor** (`actor_document::unique_component_name`); component names are unique per
  actor, never per map;
- `root = false` and `inherited = false` — a root comes from the class, never from this
  button, and an instance-authored component is never a class default;
- empty `properties` and `overrides`.

The whole resulting set is then checked with `Model::validate_component_set` (one root,
root domain, `Requires`/`Excludes`, `Cardinality`, capabilities). On failure the component
is not added and the diagnostics are joined into `last_error`.

### Properties

The **Properties** heading edits `ActorInstance::properties` for the actor's class chain
with the same table (section 3). A footnote reads *"An edited value is an override even when
it equals the class default."*

## 2. Hierarchy — actor commands

An actor node is a drag source and a drop target for the payload `EPOK_ACTOR` (the actor's
index in `scene.actors`, resolved to its UUID in the same frame). The drag tooltip reads
*"Drop on an actor to parent; drop on the map to unparent"*.

| Affordance | Effect |
| --- | --- |
| double-click, or context menu **Rename** | inline `input_text` over the node, exactly like an entity rename: Enter or blur commits, Escape discards, an empty or unchanged name is a no-op |
| context menu **Duplicate** | copies the actor and its whole logical branch |
| context menu **Delete** | removes the actor and its whole logical branch |
| context menu **Move to Map Root** | `logical_parent = None`, `attach = None` |
| context menu **Convert class...** | permanently disabled (see below) |
| drag an actor onto another actor | reparents the dragged actor under the drop target |
| drag an actor onto the empty area below the tree | moves it to the map root |
| `Delete` key while the Hierarchy has focus | deletes the selected actor when the selection is an actor, otherwise the selected entity, as before |

The grey line *"Duplicate / Delete includes child actors"* appears when the actor has
children. Opening the context menu selects the actor, as the entity menu does.

### Reparent rules

`logical_parent` is what the author sees; `attach` is the spatial relationship. A reparent
sets them together:

| Situation | `logical_parent` | `attach` |
| --- | --- | --- |
| both actors resolve to the **same spatial domain** (`World3D`, `World2D` or `UI`) | the new parent | `Attachment { actor: parent, component: None }` — the parent's root component |
| the two actors resolve to **different domains**, or either is `Domain::None`, or either class does not resolve | the new parent | cleared |
| dropped on the map root | `None` | cleared |

A parent that would close a cycle is rejected by `Scene::validate_with_model`
(`Actor \`X\` is its own ancestor`) and nothing is applied. Dropping an actor on itself is
refused before validation.

### Why there is no "Move to domain"

**Convert class...** is always disabled, with the tooltip:

> Changing an actor's domain (Actor3D to UIActor, say) replaces its root and its domain
> components. It is an explicit conversion, not a move, and is not offered yet.

That is the design decision, not a missing feature: a drag expresses hierarchy, and a
domain change rewrites the root component and every domain-scoped component on the actor.
It has to be an action an author asks for by name.

### Duplicate and delete semantics

`Scene::duplicate_actor_branch(id)` copies the actor and every actor whose `logical_parent`
chain reaches it. Identities come from `actor_document::fresh_identities` +
`remap_actor` (the P4 helpers), so actors, components, `attach`, `attach_parent` and
intra-branch `logical_parent` are all renumbered while references that leave the branch keep
their target. Names are suffixed `.001`, `.002`, … against both actors and entities, exactly
like `Scene::duplicate_branch`. `legacy_entity` is cleared on the copies: a copy is a new
placement, not a second view of the same legacy entity.

`Scene::delete_actor_branch(id)` removes the same set, then clears every surviving
`logical_parent` and `attach` that pointed into it, including an `attach` that named a
*component* of a removed actor. Deleting an actor that is no longer in the map is refused
(`That actor is no longer in this map.`) rather than recorded as an empty history step.

## 3. The shared property table

`gui::reflected_property_table` renders the reflected properties of one class chain and
returns at most one `PropertyAction` per frame, which keeps it a pure renderer over a
borrowed document. It is used for `ActorInstance::properties` and for every
`ComponentInstance::properties`.

- Properties come from `Registry::properties(cpp_name)`, which already flattens the chain;
  the class is resolved through the `Model` first so a `class_id` works as well as a name.
  `editable = false` properties are skipped.
- Values are edited with `blueprint_refs::inspector` — the same editor the ScriptBinding
  inspector uses, so asset, entity and class references get their pickers here too.
- A property with no authored value shows the class default.
- **Override marking is explicit.** Editing a value inserts it into `properties` *and* into
  `overrides`, even when the new value equals the default: "I chose this" and "I did not
  choose" are different authored states, and the cook reads `overrides` first.
- An overridden property shows a green `override` marker and a **Reset to inherited** button
  that removes the key from both `properties` and `overrides`.
- A value whose property the class no longer declares is shown in amber as
  `Preserved orphan: key = value` with a **Discard orphan** button. It is never dropped
  silently.

## 4. History coalescing

`Editor::changed()` keeps its meaning: one call, one undo step. Two new methods sit beside
it, both sharing the same body (`apply_change` + `finish_change`), so Blueprint override
recording, the stale-build flag, the dirty flag and the bake state behave identically:

| Method | Meaning |
| --- | --- |
| `changed()` | a discrete edit. Records one step and **ends any open run**. |
| `changed_coalesced(key: &'static str)` | a frame of a continuous gesture named `key`. The first call of a run records one step; every further call with the same key only moves the history baseline forward. |
| `end_coalesced()` | closes the open run, so the next gesture with the same key records its own step. Called when the mouse is released. |

Underneath, `History<T>` gained `run: Option<&'static str>`, `record_coalesced` and
`end_run`. A run is ended by: a `record` (discrete edit), a different key, an `undo`, a
`redo`, a `reset` (a different document), or `end_run`. Undoing once after a fifty-frame
drag returns to the value the drag started from.

Keys in use:

| Key | Site |
| --- | --- |
| `gizmo-drag` | `gizmo.rs` — translate / rotate / scale handles in the 3D viewport |
| `hud-rect-drag` | `hud_editor.rs` — moving and resizing a `RectTransform` in the Canvas editor |
| `actor-2d-drag` | `gui.rs` — dragging an Actor2D in the 2D world view |
| `scene-blueprint-sync` | `editor.rs::sync_embedded_blueprint` — the P6 embedded Blueprint reporting an edit back into the map |

Tests: `scene_view_mode::tests::a_coalesced_run_is_one_step_and_a_different_key_breaks_it`
(50 coalesced calls collapse to one step; a different key, a discrete edit and `end_run`
each break the run) and `editor::tests::a_coalesced_gesture_records_one_scene_history_step`
(the same through a real `Editor`).

## 5. The 2D world view

`SceneViewMode::TwoD` now draws a World2D authoring view. It is pure ImGui draw-list
rendering — no wgpu, no render target, no change to the 3D preview, which resumes untouched
when the author switches back.

### Coordinate convention

Taken from `runtime/world2d.hpp` so the editor and the runtime agree:

```text
screen = viewport_centre + (world - camera.centre) * zoom * 32
```

- **Y is up in world space, down on screen** — the only sign difference from the runtime
  formula. The editor camera has no rotation (the runtime's `R(-camera.rotation)` term is
  identity here), because a rotated authoring view makes the handles ambiguous.
- **32 screen pixels per world unit at zoom 1** — `scene_view_mode::PIXELS_PER_UNIT_2D`,
  which is `epok::pixels_per_unit_2d`. A 2D world unit is not a pixel.
- Rotation is degrees, counter-clockwise (+X towards +Y).
- Positions are clamped to ±8192 world units, the `world2d_position_limit` of the runtime.
- Zoom is clamped to `[0.05, 16]` — editor ergonomics, not a document rule.

`SceneView2D` (in `src/scene_view_mode.rs`) owns `centre`, `zoom`, `world_to_screen`,
`screen_to_world`, `pan_pixels`, `zoom_at` and `reset`, and is unit tested against the
formula above including the round trip, the cursor-anchored zoom and both clamps.

### What is drawn

- A two-density grid whose spacing steps through 1, 5, 25, 125, 625 world units, choosing
  the first that keeps lines at least 12 px apart, with the multiple-of-five lines brighter.
- The world axes through the origin: +X red, +Y green, matching the 3D viewport's gizmo.
- Each **World2D** actor — that is, each actor whose class resolves to `Domain::World2D` —
  as the one unit square centred on its origin, transformed by its root component's
  rotation and scale, filled translucent blue and outlined; the selected actor is orange and
  thicker. The actor's name is drawn next to its origin. An actor with no root component is
  labelled `no root component` in amber and drawn at identity.
- Actors are drawn back to front by `draw_order`, ties broken by document order, which is
  the ordering `draw_key_2d` uses at runtime.

### Reading and writing the transform

`runtime/object_model.hpp` declares the member as a single `Transform2D transform`, so a
document may carry either flattened override keys or one nested object.
`scene_view_mode::read_transform_2d` accepts both and falls back to the runtime identity per
key:

| Key | Type | Default |
| --- | --- | --- |
| `position` | `[x, y]` numbers, world units | `[0, 0]` |
| `rotation` | number, degrees | `0` |
| `scale` | `[x, y]` numbers | `[1, 1]` |
| `draw_order` | integer | `0` |

A key that is absent, of the wrong type, or malformed falls back to its default rather than
hiding the actor. `write_position_2d` keeps the shape the document already uses — a nested
`transform` object stays nested, anything else writes the flat `position` key — and always
marks the key as an override, because a drag is an explicit edit.

### Interaction

| Gesture | Effect |
| --- | --- |
| middle or right mouse drag | pan; the world point under the cursor stays under it |
| mouse wheel | zoom anchored on the cursor |
| **Reset View** button | camera back to origin at 100 % |
| left click | selects the topmost actor under the cursor (highest `draw_order` first), or clears the selection on empty space |
| left drag | moves the selected actor, writing `position` into its root component through `changed_coalesced("actor-2d-drag")`, so the whole drag is one undo step |

The header reads `2D World`, the zoom percentage and `32 px per unit`, over the hint line
*"World Y is up, 32 px per unit at 100%. MMB/RMB drag: pan | wheel: zoom | LMB: select and
move"*. `Ctrl+Z` / `Ctrl+Y` work here through the shared `scene_shortcuts`.

Empty states, which are what a project without resolvable reflection data sees:

- the model declares at least one placeable World2D actor class →
  *"No World2D actors in this map. Create one from the Hierarchy's Actor > 2D menu."*
- otherwise → *"This project declares no placeable World2D actor class. Compile the project
  so its reflection data resolves, or derive an Actor2D subclass."*

## 6. MCP changes

### `scene_set_actor`

Two new arguments, both optional, both applied to the same candidate scene as the existing
`active` / `name` / `properties` and validated together before anything is published.

| Argument | Shape | Rules |
| --- | --- | --- |
| `parent` | actor UUID, or `null` | Mirrors the Hierarchy drag exactly: `null` moves the actor to the map root; the spatial `attach` is set only when both actors resolve to the same spatial domain and is cleared otherwise. An unknown id, the actor's own id, or a parent that would close a cycle is refused and nothing is applied. |
| `components` | `{"add": [...], "remove": [...]}` | `remove` first, then `add`, then the whole set is validated. Any other key is refused (`components: unknown operation X; use add and remove`). |

`components.remove` takes component UUIDs. The root and `inherited` components are refused
with the same sentences the Inspector tooltips show. A removed component's `attach` and
`attach_parent` references are cleared.

`components.add` takes `{"class": "<cpp_name or class id>", "name": "<optional>"}`. Each
entry is checked with `Model::validate_component`, gets a fresh UUID, a name unique inside
the actor (defaulting to the unqualified class name), `root = false` and `inherited = false`.
Adding requires a resolvable model. The resulting set is checked with
`Model::validate_component_set`.

The reply now carries `logical_parent`, `attach` and the component list
(`id`, `name`, `class`, `root`) beside `revision` / `id` / `saved`.

### `scene_actors`

Each actor gains `attach`. Each component entry gains `class_id`, `inherited`, `overrides`
(which keys the author set explicitly, distinct from `properties`, because a value may equal
the class default and still be an override) and `removable` (false for the root and for
class defaults).

### Shared helpers

`mcp_tools` exports the rules so the UI and the tools cannot drift:
`validate_actor_components`, `component_removal_refusal` and `implied_attachment`.
`Editor::add_actor_component`, `remove_actor_component` and `reparent_actor` call them.

## 7. Deferred

| Work | Note |
| --- | --- |
| Converting an actor's class or domain (`Convert class...`) | Explicit conversion; needs a root/component rewrite plan and a preview of what is lost. |
| Setting `root` on an added component, or re-rooting an actor | The root still comes from the class chain only. |
| Editing a component's `attach_parent` (intra-actor attachment) | Read-only; the document supports it, the UI does not expose it. |
| Reordering components | The list is document order. |
| Reparenting *into* a specific component (`Attachment::component`) | The UI always attaches to the parent's root. |
| Rotation and scale handles in the 2D view | Only position is draggable; rotation, scale and `draw_order` are edited numerically in the Inspector. |
| Camera2D authoring and a 2D grid-snap | Not started. |
| Sprite/collider preview in the 2D view | Every actor is drawn as its one-unit footprint; there is no authored 2D geometry yet. |
| Multi-select of actors | Selection is still a single `Option<Uuid>`. |
| The conversion action for hybrid 3D+UI entities that `actor_view` diagnoses | Still open from P7. |

## 8. Validation run on this machine

| Step | Result |
| --- | --- |
| `cargo build --locked --bins` | succeeds |
| `cargo test --locked scene_view_mode` | 6 passed, 0 failed |
| `cargo test --locked mcp` | 10 passed, 0 failed |
| `cargo test --locked editor` | 71 passed, 2 failed — both on the P0 baseline list |
| `cargo test --locked gui` | 1 passed, 1 failed — the pre-existing shared-ImGui `gui::interaction_tests::scene_clicks_and_hierarchy_context_menu_use_real_imgui_events` |
| `cargo test --locked` | **466 passed, 7 failed, 33 ignored** — the 7 are exactly the P0 baseline list, no new failures (P7 was 426, P10's merge base 460) |
| `cargo clippy --locked --all-targets -- -D warnings` | 25 errors, exactly the pre-existing baseline count; none added |
| `cargo fmt` | the repo is not fmt-clean; every hunk rustfmt reports inside this phase's own code was applied by hand, and no unrelated file was reformatted |
| `cargo run -- --project examples/sample-game --screenshot` | renders; the editor comes up with the 3D view and the Hierarchy as before |
| `cargo run -- --project examples/sample-game --screenshot-2d --screenshot` | renders the 2D world view: the two-density grid, the red/green world axes through the origin, the `2D World / Reset View / 100% | 32 px per unit` header, the convention hint line and the empty state |

MIPS builds, standalone exports and emulator runs were not executed (no SDK on this machine)
and nothing about them is claimed.

**The sample project has no resolvable reflection model here** (libclang 18.1.1 is missing,
which the editor's own "Missing editor dependencies" dialog reports). `--add-actor` confirms
it: `Placeable classes:` comes back empty. Every affordance that needs a `Model` — the Add
Component popup, the component property tables, the 2D actor rectangles — was therefore
exercised by unit tests against a hand-built class table (`mcp_tests::actor_catalog`, now
carrying `epok::Actor2D`, `epok::SceneComponent2D` and `epok::AudioComponent` as well) and by
its empty-state paths on screen, not by a screenshot of real content.

## 9. Tests added

`src/scene_view_mode.rs` (3 new, 6 total):

- `a_coalesced_run_is_one_step_and_a_different_key_breaks_it` — 50 coalesced calls with one
  key produce one step; a different key, a discrete edit and `end_run` each break the run.
- `the_2d_camera_matches_the_runtime_projection` — the centre and Y-up sign, the round trip
  panned and zoomed, panning dragging the world with the cursor, cursor-anchored zoom, and
  both zoom clamps.
- `a_2d_transform_reads_flat_or_nested_keys_and_defaults_to_identity` — flat keys, nested
  `transform`, per-key identity fallback, malformed values, and that writing keeps the
  document's shape and always marks an override.

`src/editor.rs` (2 new):

- `actor_operations_rename_duplicate_delete_reparent_and_edit_components` — rename commit and
  discard; same-domain and cross-domain reparent and their attachments; cycle refusal leaving
  the document untouched; add with fresh identity, unique name, `root`/`inherited` false;
  the Add Component grouping and its contents; root, inherited and domain refusals;
  cardinality; duplicate producing a fully distinct identity set; delete taking the branch;
  refusing to delete nothing; undo walking back.
- `a_coalesced_gesture_records_one_scene_history_step` — 50 coalesced frames undo to the
  start in one step, and a different key and a discrete edit each record their own.

`src/mcp_tests.rs` (1 new):

- `scene_set_actor_reparents_and_edits_the_component_set_like_the_inspector_does` — the whole
  of section 6, including the refusals and the advertised-schema type checks.

## 10. Files

`src/gui.rs`, `src/editor.rs`, `src/scene_view_mode.rs`, `src/mcp_tools.rs`,
`src/mcp_tests.rs`, `src/hud_editor.rs`, `src/gizmo.rs`, `src/scene.rs` (five appended
`pub` helpers: `actor_index`, `actor_branch`, `unique_actor_name`, `delete_actor_branch`,
`duplicate_actor_branch`), `src/actor_document.rs` (two appended `pub` helpers:
`short_class_name`, `unique_component_name`), `src/main.rs` (one new screenshot flag,
`--screenshot-2d`).

## 11. Open questions for later phases

- **`--add-actor` splits the class and the name on the first `:`**, so
  `--add-actor epok::Actor2D:Sprite` is read as class `epok`. The P10 CLI needs
  `rsplit_once(':')`, or a different separator; it was left alone here because `main.rs`
  belongs to the cook/runtime side of this initiative. No test covers it because no machine
  in this initiative has a resolvable model for the sample project.
- The 2D view draws every actor as the same one-unit square. Once `Sprite2DComponent` and
  `Collider2DComponent` carry authored extents, the footprint should come from them, and
  the picking test should follow.
- `changed_coalesced` takes a `&'static str`. A gesture that needs to distinguish *which*
  object is being dragged (two gizmo drags on different entities without an intervening
  release) would need the key to carry the identity. In practice the mouse release between
  them calls `end_coalesced`, so this has not been needed.
- Component names are unique per actor, actor names are unique per map (against entities
  too). Nothing enforces the actor rule on a document written by hand or by MCP; the
  Inspector and the Hierarchy only guarantee it for names they generate.
