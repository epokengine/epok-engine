# P9 — Services and preview (2026-09-13)

Scope of this task: the shared `AudioComponent` contract and the service quarantine in
`runtime/object_model.hpp`, a new host test `tests/runtime/actor_services.cpp`, the
`TargetCapabilities` model in `src/object_model.rs`, the actor-side half of
`TimelineRequires=` in `src/timeline_compile.rs`, and the native-preview phase boundary and
protocol version across `src/hud_native.rs`, `src/hud_simulation.rs`,
`native/hud_preview.h` and `native/hud_runner.cpp`.

`runtime/main.cpp`, `runtime/scene_service.hpp`, `runtime/lifecycle.hpp` and the editor
document/scene modules were **not** modified: this phase defines the contracts and proves
them on the host, and the wiring that makes the cooked build install the audio quarantine
hook and drive `dispatch_trigger` from the collision services is listed under *Deferred*.

The plan file the task referenced (`knowledge/initiatives/bp-scene-optimization.mdy`) does
not exist in this tree; the contracts below follow `design.md` sections 2, 3, 5 and 6 and
the P9 task statement.

## 1. Audio: who starts, who stops, who releases

An `AudioComponent` is a **view** over an `epok::AudioSource`. Two kinds of storage exist,
and exactly one path may start a given source — that is the whole point of the table.

| Storage | Bound by | `play_on_start` fires in | Stops on | Storage released by |
| --- | --- | --- | --- | --- |
| **Legacy slot** (`bind_slot(Entity&)`, `source == &entity.audio`) | Scene-bank migration of an entity that already had an `AudioSource` | The **legacy** path only: the scene bank's bank-load loop and `bp::activate_spawn_audio` (`runtime/lifecycle.hpp`). `AudioComponent::begin_play` deliberately does nothing. | `lifecycle.hpp::set_active` (slot table walk) **and** the component's `on_disable`/`end_play`; both are idempotent because stopping a stopped source is a no-op | The scene bank. The component never holds the slot back (`releasable()` is always `true` for slot storage); `create_entity` keeps its existing `music_active`/`music_requested` check |
| **Component-owned** (`bind_local()`, `source == &local`) | An actor that owns its audio, with no legacy entity behind it | `AudioComponent::begin_play`, **once**, when `source->enabled && source->play_on_start && owner_active()` and it is not already playing | `on_disable` (via `Level::set_active`) and `end_play` (via teardown) | The registry, but **only** when `releasable()` is true — see the quarantine below |

`owner_active()` folds the logical parent chain through `Level::actor_active`, so audio on
an actor under an inactive parent does not start. `begin_play` is also idempotent: a
component added after its owner began play begins immediately, and re-entering `begin_play`
on an already-playing source adds no second play.

### 1.1 Why the slot case must not play

A migrated entity is reachable from both sides: the scene bank still owns the slot and
still runs `activate_spawn_audio` over it, while the actor now also owns a component that
views the same `AudioSource`. If both started it, a spawned entity with `play_on_start`
would key on twice — audibly, and with a stolen SPU voice. The rule is therefore stated in
the storage, not in a caller: `owns_source()` is false for slot storage, and `begin_play`
returns immediately.

### 1.2 Quarantine (`releasable()`)

`runtime/music.hpp` keeps `music_active` and `music_requested` pointing at an `AudioSource`
across asynchronous CD lookup/stop completions. That is exactly why `create_entity`
(`runtime/lifecycle.hpp`) refuses to reuse a legacy slot whose `audio` is one of them.
Component-owned storage needs the same rule, because the object pool would otherwise
construct a new component over memory the XA consumer still writes to.

* `ActorComponent::releasable()` — new virtual, default `true`.
* `AudioComponent::releasable()` — `false` while the installed predicate says the XA
  consumer retains `&local`. Slot storage always returns `true`.
* `ObjectRegistry::finish_release()` skips a quarantined instance instead of returning it
  to its pool, and counts it in `ObjectStats::quarantined` (a **gauge**, reset each pass).
  The handle is dead either way — `release()` already bumped the generation — so the only
  effect is that the slot and pool entry are not reused yet.
* `ObjectRegistry::collect_quarantined()` retries. The scene bank calls it once per frame,
  next to the point where the legacy path re-checks `music_active`.

The predicate is installed rather than hard-wired:

```cpp
inline bool (*audio_source_retained)(const AudioSource*) = nullptr;
```

`object_model.hpp` cannot name `music_active` directly: `runtime/music.hpp` defines it as an
`inline` variable, and an earlier `extern` declaration of the same name is ill-formed
([dcl.inline] requires the first declaration to be the inline one). With no hook installed
nothing is quarantined and release is immediate, which is the correct behaviour for a build
without the XA music service.

## 2. Target capabilities

`KNOWN_CAPABILITIES: &[&str] = &["audio"]` is replaced by a `TargetCapabilities` value built
from the target.

| Capability | PSX provides | Source |
| --- | --- | --- |
| `audio` | yes | SPU + XA services |
| `hud` | yes | `runtime/hud_core.hpp` |
| `sprites` | yes | sprite rendering |
| `particles` | yes | particle/effect runtime |
| `timelines` | yes | timeline runtime |
| `physics3d` | yes | `runtime/collision.hpp` |
| `physics2d` | **only if `runtime/world2d.hpp` is cooked** | `project::runtime_sources()` is the exact list the cook copies into the build, so asking it *is* the "is `world2d.hpp` present" question. It is present in this repository, so PSX provides `physics2d` today. |

API:

```rust
TargetCapabilities::psx()                       // this repository's cook
TargetCapabilities::psx_with_world2d(bool)      // explicit, for tests and trimmed targets
TargetCapabilities::none() / .with(n) / .without(n) / .provides(n) / .names()
Model::validate_capabilities(&TargetCapabilities) -> Vec<Diagnostic>
```

`validate_capabilities` walks the whole class table and returns one `unknown-capability`
diagnostic per class/capability pair the target does not provide, sorted for a stable build
log. `Model::validate_component_set` keeps its existing signature (`src/actor_document.rs`
calls it) and validates against `TargetCapabilities::psx()`; the explicit entry point is for
a caller that knows its target. The diagnostic code, message and shape are unchanged, so
nothing downstream had to move.

## 3. Timeline requirement mapping

`TimelineComponentRequirement` is expressed in two vocabularies. On a legacy entity it
means "this entity member is present and enabled" (`runtime_member()`, enforced by
`timeline::validate_components`). On an actor the same requirement means "this component
class is on the actor".

| `TimelineComponentRequirement` | Legacy entity member | Actor component (design.md §2) |
| --- | --- | --- |
| `Camera` | `camera` | `epok::Camera3DComponent` |
| `AudioSource` | `audio.enabled` | `epok::AudioComponent` |
| `Light` | `light.enabled` | `epok::Light3DComponent` |
| `RectTransform` | `rect.enabled` | `epok::RectTransformComponent` |
| `Text` | `text.enabled` | `epok::TextComponent` |
| `Image` | `image.enabled` | `epok::ImageComponent` |
| `ProgressBar` | `progress.enabled` | `epok::ProgressBarComponent` |
| `PaletteAnimator` | `palette_animator.enabled` | *(none reserved)* — not checked on actors |
| `ParticleEmitter` | `particle_emitter.enabled` | *(none reserved)* — not checked on actors |

`object_model::timeline_requirement_component(req) -> Option<&'static str>` is the mapping.
It returns `Option` rather than `&'static str` because `PaletteAnimator` and
`ParticleEmitter` have no reserved UUID in design.md section 2; returning an invented class
name would manufacture a false diagnostic for every actor. Introducing those two classes is
the only change needed to complete the table.

`Model::validate_timeline_requirement(owner, components, req)` produces a
`missing-timeline-component` diagnostic naming both the actor and the component, and
`timeline_compile::verify_actor_requirements` walks the adapter's ancestry the same way
`timeline::validate_components` does, so an inherited `TimelineRequires=` still applies.

Two deliberate non-failures: a requirement with no reserved class, and a reserved class the
registry has not reflected yet (`Camera3DComponent` today), both return `Ok`. Neither check
replaces the other, the set of requirements is unchanged, and no existing timeline adapter
test was weakened — `requires_excludes_cardinality_and_capabilities` only changed its
fixture capability from `physics2d` (which PSX now legitimately provides) to `shader-graph`,
which no target provides.

## 4. Preview phase boundary and protocol

**The edit-time preview does not run BeginPlay; the simulation does.** That sentence is now
a type, a protocol field and a check on both sides rather than a convention.

| | `PreviewPhase::Edit` | `PreviewPhase::Simulate` |
| --- | --- | --- |
| Entered by | the automatic preview (`--edit`) | the optional **Interact** session |
| Runs | property binding and public `editor_preview(Transform&)` construction hooks | the ordinary gameplay lifecycle |
| Does **not** run | BeginPlay/`start`, `update`, `frame_update`, game clock, input | — |
| Reported as | no capability bits | `EPOK_HUD_PREVIEW_CAP_SIMULATE` |

The child process fixes its phase at startup from `--edit` and reports it in **every** frame
header; `Session::accept` rejects a frame whose phase disagrees with the phase the session
asked for, so the editor can never believe a frame came from a phase that did not produce
it.

### 4.1 Protocol version

| Constant | Value | Defined in | Checked in |
| --- | --- | --- | --- |
| `EPOK_HUD_PREVIEW_MAGIC` / `HUD_PREVIEW_MAGIC` | `0x31445548` ("HUD1") | `native/hud_preview.h` / `src/hud_native.rs` | `hud_simulation::read_frame` |
| `EPOK_HUD_PREVIEW_PROTOCOL_VERSION` / `HUD_PREVIEW_PROTOCOL_VERSION` | **2** (was implicit 1) | both | both |
| `..._CAP_SIMULATE` | `1` | both | `read_frame` → `Frame::phase` |
| `..._CAP_BLUEPRINT` | `2` | both | `Frame::blueprint_support()` — **never set** |

Version 2 adds the version and capability words to the frame header, immediately after the
magic. A cached executable built from an older header is now rejected with a message telling
the author to restart the preview, instead of being decoded with the wrong field layout.

### 4.2 Blueprint diagnostic

`blueprint_support` is `false` in the handshake because the runner links C++ controllers
only. A scene whose entity is driven by a Blueprint class previously fell into the generic
"unsupported component" message, which reads as a missing feature. It now gets its own:

> Blueprint logic is not simulated in the native preview. This preview compiles C++
> controllers only; use Play or the console build to run Blueprint classes. `<Name>` is
> driven by a Blueprint class.

The point is that nothing is rendered as if the Blueprint had run. Non-`cpp`, non-Blueprint
providers keep the existing generic message.

## 5. Other service adapters

`LegacyBehaviourComponent` already forwarded `start`/`update`/`frame_update`/`on_enable`/
`on_disable`/`on_destroy` exactly once each. P9 adds:

* **Pause semantics, documented and tested.** `frame_update` runs once per rendered frame
  **even while the simulation clock is paused**; `tick`/`update` does not. `main.cpp` drives
  the two from separate places (`Level::frame_update` every frame, `Level::tick` only for
  the fixed steps the Time service schedules), so a pause menu keeps receiving input edges
  through `input.frame_pressed()`. An inactive actor receives neither.
* **Trigger forwarding.** `ActorComponent::on_trigger(EntityHandle, TriggerPhase)` is a
  new forward-only virtual (default empty); `LegacyBehaviourComponent` forwards it to
  `Behaviour::on_trigger`. The Level does not own collision, so the entry point is a free
  function:

  ```cpp
  size_t dispatch_trigger(Level&, ObjectId actor, EntityHandle other, TriggerPhase);
  ```

  It resolves the actor, skips inactive/doomed/unknown actors, opens an
  `ObjectDispatchScope` (so a destroy requested inside the callback is deferred), and fans
  the event out to every component in registration order, returning the number reached.
  Nothing in the object model *generates* trigger events; the collision services call this.

## 6. Deferred

| Item | Why | Where it lands |
| --- | --- | --- |
| Installing `audio_source_retained` from `music_active`/`music_requested`, and calling `collect_quarantined()` per frame | `runtime/main.cpp` and `runtime/scene_service.hpp` are owned by another phase | Level/World wiring phase |
| Calling `dispatch_trigger` from `runtime/collision.hpp` / `runtime/world2d.hpp` | same | Level/World wiring phase |
| Positional 2D/3D audio (attenuation, panning, a `SceneComponent`-attached audio source) | `AudioSource` has no position; the SPU path is mono per voice | after the spatial component adapters exist |
| `Camera3DComponent`, `Light3DComponent`, `Text/Image/ProgressBarComponent`, `PaletteAnimatorComponent`, `ParticleEmitterComponent` adapters | reserved IDs only (design.md §2); the timeline mapping is ready for them | P3/P8 component adapters |
| Full animation/timeline component adapters (binding a timeline to an actor rather than an entity) | needs the component classes above and the actor timeline binding UI | later |
| Surfacing the preview capability list in the editor UI (the `#[allow(dead_code)]` on `blueprint_support`/`runs_begin_play`) | the protocol reports it; no panel displays it yet | preview UI phase |
| `AudioComponent` positional/priority authoring in the Inspector | component add/remove UI is P9-editor, a different task | editor phase |

## 7. Validation

| Check | Result |
| --- | --- |
| `clang++ -std=c++20 -fsyntax-only -Wall -Wextra -fno-rtti -fno-exceptions` on `runtime/object_model.hpp` | clean |
| `tests/runtime/verify_spatial.py` (now including `tests/runtime/actor_services.cpp`) | pass, exit 0 |
| `tests/runtime/verify_blueprint_runtime.py` | pass (MIPS header validation skipped: no SDK on this host) |
| `cargo test --locked object_model` | 18 passed, 0 failed (3 new) |
| `cargo test --locked timeline` | 33 passed, 0 failed, 2 ignored (1 new) |
| `cargo test --locked hud` | 18 passed, 0 failed (3 new) |
| `cargo test --locked` (full) | see below |
| `cargo clippy --locked --all-targets -- -D warnings` | 18 distinct pre-existing findings; **none added** (five introduced during this task were fixed before committing) |
| `cargo build --locked --bins` | clean |
| MIPS build / emulator | not run (no SDK on this host) |

New host tests in `tests/runtime/actor_services.cpp`:

1. a slot-backed `AudioComponent` never adds a second play on `begin_play`;
2. component-owned audio plays exactly once, and not when disabled, not when
   `play_on_start` is false, and not when the owner or its logical parent is inactive;
3. owned audio stops on `on_disable` and again on `end_play`;
4. a stub `music_active` pointing at owned storage blocks pool reuse, and clearing it
   releases the storage on the next `collect_quarantined()`; with no hook installed nothing
   is quarantined;
5. audio is accepted by 3D/2D/UI actors and stays `Cardinality=Multiple`;
6. `frame_update` runs while paused and `update` does not; an inactive actor gets neither;
7. `dispatch_trigger` fans out to every component, and delivers nothing to an inactive,
   destroyed or unknown actor.
