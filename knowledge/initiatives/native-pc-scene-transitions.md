# Native PC scene transitions: implementation review

Reviewed and implemented 2026-10-24. The original findings below led to the
resident scene-bank implementation described here.

## Implementation

- Native preparation now uses `scene_bank::load_selected` and
  `header_with_templates`, preserving profile selection, initial scenes, unsaved
  overrides, global resource IDs and the reusable actor pool. All included banks
  contribute to the artifact key. Native registry capacity matches across all
  translation units.
- `runtime/scene_lifecycle.hpp` shares requests, validation, counters, retirement
  and bank startup with the PSX scene driver. PSX keeps CD/GPU/quarantine waits;
  Native resets transforms, collisions, camera, time/input, 2D services, audio
  ownership and Blueprint continuations. Bank initialization resets navigation
  and visibility state. Callback requests remain queued.
- EPN3 identifies each frame's bank/generation and carries bounded loading text,
  RGB555 image data, phase and gain. The host switches geometry and active state
  while retaining global HUD texture/font indices. Playback tokens prevent stale
  completions from touching reused actor storage.
- The GPU renderer invalidates its static-geometry cache when the bank or
  generation changes, when entering Native Play and when the frame counter resets.
  A GPU readback test checks A/B/A and Reset using distinct solid-color meshes.
- Shared `TransitionState` drives fade out/loading/fade in and independent audio
  gain. Loading is presented before retirement, including zero-duration fades.
  Pause freezes the transition; Reset returns to bank zero; Stop closes the
  process. Session state and native card files survive scene changes.
- Camera recordings remain scoped to bank zero, skip transition frames, and
  include scene name and generation to keep visibility-bake samples separated.

The compiled native fixture tests 24 A/B transitions, name/index requests,
invalid/excluded targets, different actor counts/cameras/HUD textures, lifecycle
order, stale handles/audio, collision/camera reset, persistent memory-card data,
no loading-time catch-up, zero/custom fades, pause in each transition phase,
queued callback requests, Stop/Reset and inactive unsaved-bank cache changes.
Unit tests cover protocol bounds/truncation and loading pixels. Existing
Play-profile tests cover initial maps, Whole Game discovery and missing files;
the PSX runtime transition harness continues to exercise the shared lifecycle.

```text
cargo test --locked --bin epok-editor native_play::tests -- --include-ignored --skip project_runner
python tests/runtime/verify_spatial.py spatial transform_cache scene_transitions
cargo test --locked --bin epok-editor native_scene_changes_replace_static_geometry_and_reset_caches -- --ignored
```

The native integration tests require a host C++ compiler and a built
`epok-header-tool`. Set `EPOK_NATIVE_PROJECT` for the separate real-project
120-frame smoke test. Native banks remain resident: external scene files,
CD/PCDrv latency and existing unsupported hardware services are outside this
implementation.

## Original execution path (before implementation)

- `src/play.rs::input` selects the initial scene according to Current Scene,
  Whole Game or Selected Scenes and records an unsaved included-scene override.
- `src/native_play.rs::prepare` passes one scene to
  `src/hud_simulation.rs::build_runner`. The latter emits one `scene.hh`, one
  resource set and one actor class table. It does not collect the other banks.
- `native/native_play_runner.cpp` stores `request_scene` as a string. It does not
  resolve names or indices against exported banks, ignores `TransitionOptions`,
  always reports scene zero and stops updating after a request.
- `src/native_play.rs::execute` turns that request into an unsupported-transition
  error. Each frame is applied to the original `input.scene`; host audio is also
  initialized from that one scene.

Removing the error would leave the child frozen and the editor displaying the
old resource set. Restarting a separate executable per scene would lose process
state, including the native memory-card driver's in-memory files.

## Original implementation requirements

| Area | Work | Existing pieces to reuse |
| --- | --- | --- |
| Build inputs | Collect all included scenes, honor the initial scene and unsaved overrides, resolve their assets/scripts, and include every bank in cache invalidation. | `src/scene_bank.rs::load_selected`, the scene dependency system and Play profile. |
| Generated banks | Emit a shared actor pool, bank load functions, class tables and consistent texture/audio indices for native compilation. Keep the first implementation resident in host memory. | `src/scene_bank.rs::header_with_templates_for` and resource aggregation. |
| Request semantics | Resolve names/indices, reject excluded or invalid targets, expose the active scene and transition counters, and retain requests made during lifecycle callbacks for the next safe boundary. | `runtime/scene_service.hpp`. |
| Lifecycle | End outgoing scripts/actors, invalidate old handles, stop audio, clear scene-owned services, load the new bank and run initialization/begin-play. Preserve application/session state. | `unload_actor_bank`, `load_actor_bank`, generations and console reset ordering. |
| Host services | Replace the native empty `reset_runtime_services`/`remove_runtime_owner` hooks as needed; reset camera, transforms, collisions/triggers, navigation, 2D state and relevant script continuations. | Console service reset logic, with native adapters instead of GPU/CD calls. |
| Frame protocol | Carry an active bank identifier and a scene generation. Switch the editor's scene/resources before applying actor/HUD snapshots; prevent old audio completions from affecting a reused source. | EPN2 transport, `apply`, `HostAudio` and native frame consumption. |
| Presentation and clock | Apply transition defaults/overrides, render fade/loading/fade-in, multiply audio gain without rewriting authored volumes, and resynchronize time so loading does not produce catch-up simulation ticks. | `runtime/transition.hpp` and console transition tests. |
| Session controls | Define pause, reset, stop and scene-request behavior during transitions; reset should return to the chosen initial bank. Keep camera recordings associated with their scene. | Native control loop and Play profile. |

The console scene service currently calls GPU/CD-specific hooks. Share its
portable lifecycle and request ordering through explicit host adapters instead
of copying a second scene manager with subtly different semantics. Native PC
does not need the PSX scene-file streaming path to support resident transitions.

## Acceptance checks

1. A -> B -> A by both name and index, with different actor counts, cameras,
   textures, HUD and audio; verify rendered resources as well as bank counters.
2. Current Scene rejects excluded destinations. Selected Scenes honors its
   initial scene; Whole Game and included unsaved edits use the same inputs as
   PlayStation. Editing a noninitial scene invalidates the native cache.
3. End-play precedes teardown and begin-play follows loading. Old handles cannot
   resolve into the new bank. Requests from end-play/begin-play remain ordered.
4. No old audio, triggers, camera overrides, navigation or script continuations
   survive accidentally. Intended application state and memory-card data survive.
5. Zero/default/custom fade durations, loading text/image, audio gain, pause,
   stop and reset during each phase; no load-time simulation catch-up.
6. Repeat transitions many times and check stable resource usage, malformed
   protocol rejection, no stale snapshots, and continued PSX test compatibility.
