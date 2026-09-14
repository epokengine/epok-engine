# Local testing for Epok maintainers

Use debug builds for local development and iteration: `cargo build --locked --bins`.
Development desktop shortcuts must point to `target/debug/epok-editor.exe` on
Windows (or `target/debug/epok-editor` on other hosts). Build release only when
explicitly requested for release validation or distribution.

The reusable timeline component library has a dedicated acceptance command:
`python tests/integration/verify_timeline_adapters.py`. It installs project-owned
adapter source, uses the real Clang extractor and native/Blueprint ancestry,
rejects required missing components, cooks two scene sequences and inspects
actual component values at event time and after restoration in PCSX-Redux.
It also rebuilds the standalone export byte-identically. The retained fixture
and linked section sizes are reported in
`artifacts/timelines/phase2-component-adapters.json`. This validates component
state, not audible output, pixel parity or physical-console frame rate.
The normal Blueprint runtime harness additionally runs
`test_timeline_adapters.cpp` against real PsyQo headers and compiles the library
with the pinned MIPS compiler.

The Blueprint feature integration also includes `BP_AudioOverrides`, derived from
its existing resource-bearing Blueprint. It checks that inherited numeric-default
edits preserve both normal/debug audio banks after source observation and full
staging; inherited AudioClip overrides must invalidate and stage the newly
selected payload while leaving the other destination stale. Restoring source
must rebuild both destinations to their original executable bytes.
The fixture also edits a native numeric default, a scene script value and a
template script value before full MIPS staging; both audio banks must remain
unchanged. Focused Rust tests cover script member IDs/override bookkeeping,
missing-class identity, native metadata failure/repair and independent banks.

Run checks from the repository root. Full validation is performed locally;
GitHub Actions enforces the release branch and version policy.

`make check` runs the editor checks below sequentially and stops on the first failure. Both `make check` and `make test` build the companion reflection extractor before running tests. On Windows after setup, use `.\.tools\mips\bin\make.exe check` if Make is not on PATH. The root Makefile also exposes `test`, `lint`, `fmt-check`, `build` and `release` individually; native emulator checks remain separate.

## Editor checks

Rustup selects the pinned toolchain. These checks do not launch the emulator:

```powershell
cargo fmt --all -- --check
cargo build --locked --bin epok-header-tool
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo build --locked
```

Tests that create Actor projects or stage native classes execute the companion
`epok-header-tool` binary. `cargo test` builds that binary's test harness, not
the standalone extractor. Run the explicit build above in a fresh checkout;
the PR and release workflows perform the same prerequisite before testing.

Unit tests cover scene hierarchy, transforms, export generation, lighting, HUD, picking, transport decoding, asset recovery/import transactions, WAV/MP3/ADPCM quality and XA sector spacing and Dear ImGui interaction without a desktop window. Tests requiring a graphics adapter or real emulator are explicitly ignored by the default run.

Dependency checks include PATH discovery, missing SDK files, configuration persistence,
backup preservation, and real ImGui warning/settings interactions. On Windows,
`python tests/integration/verify_dependency_setup.py` runs offline installer checks
with temporary archives: selected-package isolation, companion-file repair,
checksum rejection, and archive path containment. Capture the settings page with
`--project <folder> --screenshot-dependencies --screenshot <file.png>`.

After SDK setup, `python tools/extract_hud_font.py --check` verifies that the committed editor HUD font matches the pinned PsyQo font. Omit `--check` to regenerate the bitmap when intentionally updating that resource.

## MCP integration

After `cargo build --locked`, run `python tests/integration/mcp.py` to launch an isolated editor project and exercise HTTP/stdio negotiation, scene edits, Undo/Redo, camera changes, real Scene/HUD/editor screenshots and audio import. Add `--emulator` to compile and run the sample in PCSX-Redux, send controller input, Pause/Step, capture Game and Stop. `--editor <path>` selects an alternate binary, including a release build. These checks need a desktop GPU; the emulator option also requires SDK setup. Preferences and captures stay in the test's `.epok/mcp-integration-*/` directory, leaving the user's MCP preference unchanged.

Unit tests also cover disabled defaults, key migration, port conflicts, Host/Origin/authentication rejection, listener shutdown, stale edits, atomic batch failure, file backups and cancelled/expired queued requests. `--screenshot-mcp-settings` opens the AI / MCP preference page for visual checks.

## Native PSX checks

`python tests/integration/verify_memory.py` builds an isolated project using the
saved serial Play profile without opening a serial port or emulator. It checks
current-scene versus whole-game MIPS outputs, shared texture deduplication, BSS
accounting, SPU residency, exclusion of stale cached payloads and unchanged-build
repeatability. Reports and real editor captures of the analyzer and Play menu are
kept under `artifacts/memory/<timestamp>`. `--screenshot-memory` opens an existing
report for visual QA; `--screenshot-play-menu` opens the toolbar menu.

Default tests also verify treemap area/containment, malformed ELF rejection,
symbol alias accounting, complete/fragmented NOTPSXSerial progress records, and
the compact modal's size, continued workspace rendering and keyboard/MCP lock.

`python tests/runtime/verify_frame_clear.py` checks the real PsyQo clear packets
for all ten video modes and both buffer parities. Interlaced rendering clears
with a rectangle at `(0,0)` and preserves the displayed field; progressive
rendering retains the alternating 240-line buffers. This checks packet contents
and VRAM bounds; field timing still needs testing on physical hardware.

On 2026-09-11, the user tested the isolated Ironwood title through Unirom serial
on a Japanese SCPH-7500 with composite AV into an HDMI converter. The 640 x 480i
build still flickered and turned white after the field-clear fix. The adapted
320 x 240p build (`ironwood_menu_240p.exe`, SHA-256
`7712b6649358ef76c13fec659157974e256c9c4f1257ab9df1ed741ead769da1`)
was reported stable, with cropping at the top and bottom. This is evidence for
that title and output chain, not acceptance of all video modes or proof that
the converter caused the interlaced failure. The next comparison moves the
diagnostic counter and help text inside 16-pixel vertical margins while keeping
the same 320 x 240p GPU setup. These isolated title tests omit XA music and other
scene banks. Emulator captures alone do not validate analog output timing.

Safe native iteration is exercised with a real editor state and its existing
build/Play worker (no GPU window is needed):

```powershell
cargo test --locked native_source_edit_restarts_real_play -- --ignored --nocapture --test-threads=1
```

This creates a separate sample project, edits C++ during Play, verifies a new
emulator process and executable, introduces a compiler error, repairs it, and
checks recovery and Stop. It also edits the in-memory scene during Play, verifies a new process/executable
without changing the saved scene, and checks that the lighting bake does not
cause another restart. It then adds a real external C++ include, changes its
contents while preserving its modification time, checks a changed executable
and new emulator process, deletes the include, and repairs it to verify recovery.
It requires the pinned MIPS tools and PCSX-Redux and
must run alone on port 8077. Retain its printed fixture path for diagnosis.
Default unit tests cover stale graph propagation, unrelated timeline cache
preservation, source removal, reflected-member removal, cache provenance
migration/relocation, invalid-source debounce and deferred unsaved-effect builds.
Staging tests additionally cover reflected-member changes reaching generated
headers, independent rebuild/export destinations, failed and removed outputs,
and conflicting dependency snapshots within one generation batch. The
audio-bank regression uses real imported ADPCM and typed event resource cooking:
it checks first/last clip selection, timing/curve isolation, duplicate resource
references, malformed/ambiguous source observation, repair and independent export
certificates. The combat fixture checks that the actual bank header depends on
the embedded timeline's audio selection instead of its complete playback output.
Blueprint audio regressions compile real graphs and resource banks, covering
scalar-logic isolation, inline/literal-node references, first/last clip changes,
template-only audio, malformed sources and independent output certificates.
`verify_blueprint_features.py` also edits graph logic and adds a clip through the
CLI, checks bank staleness before staging, and requires byte-identical normal and
debug executables after restoring the source and rebuilding both destinations.
Build-ticket regressions use actual generated Blueprint artifacts and disk edits
at the before/after-compilation boundary, without an editor watcher. They cover
layout-only changes, changed defaults, additions (including the first Blueprint),
removals, unreadable sources, independent export recovery and mixed scene/compiler
membership snapshots. Older staging manifests without class-set provenance are
rejected until regenerated. The feature integration verifies the captured class
set and the source-manifest dependency in both normal/debug MIPS destinations.
The effect source integration verifies stale signatures after an invalid instance override;
the combat integration checks generated effect provenance in both destinations
against the actual written header bytes. On Windows it launches PCSX-Redux's
installed `.main` directly, so timeout cleanup owns the emulator process.
Scene provenance tests exercise saved versus unsaved inputs, independent exports,
registered-map edit/deletion/parse failure/recovery, document switching and
lighting-cache updates. Combat acceptance verifies the actual generated scene
header hashes and root/additional-map dependencies in both build and export.

Native application input certification has a separate real Make/GCC regression:

```powershell
cargo test --locked native_make_inputs_detect_timestamp_preserving_edits_and_new_include_resolution -- --ignored --nocapture --test-threads=1
```

It builds an owned project with spaces in its path and an external header with a
space and `#` in its name. It verifies target-specific `-O2` dependency selection,
unchanged-build reuse, byte-different MIPS output after a timestamp-preserving
header edit, and rejection/recovery when `__has_include` starts selecting a newly
created file during compilation. Compiler reports must include the SDK archive,
libgcc and linker script. The fixture is retained and does not launch an emulator.
The combat integration checks these actual compiler inputs, including its cooked
effect header, alongside the executable certificate and standalone rebuild.
SDK archive construction has a separate isolated real-tool check:

```powershell
cargo test --locked sdk_archive_tracks_real_assembly_inputs_source_membership_and_failed_rebuilds -- --ignored --nocapture --test-threads=1
```

It creates a miniature SDK using the pinned Make rules and real GCC/assembler/ar.
It checks C/C++ and assembly includes, `.incbin`, preserved-timestamp edits,
unchanged archive reuse, removed archive members, failed-build retention and
recovery, mid-build mutation, new include selection after capture and nonblocking
lock ownership. Pre-existing shared SDK objects and an archive must retain their
exact bytes. All authored mutations remain inside the retained fixture.
Combat and Blueprint feature integration additionally inspect the real SDK
certificate and each build destination's private archive dependency. Standalone
rebuilds use fresh private SDK objects through the exported launchers.

`python tests/integration/verify_particle_effect_load.py` measures one effect,
eight effects with 64 emitters and 256 particles, deliberate effect/burst/emitter
overflow, and cleanup in a scene containing twelve meshes, textured characters
and HUD. It checks actual MIPS output, an identical standalone rebuild and two
serialized PCSX runs. The retained fixture and `artifacts/timelines/phase5-vfx-load.json`
contain completed native frame samples, code/data/BSS sizes and pool sizes.
Capacity assertions are correctness checks; measured frame costs and dropped
steps determine the tested workload's performance. Keep the SDK and emulator
free of other native validation runs while executing it.

Blueprint foundation checks after `cargo build --locked --bins` and SDK setup:

```powershell
cargo test --locked blueprint_creation_dialog_reflects_inherits_attaches_and_undoes -- --ignored --nocapture
python tests/integration/verify_reflection.py --emulator
python tests/integration/verify_projects.py
python tests/integration/verify_project_registration.py
```

The ImGui test creates/attaches inherited native classes through production controls using real Clang. Reflection acceptance rebuilds a standalone export and checks inherited behavior, independent properties and scene-bank resets in emulator RAM; do not run it alongside another Play session on 8077. `--keep` retains its isolated temporary project for diagnosis. Registration tests use a unique HKCU test subtree and do not alter live file associations. Capture the real dialog with `--screenshot-script-dialog --screenshot <path>`; live Windows click-through and double-click launch still need an unlocked desktop.

For runtime features, run the host C++ suites and emulator acceptance checks:

```powershell
python tests/runtime/verify_spatial.py
python tests/runtime/verify_sprites_particles.py
python tests/runtime/test_compare_runtime.py
python tests/runtime/test_stream_pool_layout.py
python tests/integration/verify_rpg.py
python tests/integration/verify_streaming.py
python tests/integration/verify_streaming_xa.py
```

The two host runners cover input, time, collision, transform caching, lifecycle,
Memory Card recovery, utility services, palettes, frustum/polygon processing and
sprite/particle pools. They require a C++20 compiler and the pinned Nugget sources.
Streaming checks use isolated projects on ports 8092 and 8093 and require the
configured CD authoring tools. For diagnostic assertions, benchmark methodology
and regression gates, see the internal
[performance validation workflow](../performance-testing.md).
All native acceptance checks require a built editor, configured MIPS tools and PCSX-Redux;
run them sequentially. The RPG check covers sprite depth/blending,
camera projection, resource counters, palette animation, scene transitions,
object handles, particle capacity/expiry and input/timing behavior. Feature limits
are documented under [Sprites and particles](../../docs/sprites-particles.md),
[Cameras and resources](../../docs/camera-resources.md), [Runtime services](../../docs/runtime-services.md)
and [Input and collision](../../docs/input-collision.md).

Run dependency setup first. Integration scripts use Python 3.10 or newer and its standard library; no Python packages are required. The scripts default to `target/debug/epok-editor.exe`. Close a running editor before rebuilding that executable.

```powershell
cargo run --locked -- --project examples/sample-game --build-psx
python tests/integration/verify_projects.py
python tests/integration/verify_audio_assets.py
python tests/integration/verify_bgm.py
python tests/integration/verify_pipeline.py
python tests/integration/verify_hierarchy.py
python tests/integration/verify_hud.py
python tests/integration/verify_display.py
python tests/integration/verify_lighting.py
python tests/integration/verify_blockout.py
cargo test --locked live_video_input_pause_step_and_cleanup -- --ignored --nocapture
```

Run these **sequentially**, with no other Play session using port 8077.

| Check | Evidence |
| --- | --- |
| Projects | External folders with spaces, creation, version rejection, overwrite protection, relocation, startup scene, cache regeneration and basic/sample builds |
| Audio/assets | Closed-project moves, source/cache loss, UUID conflicts, snapshot reimport, actual SPU decoded samples, loops, stop/play, volume/pitch and 24-voice priority |
| Pipeline | PS-X EXE output, incremental script objects, C++ failure/recovery, animation, pause/resume and cleanup |
| Hierarchy | VRAM comparison of nested and equivalent world transforms, parented cameras and animated parents |
| HUD | Pixel equality for authored and C++-created HUDs; runtime component updates |
| Display | All ten native output modes, default 640 x 480, projection aspect, HUD corners/font and emulator filtering |
| Lighting | Baked/GTE agreement, normals, light budgets, range, shadows, native creation and saved bake caches |
| Blockout | Mesh UUID relocation/conflicts, material slots and instance overrides, window openings, chunk rejection, near-plane clipping, backface rejection and sloped baked/GTE normal agreement measured from native VRAM/RAM |
| Live bridge | Changing video, controller press/release, stable pause, one-VBlank step and process cleanup |

Hierarchy, HUD and lighting accept `--exe <path>` for an alternate editor build. Verification creates isolated temporary game projects under `.epok/`; the pipeline check also builds the sample project. Reports, screenshots and memory captures go to ignored `artifacts/`.

## Blueprint acceptance

Build both editor and reflection extractor with `cargo build --locked --bins`.
The ordinary Rust suite covers schema/IR/compiler, stable identities, linked
templates, typed references, editing transactions, and debug protocol validation.
Additional target and native UI checks are:

```powershell
python tests/runtime/verify_blueprint_runtime.py
python tests/runtime/verify_blueprint_bridge.py
cargo test graph_canvas_creates_connects_drags_and_undoes_using_imgui_events -- --ignored --nocapture
python tests/integration/verify_blueprints.py --keep --emulator
python tests/integration/verify_blueprint_features.py --keep --emulator
python tests/integration/verify_blueprint_performance.py --keep --emulator
```

Serialize emulator tests. The runtime harness runs host contract assertions and
compiles actual pinned MIPS/PsyQo/EASTL headers; the Lua protocol harness mocks
emulator services and is not a real-breakpoint acceptance substitute.
For that acceptance, set `EPOK_BP_DEBUG_PROJECT` to the retained inheritance
fixture's relocated project, then run
`cargo test live_blueprint_breakpoint_snapshot_step_resume_and_cleanup -- --ignored --nocapture`.

The performance fixture compares equivalent arithmetic at 1/16/64 instances
under common `-Os` optimization, verifies native RAM results, and reports linked
sections, the measured function's prologue frame, and guest interpreter cycles.
It does not measure peak stack/RAM, hardware timing, or FPS. Native/debug builds
are kept separate. See `docs/blueprints.md` for reproducible example generation
and `design-qa.md` for actual desktop evidence.

## Screenshots and profiling

TimelineAsset checks (after building both editor/extractor binaries):

```powershell
rtk cargo test --locked timeline
rtk cargo test --locked timeline_controls_add_undo_redo_validate_and_save -- --ignored --nocapture
rtk proxy python tests/runtime/verify_blueprint_runtime.py
rtk proxy python tests/integration/verify_timeline_assets.py
rtk proxy python tests/integration/verify_timeline_prototype.py --emulator
rtk proxy python tests/integration/verify_timeline_director.py --emulator
rtk cargo test --locked particle_effect
rtk cargo test --locked effect_controls -- --ignored --test-threads=1
rtk proxy python tests/integration/verify_particle_effect_assets.py
rtk proxy python tests/integration/verify_particle_effect_runtime.py --emulator
rtk proxy python tests/integration/verify_blueprint_playback.py
rtk proxy python tests/integration/verify_blueprint_playback.py --direct-assets
rtk proxy python tests/integration/verify_blueprint_playback.py --subscribe-markers
rtk proxy python tests/integration/verify_timeline_combat.py
```

The runtime runner includes cooked Q12 curve/marker tests. The asset integration
uses real extraction, rename/permission invalidation, explicit stale retention,
deterministic table timestamps, duplicate IDs, and MIPS syntax checking. The
Phase 0 fixture independently rebuilds both standalone exports and compares
Rust host, cooked MIPS, and handwritten MIPS samples and marker counts. It
measures guest cycles and linked sections under common `-Os`; it is not a
runtime director, effect-pool, or hardware acceptance test. Reports stay under
ignored `artifacts/timelines/`. Run emulator checks sequentially.

The director fixture cooks a gate/camera sequence with native and Blueprint
targets and a particle burst. It rebuilds an identical standalone export and
checks event-time values, restoration, inactive-owner freezing, dynamic template
components, destroyed/reused targets, owner cancellation, and scene replacement
in PCSX-Redux RAM. Its report includes linked sections and bounded pool sizes;
completed-frame simulation scanlines include other systems and are not isolated
director CPU timing or physical-console performance.

The effect-source check uses the shared Clang catalog and embedded timeline
compiler, verifies typed layer property/event identity, scene effect cooking,
MIPS adapter syntax, stable semantic hashes and unchanged particle limits.
The runtime fixture adds immutable sprite/emitter layers, stable reorder/seed
assignment, a full MIPS build and an identical standalone rebuild. Its optional
PCSX probe checks the production tick path, transient saturation, no entity
allocation, local pause, exactly-once burst/marker delivery and bounded drain.
The report is `artifacts/timelines/phase3-runtime.json`. These checks do not
constitute Phase 3 fireball/editor or Blueprint bridge acceptance.

The playback bridge fixture uses generated combat graphs with typed component
Play and marker/completion waits. It asserts one damage application at Impact,
stop cancellation, inactivity, owner destruction, graph reentry and scene
replacement on PCSX-Redux, then verifies a byte-identical standalone rebuild.
`artifacts/timelines/phase4-bridge.json` records MIPS sizes, continuation/frame
storage and completed-frame simulation scanlines. `--direct-assets` removes all
scene playback components and checks typed required/optional UUID binding pins,
asset-only reachability and independent export rebuilding. `--subscribe-markers`
checks repeated Impact delivery, a latent reached branch, stop cancellation and
the unchanged eight-slot continuation table. Their evidence is recorded in
`phase4-direct-assets.json` and `phase4-subscriptions.json`.

The authored combat fixture copies `examples/timeline-spell`, then instruments
its real `BP_Fireball` graph. It verifies captured target identity, one 25-point
Impact hit, completion after particle drain, cancellation before another Impact,
owner destruction, scene replacement and an identical standalone export. It
captures the game HUD and effects for visual inspection; `phase4-combat.json`
records the fixture, counters and MIPS section sizes. The older Fireball visual
fixture explicitly selects the retained presentation scene (`Main.epokmap`),
since the example now starts in `Combat.epokmap`.
The combat check also verifies actual scene, script and executable hashes in the
artifact dependency graph, plus the separate build/export stage manifests. The
`staging_files` unit tests exercise native-body invalidation, retained stale
exports, unchanged writes, abandoned output files, tampered staged inputs and a
build ticket superseded by changed compiler options. The explicit native Play
restart test additionally checks compiler-error recovery and an unsaved scene
edit without rewriting the saved document.

The `certification_rechecks` tests use production scene/resource staging and
build tickets without an editor watcher. They cover saved versus unsaved inputs,
registered scene edits and parse failures, map membership, rendering settings,
import settings, equal-length package checksum corruption, duplicate/missing
packages, independent exports and recovery only after restaging. Unimported raw
source edits and unrelated corrupt packages must not reject an independent build.
The explicit `linked_refresh_precedes` test also exercises Play's prepared-scene
staging entry point: inherited obsolete texture/audio references are removed by
the existing template refresh, while source/audio provenance still matches the
submitted saved scene and the resulting build ticket can be certified.

`legacy_metadata_changes` stages through the production catalog and covers
`.epokscript` edits, membership changes, unreadable files, mixed reads in one
capture, old staging without metadata coverage and independent export recovery.
Run `rtk cargo test --locked reflection_capture_rechecks -- --ignored --test-threads=1`
with the pinned SDK/extractor available. That fixture uses a real external include
and checks extraction/cache-hit provenance, include mutation and removal, captured
tool inputs, SDK selection changes and regeneration after repair. It edits only
its own fixture; it does not modify installed tools or the SDK.

For the dependency viewer, run the isolated interaction check:

```powershell
rtk cargo test --locked dependency_navigation_uses_real_clicks -- --ignored --test-threads=1
```

It uses actual ImGui clicks to navigate a missing input and a transitive consumer,
then checks a corrupt graph read preserves only a labeled diagnostic snapshot
and never writes to the graph. The ordinary unit suite checks shortest consumer
paths through cycles and missing nodes. Capture a built fixture with
`--screenshot-artifact-dependencies --screenshot artifacts/timelines/dependencies.png`;
inspect both columns, full-ID tooltips and stale/error text. The panel is opened
normally through Window > Artifact Dependencies.
The Blueprint compiler regression `selected_class_references_invalidate` covers
ClassRef defaults, inline literals and literal nodes through actual compilation,
dependency observation, reparenting rejection and repair; unrelated Blueprints
retain their records. The Blueprint feature integration also checks separate
release/debug executable hashes and stage manifests, including the patched
debugger `sources.mk` signature and its compiler-option dependency.
It also verifies actual audio payload/display header hashes and their precise
provider inputs. The unit test `staged_audio_payloads_follow` uses full project
staging, two imported clips, native/scene edits, snapshot reimport, a superseded
asset index, missing clips and removal from the stage manifest. It verifies old
exports stay stale and saved scene bytes remain unchanged. The display provenance
test checks real project settings, geometry-only changes, display regeneration
and malformed settings without certifying an old header on repair.
The audio test also checks actual `audio-bank.hh` hashes and source-selection
dependencies. Native body edits, scene transforms and AudioSource volume/pitch
leave its record unchanged; selected-package reimport or a changed clip assignment
stales it. Saved-scene selection changes affect its export, while the open editor
scene has independent inputs. Repairing the saved source does not certify its old
bank; regeneration is required. The normal/debug Blueprint and streaming/XA
integrations verify these bank records against the actual generated headers.
The Blueprint audio-selection regression also stages real banks after numeric
variable edits, null/changed/removed AudioClip declarations and editor-flag
changes. It checks separate `audio-catalog` provenance and independent export
staleness. The catalog regression checks inherited null declarations, declaration
order and failed-catalog repair. `verify_blueprint_features.py --emulator --keep`
checks numeric declaration/default isolation and changed clip defaults through
the production CLI before rebuilding normal/debug MIPS outputs and running PCSX.
Template audio tests also stage real banks after spatial/construction/playback
edits and explicit AudioSource removal. Linked-instance checks apply and refresh
actual overrides, compare effective clip selection, and preserve unknown member
data on failure. Both projections use the existing component applicator.
The `required_scene_assets_ignore_unrelated_errors` test runs the production
timeline/effect preparation paths with unreadable unrelated files, required
source corruption, duplicate effects and cross-family UUID collisions. It also
checks a missing required timeline leaves an independent preview unchanged.
`malformed_effects_and_uuid_collisions` covers source observation, retained stale
preview data and recovery; the editor watcher regression checks that other edits
remain observable while an unchanged catalog error persists. The authored combat
integration includes unrelated malformed files during MIPS build and standalone
export, then corrupts its used effect and requires build rejection and stale
stage provenance. `playback_watch_uses_selected_stage_edges` checks independent
normal/debug targets, unchanged errors, semantic rename stability, removal/repair
and changes published by a preview before the next watcher poll. The playback
picker test checks retained choices, ambiguous-ID exclusion, unchanged authored
nodes and canvas revalidation without repeated requests for an unchanged error.
`playback_changes_during_staging_or_compilation` changes actual source files at
build-ticket boundaries and before export publication, requiring stale outputs
to be rejected while unrelated malformed files remain harmless.
The explicit real Play restart test now adds malformed unused timeline/effect
files and requires an unchanged live emulator process. It corrupts a used
timeline, verifies Play stops and the replacement fails, then repairs it and
requires a fresh emulator with the original executable before exercising C++
edit/error/recovery and unsaved scene restart.
It also creates all seven editor presets through `--preset` and cooks their
ordinary timeline/emitter/sprite data in one scene. The explicit ImGui effect
test clicks Add Fire layer, Undo, Redo and Save and checks layer/slot/track IDs,
whole-document restoration, embedded serialization and external-write rejection.
`--open-effect` plus `--screenshot` captures the effect editor; inspect the image
for reflection diagnostics as well as layout errors.

For the asset window, use `--project examples/timeline-spell --open-timeline
assets/Timelines/Cast.timeline.json --screenshot artifacts/timelines/editor.png`.
The ImGui interaction test owns a context and is explicitly serialized.

```powershell
cargo run --locked -- --screenshot artifacts/projects-hub.png
cargo run --locked -- --project examples/sample-game --screenshot artifacts/editor.png
cargo run --locked -- --project examples/sample-game --screenshot-game --screenshot artifacts/editor-game.png
cargo run --locked -- --project examples/sample-game --profile-editor
cargo run --locked -- --project examples/sample-game --profile-scene
cargo run --locked -- --project examples/sample-game --profile-scene-cpu
```

These commands need a desktop graphics adapter. Game screenshots also need the emulator. Captures are taken from application/emulator rendering. Inspect them before copying selected product images to `docs/images/`; keep machine paths and logs out of published screenshots.

Add `--window-size 1024x720` (or `1280x720`, `1440x900`) to reproduce compact and wide layouts. Check Scene/Game toolbars, Inspector fields, asset columns, settings footers and floating tools at each size. `--screenshot-hud`, `--screenshot-lighting`, `--screenshot-imports`, `--screenshot-import-dialog`, `--screenshot-project-settings` and `--screenshot-editor-preferences` open their respective views. Audio captures need an imported AudioClip in the project.

The ImGui interaction check includes a real modal regression: it verifies that normalizing reordered draw indices for imgui-wgpu 0.25 preserves each command's geometry, preventing modal dimming from covering dialog controls.

Timing depends on the scene, machine and build mode. GPU preview timings do not measure native PSX game performance.

For a project containing an EditableMesh, add `--screenshot-blockout --screenshot artifacts/blockout.png` to open and capture its Blockout panel. Unit tests also exercise primitive winding, welded corners, ramp/triangle extrusion and subdivision, validation, stale-write protection, shared editing, extraction Undo/Redo, same-asset baked shadow occlusion and real ImGui editing buttons.

Blockout keyboard tests cover E/Q extrusion, inward displacement, edge picking, Q bevel and navigation shortcut suppression. Topology tests verify watertight bevel/ramp generation and atomic rejection of unsupported cuts. Camera tests check in-place free look, movement directions and frame-rate-independent flight; mouse capture also needs desktop verification on the target window system.

## Native runtime profiling

For Timeline/VFX relocation and standalone portability on Windows:

```powershell
rtk proxy python tests/integration/verify_timeline_relocation.py --emulator
```

The verifier builds a copy of `examples/timeline-spell`, copies its caches into
a second project and copies its export into a separate folder. It checks the
resolved fixture paths before renaming its own original folder so the old path
is unavailable. Both rebuilds must produce the original executable byte-for-byte;
authored files and UUIDs must remain unchanged. It also verifies v2 relative
script-manifest paths, fresh build provenance, portable generated/debug sources
and exported guides. `--emulator` adds a Play startup/stop smoke test from the
copied project; it is not a new RAM, visual or performance measurement. All
fixture folders are retained, and `artifacts/timelines/phase5-relocation.json`
records the outcome. The host provider unit test covers v1 regeneration,
equivalent manifests in different roots and rejection before partial writes.

The editor now links `native/effect_preview.cpp` with a host C++20 compiler and
the pinned Nugget headers. Initialize the SDK with setup before Cargo builds;
Windows uses the existing Visual Studio C++ prerequisite. Only host callback
wrapper/display stubs differ; production timeline/effect/particle Q12 kernels
are compiled directly. macOS host preview compilation still needs verification.

`python tests/integration/verify_particle_effect_preview.py` compares 96 fixed
steps of the editor's C ABI trace with cooked MIPS execution in PCSX. It checks
all live particle state hashes, lifecycle/diagnostic counters, sprite values,
atlas regions, world matrices and deterministic replay. The probe deliberately
executes a bounded trace in one update; do not use its frame duration as a
performance measurement. The report is written to
`artifacts/timelines/phase3-preview-parity.json`. The regular effect runtime
harness remains responsible for lifecycle, capacity and runtime timing checks.

`cargo test --locked particle_effect_preview -- --test-threads=1` exercises the
host bridge ABI, pause, resolver ownership, replay and broken binding behavior.
Add `--fireball` to `verify_particle_effect_preview.py` for all 350 steps of the
seven-layer textured example. `verify_timeline_fireball.py` independently builds
and boots a copy of the example, captures each spell stage, checks marker
revisions and cleanup, records completed-frame simulation costs, and rebuilds
the standalone export byte-identically. It writes `phase3-fireball.json` and
`fireball-stage-*.png` under `artifacts/timelines`. The effect runtime harness
also checks per-instance typed overrides and unchanged unmodified plays;
the asset harness rejects orphaned/type-mismatched overrides without rewriting
the scene. Retiming tests preserve all IDs and reject collapsed curve keys.
For manual visual checks, open an effect with `--open-effect <source-path>` and
capture it with the normal `--screenshot` option. Host GPU pixels are not a PSX
rasterization reference, even though simulation and cooked sprite values match.

The included profiler defaults to `examples/rpg-2-5d-demo`. It uses that project's optional `Local.epokconfig`, otherwise the engine's local or default tool configuration. Build the editor first and keep the selected project and emulator port free during each run.

```powershell
cargo build --locked
python tools/profile_runtime.py --output artifacts/performance/demo
python tools/profile_runtime.py --output artifacts/performance/demo-detail --detail
python tools/profile_runtime.py --output artifacts/performance/demo-validation --validate-gte
python tools/compare_vram.py artifacts/performance/demo/vram.bin artifacts/performance/demo-validation/vram.bin --png artifacts/performance/diff.png
```

Run these commands from the engine checkout. `--project` selects a different game project; `--optimization Os` or `--optimization O2` compares runtime compilation options. Detail and validation timers add work, so do not interpret their frame times as release performance. The helper invalidates the instrumented runtime object after a run so the next normal build can rebuild it.

The profiler generates `profile.json`, `vram.bin` and logs in the requested output directory. VRAM comparisons require matching scene state and display settings; animated effects can differ between captures. See [Performance](../../docs/performance.md) for counter semantics and measurement limits.

`python -m unittest discover -s tools -p test_profile_runtime.py` checks linked
counter layouts, unavailable services, active gauges, capture deltas, resets and
saturation. Use `--project examples/timeline-spell` for a native spell capture;
the profiler's `playback` section reports sequences, effects and particles along
with frame costs. Startup work remains visible in the first/last cumulative
values even when the sampled frames start after the spell has completed.

## Clean-checkout verification

`python tests/integration/verify_splash.py` captures the real 640 × 400 project
splash, a restored 1024 × 720 editor and the Hub after a failed open. It checks
window proportions at the monitor's DPI and keeps captures under `artifacts/splash`.
Manually check opening from a maximized Hub and on a secondary monitor as well.

Before publishing structural or dependency changes, validate a separate checkout or source snapshot containing only publishable files. Run setup with an empty `.tools/`, initialize the pinned SDK without nested recursion, and build with a fresh Cargo target directory.

Compile the sample PS-X EXE with no pre-existing SDK objects. Check the standalone C++ export and its README/license files. Finish with the runtime integration checks. Cargo's downloaded dependency cache may be reused, but previous compiled project/SDK outputs must not be needed.

After validation, verify that generated files remain ignored and Nugget source remains clean.

`verify_bgm.py` imports a generated stereo MP3 as SFX and XA, removes the source/cache and moves the package while closed, then builds/boots BIN/CUE. It DMA-reads the emulated SPU CD capture buffers and checks stereo decoding alongside a sample voice, loops, stop/restart/retrigger, music priority, switching and mono one-shot completion. Evidence is saved as `artifacts/bgm-*`. It needs the complete setup tools and port 8077. Physical-console XA timing/audio remains unverified.


Skeletal validation: `cargo test --locked skeletal_tests` compares quantized poses with the FBX evaluator and checks reimport consistency. `python tests/integration/verify_skeletal.py` compiles for MIPS, compares native poses, verifies changing GPU output in PCSX-Redux and captures the UI.

Python validation and migration tools require `python -m pip install -r tools/requirements.txt`. See [document formats](../../docs/formats.md) for YAML and the UniQo-to-Epok migration.
