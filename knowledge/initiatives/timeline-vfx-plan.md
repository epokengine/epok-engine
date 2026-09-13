# Timelines, VFX, and particle effects

Date: 2026-09-08. Implementation update: 2026-09-10. The final audit reopened
and corrected Phase 2's reusable component-adapter gap. All engine delivery
phases 0–5 now have acceptance evidence, including current MIPS, standalone
export, real Play restart and emulator checks.

The engine plan and the additional authorized Ironwood integration are accepted.
Fireball, Blizzard, Lightning and healing VFX now accompany the existing skills.
The final game build, real combat input test and independent export rebuild pass.
No delivery phase remains open. Physical PSX timing and further Ironwood memory
and frame-time optimization remain follow-up work, not unverified acceptance claims.

## Implementation record

### Additional Ironwood delivery — passed (2026-09-10)

- Four ordinary ParticleEffects embed the shared TimelineAsset format. Their
  persistent scene anchors bind through reflected EntityHandle authoring fields.
  The game presents committed combat events without changing its portable rules.
  No particle budget or alternative playback system was introduced.
- Current MIPS disc build and PCSX-Redux 2 MB interpreter checks passed. Real
  controller input cast all four spells across 18 actions: Fireball played twice,
  Blizzard and Lightning four times each, and healing four times. SP costs were
  8/16/16/8. All effects and particles cleaned up with zero missing bindings,
  rejected effects, dropped particles or dropped sprites. Four native captures
  were reviewed; sprite depth bias keeps impacts visible over enemy billboards.
- Independent export `psyqo-1788972689008` rebuilt successfully with `build.ps1`
  and produced the byte-identical tested executable. SHA-256:
  `3c39063383e73511e7308ebcfa574e230984da12746e9df0988e02e2a248a95f`.
- Game text/data/BSS total 1,931,580 bytes; this is close to the 2 MB target and
  does not establish peak stack/heap headroom. Whole-game peak simulation work
  was 2,424 scanlines in this run. No 60 FPS or physical-hardware claim is made.
- Game-specific instructions and evidence are in Ironwood's
  `docs/spell-effects.md`, `docs/spell-effects-verification.md` and
  `.epok/spell-verification/`. Six original game files and hashes are backed up
  under `.epok/backups/timeline-vfx-20260910/`. Existing entity content was preserved;
  loader-generated UUIDs and the explicit native class-binding migration were used.

This plan predates the engine rename. Current code uses namespace `epok`, project
cache `.epok/`, and the `EpokEngine` repository. The prerequisite design is
[Blueprint research](blueprints/research.md); maintainer checks are in
[Testing](../maintainers/testing.md). Legacy identity salts are preserved for
serialized compatibility; they are not current product names.

### Phase 0 — passed (2026-09-09)

- Revalidated the existing UUID/reflection registry, `EntityRef` cook resolution,
  typed graph IR/AOT compilation, and bounded eight-frame continuations. The
  existing cancellation, activation, generation and scene-reset contracts are
  reused. No second reflection, entity-reference or continuation system was added.
- Added an explicit `TimelineAnimatable` native-property profile to the existing
  reflection wire schema v2. Old metadata loads with no timeline permission and
  the extractor/schema key forces regeneration. The prototype profile is writable
  Fixed, linear, absolute, leave-final. ReadOnly opt-in fails extraction. Existing
  scalar Blueprint timeline nodes remain the historical compatibility feature;
  they are not the new reusable TimelineAsset system.
- `runtime/timeline.hpp` and `src/timeline_curve.rs` evaluate cooked integer Q12
  keys using the Blueprint timeline's wide intermediate and division toward zero.
  A sorted marker table and per-play cursor prevent repeated crossing delivery.
  This is an evaluation kernel, not a runtime director or continuation scheduler.
- `verify_timeline_prototype.py --emulator` passed real reflection rejection/
  recovery, all 64 host/compiled/handwritten sample comparisons, one marker over
  130 updates, real MIPS builds and byte-identical independent export rebuilds.
  Both variants use `-Os`, 68 raw-Q12 tick advances, and PCSX-Redux's interpreter
  with 2 MB RAM. Fixture code resolves the property by persistent ID before
  generating a typed C++ assignment. Runtime execution uses no field names.
- Measurements: compiled work function 452 bytes / median 339 guest cycles;
  handwritten 180 bytes / median 59 guest cycles (active calls 11–60). Both
  behavior instances are 24 bytes; both EXEs are 178,176 bytes. Compiled linked
  text/data/BSS: 101,436 / 76,740 / 439,016 bytes; handwritten:
  101,148 / 77,028 / 439,016. Alignment explains the unchanged total image size.
  These are linked sections and interpreter cycles, not peak stack, heap, FPS,
  hardware timing, or a claim of performance parity. The table path costs more
  CPU for this tiny workload. Particle budgets have not changed.
- Runtime contract suite passed with actual PsyQo Q12 headers, including new
  endpoint/extreme/negative-slope/ordered-marker checks. Rust: 167 passed, eight
  ignored. Host editor/extractor build passed. Machine evidence is retained in
  ignored `artifacts/timelines/phase0.json`; the integration script recreates it.

### Phase 1 — asset-core acceptance passed (2026-09-09)

Initial slice (historical implementation record):

- Versioned `.timeline.json` source assets discovered by the existing asset
  scanner, with UUIDs for assets/slots/tracks/keys/markers. Typed slots use the
  existing `EntityRef` schema and class ancestry; native property references use
  existing reflection IDs. No particle-only timeline was introduced.
- Deterministic raw Q12 cooking, sorted tables/source maps, semantic signatures
  and generated `.hh` output. Labels/layout/list reorder preserve IDs and table
  timestamps. Curves hold endpoints throughout the sequence; same-priority
  absolute writes conflict even through two slots aliased to the same entity.
- Required/optional/inactive/missing/incompatible binding diagnostics and orphan
  preservation. Binding tests follow scene UUIDs through reorder/deletion; they
  never serialize runtime indices or generations. Binding tests in the asset
  window are explicitly temporary; scene component persistence is not connected.
- Native schema/class-ancestry changes invalidate the asset signature; unrelated
  classes do not. Native parse failures, including failure while opening the
  project, mark existing timeline caches stale. Unknown fields/versions and
  removed property IDs are preserved and cannot silently cook. Current-source
  compilation never uses a stale successful artifact as fallback.
- Project-browser creation/opening; an ImGui editor with curve/marker plot,
  key/slot/property reassignment, diagnostics, scrubbing, 64-entry undo/redo,
  Ctrl+Z/Y, save/discard handling and guarded atomic saves. External writes block
  overwrite and are identified as stale in the window. The initial profile is
  explicitly writable Fixed / Linear / Absolute. Serialized unsupported modes
  fail validation; they are not silently approximated.
- CLI `--new-timeline`, `--compile-timelines`, `--open-timeline`; current user
  guide at `docs/timelines.md` and a source RPG charge/Impact/Aftermath example at
  `examples/timeline-spell/`. It is an authoring example, not a playable fireball.
- Conservative authoring ceilings: 32 assets, eight slots, 16 tracks, four keys
  per track and 64 markers. These do not allocate or increase PSX particle or
  active sequence/effect budgets. Runtime pool capacities still require Phase 2
  measurements. The existing runtime budgets are unchanged.

Validation:

- Final default Rust suite: **176 passed, nine ignored**. `cargo fmt --all --
  --check` and `cargo build --locked --bins` passed.
- Explicit `timeline_controls_add_undo_redo_validate_and_save` passed real ImGui
  Add Marker, Undo, Redo, Validate and Save, including persistent marker identity.
  Native editor capture was inspected at `artifacts/timelines/editor.png`; the
  curve plot, fields, marker controls and scrolling render correctly. No locked
  desktop limitation prevented this capture.
- `verify_timeline_assets.py` passed real extraction, permission/rename/native
  syntax-failure invalidation, stale retention/recovery, unchanged-output
  timestamps, unrelated-class isolation, duplicate UUID rejection and generated
  table MIPS compilation. Evidence: `artifacts/timelines/phase1.json`.
- `verify_blueprints.py --keep --emulator` revalidated native/visual inheritance,
  executable graphs, Call Parent, Delay, independent state, bank reset,
  standalone export rebuild, relocation and real PCSX-Redux execution.
  `verify_blueprint_runtime.py` and `verify_sprites_particles.py` passed.
- Strict Clippy still fails on three incoming, unrelated changes:
  `pipeline.rs::execute` has eight arguments; the existing GUI interaction loop
  triggers `needless_range_loop`; the incoming viewport test triggers
  `field_reassign_with_default`. No timeline lint remains. These user changes
  were preserved instead of being rewritten to obtain a green unrelated check.
- No physical-console test, full effect parity, runtime director cancellation,
  or runtime sequence/effect pool measurement is claimed.

Typed asset-core extension and acceptance:

- Timeline and Blueprint sources now use version 2. Version 1 migration is in
  memory only, retains IDs, and never grants new permissions. Reflection schema
  v3 adds explicit native-field interpolation/blend capabilities and independent
  TimelineCallable/TimelineAction function exposure. The earlier v2 reflection
  Fixed profile remains readable with its original permissions.
- Fixed, Int32, UInt32 and the existing Fixed[2]/Fixed[3] vector adapters support
  Linear, Step, Smoothstep, EaseIn and EaseOut; Bool/32-bit enums support Step
  only. Numeric tracks permit Absolute/Additive policies. Runtime application
  and restoration are Phase 2 work; the core cooks every channel deterministically.
- Event tracks reuse the same asset, slots and persistent function IDs. Events
  cook up to four arguments into integer lanes, existing compact asset IDs or
  binding-slot indices. EntityHandle arguments use Blueprint IR's existing
  assignability rule. Mutable reference arguments and unsupported types fail.
  Marker/event signal tables sort by tick and persistent ID. Source slot reorder
  also preserves cooked indices and output bytes.
- Native and Blueprint property/event exposure is editable through their
  existing metadata/graph models. The timeline window edits typed keys, modes,
  events and arguments, with the same guarded save and bounded undo history.
  Asset arguments use the existing imported-asset picker/index. Property preview
  never invokes native events. `editor-typed.png` was captured and inspected.
- Consumer contract: `Compiled.dependencies` records schema/compiler versions,
  semantic TimelineAsset identity, class ancestry, selected property/function
  signatures and imported resource hashes. Consumers must record the fresh
  compiled signature, validate their UUID binding map, and refuse stale/error
  results. Missing/wrong-kind resources fail cooking. Unused native defaults do
  not invalidate curve data. Scene/effect/generated-output dependency edges are
  connected as those consumers are introduced in phases 2–5; their absence here
  is not a claim that transitive project invalidation is finished.
- Current validation: **182 Rust tests passed, nine ignored**; the explicit
  ImGui transaction test passed. C++ runtime contracts and real pinned MIPS
  headers passed. `verify_timeline_assets.py` passed real native/Blueprint typed
  properties/events, permission removal/recovery, ID stability, typed arguments,
  stale caches, semantic invalidation, source reorder and generated MIPS tables.
- Extended `verify_timeline_prototype.py --emulator` passed all original samples
  plus 64 mixed signed/unsigned/easing Rust-versus-MIPS samples in each variant.
  Both standalone rebuilds remain byte-identical. Current compiled text/data/BSS
  is 102,252 / 75,924 / 439,016; handwritten is 102,140 / 76,036 / 439,016.
  Both EXEs remain 178,176 bytes and instances 24 bytes. Measured work medians
  are 471 versus 57 interpreter cycles. Work entry sizes are 140 versus 180
  bytes, but the compiled entry now calls an outlined shared kernel: entry size
  alone is not total implementation cost. Extra parity work runs outside the
  measured interval. Particle budgets remain unchanged.
- Strict Clippy still reports only the same three incoming unrelated errors
  above. No physical-console validation is claimed.

Phase 2 must now connect scene persistence, typed runtime accessors, lifecycle,
bounded pools, statistics, staging/export and the gate/camera emulator scenario.

### Phase 2 — accepted

- Added `timeline_runtime.hpp` with a bounded director template (at most eight
  instances), immutable cooked target/property/event descriptors, existing
  generation-checked EntityHandles, captured values, pause/seek/stop/completion,
  owner/scene cancellation, marker revisions and a 32-entry diagnostic ring with
  overflow counters. Scene components now drive this service in the production
  fixed-step loop; production integration acceptance passes below.
- Captures occur before the first property write, including after an optional
  inactive binding becomes usable. Tracks sharing a property combine from one
  captured baseline. Absolute tracks sort before additive tracks at equal
  priority; tracks sharing a target must agree on restoration. Cross-instance
  claims on the same entity/property are rejected with an observable conflict,
  preventing one director from restoring over another director's live output.
- Events execute in crossing order with property values sampled at the event
  timestamp. Marker revisions remain observable independently of diagnostic
  overflow. Forward seek skips crossing events/markers and only reconstructs explicitly
  idempotent actions; backward runtime seek is rejected. Callback reentry cannot recursively advance the director
  or overwrite an instance whose dispatch stack is still active.
- Each advance accepts at most one asset-duration of time, split into at most
  two segments across a wrap. Ordinary fixed-step remainder survives looping;
  excess large-dt catch-up is discarded and counted. Work never expands into
  an unbounded number of loop iterations.
- Initial C++ host contract tests and real pinned MIPS template compilation
  passed for restoration, event-time samples, target destruction/reuse/type
  failure, pause, scene cancellation, owner destruction during events, restart
  generations, seeking, looping and deterministic pool/conflict rejection.
  The production gate/camera emulator integration and capacity measurements now
  pass; no additional continuation pool was introduced.
- Scene v4 adds optional TimelineComponent authoring data, typed UUID bindings,
  Inspector editing through existing entity undo/redo, and automatic playback.
  Plain scenes retain v3 and the original legacy entity-identity salt. Capture,
  placement, duplication and template refresh remap binding UUIDs; external
  template targets are rejected. Dynamic templates register components after
  typed initializers and before start callbacks through an optional extension
  to the existing reservation/rollback path. Focused Rust identity tests and
  C++ configuration-failure rollback tests pass.
- Generated adapters use the existing class ancestry/behaviour lookup and
  dispatch quarantine. Native-only timeline games enable that same metadata
  without allocating Blueprint spawn pools. Staging cooks current sources,
  required bindings and referenced resources; generated scene headers install
  the shared asset tables and components. Owner removal and scene reset cancel
  sequences through existing lifecycle hooks. The spell example builds for MIPS
  with its first scene-owned component; its visual RPG spell remains unfinished.
- Added a production gate/camera integration harness covering typed native and
  Blueprint targets, event-time samples, restoration, pause/deactivation,
  destroyed/reused targets, owner cancellation, scene replacement, linked RAM,
  completed-frame simulation timing and standalone export rebuilding. This
  harness passes MIPS, identical standalone export rebuilding and PCSX-Redux.
- Emulator evidence (`artifacts/timelines/phase2.json`): two sequences completed,
  two cancelled, exactly one event at gate Y=8192 Q12, two Impact markers, eight
  particles spawned, and 56 invalid-target skips without touching the reused
  target. Diagnostic overflow (25) remained bounded and observable. Template
  component registration also passed on a dynamically spawned Blueprint.
- Under common `-Os`, linked text/data/bss are 129996/93232/445416 bytes. The
  eight-instance director occupies 5568 bytes and eight components occupy 704
  bytes. Maximum observed completed-frame simulation was 54 scanlines (~3.46 ms)
  in this small fixture; that includes other simulation systems and is not an
  isolated director CPU budget, a maximum-load benchmark or physical-PSX timing.
  Phase 0's isolated kernel CPU comparison remains the curve-cost evidence.
- The integration caught and fixed generated class-count storage being optimized
  away when a separate native translation unit called the existing Blueprint
  spawn API. The generated inline constant now explicitly retains external
  storage. Runtime adapters also collision-check compact IDs with the existing
  Blueprint identity function before emitting tables.
- Verification: 183 Rust tests passed, nine explicitly ignored; runtime host and
  real pinned MIPS tests passed, including backward-seek rejection and template
  rollback. Native Inspector screenshot was inspected. Strict Clippy remains
  blocked only by the three incoming pipeline/gui/viewport findings recorded
  earlier; no unrelated work was reverted. Hardware validation is unavailable.

### Phase 3 — acceptance passed (2026-09-09)

The next implementation extends the same reflected target and director binding
contracts to generation-checked internal effect layers. ParticleEffect embeds
TimelineAsset directly, uses existing emitter/sprite data and the single 256
particle pool, and allocates transient effect state independently of scene
entities. No particle-only timeline or parallel reflection registry is planned.

- Implemented ParticleEffect source v1 with at most eight persistent layers,
  existing Sprite/Emitter authoring payloads, seed, and an embedded TimelineAsset
  v2. It shares TimelineAsset migration, validation, semantic compilation and
  preview caches. Layer/slot/name/reorder tests preserve UUIDs; existing emitter
  validation rejects a 129-particle layer. CLI source creation and validation are
  available; basic effect editing and component selection are now implemented
  below, while visual preview and artistic controls remain unfinished.
- Reflection schema v4 adds EffectLayerRef as an internal typed UUID reference.
  The SDK's explicit `EffectLayer` declaration is extracted by the same Clang
  manifest and is neither a Behaviour nor Blueprint-spawnable. Native registries
  merge that manifest even when behaviour chains are already present. There is
  no second reflection registry or class identity algorithm. Exposed layer
  controls include enabled/playing, opacity, size, offset, color, rate, velocity,
  play/stop actions and bounded burst requests with a dropped-request counter.
- The same director now accepts a tagged BoundTarget containing an existing
  EntityHandle or an EffectLayerHandle. Cooked typed adapters validate the tag
  and generation before accessing either reflected receiver. A layer can own
  playback without reserving a scene entity. C++ host/MIPS tests cover mixed
  target kinds, inactive layers, event dispatch and generation reuse; the typed
  resolver is now owned by the dedicated bounded effect pool.
- Revalidated the production gate/camera emulator and identical export after
  extending bindings. Current report: text/data/bss 132172/86960/452256 bytes,
  director 5856 bytes, component pool 960 bytes, maximum observed completed-frame
  simulation 55 scanlines. Tagged handles add 288 director bytes and 256 component
  bytes over Phase 2. These are measured costs; particle/emitter budgets remain
  unchanged. This small scene is not a maximum-load effect benchmark.
- Real Clang/source integration passes native behaviour coexistence, layer
  property/event identity, embedded deterministic curves/events, and generated
  typed layer-adapter syntax against real MIPS headers. Broken effect sources
  preserve their bytes and invalidate their embedded timeline preview. Full
  effect consumer/staging/export invalidation remains required before shipping.
- Latest validation: 184 Rust tests passed, nine ignored; formatting and diff
  checks pass. The effect-source integration also rebuilds MIPS and verifies
  in-memory TimelineAsset v1 migration, explicit stale previews, and rejection
  of future nested Sprite/Emitter fields without rewriting original bytes.
  Strict Clippy still reports only the three incoming findings noted above.
- Implemented the eight-instance/eight-layer effect pool against the same
  director. Scene-owned and detached effects use internal generation-checked
  layer handles without allocating entities. Local pause freezes simulation but
  preserves visibility; owner inactivity hides/freezes, owner destruction and
  scene replacement cancel, completion drains a bounded particle tail, and stop
  releases particles immediately. New instances cannot emit/render before their
  first timeline evaluation. Capacity, stale generations, owner lifecycle,
  shared-director exhaustion and burst saturation pass the host runtime suite
  and real MIPS header compilation. Host pool size is 20,280 bytes (not PSX RAM).
- Scene and effect emitters now feed one fixed 64-source/256-particle pool;
  rejected sources and particles have saturating dropped-work counters. Existing
  scene particle aging/visibility behavior is retained. Layer sprites and
  particles draw through the existing 2,048-triangle sprite renderer. No particle
  budget increase was made. Effect stats expose active/peak/spawn/drop/lifecycle
  and layer totals. Native-only games still omit the dedicated effect pool.
- Added source-v1 ParticleEffectComponent in scene v4, required external binding
  validation, template UUID remapping and runtime registration through the
  existing template callback/rollback. Reachable effects stage immutable layer
  definitions and the existing embedded timeline compiler output. Texture and
  event resources enter the existing shared resource layout; sprite/emitter
  initialization uses existing typed generators. Layers sort by persistent UUID
  before cooking, preserving runtime indices and per-layer seeds across reorder.
  Project fingerprints include effect semantic data; exact consumer isolation is
  still Phase 5 work. Per-layer component overrides are pending.
- `verify_particle_effect_runtime.py --emulator` passes a real cooked scene
  component plus seven detached plays, deterministic rejection of the ninth,
  unchanged entity count, pause, one Impact marker/event and eight particles,
  eight cancellations and one bounded normal completion across nine accepted
  plays. A play immediately after timeline completion reuses the director slot
  without erasing the previous effect's particle tail: completion is observed
  before gameplay dispatch and draining no longer depends on a sequence handle.
  Reordered layers/slots
  produce identical headers and the standalone export rebuild is byte-identical.
  PSX effect pool/component tables: 20,240 / 992 bytes; linked text/data/BSS:
  163,004 / 45,888 / 502,872 bytes. Maximum observed completed-frame simulation
  cost is 190 scanlines, including all other systems and the eight-effect spawn
  burst. This is a small untextured fixture, not isolated effect timing, physical
  hardware or worst-case gameplay acceptance. Evidence is retained in ignored
  `artifacts/timelines/phase3-runtime.json`.
- Regression verification after the shared particle refactor: 185 Rust tests
  passed, nine ignored; host sprites/particles and Blueprint/timeline/effect
  contracts pass against actual PsyQo headers. The gate/camera MIPS and
  standalone/export/emulator checks still pass: text/data/BSS
  133,212 / 55,200 / 488,360 bytes, maximum observed completed-frame simulation
  56 scanlines. This fixture does not link the dedicated effect pool. Formatting
  and diff checks pass; strict Clippy retains only the three incoming findings.
- Added a single editor document owner for standalone TimelineAsset or
  ParticleEffect containing that same TimelineAsset. The existing timeline
  editor now saves and snapshots the complete effect, including layers, in its
  64-entry undo/redo history; there is no second timeline authoring model.
  Project creation/opening, `--open-effect`, the entity effect Inspector and
  typed external bindings are connected. Layer add/remove/reorder preserves
  persistent identity, and layer property/event controls use the same reflected
  timeline selectors. Sprite and emitter controls share their existing Inspector
  implementation. Atomic-save conflict protection covers the whole document.
- Fire, Smoke, Sparks, Impact, Projectile, Aura and Rune presets produce ordinary
  source layers and shared opacity/event tracks. `--new-particle-effect Name
  --preset Fire` exposes the same preset builder for repeatable authoring.
  The source integration validates and MIPS-cooks all seven presets in one
  eight-effect scene. Real ImGui interaction passes Add Fire layer, Undo, Redo,
  Save, ID retention and external-write rejection; the standalone Timeline
  interaction test still passes. The editor screenshot was inspected at
  `artifacts/timelines/effect-editor.png`. It revealed a missing SDK-class merge
  in editor metadata; the editor now uses the authoritative full native registry
  just like effect cooking. Reflection refresh must finish before starting the
  source watcher debounce interval, preventing a slow extraction from launching
  a build in the same detection tick.
- Earlier authoring validation: 186 Rust tests passed, ten ignored; explicit
  effect and standalone Timeline ImGui tests pass. The seven-preset MIPS scene
  also rebuilds byte-identically from its standalone export. Formatting/diff
  checks pass and strict Clippy retains only the three incoming findings.
  An initial full-suite run hit a transient Windows socket-buffer error; serial
  verification passed. A subsequent watcher test exposed the real slow-refresh
  debounce issue fixed above; both its focused regression and the complete
  serial suite passed. Visual preview/PSX parity was still pending at that point;
  the subsequent preview validation is recorded below.
- Added the dedicated visual effect preview using the actual C++ director,
  effect pool, particle pool and PsyQo Q12 headers, statically linked into the
  editor through bounded typed C ABI tables. The adapter maps only the SDK's
  explicit reflected layer properties/functions; it does not interpret offsets
  or reload native libraries. Asset reconstruction validates with the same
  timeline compiler and production texture/atlas checks. The host bridge is
  never staged into PSX builds or standalone exports.
- The effect window now has an eight-second fixed-step loop, pause/restart,
  bounded seek-by-replay, camera/background controls and visible particle,
  dropped-work, event and marker counters. Rendering reuses SceneGpu's sprite,
  atlas and blend path with neutral asset lighting. A single preview owns the
  SDK layer resolver; acquiring it never blocks an editor frame. Stale source
  or reflection errors stop preview playback. Optional absent external work
  is skipped; required external bindings require the PSX Game view.
- `verify_particle_effect_preview.py` passes 96 identical 68-Q12 steps against
  cooked MIPS tables in PCSX-Redux: hashes of every live particle's index,
  generation-resolved layer, age, lifetime, position and velocity; all counters;
  sprite counts, size, pivot, color, atlas regions and world matrices. The
  fixture includes two bursts, a marker, a Smoothstep opacity curve, a partial
  atlas row, draining/completion and repeatable seeds. Evidence is in
  `artifacts/timelines/phase3-preview-parity.json`. Its standalone export
  rebuilds byte-identically. This is simulation/table-adapter parity, not
  GPU/PSX pixel equivalence or physical-console performance measurement.
- Current preview checks: 187 Rust tests passed, ten ignored. The focused
  native bridge test covers ABI layout, exclusive ownership, pause visibility,
  repeatable seed replay, required external rejection and optional work skips.
  The real ImGui whole-effect undo/save test passes. The Fire preview screenshot
  and enlarged particle region were inspected. Strict Clippy retains only the
  three incoming findings. Windows host C++20 was tested; macOS and physical
  hardware were unavailable. The older Rust Scene particle preview remains
  unchanged and is not the effect parity reference.
- Artistic controls now map density, violence/speed, direction, chaos/spread,
  scale and brightness onto the existing validated emitter/sprite payloads.
  Burst density edits shared timeline event keys. Duration/retime preserves all
  key/event/marker IDs and rejects collapsing distinct curve keys. Advanced
  controls remain available; preview transport/counters stay visible while the
  authoring child panel scrolls. No second parameter or particle timeline model
  was introduced. Real ImGui effect and standalone timeline editing tests pass.
- ParticleEffectComponent v2 stores explicit per-layer/property UUID overrides
  using the shared typed literal format. V1 components migrate without identity
  changes. Full reflection validates animatable exposure, exact type and values;
  the existing component cooker consumes bounded typed initializer bodies
  prepared from that manifest. This also fixes reduced legacy scene/template
  metadata lacking the SDK layer class, without introducing another registry.
  Initialization precedes director binding/capture, preserves asset defaults,
  and allocates no runtime storage beyond one function pointer per component.
  Source tests reject orphaned/type-changed overrides while preserving bytes;
  host and PCSX tests verify independent overridden/default instances.
- The seven-layer `examples/timeline-spell` Fireball has cast sigil, projectile,
  travelling embers, flash, sparks, smoke and residual glow. Eleven shared
  tracks/events and four persistent markers author all timing. The presentation
  harness repeats after cleanup; gameplay damage is deliberately the following
  Blueprint phase. Existing original sample pixel art supplies the atlas.
- `verify_timeline_fireball.py` passes real MIPS, five PCSX stage captures,
  one revision for each marker, complete cleanup, zero dropped particle/sprite
  work and a byte-identical independent export. Initial 48-spark authoring
  measured 67 peak particles and 270 maximum completed-frame simulation
  scanlines. Reducing the authored burst to 32 preserves the stage composition
  and measures 51 peak particles / 76 total emitted / 182 maximum simulation
  scanlines. No global budget changed. These are fixture/frame samples, not
  isolated kernel costs, worst-case guarantees, hardware timings or 60 FPS claims.
- `verify_particle_effect_preview.py --fireball` matches all 350 steps against
  cooked MIPS/PCSX, including every live particle state hash, all sprite values,
  atlas regions, matrices, counters and deterministic replay. The adapter also
  preserves SDK default rate/velocity for sprite layers. Evidence is retained
  in `phase3-fireball.json`, `phase3-fireball-parity.json` and stage screenshots
  under `artifacts/timelines`. The effect runtime saturation/override fixture
  passes again: text/data/BSS 164588/46352/502904 bytes; pool 20240 bytes;
  eight components 1024 bytes (32 additional bytes for initializers); maximum
  completed-frame simulation 190 scanlines. Its standalone export is identical.
- Current checks: 189 Rust tests passed, ten ignored; explicit ImGui checks,
  MSVC runtime contracts, real MIPS syntax/cooks, emulator scenarios, formatting
  and diff checks pass. Strict Clippy retains three incoming findings. macOS
  host compilation and physical PSX checks remain unavailable. The phase's
  authored fireball, deterministic capacity/drop behavior, artist controls and
  instance overrides satisfy its acceptance gate. Phase 4 is next: typed
  playback/effect handles and marker/completion/cancellation waits through the
  existing eight-continuation Blueprint contract.

### Phase 4 progress — typed component playback and latent waits

- Blueprint source v3 adds typed sequence/effect handles and playback nodes;
  v1/v2 migration stays in memory until Save. Reflection schema v5 recognizes the
  existing runtime handle declarations. Handles can be transient graph values or
  non-editable variables with null defaults; serialized slots/generations and
  animatable handle properties are rejected. No parallel identity system exists.
- Component Play, Stop, Pause, Resume, effect-to-sequence conversion and Burst
  nodes use the existing director/effect services. Burst queues work on enabled,
  playing emitter layers, retains the per-layer 256-request bound and reports
  dropped requests through existing effect counters; sprite layers are skipped.
- Wait for Marker, Sequence Complete and Effect Complete use the existing
  `Continuations<8>` slots. Effects complete after their bounded particle drain.
  Marker nodes select a persistent timeline/marker pair from source documents,
  including embedded effect timelines; deleted markers fail compilation without
  rewriting the node. Runtime asset identity and handle generation are checked.
- A compatible external-signal extension marks an existing continuation ready;
  it never invokes gameplay. Generated read-only observation runs before gameplay
  dispatch and before a terminal director/effect slot is reused, preserving a
  reached/completed result in the graph frame. Inactive/paused consumers retain
  the result. Dispatch revalidates the owner, and newly scheduled waits require
  a later continuation advance. Retriggering the graph replaces its frame;
  owner destruction and scene replacement cancel it without running gameplay.
- Host runtime tests cover exactly-once markers, invalid asset/marker/handle
  identity, terminal slot reuse, effect drain, pause, capacity and cancellation.
  `verify_blueprint_playback.py` passes generated C++, real MIPS, PCSX-Redux and
  a byte-identical independent export. Its combat graphs apply captured damage
  once at Impact, preserve inactivity, stop through Cancelled, and do not apply
  damage after destruction or scene replacement. MIPS text/data/BSS are
  182692/56920/515352 bytes; the eight-continuation container remains 204 bytes
  and each playback-wait frame adds 32 bytes. The fixture reports zero dropped
  particles and 304 maximum completed-frame simulation scanlines, including
  other systems/scene work; this is not an isolated observer cost or 60 FPS claim.
  Evidence: `artifacts/timelines/phase4-bridge.json`.
- Validation at this checkpoint: 191 Rust tests passed, ten ignored; the explicit
  ImGui canvas test and nine editor model tests pass. Host C++/MIPS runtime checks,
  editor/reflection builds, formatting and diff checks pass. Strict Clippy reports
  the same three incoming findings in pipeline/gui/viewport; no new lint finding
  was introduced. A concurrent Windows test relink hit an executable lock, and an
  attachment fixture failed during that concurrent run; serialized reruns of the
  focused test and full suite pass. Physical PSX and macOS host checks remain
  unavailable. The Blueprint editor now refreshes marker choices on opening and
  source/reflection refresh; source-catalog errors remain visible.
- At that checkpoint direct asset plays, subscriptions and the authored combat
  example remained. Their implementation and validation follow below.

### Phase 4 - accepted: direct assets, subscriptions and authored combat

- Direct Play TimelineAsset and Spawn ParticleEffect nodes specialize their
  typed binding pins from the existing source asset UUID and persistent external
  slot UUIDs. Required unconnected/null/incompatible bindings fail compilation;
  optional absent work uses the director's existing bounded skip diagnostics.
  Internal effect layers are supplied by the existing pool. Get/Make Transform
  provide typed placement, and a null effect owner explicitly requests detached
  playback. This is compile-time asset selection, not dynamic asset-variable
  dispatch. Each direct play is independent; callers stop an earlier handle before
  restarting a sequence that would conflict over the same properties.
- Staging includes assets, resources and typed adapter translation units reached
  only through Blueprint nodes. These use the existing timeline/effect compilers
  and services. Removing/reordering/renaming source slots retains UUID connections;
  removed slots diagnose stale wires instead of rewriting authored data. Editor
  labels expose friendly required/optional slot names while pin identity stays
  persistent. Undo/redo includes playback nodes and asset selections.
- Direct-asset evidence: `phase4-direct-assets.json`, fixture
  `epok-blueprint-playback-n80sebtm`, passes real MIPS, PCSX-Redux and an identical
  standalone export with no TimelineComponent or ParticleEffectComponent in the
  scene. Linked text/data/BSS: 189716/58088/515504 bytes. Captured Impact damage,
  completion, stop, inactivity, destruction and scene replacement pass. Particle
  drops are zero; maximum completed-frame simulation is 304 scanlines including
  unrelated work and scene replacement, not an isolated VFX timing claim.
- Subscribe to Marker compiles a captured listener frame using the same
  `Continuations<8>` scheduler. Registration continues Next immediately; a parent
  Delay does not overwrite the listener. Reached branches can themselves suspend.
  Future crossings are retained as bounded revisions, including while inactive,
  during a latent branch and before terminal slot reuse. Completion/cancellation
  follows the captured crossings. Reaching the node again replaces its frame;
  Return from a listener branch ends it. Owner destruction/scene replacement
  cancel through existing lifecycle hooks. No listener pool was added.
- A per-frame epoch closes a compatible continuation reentry gap: if a callback
  invokes the same event/subscription, the older running state machine stops
  before it can overwrite the new frame's program counter. Captured arguments
  and temporaries belong to the listener. Subscription playback state is 40
  bytes on MIPS, ordinary wait state 32 bytes; both also use compiler-generated
  captures/program counters/epochs. The eight-slot table remains 204 bytes.
- The startup example is now `Combat.epokmap` with authored `BP_Fireball` and a
  target HP display. Blueprint spawns the seven-layer effect, captures its victim
  and power, waits for Impact, invokes a checked target call once, then waits for
  particle drain. Native code supplies input/HP/HUD actions. The original Main
  presentation scene and charge TimelineComponent remain available. A stable-ID
  authoring generator recreates the combat source; no runtime interpreter exists.
- The authored combat integration passes MIPS, PCSX and identical export:
  `phase4-combat.json`. It records one 25-point hit, one normal completion, one
  stopped-play cancellation, no damage/completion after owner destruction, and
  successful scene replacement. Changing the next cast's target during the wait
  does not redirect captured damage. Game/Blueprint screenshots are inspected.
  Foreign Blueprint receiver includes now follow the selected class ancestry;
  unrelated SDK effect-layer descriptors cannot masquerade as project headers.
  Captured parameter connections are visible on the canvas and support disconnect
  with Undo. Focused regression tests cover both integration fixes.
- The final nonblocking subscription/reentrant-callback fixture passes MIPS,
  PCSX-Redux and identical standalone export (`phase4-subscriptions.json`,
  `epok-blueprint-playback-qbqr6hbi`). Two consumers deliver three hits each;
  captured powers total 46 Q12 units, seven registrations continue Next immediately,
  and three independent parent Delay completions survive the deliberate
  reentry/destruction/scene-cancellation cases. Text/data/BSS:
  192308/57544/515872 bytes; continuation table 204 bytes; playback subscription
  state 40 bytes. Particle drops are zero. Maximum completed-frame simulation is
  304 scanlines, including other systems and scene replacement.
- Final verification: 195 Rust tests pass, ten ignored; explicit real ImGui canvas
  interaction passes. Host C++ contracts and pinned MIPS header checks pass.
  Component playback and the authored combat fixture pass again after the
  continuation epoch change. The Blueprint/game screenshots show the complete
  graph, captured parameter wires, HP=75 after one hit, and cancellation without
  another hit. Formatting and diff checks pass. Strict Clippy retains only the
  three incoming pipeline/gui/viewport findings. Physical PSX and macOS host
  checks remain unavailable. Phase 4's authored-combat/lifecycle gate is accepted.
  Phase 5 exact consumer invalidation and dependency visualization are next.
  Ironwood's authorized Fireball, Blizzard, Lightning and healing effects follow
  engine acceptance.

Phase 5 implementation in progress:

- Added `verify_particle_effect_load.py` for native load measurements with twelve
  ordinary meshes, the existing textured characters and HUD. The authored effect
  uses eight emitter layers, eight vector-property tracks and eight event tracks
  within the shared sixteen-track limit. One effect, eight effects/64 emitters/
  256 particles, rejected excess effects, saturated burst requests, a competing
  scene emitter and final cleanup run as separate phases in one executable.
  The verifier builds MIPS, rebuilds a standalone export, and replays PCSX twice
  to compare capacity outcomes. Initial correctness gates pass with identical
  standalone bytes and repeatable overflow/cleanup (`phase5-vfx-load-baseline.json`,
  fixture `epok-vfx-load-07x9z44g/Game`). The baseline is 130 scanlines per frame;
  one effect has a 554-scanline median and eight effects 5,171 scanlines, with
  1,424 cumulative dropped simulation steps. These results are too costly for a
  routine gameplay workload despite correct memory/capacity bounds.
  A compatible optimization now composes pure translations without multiplying
  an unchanged basis by identity, and caches the sprite camera-facing bases per
  renderer/frame. General scale/shear retains the original integer rounding;
  unit-scale sprites reuse the same model-view basis and recompute translation.
  Host Q12 translation differential tests pass. Native billboard differential
  tests also pass across fixed/upright/spherical modes, singular and sheared
  transforms, negative values, repeated reads and changed views. The optimized
  load passes MIPS, byte-identical standalone rebuilding and both PCSX runs
  (`phase5-vfx-load-optimized.log`, fixture `epok-vfx-load-1lh4owhn/Game`). At the
  example's 640x480 setting, median frame cost changes from 554 to 469 scanlines
  for one effect and 5,171 to 4,251 for eight; cumulative dropped steps change
  from 1,424 to 1,001. The maximum workload remains unsuitable for routine
  gameplay. These are hard capacities, not recommended simultaneous workloads
  or a 60-FPS target. Sequence/effect/particle capacities remain unchanged.
  The camera cache adds 248 bytes of BSS; instrumented text/data/BSS totals are
  221,516 / 89,776 / 557,616 bytes. Pool sizes remain 5,856 / 20,240 / 36,104 bytes
  for sequences/effects/particles. Nine plays cancel without retained particles
  or extra entities; the excess effect, 49,152 excess pending requests, 16,384
  pool-rejected particles and competing emitter are observable and repeatable.
  Final host suites pass 231 Rust tests (15 ignored), actual PsyQo particle and
  Blueprint/timeline/effect lifecycle tests, plus all 350 Fireball host-preview
  versus cooked MIPS snapshots (`phase5-load-preview-parity.log`, fixture
  `epok-effect-preview-7x3317sm/Game`). Particle state, random seeds, markers,
  quads and their Q12 transforms match. Strict Clippy retains only the same three
  incoming findings (`1788960434_cargo_clippy.log`). Physical PSX remains untested.
- External native dependency observation now reads the compiler/extractor's
  recorded host paths on one background worker, with a one-second minimum start
  interval. Raw observation history is independent of build graph publication;
  content edits with preserved timestamps, deletion and repair propagate through
  the selected stage/native-build/executable's retained edges. A result cannot
  overwrite provenance published since its read began. The editor uses the
  existing Stop, reflection refresh and rebuild/relaunch path. The focused
  dependency test passes. The expanded real Play regression passes (371.83
  seconds, `phase5-external-watch-play.log`, retained fixture
  `epok-native-restart-37520-1788957956096913600`): an actual external C++ header
  edit with a preserved timestamp produces different MIPS bytes and a new PCSX
  process; deletion stops Play and repair restores the expected executable.
  A follow-up distinguishes an unfinished observation from successful error
  recovery, with its own pending/error/recovery regression. The final host suite
  passes 231 tests, 15 ignored (`phase5-external-watch-tests.log`).
  New external include membership is discovered at compiler capture; arbitrary
  external directory scanning and portable source packaging remain separate work.
- The general native profiler now decodes linked sequence/effect/particle stats
  with linked-layout checks. It distinguishes active gauges, persistent peaks,
  cumulative work, capture deltas, unavailable services, observed resets and
  saturation. Seven profiler tests pass. A real Fireball MIPS/PCSX capture passes
  (`artifacts/performance/phase5-fireball-playback/profile.json`, 89 samples):
  one completed effect, two cancellations, seven peak layers and 51 peak particles
  with no particle drops. The fixture replaces the scene; particle counters reset
  and the profiler correctly reports their whole-capture deltas as unavailable.
  The existing performance-layout lookup also now uses the actual four-character
  `epok` namespace, restoring appended frame fields. Observed frame costs span
  22–811 scanlines (95th percentile 607), including this lifecycle fixture's scene
  transition; this is neither a 60-FPS guarantee nor worst-case gameplay or
  physical-PSX acceptance. Capacity budgets were not increased. Host binaries,
  formatting and diff checks pass; strict Clippy retains only the three incoming
  pipeline/gui/viewport findings (`1788958553_cargo_clippy.log`).
- The source watcher now reports invalid source reads/parses, stops an obsolete
  build/Play through the existing worker control channel, clears the old Game
  frame and waits for Finished before starting a replacement. Auto compile
  preserves Play intent across validation/compilation errors and deferred builds;
  an explicit Stop cancels that intent. Open unsaved Timeline/ParticleEffect
  documents now use the same Build/Play guard as Blueprints.
- Added host artifact provenance using the compiler's existing dependency IDs
  and signatures. Timeline previews publish their exact reflected ancestry,
  property/function, source and resource footprint. Native metadata observations
  and valid source edits/deletions invalidate only affected cached timelines,
  retaining old edges on removal/failure. Recovery does not certify consumers
  until they are validated again. Existing preview footprints migrate into the
  versioned graph; no second reflection or asset registry is introduced.
- Focused cache tests cover property rename/removal, unrelated cache byte
  preservation, source deletion/recovery and project relocation. The real
  `native_source_edit_restarts_real_play_and_recovers_after_compiler_failure`
  check passes MIPS compilation and PCSX-Redux relaunch, different process IDs
  and executable content after editing C++, no old Game frame after a compiler
  error, recovery to the original executable and explicit Stop cleanup.
- Earlier checks: 202 Rust tests pass, 11 ignored; the final focused CLI
  parse-failure regression and explicit real Play restart check pass. Host logs
  are retained in `artifacts/timelines/phase5-host-tests.log` and
  `phase5-native-restart.log`. The latest native restart fixture is
  `epok-native-restart-48396-1788931423289624500` under the host temporary folder.
  TimelineAsset source/cook/cache acceptance passes again, including MIPS tables
  and generated native/Blueprint accessors (`artifacts/timelines/phase1.json`,
  fixture `epok-timeline-assets-p70vg6zt/Game`).
- The authored combat regression also passes MIPS, PCSX-Redux and a byte-identical
  standalone export (`artifacts/timelines/phase4-combat.json`, fixture
  `epok-timeline-combat-nfuqyy29/Game`). Text/data/BSS remain
  211,188 / 69,384 / 517,064 bytes; one hit leaves 75 HP, normal draining and
  cancellation/owner destruction/scene replacement pass, and particle/sprite
  dropped counts remain zero. The inspected emulator image shows HP=75, one hit
  and stopped Fireball. The maximum observed completed-frame simulation is 209
  scanlines including gameplay; this is not physical-console performance proof.
  Editor build, formatting and diff checks pass. Strict Clippy still reports only
  the three incoming pipeline/gui/viewport findings. macOS and physical PSX
  validation remain unavailable.
- This is not Phase 5 acceptance: the complete Blueprint/effect/scene/generated
  output/staging/export graph, dependency viewer, exact broken-catalog isolation
  and profiling acceptance remain. The watcher still conservatively invalidates
  all cached previews when source/reflection discovery fails, and global build
  fingerprints still include all authored timelines/effects.
- Blueprint generation now captures per-class dependency footprints from the
  exact snapshots validated by the compiler, including reflected ancestry,
  debugger property layouts, referenced/overridden functions, typed class
  references, parent/callee Blueprint sources, imported resources (including
  resolved template resources), direct timeline/effect plays and marker IDs.
  Persisted-source compilation publishes these into the existing
  artifact graph; unsaved canvas compilation remains side-effect free. Source
  removal and failed compilation keep prior dependency edges and last valid
  generated signatures. Timeline/effect/marker source observations now propagate
  to those Blueprint outputs. Generated target staging and the dependency viewer
  remain pending; these host records never replace current-source cook checks.
- Generated Timeline and ParticleEffect headers now publish their actual content
  signatures and validated compiler footprints per staging destination. Effect
  headers additionally depend on their source, referenced texture revisions and
  the texture mappings they use in that destination. Instance overrides and
  entity bindings are emitted in `scene.hh`, so they remain scene-generation
  dependencies rather than being attributed to a context-free effect header.
  A failed staging operation marks prior playback headers stale, retaining their
  signatures; a successful removal drops them from the current target's edge set.
  Build and standalone export destinations require independent regeneration.
  Conflicting source signatures within one captured batch fail staging. Before
  publishing, the batch also rejects a conflicting observation or newly recorded
  input failure made after it started; an old worker cannot clear that diagnostic.
  `stage-playback:*` explicitly covers the playback headers only; complete scene,
  native output, resource packaging and executable/export provenance remains.
- The combat integration now launches the installed PCSX-Redux `.main` binary
  directly on Windows, matching Play's existing process ownership. A timed-out
  updater wrapper had left its child alive; that specific child was terminated
  and the direct-process repeat passed MIPS, identical standalone reconstruction,
  Impact/completion/cancellation, owner destruction and scene replacement
  (`epok-timeline-combat-bkb10dle/Game`). Timeout cleanup now waits for the owned
  process and escalates to kill only if termination itself times out.
- Playback staging verification: 205 Rust tests pass, 11 ignored, including
  destination-specific invalidation/recovery, removed headers, inconsistent
  snapshots and preservation of a newer watcher observation. The full log is
  `artifacts/timelines/phase5-staging-tests.log`. Effect source acceptance passes
  MIPS generation, stale header provenance after an orphaned instance override,
  recovery and all seven presets (`epok-effect-assets-7ly_jxhe/Game`). Combat
  acceptance passes again with generated-header signature/edge assertions for
  both build and standalone export (`epok-timeline-combat-uu2p0n3f/Game`, report
  `artifacts/timelines/phase4-combat.json`). Text/data/BSS remain
  211,188 / 69,384 / 517,064 bytes; one hit leaves 75 HP, no particles/sprites are
  dropped, and the maximum observed completed-frame simulation remains 209
  scanlines including gameplay. The stopped-state screenshot was inspected.
  A test-only Windows verbatim-path comparison failed before emulator launch;
  the harness now normalizes that prefix and the full repeat passes. Editor
  build, formatting and diff checks pass; strict Clippy still reports the same
  three incoming pipeline/gui/viewport findings. Physical PSX and macOS remain
  unvalidated. These checks do not complete the outstanding Phase 5 graph,
  dependency viewer or profiling acceptance.
- Scene generation now participates in the same host dependency batch. Build,
  Play and Export carry an explicit input origin: a saved document snapshot,
  the editor's in-memory document, or an anonymous caller-supplied scene. A scene
  opened in the editor is never mislabeled as the project's startup file. Saved
  and editor snapshots use distinct project-relative document keys, keeping
  existing scene serialization and all entity/asset UUIDs unchanged. Scene-bank
  loading returns the exact migrated input snapshots used for that generation;
  template refresh and resource resolution still run through existing code.
- `generated-scene:<destination>/scene.hh` records its actual generated bytes,
  root/additional scene inputs, scene registry, render settings, consumed script
  catalog, Blueprint sources, prepared playback headers, resource revisions and
  reflected effect override properties (including properties with no track).
  The editor observes consumed saved maps and its current document; changed,
  deleted or malformed maps retain stale consumer edges, and reopening another
  document stales the previous in-memory snapshot. Failed staging stales the
  prior scene header. Derived lighting caches are excluded from the authoring
  signature so a LightingBaked event cannot restart its own Play. Playback-only
  aggregates stay independent of scene-only edits. Native staged files, the
  complete resource/executable/export chain, dependency visualization and exact
  isolation of broken catalogs remain unfinished Phase 5 acceptance work.
- Scene provenance verification passes 208 Rust tests, 11 ignored, with the full
  log in `artifacts/timelines/phase5-scene-tests.log`. The expanded real Play
  restart test passes C++ edits, compiler-error recovery, a subsequent unsaved
  scene edit producing a new process/executable, unchanged saved scene bytes,
  stable Play after the lighting-cache update and explicit Stop
  (`phase5-scene-restart.log`, fixture
  `epok-native-restart-29780-1788936093685875000`). Combat, MIPS and identical
  standalone export pass with scene-header hashes and dependency assertions in
  `epok-timeline-combat-ruemotkz/Game` (`phase4-combat.json`). Text/data/BSS remain
  211,188 / 69,384 / 517,064 bytes; one hit leaves 75 HP, dropped particle/sprite
  counters stay zero and maximum observed completed-frame simulation remains
  209 scanlines including gameplay. The stopped-state screenshot was inspected.
  Strict Clippy retains only the three incoming pipeline/gui/viewport findings;
  formatting and diff checks pass. Physical PSX and macOS remain unvalidated.

- Native/script/stage provenance now extends the same dependency graph. Native
  files retain their exact source-byte snapshots; generated Blueprint headers
  and debug metadata depend on the compiler's captured footprints. Successful
  writes through the existing stage writer, including unchanged files, form a
  destination-specific manifest. Abandoned output files are excluded. Generated
  scene headers depend on script headers; body-only C++ changes flow through
  their own staged outputs. Runtime files use their embedded source signatures.
  Resource-generated files still use conservative footprints pending finer
  provider integration; this does not satisfy exact consumer isolation yet.
- Build tickets check the actual staged bytes and native observations before
  and after compilation. Starting a compile marks the previous executable stale;
  a changed stage or compiler-option snapshot cannot publish or launch its
  result. A superseded ticket cannot invalidate a newer completed ticket.
  Debugger/disc preparation extends the captured manifest explicitly. Standalone
  exports publish their own stage and documentation dependencies; rebuilding one
  destination does not certify an older export. Tool/SDK contents and full
  external-source freshness still require further coverage; no immutable
  operating-system snapshot or native hot reload is claimed.
- Stage/build verification passes 211 Rust tests in total, with 11 ignored
  (`artifacts/timelines/phase5-build-tests.log`). Three focused tests cover native
  body invalidation, stale exports, exact write manifests, old-output exclusion,
  tampering and superseded compilation. The authored combat integration passes
  MIPS, identical standalone rebuild and PCSX-Redux with script/stage/executable
  graph assertions (`epok-timeline-combat-adl0tywu/Game`, `phase4-combat.json`).
  Section sizes remain 211,188 / 69,384 / 517,064 bytes; the stopped-state image
  shows 75 HP and one hit, with zero dropped particles/sprites. Real Play restart
  and compiler recovery also pass after build-ticket integration
  (`phase5-build-restart.log`, `epok-native-restart-53796-1788938247785646500`).
- Added Window > Artifact Dependencies using the existing graph, with ID/path/
  diagnostic search, stale filtering, input/direct/transitive consumer links,
  shortest-path tooltips, missing-node navigation, retained signatures and
  originating stale reasons. History is bounded to 64 selections. It reads only
  while open, at most once per second unless explicitly refreshed. Failed reads
  retain a visibly labeled last snapshot and never certify artifacts. A Recorded
  label means no known invalidation in the snapshot; Build/Export still validate.
  The actual ImGui navigation/read-failure test passes and the real editor capture
  `artifacts/timelines/phase5-dependencies.png` was inspected. A first interaction
  test clicked coordinates from the previous selection's layout; the harness now
  renders the committed selection before the next click, matching normal frames.
  User/architecture/testing documentation describes the viewer and current limits.
- The viewer regression suite passes 212 Rust tests in total, 12 ignored
  (`phase5-dependency-viewer-tests.log`); its explicitly serialized ImGui test
  also passes. Native and Blueprint-debug MIPS builds plus PCSX RAM assertions
  pass in `epok-bp-features-joel_yc4/Blueprint Features`
  (`phase5-build-blueprint-features.log`). Debug output remains separate from the
  normal target. The viewer's new Clippy finding was fixed; only the three
  incoming pipeline/gui/viewport findings remain.
- Closed a concrete ClassRef footprint gap: selected classes in property defaults,
  inline literal inputs and literal nodes now contribute their authoritative
  ancestry, alongside the declared base. The compiler regression exercises all
  three forms, verifies a reparented choice stales only its consuming Blueprint,
  requires compilation to reject the incompatible choice and revalidates after
  repair. Runtime factories and class resolution remain unchanged. Dynamic class
  factories are still represented by the existing scene/catalog generation and
  require the remaining exact-consumer audit, not a parallel class system.
- Current verification after the ClassRef correction: 213 Rust tests pass, 12
  ignored (`phase5-class-reference-tests.log`); the separate real ImGui navigation
  check passes. The expanded feature integration verifies release/debug stage
  and executable hashes, debugger source-list patch provenance, and actual PSX
  RAM assertions (`phase5-class-reference-features.log`, fixture
  `epok-bp-features-g2jhgdll/Blueprint Features`). The PS-X executables are
  227,328 bytes (release) and 235,520 bytes (debug). Strict Clippy still reports
  only the three incoming findings. Physical PSX and macOS remain unvalidated;
  the recorded emulator measurements do not establish worst-case game budgets.
  The viewer also passes real-click verification after sizing its first window
  to the available display. The 1024x720 capture
  `artifacts/timelines/phase5-dependencies-compact.png` was inspected: controls,
  close button and both scrollable columns remain within the window. The latest
  editor binary builds; formatting and diff checks pass.

- Resource providers now return their successful output hashes and existing input
  signatures to the same staging batch. `generated-resource:<destination>/<path>`
  records participate in normal transitive invalidation, removed-output handling,
  stale failure retention and exact write-manifest verification. This is host
  provenance data, not another asset loader, runtime table or identity system.
- SPU-ADPCM and XA payloads depend only on the imported package snapshot actually
  used by the audio provider. Unrelated native or scene changes no longer stale
  their bytes. Audio staging consumes the already selected project asset index
  and checks loaded package ID/revision before conversion, rejecting a package
  reimported or replaced after selection. Reimport/delete invalidation still
  propagates to each build/export; rebuilding one destination cannot certify
  another. Shared audio-bank selection and disc composition remain conservative.
- `display.hh` records the existing Rendering generator's validated settings
  output. The scene observer reads project settings once and derives both the
  complete scene-settings signature and the display projection from that same
  snapshot. Scene transforms and geometry-only triangle-budget changes leave the
  display record unchanged; display changes or unreadable settings stale it.
  Repairing settings does not certify an old generated header. HUD budgets,
  geometry providers, build lists, catalog isolation and external toolchain
  coverage still need the remaining exact-consumer work.
- Focused tests exercise real project staging with two imported clips, native and
  scene edits, package reimport after selection, removal, stale exports and
  unchanged saved scene bytes. Display tests exercise actual project settings,
  an unconsumed geometry field, a changed resolution, regeneration and malformed
  settings/recovery. Both pass. Expanded MIPS feature and XA fixtures assert
  actual resource hashes and single-package dependencies rather than accepting
  stage metadata alone.
- Resource provenance verification passes 215 Rust tests, 12 ignored
  (`artifacts/timelines/phase5-resource-snapshot-tests.log`). The normal/debug
  Blueprint feature fixture passes MIPS and PCSX RAM assertions with actual
  audio/display output hashes (`phase5-resource-features.log`,
  `epok-bp-features-rqm_v4x6/Blueprint Features`). The authored Fireball passes
  MIPS, a byte-identical standalone rebuild and emulator lifecycle checks
  (`phase5-resource-combat.log`, `epok-timeline-combat-ifmxbjhy/Game`); its stopped
  image was inspected. Text/data/BSS remain 211,188 / 69,384 / 517,064 bytes,
  with one hit leaving 75 HP and no dropped particles/sprites.
  XA conversion, resource hashes, MIPS linking and CD image generation pass the
  streaming/XA build fixture (`phase5-resource-xa.log`,
  `artifacts/streaming-xa/1788940333125734400`). The first XA attempt exposed the
  fixture's copied relative encoder/packager paths; the verifier now preserves
  their configured tool locations without changing user configuration. This run
  used `--build-only` and does not claim new XA playback measurements. Formatting,
  editor build and diff checks pass. Strict Clippy retains the same three incoming
  findings; physical PSX and macOS remain unvalidated. Phase 5 stays in progress.

- Timeline/effect catalog observation now preserves independent valid documents
  through unreadable files and duplicate/cross-family UUIDs. It uses the existing
  typed loaders and namespace identities; removed/unreadable/ambiguous known
  sources stale their transitive consumers without hiding other source changes.
  Recovery preserves stale previews until fresh validation. Open asset catalog
  failures are separate from native reflection failures and affect only that
  document. Polling continues while another document's error remains unchanged.
- Scene preparation and typed Blueprint direct-play/marker compilation load
  referenced timelines/effects from the same partial observations. Required
  missing or ambiguous assets still fail; strict all-source validation commands
  retain project-wide diagnostics. Missing required timeline preparation no
  longer invalidates every preview. Automatic Play's global fingerprint error
  gate, asset pickers and native/Blueprint catalog isolation still need work;
  selective asset cooking alone does not accept Phase 5.
- Selective-source verification passes 217 Rust tests in total, 12 ignored
  (`artifacts/timelines/phase5-selected-assets-tests.log`). The authored combat
  integration passes with unrelated malformed timeline/effect files present:
  MIPS, byte-identical standalone export and PCSX-Redux lifecycle/RAM assertions
  (`phase5-selected-assets-combat.log`, `epok-timeline-combat-ef33jso3/Game`).
  Corrupting the used effect subsequently rejects the build and stales its stage;
  source restoration does not certify that old artifact. Text/data/BSS remain
  211,188 / 69,384 / 517,064 bytes; one hit leaves 75 HP with no dropped work.
  The inspected stopped-state image agrees. Maximum observed completed-frame
  simulation remains 209 scanlines including gameplay, not worst-case or physical
  hardware acceptance. Real native-edit/rebuild/Play restart, compiler-error
  recovery and unsaved-scene restart pass (`phase5-selected-assets-restart.log`).
  Both explicitly serialized Timeline/ParticleEffect ImGui edit/undo/save checks
  pass (`phase5-selected-assets-ui.log`). Editor build, formatting and diff checks
  pass; strict Clippy retains only the
  same three incoming findings. Physical PSX and macOS remain unvalidated.

- Auto compile now observes timeline/effect sources independently of the coarse
  native/Blueprint fingerprint. Its editor-local signature history survives
  preview cache publication, and changed IDs trigger restart only through the
  selected stage's existing transitive dependency edges. Unused malformed files
  do not block Play. Required source failure stops old Play and fails the normal
  replacement cooker; source repair requests a new build. Global capacity/graph
  read errors remain explicit blockers. A build request establishes its source
  observation baseline before launching the worker to avoid restarting itself.
- Component and Blueprint pickers now use the same unambiguous typed source
  view, retaining valid choices alongside catalog diagnostics. Duplicate IDs
  cannot be selected; source errors never rewrite nodes or component bindings.
  Changes to assets selected by open Blueprint nodes request canvas validation,
  including unsaved nodes, without repeating it for an unchanged catalog error.
- Build tickets reobserve current playback sources before and after compilation;
  exports do so before manifest publication. This closes the playback-source
  observation gap when a file changes during initial staging or compilation,
  including when the watcher has not yet seen a consumer graph. It reuses normal
  stale propagation and never certifies changed output. Other external inputs
  and toolchain freshness still require their remaining audits.
- Selective Play verification passes 220 Rust tests, 12 ignored
  (`artifacts/timelines/phase5-selective-play-tests.log`). The expanded real Play
  test passes unchanged emulator identity with malformed unused files, used
  timeline failure/repair, C++ edit/error/repair and unsaved scene restart
  (`phase5-selective-play-restart.log`). The source-watch, picker and build-ticket
  regressions cover independent normal/debug stage edges, preview publication,
  unchanged-error stability, preserved nodes and edits during build/export.
  Both explicitly serialized Timeline/ParticleEffect ImGui controls tests pass
  (`phase5-selective-play-ui.log`). The real Blueprint capture
  `phase5-selective-play-blueprint.png` was inspected and shows valid authored
  Fireball nodes and successful validation with unrelated broken source files.
- The authored combat integration passes current-source MIPS build, identical
  standalone export and PCSX RAM/lifecycle checks after ticket revalidation
  (`phase5-selective-play-combat.log`, `epok-timeline-combat-j2du2qbw/Game`).
  The used-effect corruption check still rejects stale output. Linked
  text/data/BSS remain 211,188 / 69,384 / 517,064 bytes; one hit leaves 75 HP,
  particle/sprite drops remain zero and maximum observed completed-frame
  simulation remains 209 scanlines including gameplay. These are emulator
  fixture observations, not worst-case gameplay or physical-console acceptance.
  Editor build, formatting and diff checks pass; strict Clippy retains only the
  three incoming pipeline/gui/viewport findings. Physical PSX and macOS remain
  unvalidated; Phase 5 remains in progress. The native and Blueprint-debug feature
  regression also passes MIPS, separate stage/executable provenance and PCSX RAM
  assertions (`phase5-selective-play-features.log`,
  `artifacts/blueprints/features-validation.json`).

- Generated script manifests now use v2: emitted files/native sources are
  manifest-relative and authoring dependencies are project-relative, all with
  forward slashes. Host compiler dependency paths remain unchanged in memory;
  only portable serialization changes. Regeneration replaces v1 generated
  manifests without migrating or rewriting authoring files. Invalid/outside
  project dependency paths fail before artifact writes. Standalone exports now
  also carry the Timeline and Blueprint guides through their normal documented
  output/provenance path.
- `verify_timeline_relocation.py --emulator` passes a real Windows Fireball
  project copied with caches plus a separate export copied without compiler
  intermediates. The verifier checks both resolved paths before renaming its own
  original fixture, leaving its former path unavailable. Both MIPS rebuilds
  produce the same 268,288-byte executable; source bytes/UUIDs are unchanged,
  generated/debug sources contain no original root and the copied build has
  fresh stage/executable provenance. Play starts/stops from the copied project.
  Evidence: `artifacts/timelines/phase5-relocation.json` and `phase5-relocation.log`,
  fixture `epok-timeline-relocation-u20lw5n5`. This is an emulator startup smoke
  check, not new RAM/rendering/performance acceptance. SDK/toolchain identity
  remained fixed; macOS and physical PSX remain unvalidated.
- Portability verification passes 221 Rust tests, 12 ignored
  (`artifacts/timelines/phase5-relocation-tests.log`). The focused provider check
  additionally verifies both parent-relative escapes and an actual external file
  are rejected before writing output. Current editor build passes; strict Clippy
  retains only the three incoming pipeline/gui/viewport findings. Documentation
  describes generated-manifest v1 regeneration, path bases and the relocation
  verifier's exact scope. Remaining Phase 5 work includes shared-resource and
  native/Blueprint dependency precision, external/toolchain observations and
  worst-case gameplay profiling; Windows relocation does not complete the phase.

- Shared audio-bank output now participates in provider provenance with the
  actual generated header hash and loaded package snapshots. Authored scenes
  contribute `audio-selection:<scene input>` signatures: component clip IDs plus
  script/linked-template values. Ordinary spatial transforms and AudioSource
  volume/pitch/playback controls do not stale the bank. Clip selection changes
  propagate through only the corresponding saved/editor source identity.
  Saved-file scene and audio projections share a single migrated read snapshot;
  source repair leaves consumers stale until successful staging.
- This narrows component selection without claiming complete resource precision.
  Native registry defaults, Blueprint/template values and playback resource
  arguments retain their existing conservative footprints. No second resource
  collector or asset registry was added. Audio-bank bytes, SPU budgets and runtime
  behavior are unchanged; disc composition, HUD, geometry, build-list and external
  toolchain precision remain Phase 5 work.
- Audio-selection verification passes 221 Rust tests, 12 ignored
  (`artifacts/timelines/phase5-audio-selection-tests.log`). The expanded real
  staging test checks header hashes, native/transform/control edits, selected
  package reimport, saved/editor clip changes, failure retention and source
  restoration without implicit certification. Native and Blueprint-debug MIPS
  builds plus PCSX RAM assertions pass with actual bank provenance checks
  (`phase5-audio-selection-features.log`, `epok-bp-features-_gr74dse/Blueprint Features`).
  The XA/streaming build fixture passes conversion, bank/payload hash assertions,
  MIPS linking and CD image generation (`phase5-audio-selection-xa.log`,
  `artifacts/streaming-xa/1788944689565173600`). Its build-only run does not claim
  new XA playback validation. Current editor build, formatting and diff checks
  pass; strict Clippy retains only the three incoming findings.
- Authored Fireball also passes MIPS, byte-identical standalone rebuild and
  PCSX lifecycle/RAM assertions after audio-bank provenance integration
  (`phase5-audio-selection-combat.log`, `epok-timeline-combat-3ewwf362/Game`).
  Linked text/data/BSS remain 211,188 / 69,384 / 517,064 bytes; one hit leaves
  75 HP, particle/sprite drops remain zero and the fixture's maximum completed
  simulation frame remains 209 scanlines including gameplay. Physical PSX and
  macOS remain unvalidated. These checks do not complete remaining Phase 5
  dependency precision or worst-case gameplay profiling.

- Timeline audio selection now reuses the typed event resource traversal used by
  project cooking. Each used standalone or embedded timeline records a
  `timeline-audio:<UUID>` input, including an empty selection. The bank no longer
  depends on complete generated playback headers: visual curves, marker ticks,
  event timing and repeated references do not change its clip selection. Adding
  the first clip or removing the last one invalidates the dependent bank.
- Source observation includes these projections in unreadable/removed/ambiguous
  identity invalidation. Repairing a source does not certify retained bank output.
  Scene audio projections also include selected timeline/effect asset sets, so
  changing component resource reachability cannot hide new or removed audio.
  Compilation still resolves and validates through the existing resource index;
  no runtime representation, budget or resource resolver changed. Conservative
  native/Blueprint defaults and script/template selection remain follow-up work.
- Timeline audio-selection validation passes 222 Rust tests, 12 ignored
  (`artifacts/timelines/phase5-timeline-audio-tests.log`). The focused regression
  cooks typed event references and real imported ADPCM; it covers first/last
  selection, duplicate UUID references, timing/curve changes, source corruption,
  ambiguity, repair and independently stale build/export targets. The scene
  regression also checks replacement timeline/effect component selections.
  Editor binaries, formatting and diff checks pass; strict Clippy still reports
  only the three incoming pipeline/gui/viewport findings.
- Fireball passes MIPS, a byte-identical standalone rebuild and PCSX lifecycle
  assertions with actual embedded-timeline audio dependency and bank hash checks
  (`phase5-timeline-audio-combat.log`, `epok-timeline-combat-qsf3udjy/Game`).
  Linked text/data/BSS remain 211,188 / 69,384 / 517,064 bytes. Impact leaves
  75 HP with one hit; particle/sprite drops remain zero and maximum observed
  completed-frame simulation cost remains 209 scanlines including gameplay.
  These checks do not establish worst-case gameplay or physical PSX performance.
- Native and Blueprint-debug MIPS resource builds and PCSX RAM assertions also
  pass after timeline audio-selection integration
  (`artifacts/timelines/phase5-timeline-audio-features.log`). Physical PSX and
  macOS host validation remain unavailable. Phase 5 remains in progress: the next
  resource refinement is typed Blueprint/script/template audio selection, followed
  by the remaining shared-resource and external-toolchain dependencies and final
  profiling acceptance.

- Blueprint graph audio selection now reuses the existing resource cooker's
  typed literal traversal for both literal nodes and inline input values.
  Staging and source observation share `blueprint-audio:<UUID>` signatures;
  banks no longer depend on complete Blueprint graph sources. The selection
  includes directly played timeline/effect references through the existing
  playback reference collector, so changed resource reachability remains visible.
- Scalar graph logic edits and equivalent inline/literal-node rewiring do not
  stale the bank. Added/removed clips do. Removed or unreadable Blueprint sources
  invalidate retained selection records, and restoration still requires staging
  before dependent output is current. Parent/default/declaration/template fields
  keep conservative coverage, as do native catalog and scene-instance inputs;
  this does not claim complete script/template dependency precision. Asset
  resolution, runtime data, particle budgets and authored formats are unchanged.
- Blueprint source observation now leaves duplicate class identities unresolved
  instead of publishing the last discovered copy. Existing typed source lists
  supply the identity counts; compiler validation and asset ownership are
  unchanged. Both Blueprint and audio-selection provenance remain stale until
  the ambiguity is repaired and dependent output is regenerated.
- The expanded production CLI feature check passes scalar graph isolation,
  added literal clips, independent normal/debug banks, stale source restoration,
  clip removal and byte-identical restored executables, followed by PCSX RAM
  assertions (`phase5-blueprint-audio-features.log`, retained fixture
  `epok-bp-features-ayd1n9bs/Blueprint Features`). Normal/debug EXEs are
  227,328 / 235,520 bytes. The final editor also passes the dedicated duplicate
  identity CLI case in that fixture, including failed compilation, stale bank
  diagnostics, repair and identical normal/debug rebuilds
  (`phase5-blueprint-audio-identity.log`). These verify generation and runtime
  resource contracts, not audible output or worst-case performance.
- Final Blueprint audio-selection verification passes 223 Rust tests, 12 ignored
  (`artifacts/timelines/phase5-blueprint-audio-tests-final.log`). An earlier run
  exposed a layout-invariance fixture that captured its baseline before source
  observation; its setup now matches production and retains the full graph
  equality assertion. The final suite includes duplicate source identity and
  repair coverage. Editor binaries, formatting and diff checks pass; strict
  Clippy retains only the three incoming findings.
- Fireball again passes MIPS, byte-identical standalone export rebuilding and
  PCSX gameplay/lifecycle assertions after Blueprint audio provenance changes
  (`phase5-blueprint-audio-combat.log`, `epok-timeline-combat-zhjfjwq9/Game`).
  Linked text/data/BSS remain 211,188 / 69,384 / 517,064 bytes; Impact produces
  one hit and 75 HP, with zero particle/sprite drops and a maximum observed
  completed-frame simulation cost of 209 scanlines including gameplay. Physical
  PSX and macOS validation remain unavailable. Phase 5 still needs inherited
  default/template and shared-resource precision, remaining external dependency
  coverage and worst-case profiling acceptance before Ironwood spell authoring.

- Build tickets now reread current Blueprint sources through the existing typed
  loader before compilation and before executable certification; standalone
  export rereads them before publication. Observation does not recompile or use
  cached descriptors. Parse/discovery failures invalidate known Blueprint and
  class-set provenance instead of leaving old targets current.
- Added `blueprint-sources` membership provenance from the actual compiler input
  IDs, including empty sets and duplicate counts. Scene/resource preparation and
  late script generation must agree on captured membership; additions/removals
  during preparation are rejected as mixed snapshots. Factory glue, scene
  generation and audio resource reachability consume membership while individual
  class headers retain their own dependencies. Builds/exports reject older stage
  manifests without this coverage and regenerate through normal staging. No
  authored schema or runtime representation changed. This closes Blueprint
  source freshness at the existing certification checkpoints, not all external
  dependency coverage or an operating-system-wide atomic snapshot guarantee.
- Blueprint certification verification passes 225 Rust tests, 12 ignored
  (`artifacts/timelines/phase5-blueprint-certification-tests.log`). Deterministic
  build-ticket tests mutate actual Blueprint files between capture and completion,
  exercise layout-only changes, changed defaults, additions, removals, unreadable
  sources, independent exports and conflicting scene/compiler input sets. A
  separate native-only fixture covers the first Blueprint and rejects retained
  manifests without membership provenance. Both fixtures regenerate successfully.
  Editor binaries, formatting and diff checks pass; strict Clippy retains only
  the three incoming findings.
- The production feature integration passes native and Blueprint-debug MIPS
  builds and PCSX RAM assertions with captured membership/hash checks on both
  staged source manifests (`phase5-blueprint-certification-features.log`,
  `epok-bp-features-z41v8nc2/Blueprint Features`). Audio selection, duplicate
  identity rejection, repair and byte-identical normal/debug rebuild checks also
  pass. EXEs remain 227,328 / 235,520 bytes; this does not claim physical-console
  or worst-case gameplay performance.
- Fireball passes the final MIPS/PCSX lifecycle check and a byte-identical
  standalone export rebuild with Blueprint certification enabled
  (`phase5-blueprint-certification-combat.log`, `epok-timeline-combat-6t_x897m/Game`).
  Linked text/data/BSS remain 211,188 / 69,384 / 517,064 bytes; one Impact leaves
  75 HP, particle/sprite drops are zero, and maximum observed completed-frame
  simulation cost remains 209 scanlines including gameplay. Physical PSX and
  macOS remain unvalidated. Phase 5 continues with inherited/template resource
  precision, remaining scene/resource/toolchain freshness and profiling acceptance;
  Ironwood spell authoring still follows engine acceptance.

- Build and export certification now share one source-observation path. It
  reuses the scene observer's migrated saved-document snapshots, map registry
  and rendering settings at both build checkpoints and before export publication.
  Without an active editor context, observation never replaces or closes
  submitted unsaved scene inputs. The editor retains ownership of those changes.
- The same checkpoints obtain a fresh asset index through the existing scanner
  and observe recorded package dependencies. Package decoding and checksum
  validation catch changed settings, equal-length payload corruption, removal
  and duplicate identities without relying on the editor's timestamp cache.
  Unrelated broken packages and unimported raw-source edits do not invalidate
  independent outputs. Repair retains stale consumers until successful staging.
  This does not add an asset resolver or change runtime representations/budgets.
- Focused production-staging tests pass saved/editor separation, registered map
  edits, registry membership, invalid scene/settings reads, package mutation and
  independent build/export recovery. Full regression passes 227 Rust tests,
  12 ignored (`artifacts/timelines/phase5-scene-resource-certification-tests.log`).
  Editor binaries, formatting and diff checks pass; strict Clippy retains only
  the three incoming pipeline/gui/viewport findings. Remaining
  external-source/toolchain coverage, inherited/template dependency precision
  and worst-case profiling still gate Phase 5 acceptance.
- Fireball passes MIPS, byte-identical standalone rebuilding and PCSX gameplay/
  lifecycle assertions with scene/resource certification enabled
  (`phase5-scene-resource-certification-combat.log`, retained fixture
  `epok-timeline-combat-sawwiyg1/Game`). Linked text/data/BSS remain
  211,188 / 69,384 / 517,064 bytes; one Impact leaves 75 HP, particle/sprite
  drops remain zero and maximum observed completed-frame simulation cost remains
  209 scanlines including gameplay. This is not worst-case or physical-console
  performance acceptance; physical PSX and macOS remain unvalidated.
- Certification exposed a prepared-scene boundary: Play refreshes linked
  Blueprint instances before lighting preparation, so resource projections must
  come from the original submitted input rather than that resolved scene.
  `stage_prepared` now carries both existing snapshots to staging; source/audio
  provenance uses the submitted document and resource cooking uses prepared data.
  The explicit real SDK/template regression passes inherited obsolete texture
  and audio removal, preserved instance overrides and successful saved-input
  certification (`phase5-scene-resource-certification-linked.log`).
- The production Blueprint feature integration passes normal/debug MIPS builds,
  typed resources, independent bank invalidation and PCSX RAM assertions
  (`phase5-scene-resource-certification-features.log`, retained fixture
  `epok-bp-features-kzmpty6f/Blueprint Features`). Executables are 227,328 / 235,520
  bytes. After the prepared-scene follow-up, the final editor rebuilds both
  destinations with byte-identical executables and fresh executable signatures
  matching their actual files. It does not claim audible output or worst-case
  performance.
- Final verification after the prepared-scene correction again passes 227 Rust
  tests, 12 ignored (`phase5-scene-resource-certification-tests-final.log`), in
  addition to the explicit linked-instance check. The final Fireball integration
  passes MIPS, identical standalone export rebuilding and PCSX lifecycle checks
  (`phase5-scene-resource-certification-combat-final.log`, retained fixture
  `epok-timeline-combat-y_2v3_yf/Game`), with unchanged section sizes, damage and
  dropped-work results. Formatting/diff checks pass; strict Clippy still reports
  only the three incoming findings. Next: capture native metadata/reflection and
  external SDK/toolchain inputs across staging and certification, then finish
  remaining resource precision and profiling acceptance. Ironwood authoring
  remains after engine acceptance.

- Native metadata reads now feed the existing staging capture. The native catalog
  records the exact `.epokscript` bytes and source membership; Clang discovery
  records the already validated include hashes and tool inputs from its existing
  cache key on both fresh extraction and cache hits. Conflicting reads within one
  staging operation fail instead of combining metadata versions.
- Scene and bank consumers retain these inputs in the versioned artifact graph.
  Certification rereads file bytes and reflection configuration without invoking
  a second parser or accepting cached declarations as current. Project paths stay
  relative; external SDK/tool/include identities are host provenance only. Older
  scene manifests lacking metadata membership coverage require normal restaging.
  This preserves conservative metadata dependencies; full linker/make inputs,
  external source packaging, resource precision and profiling remain pending.
- Metadata regression verification passes 228 Rust tests, 13 ignored
  (`artifacts/timelines/phase5-native-metadata-tests.log`). The explicit real
  extractor test also passes (`phase5-native-metadata-reflection.log`), covering
  external include mutation/removal, cache-hit capture, recorded tool inputs,
  SDK selection changes and recovery. Its initial equality check exposed a
  fixture recreating scene UUIDs; the corrected fixture reuses a saved scene and
  retains the full stage-record comparison. Editor build, formatting and diff
  checks pass; strict Clippy retains only the three incoming findings.
- Fireball passes MIPS, a byte-identical standalone rebuild and PCSX gameplay/
  lifecycle assertions (`phase5-native-metadata-combat.log`, retained fixture
  `epok-timeline-combat-9apnbivi/Game`). Text/data/BSS remain
  211,188 / 69,384 / 517,064 bytes, one Impact leaves 75 HP, particle/sprite drops
  remain zero and maximum observed completed-frame simulation cost is 209
  scanlines including gameplay. Windows project/cache relocation and isolated
  standalone rebuilding also pass with identical MIPS output and preserved
  authoring IDs (`phase5-native-metadata-relocation.log`, retained fixture
  `epok-timeline-relocation-psr6gzrl`). This relocation run does not launch Play;
  physical PSX and macOS remain unvalidated.
- The final production Blueprint feature integration passes normal/debug MIPS
  builds, typed resource selection, independent bank invalidation, duplicate
  identity repair and PCSX RAM assertions with native metadata provenance enabled
  (`phase5-native-metadata-features.log`, retained fixture
  `epok-bp-features-s63c66e1/Blueprint Features`). Executables remain
  227,328 / 235,520 bytes. This does not establish audible output or worst-case
  performance. Next: capture the actual Make/link inputs, including selected
  tools and SDK/compiler archives, and finish external-source refresh/packaging
  coverage before remaining resource precision and profiling acceptance.

- Native application input capture now queries the configured Make rules and
  fresh GCC `-M` dependency output before and after compilation. It records
  Makefiles, selected options, transitive C/C++ inputs, direct linker scripts,
  consumed SDK/libgcc archives and selected compiler/linker executables in the
  existing provenance graph. The executable depends on this capture separately
  from staging/build options, preserving Blueprint-debug source certification.
  No runtime reflection, asset resolver or PSX metadata format was added.
- Dependency recipes use the same compiler flag order as object recipes, and
  `main.dep` shares `main.o`'s target-specific Release optimization. A changed or
  previously unsuccessful native build forces application objects and linking
  through Make's explicit stamp prerequisite, including timestamp-preserving
  edits. Unchanged certified builds retain normal Make reuse. Post-build queries
  detect newly selected includes as well as changed existing files; failure
  prevents executable certification and launch. Worker cancellation still uses
  the existing bounded process polling/Stop path. Standalone builds require no
  editor capture or stamp.
- The explicit Make/MIPS regression passes (95.90 seconds,
  `artifacts/timelines/phase5-build-inputs-native.log`), covering paths with spaces
  and `#`, target-specific optimization, unchanged reuse, changed executable bytes
  after preserved-timestamp edits, and `__has_include` membership changes during
  compilation with rejection/recovery. Its owned fixture is retained at
  `epok-build-inputs-8e8c9ba9-eaad-4bb8-ae36-d10dbe99ffef/Game With Spaces`.
- This is application input certification, not complete toolchain acceptance.
  SDK archive construction provenance and stale SDK object recovery, nested custom
  linker scripts, dynamic tool components, external source refresh/packaging and
  remaining resource precision/profiling are still pending. Changed native inputs
  currently force the whole application; per-object invalidation is unfinished.
  Phase 5 remains in progress and Ironwood authoring follows engine acceptance.
- Application input regression checks pass 229 Rust tests, 14 ignored
  (`phase5-build-inputs-tests.log`), plus the explicit native Make regression.
  Editor/extractor builds, formatting and diff checks pass. Strict Clippy retains
  only the three incoming pipeline/gui/viewport findings. Fireball passes MIPS,
  a byte-identical standalone export rebuild and PCSX gameplay/lifecycle checks
  (`phase5-build-inputs-combat.log`, fixture `epok-timeline-combat-guhxdw0p/Game`).
  Its actual executable provenance includes the generated effect header and
  consumed compiler/SDK inputs. Text/data/BSS remain 211,188 / 69,384 / 517,064
  bytes, one Impact leaves 75 HP, particle/sprite drops are zero and maximum
  observed completed-frame simulation cost remains 209 scanlines including
  gameplay. Physical PSX, macOS and worst-case gameplay remain unvalidated.
- A portability review removed the inspection target's use of Make's `file`
  function. It now emits prefixed `info` records captured by the existing worker
  output readers, without shell quoting or a newer Make installation solely for
  reporting. The host replaces the previous report with each successful query's
  captured output; missing records cannot reuse an earlier manifest. Compiler
  diagnostics remain visible and emulator output retains its existing behavior.
  This avoids a known newer-function requirement; macOS execution is still
  unvalidated. Final transport/regression checks follow this correction.
- The final report transport passes the real Make/MIPS regression again (94.88
  seconds, `phase5-build-inputs-native-final.log`, retained fixture
  `epok-build-inputs-f1afed65-c2d2-4af5-8f10-296f163eacda/Game With Spaces`). Windows
  relocation with copied caches, unavailable original root, independent export
  rebuilding and relocated Play also passes (`phase5-build-inputs-relocation.log`,
  fixture `epok-timeline-relocation-2uozc1h8`). The 268,288-byte MIPS output is
  identical and authored bytes/IDs are preserved. The full production feature
  integration passed normal/debug builds and PCSX RAM checks immediately before
  this transport-only adjustment (`phase5-build-inputs-features.log`, fixture
  `epok-bp-features-3r7s4i14/Blueprint Features`); final rebuild checks reuse those
  exact executables as their comparison baseline.
- After the report transport correction, the full Rust suite again passes 229
  tests, 14 ignored (`phase5-build-inputs-tests-final.log`). Final production
  rebuilds of both retained feature destinations are byte-identical at 227,328 /
  235,520 bytes, with fresh native-input/executable certificates and no report
  records leaked into the build log (`phase5-build-inputs-transport.log`). Host
  binaries, formatting and diff checks pass; strict Clippy retains only the three
  incoming findings. SDK source construction remains the next concrete coverage
  step: the configured assembler's `--MD` probe correctly reports the real
  `crt0cxx.s` and `hwregs.inc` inputs, which Nugget's empty assembly `.dep` rule
  currently omits. This probe does not yet wire SDK certification into builds.
- The final real editor/Play restart regression passes in 142.84 seconds
  (`phase5-build-inputs-restart.log`, retained fixture
  `epok-native-restart-31704-1788953753773539700`). It verifies unchanged Play for
  unrelated broken playback sources, rejection/repair of a used timeline, changed
  emulator PID and executable after a native edit, compiler-error Stop/recovery,
  unsaved scene rebuild without changing the saved document, and no spurious
  restart from lighting preparation. Native input capture remains a worker-side
  build checkpoint; it does not patch running C++ or add a PSX hot-reload path.

- SDK construction now participates in the existing artifact graph as
  `native-sdk:<build target>`, consumed by the application's native-build node.
  The configured Nugget Makefiles and inherited build options supply the source
  and object lists. GCC reports C/C++ include dependencies; assembler `--MD`
  reports assembly includes and `.incbin` inputs. SDK signatures include actual
  file/tool bytes and the private archive's bytes. No second asset, reflection,
  entity-reference or continuation registry was added.
- Changed SDK inputs force fresh compilation, including edits that preserve file
  dates and changed wildcard source membership. The adapter preserves original
  object target names and their target-specific flags while writing compiled
  objects into an invocation-owned directory. Archive construction uses a new
  archive so removed sources cannot leave old members behind. Shared SDK runtime
  objects/libraries are preserved. Generated dependency/probe files and a marked
  Make adapter remain host build metadata under the configured SDK.
- SDK capture/build uses a nonblocking OS file lock shared by editor invocations.
  A busy SDK reports an error rather than blocking an editor frame. Failed or
  changed-input builds retain stale prior archives; successful archives publish
  through the existing atomic file writer with revision checks. Application
  capture checks that its consumed private archive matches SDK certification,
  then repeats SDK/application dependency queries before executable publication.
  This does not claim an atomic snapshot against arbitrary external SDK tools.
- Standalone Windows/Unix launchers now build private SDK objects and a fresh
  archive from current sources, then force application recompilation. Plain Make
  dispatches to these launchers; explicit `make all` remains the ordinary Nugget
  incremental path. Exports need neither the editor nor its certificate cache.
  The new Unix launcher is staged with the existing runtime source bundle.
- The SDK regression uses a separate miniature SDK with the pinned Make rules
  and real MIPS GCC/assembler/ar. It passes assembly/binary inclusion, unchanged
  cache reuse, preserved-timestamp edits, obsolete archive-member removal,
  failed-build retention/repair, mid-build mutation and newly selected includes
  after capture, plus nonblocking lock ownership (`phase5-sdk-inputs-isolated.log`,
  56.89 seconds, fixture `epok-sdk-inputs-4c303e85-a947-4d19-8d4b-5908fd9d175b`).
  Additional final shared-object and production/export checks are in progress.
  Phase 5 still needs external-source refresh/packaging, remaining dependency
  precision and profiling acceptance. Native hot reload remains out of scope;
  Ironwood spell authoring still follows engine acceptance.
- Final SDK isolation checks pass, including exact preservation of a pre-existing
  shared object and archive (`phase5-sdk-inputs-complete.log`, 53.73 seconds,
  fixture `epok-sdk-inputs-8cb66caa-ca80-46a9-b37f-ed3df88cc22a`). The default Rust
  suite passes 229 tests, 15 ignored (`phase5-sdk-tests.log`); formatting/diff
  checks and host binaries pass. Strict Clippy still reports only the three
  incoming findings. The Unix launcher passes Bash syntax checking on Windows;
  macOS execution is not validated by that check.
- Fireball passes MIPS, private SDK provenance assertions, byte-identical
  standalone rebuilding through the new Windows launcher and PCSX gameplay/
  lifecycle assertions (`phase5-sdk-combat.log`, retained fixture
  `epok-timeline-combat-dcqbtb_z/Game`). Text/data/BSS remain 211,188 / 69,384 /
  517,064 bytes. One Impact leaves 75 HP, particle/sprite drops remain zero and
  maximum observed completed-frame simulation cost remains 209 scanlines. This
  verifies the existing fixture, not worst-case gameplay or physical-console
  performance. Normal/debug and final relocation checks continue below.
- The production Blueprint feature integration passes normal/debug MIPS builds,
  private SDK dependencies, typed resource invalidation, duplicate identity
  repair and PCSX RAM assertions (`phase5-sdk-features.log`, retained fixture
  `epok-bp-features-3qyajm9_/Blueprint Features`). Executables remain 227,328 /
  235,520 bytes. Native SDK input changes now reach both independent destinations
  through their SDK certificates and actual archive/source dependencies.
- Final Windows relocation passes with copied caches, the original root made
  unavailable, preserved authoring bytes/IDs, a byte-identical 268,288-byte MIPS
  executable and relocated Play (`phase5-sdk-relocation.log`, fixture
  `epok-timeline-relocation-jf28h5w3`). The isolated export invokes plain Make,
  exercising its default dispatch through the Windows standalone launcher and
  fresh private SDK construction. Running the same export through `build.sh`
  under the available Windows Git Bash also produces identical bytes
  (`phase5-sdk-shell.log`, detailed output `phase5-sdk-shell-build.log`). This
  covers both launchers on this host; it does not establish macOS or physical
  PSX support. No tracked Nugget sources/gitlink were changed. Next: external
  source refresh/packaging and remaining consumer precision/profiling acceptance;
  the complete initiative and Ironwood spell authoring are not yet finished.

- Audio catalog precision now uses the existing `legacy_registry` and
  `Registry::properties` resolution to publish `audio-catalog` separately from
  `scene-catalog`. Audio banks consume that projection in both editor builds and
  exports. Numeric Blueprint declarations/defaults and editor flags no longer
  invalidate banks; resolved AudioClip IDs/names/defaults, including null fields
  used by instance overrides, remain covered. No alternate inheritance resolver,
  asset format, runtime table or particle budget was introduced.
- Blueprint own-variable source observation keeps only explicitly typed AudioClip
  declarations, retaining duplicate entries for compiler validation and sorting
  declaration order. Catalog errors invalidate both catalog projections; source
  repair never certifies retained consumer output. Parent/default overrides,
  template/instance inputs and native metadata still retain conservative edges.
  Focused real-bank staging checks pass numeric edits, null/changed/removed clip
  declarations and independent build/export invalidation
  (`phase5-audio-catalog-focused.log`). The full Rust suite passes 232 tests,
  15 ignored (`phase5-audio-catalog-tests.log`, 152.14 seconds for editor tests),
  including inherited null fields and failure/repair coverage. Editor binaries,
  formatting and diff checks pass; strict Clippy still reports only the three
  incoming findings (`phase5-audio-catalog-clippy.log`). The first CLI run passed
  initial normal/debug MIPS builds but used
  an incorrect assertion that Blueprint-only compilation publishes the complete
  catalog. The revised harness checks source observation separately, then checks
  both banks after full numeric-change staging before continuing clip/recovery
  tests. This corrects the test scope, not the production compilation contract.
- The corrected production CLI feature integration passes numeric declaration
  and default isolation after full project staging, clip-default/graph selection,
  independent normal/debug banks, source restoration, duplicate identity repair,
  byte-identical restored executables and PCSX RAM assertions
  (`phase5-audio-catalog-features-final.log`, report
  `phase5-audio-catalog-features.json`, retained fixture
  `epok-bp-features-_td_4an0/Blueprint Features`). Current normal/debug executable
  sizes are 229,376 / 239,616 bytes. This validates resource provenance and runtime
  behavior, not audible output or a frame-rate guarantee.
- Fireball passes MIPS, byte-identical independent `build.ps1` export rebuilding,
  and PCSX gameplay/lifecycle assertions after audio catalog projection changes
  (`phase5-audio-catalog-combat.log` and `.json`, retained fixture
  `epok-timeline-combat-4u9yi1yj/Game`). Linked text/data/BSS are
  213,892 / 68,728 / 517,312 bytes; Impact applies one hit leaving 75 HP.
  Completion, stop, owner destruction and scene replacement pass, with zero
  particle/sprite drops and 181 scanlines maximum observed simulation cost in
  this fixture. Required broken effects are rejected while unrelated malformed
  assets remain isolated. This does not establish physical PSX/macOS behavior
  or worst-case frame rate. Phase 5 remains in progress: next are inherited
  default/instance/template audio projections through the existing resolver,
  remaining native/shared-resource precision and the final acceptance audit.
  Ironwood spell authoring still follows engine acceptance.

- Template and linked-instance component audio projections now reuse the existing
  stable built-in member mapping and `apply_override` typed component applicator.
  Spatial/rendering changes, activation, construction transforms/colors and audio
  playback controls do not invalidate shared banks. Resource-bearing template
  entity IDs, script bindings and selected timeline/effect assets remain covered;
  explicit AudioSource removal remains distinct from inheriting a component.
  Unknown/new members and invalid resource-bearing values remain conservative.
  Existing constructor operations are exhaustively classified so adding a new
  operation requires an explicit resource-selection decision.
- Focused tests pass actual bank staging after template edits/removal and linked
  instance refresh after clip/volume edits, including preserved data on unknown
  member failure (`phase5-template-audio-focused.log`,
  `phase5-instance-audio-focused.log`). Inherited property-default and script
  value projection remains pending; this change introduces no new reflection,
  asset schema, runtime representation or budget. The full Rust suite passes 232
  tests, 15 ignored (`phase5-template-audio-tests.log`, 147.44 seconds for editor
  tests), including a newly added resource-free template child. Editor binaries
  build successfully; strict Clippy still reports only the three incoming
  findings (`phase5-template-audio-clippy.log`). The production CLI integration
  passes template position/scale/color/volume isolation after full staging,
  numeric declarations, clip changes, independent normal/debug banks, source
  restoration, duplicate repair, byte-identical restored executables and PCSX
  assertions (`phase5-template-audio-features.log` and `.json`, retained fixture
  `epok-bp-features-796mnmk7/Blueprint Features`). Normal/debug EXEs are
  231,424 / 239,616 bytes in this fixture.
- Final Fireball checks pass MIPS, byte-identical independent standalone rebuild,
  Impact damage exactly once, drain completion, stop, owner destruction and scene
  replacement (`phase5-template-audio-combat.log` and `.json`, retained fixture
  `epok-timeline-combat-iz82mz2b/Game`). Text/data/BSS remain
  213,892 / 68,728 / 517,312 bytes; the probe records one hit, 75 HP, no
  particle/sprite drops and 181 scanlines maximum observed simulation cost.
  Required broken effects are rejected; unrelated malformed assets are isolated.
  Physical PSX and macOS remain unvalidated. Formatting/diff checks pass and the
  tracked SDK is unchanged. The next step is a requirement-by-requirement
  acceptance audit: distinguish unresolved asset/consumer correctness from
  optional finer per-field resource optimizations, without weakening any original
  plan gate. Ironwood inspection confirms existing Fireball, Blizzard, Lightning
  and Heal Wounds actions; their VFX integration remains pending engine acceptance.

- Final acceptance audit found the reusable component-adapter contract incomplete:
  `SequenceTarget` in the gate/camera test was fixture-owned. Phase 2 is reopened
  for correction rather than treating that narrow test as general authoring
  acceptance. The existing Phase 3/4 records remain historical evidence; no later
  acceptance is closed over this gap.
- Added an opt-in project-owned `TimelineAdapters.hpp` library for Transform,
  Camera, AudioSource, Light, PaletteAnimator, ParticleEmitter and HUD Rect/Text/
  Image/Progress. Project-browser/CLI installation publishes without replacing
  existing edits. Ordinary native/Blueprint attachment, reflection and staging
  discover these classes; no alternate component or reflection registry exists.
- Reflection schema v6 adds optional inherited `TimelineRequires` component
  metadata. Old manifests deserialize without a requirement; extractor/cache
  versioning forces regeneration. Required missing/disabled components fail scene
  validation, while optional bindings retain bounded runtime skip diagnostics.
  Class dependency fingerprints include requirements. Timeline compiler v5 emits
  component guards through existing generation-checked handles.
- Runtime-owned `Behaviour::timeline_sync` captures/applies one explicitly
  addressed property, using the existing compact property ID. Component writes
  and restoration happen before events, without frame polling, allocation or
  string lookup. EffectLayer accesses remain direct. The extractor now inherits
  reflection only from explicitly annotated virtual-method ancestors, so internal
  synchronization hooks do not become authoring functions. Host runtime and real
  MIPS-header checks pass; focused source tests cover inherited requirements,
  component removal/disable, migration identity and installation preservation.
  Subsequent full production integration and regression evidence follows.
- Component-adapter corrective acceptance passes (2026-09-10). All ten installed
  native classes are reflected with explicit stable IDs; a Blueprint inherits
  the camera requirement. Required missing Light/Camera components fail actual
  cooking, unknown/duplicate requirement annotations fail extraction, and source
  edits survive attempted reinstallation. Two cooked sequences exercise component
  writes at crossing time, live restoration, HUD inheritance, optional component
  disable and required-target/owner destruction. No runtime name lookup is used
  by the shipped adapters; the fixture uses names only to inspect component state.
- TimelineEmitter now exposes Play/Stop/Burst. Explicit zero-count bursts do
  nothing; the existing 256-request queue accepts bounded work and rejected
  requests increment the saturating particle dropped counter. Runtime adapter
  limits match existing authored component ranges (including 512 emission rate
  and signed HUD size), so restoration does not narrow valid initial values.
  Palette speed zero additionally supports pausing. Host tests cover extreme
  requests, counter saturation, live capture, upper-bound/signed restoration,
  and all 256 byte-color round trips using actual PsyQo Q12 headers.
- Final adapter MIPS and byte-identical standalone export checks pass in
  PCSX-Redux (`phase2-component-integration-final.log`,
  `phase2-component-adapters.json`, fixture `epok-timeline-adapters-m1bkaeod/Game`).
  Linked text/data/BSS are 209,124 / 69,400 / 502,624 bytes. The probe records
  one Impact, 18 safe target skips, one cancellation, seven events and 552
  deliberately dropped particle requests from the two overflow bursts; no
  assertion failed. This is component-state validation, not audible output,
  pixel parity, physical-console timing or a frame-rate guarantee.
- The complete Rust suite passes 234 tests, 15 ignored
  (`phase2-component-tests.log`, 156.52 seconds for editor tests). Host runtime
  and real MIPS-header checks pass (`phase2-component-runtime.log`); editor
  binaries, formatting and diff checks pass. Strict Clippy reports only the
  three incoming findings (`phase2-component-clippy.log`). The tracked SDK
  remains unchanged; physical PSX and macOS checks are unavailable here.
- Fresh gate/camera lifecycle and Fireball combat integrations both pass MIPS,
  byte-identical standalone rebuilding and PCSX execution
  (`phase2-component-director.log/.json`, `phase2-component-combat.log/.json`).
  Gate/camera destruction, slot reuse, deactivation and scene replacement pass;
  maximum observed simulation remains 56 scanlines. Combat still applies one
  Impact for 25 damage, leaves 75 HP, completes/cancels correctly, drops no
  particles/sprites and reaches 181 scanlines maximum observed simulation cost.
  The next concrete phase is the Phase 5 final dependency/resource-selection and
  iteration acceptance audit, followed by Ironwood spell integration after
  engine acceptance. No Ironwood files have been modified.

- Phase 5 inherited-default precision now shares the Blueprint compiler's normal
  declaration ordering, persistent-ID checks, ancestry and default validation.
  The observer builds that existing registry without graph/resource compilation;
  no parallel reflection or inheritance resolver was introduced. Audio selection
  excludes inherited numeric/Texture defaults and retains explicit AudioClip
  overrides, including null. Unresolved declarations preserve raw defaults and
  do not grant type-based exclusions. Build certification requests fresh native
  metadata only when consumed Blueprint audio projections have explicit defaults.
- Focused tests pass actual bank staging after a native numeric-default edit,
  independent bank preservation, three-level inherited clip resolution,
  explicit null overrides, unknown default IDs and duplicate identity rejection
  (`phase5-inherited-audio-focused.log`). The complete Rust suite passes 235 tests,
  15 ignored (`phase5-inherited-audio-tests.log`, 140.47 seconds for editor tests).
  Host binaries and formatting pass. Production normal/debug MIPS and emulator
  validation with an additional Blueprint-derived audio-default consumer is in
  progress. Native metadata and script-valued instance/template projections remain
  pending; this change does not alter runtime budgets or Ironwood content.

- Final audio dependency implementation now uses existing bound-class and
  inherited-property resolution for scene, template and linked-instance script
  values. Numeric values and their member/override bookkeeping do not select
  audio; unknown classes/members remain conservative. Source observation reads
  current native and Blueprint declarations through the shared compiler pass,
  including native-only projects, and publishes the audio catalog without
  compiling graphs or trusting cached metadata. Raw native inputs remain stage
  and executable dependencies but are no longer direct audio-bank dependencies.
- Eleven focused audio tests pass, including actual independent bank staging,
  native numeric edits, metadata failure/repair, script-value isolation and
  persistent missing-class identity (`phase5-audio-final-focused.log`). The prior
  inherited-default integration exposed a test-baseline error after an intentional
  bank rebuild; its source-only assertion now compares each destination against
  its own last-built signature. The final original-executable byte comparison is
  retained. Full Rust passes 237 tests, 15 ignored (168.51 seconds for editor
  tests in the first run; the final editor-observer run takes 178.59 seconds,
  `phase5-audio-final-tests.log`); formatting/diff checks pass. Strict
  Clippy reports only the same three incoming pipeline/gui/viewport findings
  (`phase5-audio-final-clippy.log`). Production MIPS validation remains in
  progress; this is not yet a new acceptance claim. No Ironwood content changed.
- The corrected inherited-default production integration passes normal/debug
  MIPS builds, independent bank invalidation, byte-identical restored executables
  and PCSX RAM assertions (`phase5-inherited-audio-features-final.log`, report
  `phase5-inherited-audio-features.json`, fixture `epok-bp-features-8w8p29l4`).
  Normal/debug EXEs are 233,472 / 241,664 bytes. The final combined source-selection
  integration is checking the later script/native projections as well.
- Editor scene observation now runs after the existing source/reflection refresh
  and receives its validated registry or its current error. This avoids running
  Clang discovery every 300-ms scene poll. Build/export certification always
  rereads current declarations. An errored refresh cannot publish audio selection
  from retained registry data; ordinary scene observation cannot certify an
  executable. The complete Rust suite remains green after this ordering change;
  the real Play restart regression is part of final validation.
- The final combined CLI integration passes native numeric-default, inherited
  default, template-script and scene-script isolation after full MIPS staging.
  Clip edits invalidate both consumers, stage actual ADPCM, and keep the untouched
  destination stale. Duplicate Blueprint identities fail compilation; repaired
  and restored sources rebuild both original executable byte streams. PCSX RAM
  assertions pass (`phase5-audio-final-features.log/.json`, retained fixture
  `epok-bp-features-2f4sij40`, normal/debug EXEs 233,472 / 241,664 bytes). The
  remaining current-state checks are real Play restart and combat/standalone
  export regression, not further per-field dependency optimization.
- Real Play restart passes after the observer-order change: native C++ edits,
  compiler failure/recovery, unsaved scene edits, unchanged lighting-cache state,
  external-header edits with preserved timestamps, deletion and repair all
  complete through Stop/rebuild/relaunch (`phase5-audio-final-restart.log`,
  392.85 seconds, retained fixture `epok-native-restart-42544-1788970233094494900`).
  Combat MIPS/standalone/emulator regression is the final acceptance check.
- Phase 5 acceptance passes: the final authored Fireball regression builds MIPS,
  rebuilds a byte-identical standalone export and passes PCSX lifecycle checks
  (`phase5-audio-final-combat.log/.json`, fixture `epok-timeline-combat-l624hazz`).
  One Impact applies 25 damage, leaving 75 HP; completion, stop, owner destruction
  and scene replacement pass. Particle/sprite drops are zero. Text/data/BSS remain
  213,892 / 68,728 / 517,312 bytes; maximum observed simulation is 181 scanlines
  in this fixture. This closes the current-state check following the complete
  Rust suite, combined normal/debug feature integration and real Play restart.
- Acceptance includes the earlier dependency-viewer interaction, relocation,
  host/PSX parity and load evidence recorded above. Shared scene/project output
  inputs and conservative handling of invalid/unknown declarations are explicit
  boundaries, not a promise of arbitrary per-field incremental compilation.
  No native hot reload, extra runtime budgets or parallel identity/continuation
  system was introduced. Physical PSX and macOS checks remain unavailable;
  saturation workloads are not a 60-FPS guarantee. Strict Clippy still reports
  the three incoming unrelated findings. Ironwood spell authoring follows now.

| Phase | Current status |
| --- | --- |
| 0 | Passed; measured native prototype and Blueprint prerequisites revalidated |
| 1 | Typed asset core accepted; runtime application and consumer wiring proceed in later phases |
| 2 | Accepted after corrective audit: reusable component adapters, live capture/restoration, required/optional component guards and bounded Burst pass MIPS/export/PCSX; gate/camera lifecycle regression passes |
| 3 | Accepted: authored fireball, artist preview/controls, typed instance overrides, measured budgets and MIPS/host parity |
| 4 | Accepted: typed direct/component playback, nonblocking subscriptions, reentry/lifecycle rules and authored combat pass MIPS/PCSX/export |
| 5 | Accepted: dependency viewer, stale diagnostics, safe preview/Play iteration, profiling, docs/examples and transitive staging/export provenance have acceptance evidence. Final 237-test suite, normal/debug MIPS/PCSX, real Play restart and byte-identical combat export pass. Ironwood's additional spell integration remains. |

## Recommendation

Build one typed, compiled `TimelineAsset` system for scene sequencing, cinematics,
and VFX. A `TimelineComponent` (also presented as a *Sequence Director* in the
editor) plays an asset on a scene entity and supplies that asset's scene bindings.
Particle effects use the same representation, but carry an embedded timeline by
default for their internal layers. Do not create a separate, incompatible particle
timeline format.

The target authoring experience is a compact RPG-effect workflow: compose a few
pixel-art layers, choreograph them against sound/camera/lighting/markers, then
play the result from a Blueprint or a scene director. The target runtime is a
bounded, deterministic PSX implementation, not a general-purpose visual-effects
framework clone.

This work starts only after the Blueprint foundation and the applicable later
Blueprint milestones are complete. In particular it depends on:

1. Phase 1's versioned reflection registry, typed properties, stable entity
   authoring IDs, `EntityRef`, and transitive dependency invalidation.
2. Phase 3's compiled Blueprint graph path and event/call validation.
3. Phase 4's bounded latent-continuation contract, owner cancellation, and
   pause/deactivation semantics.

The Blueprint plan remains authoritative for reflection, entity handles, generated
C++, builds, exports, and the rule that editing during Play initially rebuilds and
restarts Play rather than attempting unsafe native hot reload.

## Product goals

- Let a VFX author make a reusable fire, smoke, impact, projectile, aura, buff,
  summoning, or spell effect from presets and a visual timeline rather than code.
- Let a level author animate scene entities, cameras, audio, lights, palettes,
  HUD, and particle effects through a reusable asset with explicit scene bindings.
- Let Blueprint play a sequence and respond to named markers without owning the
  visual choreography or polling frame counts.
- Keep every PSX-side allocation, active sequence, event queue, particle count,
  and per-frame evaluation cost bounded and observable.
- Preserve deterministic replay: the same cooked data, seed, input, and simulation
  ticks produce the same event order and particle state.

## Non-goals for the first release

- A fully generic property editor that can animate arbitrary memory, pointers,
  resources, or unannotated C++ fields.
- Runtime graph interpretation, heap allocation, reflection-by-name, or editor-only
  metadata on PSX.
- General particle collision, GPU particles, shader graph material effects,
  arbitrary sub-emitter recursion, or post-processing.
- Native C++ live patching on a running PSX executable.
- Sequencer collaboration, recording, curves with unbounded keys, nested sequences,
  or cinematic editing parity with modern desktop engines.

## Existing basis

EpokEngine already has a useful bounded particle implementation:

- `src/particles.rs` and `runtime/particles.hpp` implement deterministic emission,
  bursts, fixed pools, seeded velocity spread, lifetime, gravity, size/color
  interpolation, local/world space, and flipbooks.
- The current budgets are 64 authored emitters, 128 particles per emitter, 256
  globally, and 2,048 sprite triangles per frame.
- `ParticleEmitter` already exposes `play`, `stop`, and `burst`; particles are
  invalidated on owner generation changes.
- The authoring preview uses a bounded, fixed-step timeline-like simulation.

The new system must reuse these invariants. It may replace the authoring data shape
and generated setup code only through a versioned migration. It must not silently
increase these limits or make particle slots into entity slots.

## Architecture

```text
TimelineAsset (.timeline.json)                 ParticleEffect (.particle-effect.json)
  slots, tracks, keys, markers                    layers + embedded TimelineAsset
               |                                              |
               +------------------+---------------------------+
                                  validation
                                      |
                             compiled timeline data
                                      |
          TimelineComponent / SequenceDirector on a scene entity
                                      |
                           resolved bindings and runtime state
                                      |
                 generated C++ + staged data + MIPS / PSX runtime
```

### Timeline asset

`TimelineAsset` is a versioned asset with a UUID, display name, duration,
timebase, loop mode, slots, tracks, key IDs, markers, layout, and dependency
signature. IDs are persistent and independent of labels and list ordering. Store
the editable authoring representation in JSON and derive compiled/cached output
under `.epok/`; exports contain the cooked C++/data they need, not the editor.

An asset has **binding slots**, never concrete scene slot indices:

```text
Slot "Boss"       : Entity<Behaviour-compatible>
Slot "MainCamera" : Camera
Slot "Gate"       : Entity
Slot "EntranceFX" : ParticleEmitter
```

Tracks target either an internal object or a slot plus a reflected, explicitly
animatable property/function. They use stable `PropertyId`/`FunctionId`, not a
serialized member name or pointer. The initial target kinds are:

- `Property`: an allowed scalar, `Fixed`, bool, enum, `Vec2`, `Vec3`, color, or
  supported small struct property.
- `Transform`: position, rotation, or scale on an entity target, represented as
  validated component properties rather than a raw matrix write.
- `Particle`: `Play`, `Stop`, `Burst`, and selected effect/emitter controls.
- `Event`: a typed, registered event/function call with bounded arguments.
- `Marker`: a named signal; it has no gameplay side effect itself.
- `Audio`, `Camera`, `Light`, `Palette`, and `HUD`: narrowly defined adapters
  registered in the same catalog, not special untyped escape hatches.

Tracks have a declared blend policy. The first release supports `Absolute` and
`Additive` only, validates conflicts at authoring time, and has deterministic
ordering by priority then track ID. Two absolute tracks may not write the same
property at the same priority and overlapping time unless the author explicitly
selects a documented override policy.


### Curves and keys

The editor may show smooth handles, but the portable semantic form is deliberately
small: a maximum of four keys per curve segment group, linear/step/smoothstep/
ease-in/ease-out interpolation, and values representable by the target type.
Compile curves to Q12 or integral samples/segments. Do not evaluate host `f32`
curves at runtime and do not invent a different rounding model from `Fixed`.

Events and markers fire once when the simulation time crosses their timestamp;
large `dt` is processed in chronological order. A seek has an explicit policy:
preview may rebuild from time zero; runtime play only permits bounded forward
advance, restart, or a cooked checkpoint if a future measurement proves it useful.
This avoids replaying uncontrolled side effects after a backward seek.

### Scene component and bindings

`TimelineComponent` belongs to an entity, carries a `TimelineAssetRef`, playback
settings, and a binding table. It provides `play`, `pause`, `resume`, `stop`, and
restart operations; its enabled/active state follows the existing component and
scene lifecycle contract.

A binding stores an authoring entity UUID plus expected target type. At cooking,
the UUID resolves to a target slot and generation-aware handle; a previous-run
handle is never serialized. At runtime each use revalidates the handle:

- A missing, inactive, destroyed, type-incompatible, or generation-mismatched
  binding skips just that track/event and increments a bounded diagnostic counter.
- A reused entity slot never receives an old track because generations differ.
- The director stays alive unless its own owner becomes invalid; it can complete
  successfully with skipped bindings under a documented `BestEffort` policy.
- A `Required` slot makes validation fail for cook/build and blocks normal play
  until it is assigned. Optional slots may be intentionally empty.

The Inspector shows broken, stale, optional, and required bindings separately and
offers reassignment. Asset authors never need to edit a reusable asset merely to
point it at a new scene's boss, camera, or gate.

### Particle effects

Introduce a reusable `ParticleEffect` asset rather than duplicating emitter
configuration across every scene entity. It contains a small layer list (initially
at most eight) and an embedded `TimelineAsset` whose targets are its named internal
layers. A layer is a constrained wrapper around the current emitter/sprite data;
the first release includes a sprite/flipbook layer and a particle-emitter layer.

The particle editor opens the embedded timeline automatically. It provides presets
such as fire, smoke, sparks, impact, projectile, aura, and rune, plus a dark
looping preview. The artist edits expressive controls (density, violence, scale,
brightness, chaos, direction, and duration); these map to existing validated
particle values. Advanced controls remain available in categorized sections.

`ParticleEffectComponent` is the persistent scene form and may reference the
effect with explicitly serialized overrides. `SpawnParticleEffect` creates a
transient effect instance from a dedicated bounded pool; it returns a
generation-checked `EffectHandle`. It does not require the caller to create an
otherwise-unused scene entity. On exhaustion, the default policy is `Drop`, with
an observable counter. Any future `ReplaceOldest` policy must be explicitly chosen
by the effect and tested for determinism.

Internal effect tracks need no scene binding. External effect slots—such as a
target entity, camera, or temporary light—use the same binding model as a scene
timeline. Sub-effects are deferred: authors initially compose named layers in one
asset rather than introduce recursive spawning.

## Blueprint contract

Expose small typed nodes after the executable graph and continuation systems exist:

```text
Play Timeline (TimelineComponent) -> SequenceHandle
Play Timeline Asset (TimelineAsset, bindings) -> SequenceHandle
Stop / Pause / Resume Timeline (SequenceHandle)
Spawn Particle Effect (ParticleEffect, Transform, Seed?) -> EffectHandle
Play / Stop / Burst Effect (EffectHandle or persistent component)
Wait for Timeline Marker (SequenceHandle, MarkerId)
Wait for Timeline Complete (SequenceHandle)
```

`Wait` nodes are latent continuations and inherit Phase 4's capacity, ownership,
pause, destruction, scene-replacement, and cancellation rules. They never block a
frame. Direct marker event subscriptions are also available when waiting is not
appropriate. Blueprint gameplay determines damage, mana, target selection, and
turn progression; the VFX asset determines presentation timing. A typical spell is:

```text
Cast Fire -> Play FX_Cast -> launch projectile -> marker "Impact"
          -> apply damage -> Play FX_Impact -> wait for completion
```

## Reflection and invalidation

Add a `TimelineAnimatable`/equivalent flag to the existing reflection schema,
separate from ordinary editability and Blueprint visibility. A property must state
its allowed timeline interpolation/blend modes and its runtime adapter. A function
must be explicitly callable from timeline and state whether it is an idempotent
property action or a crossing-time event. Do not make `Editable` mean animatable.

Build a transitive dependency graph covering native annotated headers, reflection
manifests, Blueprints, timeline assets, particle effects, scenes, generated C++,
and cooked output. Dependencies include IDs and schema versions as well as files.

- A native signature/property/type change refreshes reflection and marks affected
  Blueprints, tracks, and bindings stale; deleted or incompatible references become
  recoverable diagnostics rather than being rebound by display name.
- A Blueprint event/property change invalidates callers and tracks that use its
  stable ID.
- A timeline/effect edit invalidates its preview, consumers, generated artifact,
  and only dependent scenes—not every project asset.
- The editor retains the last valid compiled artifact for navigation/preview but
  labels it stale. Cooking/export fails if used assets have unresolved errors.

The initial iteration loop is **safe data refresh plus rebuild/relaunch**. Preview
can rebuild a modified timeline/effect from zero under a fixed simulation clock.
Changing C++ layouts, generated code, native functions, component schemas, or a
running MIPS executable requires a build and Play restart. Do not claim general
native C++ hot reload until a versioned state-reconstruction design is measured and
implemented. A later hot-refresh feature must reconstruct instances from serialized
data, never patch incompatible MIPS object memory in place.

  ## Runtime and performance contract

- Compile each asset into sorted fixed-size tables of curve segments, events,
  markers, bindings, and declared resource dependencies. No runtime JSON parsing,
  string lookup, dynamic allocation, or RTTI.
- Declare and validate project-wide limits for authored timelines, active sequence
  instances, tracks per sequence, keys per track, marker queue entries, transient
  effects, layers per effect, and emitted particles. Start conservatively and set
  final values only after measurements against the existing 2 MB PSX runtime.
- Expose `sequence_stats` and extend effect/particle stats with active/peak/dropped
  counts, skipped invalid targets, event queue overflow, and per-frame work.
- Use the existing fixed simulation clock for particle and timeline advancement by
  default. Camera/HUD adapters must state if they intentionally use frame time.
- Define restoration semantics before implementation: an absolute property track
  either restores its captured initial value on stop or leaves its final value.
  Choose per track/component, serialize it, and apply it consistently on normal
  completion, cancellation, deactivation, and scene replacement.

## Authoring workflow

1. Create a `ParticleEffect` from a preset or create a `TimelineAsset` from the
   Content Browser.
2. For an effect, add up to eight named layers and use its embedded timeline;
   compose core flash, sparks, smoke, ground flash, rune, and residual layers.
3. Add property tracks and short curves; add bursts and named markers such as
   `Cast`, `Launch`, `Impact`, and `Aftermath`.
4. Preview in a fixed-step eight-second loop with camera/background controls and
   a continuously visible cost panel.
5. For a scene sequence, add typed slots, attach `TimelineComponent` to a director
   entity, bind concrete scene UUIDs, and repair any broken required slots.
6. Use Blueprint to start the effect/sequence and react to markers; use the
   timeline inspector to navigate from any marker/track diagnostic to its target.
7. Cook/build. Validation identifies the asset, track, key, reflected property,
   binding, and source location for every error.

## Delivery phases

| Phase | Deliverable | Acceptance criterion |
| --- | --- | --- |
| 0. Prerequisite audit | Verify Blueprint phases 1, 3, and 4; prototype one compiled property track and one marker on real MIPS | Fixed-value output matches host preview and handwritten C++ under the same Q12 ticks; record RAM/code/CPU measurements |
| 1. Asset core | Versioned timeline schema, IDs, bindings, typed tracks/curves/markers, editor validation, undo/redo, and cache/invalidation integration | Save/load/reorder/rename preserves IDs; broken required and optional bindings diagnose correctly; no display-name rebinding |
| 2. Runtime director | Cooked tables, bounded `TimelineComponent`, handle validation, playback lifecycle, adapters, stats, and export/staging integration | A gate and camera sequence plays in emulator; destroy/reuse/deactivate/replace-scene cases never affect the wrong entity or leak a continuation |
| 3. VFX assets | `ParticleEffect`, layers, embedded timeline UI, presets, persistent/transient effect components, and budgets | An authored fireball has cast, travel, impact, and aftermath layers; pool exhaustion is deterministic and observable |
| 4. Blueprint bridge | Playback, effect, marker, completion, and cancellation nodes using the existing latent contract | A combat Blueprint waits for `Impact`, applies damage exactly once, and safely cancels on owner destruction/scene replacement |
| 5. Iteration tooling | Dependency visualization, stale diagnostics, safe asset preview refresh, profiling, docs, and examples | Native/reflection/Blueprint/timeline edits invalidate exactly the affected consumers; C++ edits rebuild/relaunch rather than leaving stale Play state |

Do not begin a phase with unfinished acceptance criteria from the earlier one.

## Tests and validation

- Schema migration, UUID/ID persistence, deterministic serialization, rename
  redirects, and preservation of orphaned/broken data.
- Curve endpoint/interpolation/Q12 tests shared between host and runtime; crossing
  events with ordinary and clamped large `dt`; loop/restart/completion semantics.
- Required/optional binding resolution; hierarchy activation; destruction, slot
  reuse, generation mismatch, scene-bank replacement, and cancellation.
- Track conflict and blend validation; target type/visibility/access validation;
  unsupported property/function diagnostics.
- Particle layer limits, global pool exhaustion, deterministic seed replay,
  preview/runtime parity, rendering budget counters, and effect cleanup.
- Blueprint marker delivery, duplicate-event prevention, continuation capacity,
  pause/deactivation semantics, and independent simultaneous instances.
- Transitive invalidation from headers, generated reflection, Blueprints, timelines,
  effects, scene bindings, staging, export, and moved project folders.
- Clean MIPS build, standalone export rebuild, host unit tests, and relevant
  PCSX-Redux execution. Report unavailable emulator/tool checks honestly.

## Likely implementation map

The Blueprint work may use different final module names; integrate with those names
instead of duplicating registries or asset indexes.

| Area | Likely modules |
| --- | --- |
| Asset/schema/compiler | `src/timeline.rs`, `src/timeline_ir.rs`, `src/timeline_compile.rs`, `src/particle_effect.rs` |
| UI | `src/timeline_editor.rs`, particle inspector/preview extensions, asset browser integration |
| Dependencies | existing reflection/class registry, asset index, fingerprint/build graph, `project`, `pipeline`, `export` |
| Scene persistence | `scene`, migration/versioning, component Inspector, undo/redo |
| PSX runtime | `runtime/timeline.hpp`, `runtime/effects.hpp` or a focused effect module, generated scene/effect setup |
| Blueprint bridge | Blueprint node catalog, IR/backend, bounded continuation runtime |
| Validation | host unit tests, `tests/runtime/` C++ tests, integration/emulator scenarios, documentation examples |

## Risks and decisions to validate

- Measure before setting sequence and effect capacities; the renderer, sound, and
  game already share 2 MB RAM with this feature.
- Determine whether timeline data belongs entirely in generated C++ or can use a
  compact cooked binary staged alongside it, while preserving editor-independent
  export and stable debugging source maps.
- Decide camera semantics early: a camera can be an entity-backed component or a
  scene service, but tracks need one explicit target contract.
- Keep nested timelines and sub-effects out of the initial scope because recursive
  lifetime/budget/cancellation rules make worst-case work difficult to prove.
- Do not migrate every existing `ParticleEmitter` automatically until an importer
  can reproduce current scene appearance and retain a recoverable backup.

The first valuable vertical slice is a scene `TimelineComponent` that animates a
validated transform property, emits one marker, drives a small existing particle
burst, and survives invalid target destruction correctly in preview and PSX. Build
the particle-effect authoring experience only after that shared core is proven.
