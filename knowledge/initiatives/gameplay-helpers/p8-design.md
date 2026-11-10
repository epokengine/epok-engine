# Complete-game migration preparation

Status: P7D is validated. The bundled template now uses Character3D, its inherited
motor/collider, the shared query context and a project camera derived from
ThirdPersonCamera. Its 3,420-word logical camera/state comparison, economy circuit
and cold reload pass. Required checks pass 1,311 workspace tests / 88 ignored,
formatting, Clippy and debug builds. Overall P8 acceptance remains open, including
the existing 25ms failure, runtime peaks/Lua headroom and remaining complete-game
decomposition. The receipts below retain each earlier prototype's own scope.

The initial bundled complete game has 64 actors, at most five components on
one actor, 12 UI actors and seven actors containing only a SceneComponent3D and
AudioComponent. The inventory and scene hash are recorded in
`artifacts/gameplay-helpers/p8/composition-inventory.json`.

## First capacity change

Six of those sound-only actors exist solely to hold independent sound sources.
The seventh is the existing Reactor Station root, which also holds music.
AudioComponent already supports multiple independent instances on one actor,
including local source storage, reflected playback controls and lifetime hooks.

The six sound components now live on Reactor Station, retaining their component
UUIDs and authored audio properties. This leaves 58 actors and exactly
eight components on the root. Keep the terminal's separate interface source and
the root's music source, preserving all seven sound channels plus music.

DungeonGame uses explicit component references for these sound sources and music.
It resolves a reference against the live registry when using it; it does not
retain a raw AudioSource pointer across destruction or scene transitions. The
existing common AudioComponent API remains the mechanism for later Blueprint/Lua
rules. No new native audio facade is introduced.

The six removed actors have no scene references except their identity and empty
lighting-cache entries; their logical/spatial parent was already Reactor Station.
Scene comparison permits only those source moves and eight explicit bindings.
All 67 resource packages remain byte-identical. The prototype's full economy /
PocketStation circuit passes, including duplicate receipts and cold save reload.
Guard and boss routes execute with music, zero missing sounds, zero dropped
triangles and successful victory, but both still fail the existing 25ms gate.

Same-editor measurements:

| Measurement | Untouched baseline | Sound composition |
| --- | ---: | ---: |
| Authored actors | 64 | 58 |
| Linked static RAM | 1,974,360 | 1,951,752 |
| Unassigned RAM | 122,792 | 145,400 |
| EXE bytes | 1,150,976 | 1,142,784 |
| Guard maximum frame microseconds | 27,456 | 27,264 |
| Guard slow frames | 13 | 9 |

The measured capacity improvement is 22,608 RAM bytes and six actor slots.
The frame-time samples do not satisfy performance acceptance; preserve that
failure while addressing shared mechanisms. Evidence and preserved setup failures
are under `artifacts/gameplay-helpers/p8/sound-capacity/` and `sound-baseline/`.

## Following work

The shared query component passes nine real provider executions, a byte-identical
source-mode export and zero unused RAM/EXE cost after repairing optional retention.
Its receipt is `artifacts/gameplay-helpers/p8/query-context-validation.json`.
The disposable Complete integration passes the guard/boss functional scenarios
and the full economy circuit, including cold reload. Both routes still fail
25ms at 27,648us. Its 67 packages and other scene content match the sound prototype.
That prototype receipt predates the bundled integration recorded below.

The character kernel's internal query adapter passes host regressions. A second
disposable Complete prototype uses that kernel while retaining game-owned facing,
attack/animation rules and the original stationary-query policy. It passes the
same console functional scenarios and economy circuit, including cold reload;
the performance gate still fails at 27,648us. Another 1,440 host steps match the
old Complete motor exactly, including 524 idle query skips. Its receipt is
`artifacts/gameplay-helpers/p8/character-complete-recheck/validation.json`.
That controller receipt predates the bundled adoption recorded below.

SpringArm now has a shared-context solve operation and optional adaptive close
clearance. The context owns one bounded recent-ray cache, which re-queries on
consumer/input/revision changes and invalidates on typed publications, movement
and reset. Camera instances do not each carry that cache. Host checks pass the
original 720-frame camera reference and shared-cache identity/lifecycle cases.
The three-provider, three-mode console matrix and byte-identical standalone rebuild
pass. Unused Basic remains at 983,160 RAM / 436,224 EXE bytes with no context pool.
The receipt is `artifacts/gameplay-helpers/p8/shared-camera-validation.json`.

The project camera prototype derives from ThirdPersonCamera, keeps its logical
parent and detaches from the authored identity spatial root to use world poses.
Its initial runtime load failed: source actors without a linked composition did
not acquire native-default identities for existing Camera/SpringArm components.
The duplicate component caused a transactional bank rollback. Captured counters
prove one registry rejection and zero arena rejections. An isolated UI subclass
reproduces the same missing hydration. The common class refresh regression now
passes, including preservation of authored control identity/values, the saved
source and a valid bake. Root-only actors remain unchanged. Loading and Native PC
preparation also refresh unlinked source actors. The fresh
`camera-complete-fixed` project contains the same authoring inputs; its generated
camera records now use native indices 0/1/2. The full workspace passes 1,311 tests /
88 ignored, including PocketStation, and Clippy passes after a test-only style
repair. Console loading reports 58 actors / 150 components and zero registry
rejections. Guard/boss functional scenarios and the economy/PocketStation circuit
with cold reload pass. Both routes still fail the unchanged 25ms frame gate at
28,032us with 13 slow frames. All 67 resource packages are byte-identical.
The initial lobby framing was visually inspected; matched camera trajectories
are recorded below. Every-view visual acceptance remains required. Receipts are
`artifacts/gameplay-helpers/p8/native-default-refresh-validation.json` and
`artifacts/gameplay-helpers/p8/camera-complete-fixed/validation.json`.
That camera receipt predates the bundled adoption recorded below.

Optional native HUD diagnostics now separate change detection, layout and packet
emission. They run only under the existing detailed profiling flag. An initial
capture exposed partially replaced live records; the diagnostic publisher now
clears the draw identity around replacement and publishes it last. Collectors
omit zero identities. The recheck validates 476 complete phase records. Its peak
HUD invocation spends 10 / 7 / 82 scanlines on those three phases, 102 in total;
that invocation is not asserted to be the frame-time peak. The captured
exploration frame peak spends only 10 scanlines in the enclosing HUD phase.
No additional layout cache is justified by these measurements.

The matching ordinary executable remains byte-identical to the pre-instrumentation
Complete prototype and contains no profiling symbols. All 67 resource packages
remain byte-identical. Its route still fails the unchanged frame gate at 28,032us
(15 slow frames in this repeat), while combat, audio and streaming checks pass.
The instrumented route also fails the gate; its timings describe that diagnostic
build. Three native HUD/text/widget fixtures, 15 profiler-tool tests, API
regeneration and debug binaries pass. Evidence is
`artifacts/gameplay-helpers/p8/hud-phase-recheck/validation.json`.

A deterministic fixture now compares the actual old and common-camera Complete
controllers under the same logical input sequence. All 3,420 Q12/state words
match across 180 steps: 90 normal, 55 lock-on and 35 free aim. The sequence covers
orbit, simultaneous L1/R1, release, reacquisition, movement and jumping. The first
matching run opened a nearby terminal on Cross instead of jumping; the preserved
coverage note records that gap. A fresh fixture starts outside interaction range
and verifies a real 7,210-Q12 jump rise in both cases. All 67 resource packages
remain byte-identical, and the original 127 / 129 authored files are unchanged.
Evidence is `artifacts/gameplay-helpers/p8/camera-mode-trace-jump/validation.json`.

The matched fixtures use 1,991,128 / 2,018,672 linked RAM bytes, including two
test actors, an observer and a 13,680-byte trace buffer. These are not production
RAM totals. The common-camera delta is 27,544 bytes; the receipt records the
largest linked symbol changes for the next capacity review. This behavior
acceptance does not certify every rendered view, the failed 25ms gate or runtime
heap/stack peaks. Bundled adoption was still open at that fixture milestone;
the current source adoption is recorded below. Overall P8 remains open.

A MIPS call-boundary experiment was rejected. Forcing the checked SpringArm solve
and camera refresh out of line yields no linked-RAM savings against the matching
PCDrv image: both use 1,990,344 bytes. An initial comparison mistakenly used the
later CD build's 1,988,168 bytes; the receipt now records that correction and the
matching captured heap boundaries. The guard peak remains 28,032us and the boss
route reaches 29,440us. Both attributes were removed and the
accepted camera header was restored byte-for-byte. Further candidate tests were
stopped; no economy or provider acceptance is claimed for that experiment.
Evidence is `artifacts/gameplay-helpers/p8/camera-call-boundary-complete/rejection.json`.

An optional debugger observation now captures the complete previous CPU profile
at each native frame callback above 25ms. It leaves the executable and gate
unchanged. The original guard binary reports 13 slow intervals: five early
intervals have roughly 11ms outside measured frame work; combat intervals have
substantial CPU work. Automatic pad-transition serial prints occur outside the
measured work. Native pad-transition prints now require the explicit
`EPOK_PAD_TRACE=1` build option, tracked by native build provenance. The off/on
comparison preserves all 67 resource packages and the geometry archive; enabling
the trace recreates the original executable byte-for-byte. Disabling it removes
the five early slow intervals and recovers 2,048 PCDrv RAM bytes. Input and motor
negotiation remain enabled. The off guard/boss routes still fail at 28,928us /
27,648us with four / ten slow frames. These are not overall performance passes.
The change passes 1,311 workspace tests / 88 ignored and Clippy. Evidence is
`artifacts/gameplay-helpers/p8/pad-trace-complete/validation.json`. The earlier
camera receipt's RAM value came from its later CD build; its memory-scope note
distinguishes that report from the captured PCDrv executable.

A focused native test reproduces unnecessary text packet rebuilding when a
preceding rectangle border appears. The cache now compares the packet/budget
groups a node uses, preserving strict text-budget and dropped-work checks.
No cache storage or buffer capacity is added. Host command comparisons and shared
budget regressions pass, as do text reveal, dialogue widgets and the serial
kernel. API generation and debug binaries pass. Console functional routes and
the full economy/PocketStation/cold-reload circuit pass, including restoration
of the disposable test cards. Serial startup ordering passes all four required
reads. Basic remains at 983,160 RAM / 436,224 EXE bytes with no unused query pool.
The Complete PCDrv image remains at 1,988,296 linked RAM bytes, and all 67 resource
packages match. Evidence is
`artifacts/gameplay-helpers/p8/hud-independent-pools-complete/validation.json`.

The guard/boss routes still fail the unchanged frame gate at 28,736us / 25,920us
with eight / six slow frames. Their setup teleports are still host-asynchronous;
the observed changes are not a controlled claim of aggregate CPU improvement.
Those historical asynchronous samples are superseded for repeatability by an
optional deterministic harness mode. It applies setup at the native frame entry
before physics, updates the virtual pad at native polling and stops after the
exact completed frame 1,800. All eight frozen before/after guard/boss runs have
identical replay, final counters and slow-frame records within each repeated
case. Guard slow intervals change from ten to nine with the same 27,776us peak;
boss intervals change from five to four, with peaks 27,328us / 25,664us. All runs
still fail the original 25ms gate. Active-play work is grouped by simulation-step
count, but different binaries can execute different tick trajectories, so those
means do not establish an identical-combat-work speedup. Evidence is
`artifacts/gameplay-helpers/p8/deterministic-routes/comparison.json` and the
adjacent repeatability receipt. No overall P8 acceptance or bundled
controller/camera adoption is claimed.

Native CPU sampling now retains the native-only MIDI clock. Timer0 sampling
starts after sequence initialization; the music clock continues on Timer2.
The old 21-bit PC mask could alias BIOS code into RAM bins; that capture is
superseded by the corrected 29-bit physical mask. The validated 301..1,440
window collects 11,191 samples: 9,746 unambiguously attributed, 1,445 overlapping
function boundaries and zero outside. At least 1,833 samples (16.4%) belong to
the two retained-interior triangle emitters. Inlined work is attributed to its
containing linked function, and deferred interrupts limit conclusions about ISR
cost. The analysis tool preserves ambiguous bins instead of splitting their
counts; two boundary/alias/accounting regressions and the existing 15 profiler
tests pass.

Combat, music, audio, streaming and texture checks pass in that diagnostic build;
the 25ms gate still fails with 14 slow frames and a 27,136us peak. Sampling uses
2,023,112 linked RAM bytes, 34,816 above the normal image. Normal RAM remains
1,988,296 bytes with no sampling symbols. The normal EXE has one differing byte:
the source-location line for the sequence-start failure changes from 1,016 to
1,018; every other loaded byte matches. All 67 packages and the geometry archive
match. Evidence is
`artifacts/gameplay-helpers/p8/native-music-sampler-physical/validation.json`.
The first candidate selected from this profile used GTE NCLIP for retained
triangle orientation. The checked target route compares 809,659 areas against
the CPU calculation with zero differences. The unchecked candidate retains
1,988,296 RAM bytes, but its guard/boss peaks are 27,968us / 26,112us, worse than
the frozen baseline's 27,776us / 25,664us. Guard active-play one-/two-step groups
have the same frame counts; mean CPU work changes only from 245.50 / 330.84 to
245.35 / 330.34 scanlines. This does not justify changing the kernel. Both routes
still fail the unchanged gate. The experiment and its temporary counter were
removed, restoring runtime main.cpp byte-for-byte to the validated sampler
source. No economy, hardware or all-view acceptance was attempted for rejected
code. Evidence is `artifacts/gameplay-helpers/p8/retained-nclip/rejection.json`.

After query integration acceptance, migrate locomotion and camera through the P5
mechanisms while preserving the project's normal, lock-on and free-aim policies.
Split project run flow, combat, encounters, inventory/economy and terminal rules
at their observable state boundaries. Generic native operations may own bounded
queries, movement, transport and serialization; project decisions remain editable
in the selected language.

Keep the existing save schema and save-before-receipt-acknowledgement ordering.
Complete-game performance, runtime heap/stack peaks, every failure path and all
three authored implementations remain explicit P8-P10 gates in the main plan.

## Validated kernel adoption

The bundled scene and five project source files now use the validated shared
motor kernel, query context and inherited camera. A fresh public-CLI C++ project
produces executable `8f533159...` byte-for-byte, with 58 actors, 150 runtime
components and three free player component slots. Camera and query bindings are
editable typed references. The source camera keeps normal/lock-on/free-aim rules
in the project, derives from ThirdPersonCamera and uses its shared SpringArm.

Both route replays, final counters and slow-frame records match the frozen
prototype exactly. The full economy circuit passes 16 events, including cold
reload and duplicate receipt handling, and disposable cards are restored. All
67 resource packages match. Linked PCDrv RAM is 1,988,296 bytes. The workspace
passes 1,311 tests / 88 ignored, with formatting, Clippy and debug builds.
Evidence is `artifacts/gameplay-helpers/p8/bundled-foundation/validation.json`.
The current-runtime pre-adoption source build uses 1,949,704 linked RAM bytes;
the shared foundation adds 38,592 RAM bytes and 30,720 EXE bytes. The geometry
archive is byte-identical. This is a source comparison under matching PCDrv
transport and compiler options, rather than a comparison with historical CD
reports. Its 108,856 unassigned bytes do not establish runtime peaks or Lua
headroom. Evidence is `bundled-foundation/adoption-footprint.json` under the same
P8 artifact directory, with symbol changes in `adoption-symbol-deltas.json`.
The 25ms gate still fails at 27,776us / 25,664us; source adoption does not close
that gate, runtime peaks, all-view acceptance or remaining P8/P9/P10 work.

## Reflected motor boundary

The preceding bundled C++ controller shared `character3d::advance`, but its native
query adapter and direct state access could not be authored through the reflected
motor component. The new generic operations are shared-context simulation with a caller-supplied
support revision, explicit facing assignment, velocity clearing that preserves
contact state, and an option to omit speed measurement when the project owns
animation. A zero support revision always queries; cached support must validate
the body pose, motor/owner/context identities, tuning and context-reset epoch.
Invalid/stale/cross-level contexts must not move the owner or consume a pending
jump. Keep ordinary simulate/stop/reset behavior and defaults compatible.

The component passes the existing 1,440-step Complete motor oracle,
including idle query skips, jumping, changed support, external movement/rotation,
context reset/disable/destruction/reuse and multiple consumers. A rebinding
regression first reproduced stale support reuse, then passed after body pointer,
generation, owner, scale and parent were included in the cache key. Both the
failure reproduction and final passing identity log are preserved. Nine actual
provider executions pass across C++, Blueprint and all Lua modes, with identical
motion hashes; VM-source standalone rebuilds byte-identically. Final evidence:
`artifacts/gameplay-helpers/p8/motor-shared-api/providers-final/` and
`identity-host.log` in its parent directory.

The current template delta replaces Player Aqua's Actor3D with Character3D,
adopts its inherited Collider/Motor, and binds visual/camera through the native
typed properties. The project controller owns a last-motion sample rather than
motor state or a query cache. Motor tuning lives on the scene component. The
controller simulates manually and disables speed measurement for the existing
animation policy. Resetting a run now resets the query component's epoch as well
as its batch. The existing optional project camera override remains compatible.

Fresh Complete camera/economy/resource acceptance and full regression checks
pass in `artifacts/gameplay-helpers/p8/character3d-adoption/validation.json`.
The public CLI generates/exports the new composition with 58 actors, 151
components, two player extension slots and zero registry rejections. All 3,420
camera/state trace words match across 180 logical steps. The original full trace
fixture exceeded the required 64 KiB runtime reserve; a 60-frame ring now captures
all rows without lost-frame tolerance. Its baseline is word-identical to the
earlier full-buffer capture, and the new instrumented image uses 2,030,168 linked
RAM bytes. These are fixture costs, not the game's production footprint.

Nine ordinary-motor provider regressions also pass, alongside the nine shared
API executions above. Unused Basic retains exactly its prior RAM/EXE footprint
without either helper pool. The full 16-event economy circuit, duplicate receipt,
cold save reload and original-card restoration pass. All 67 resource packages
and the geometry archive match. Required checks pass 1,311 workspace tests /
88 ignored, formatting, Clippy, debug builds, API freshness and parity.

The matched source comparison uses the same current runtime/editor and PCDrv
settings: the previous kernel composition uses 1,988,296 linked RAM bytes /
1,171,456 EXE bytes; Character3D adoption uses 2,011,016 / 1,189,888, a delta of
22,720 RAM and 18,432 EXE bytes. Its remaining 86,136 bytes do not establish
runtime peaks or Lua headroom. The 25ms gate still fails: guards have 11 slow
frames with a 26,752us peak, boss has 5 with a 25,792us peak. Functional combat,
music, audio and streaming checks pass. The component boundary is accepted;
remaining game-rule decomposition and overall P8 performance acceptance stay open.

## Optional ancestor creation capacity

Class cooking now distinguishes descriptors needed for
inheritance from classes that need a creation factory. A helper retained only
as an ancestor keeps its identity, parent, domain and callback/default metadata,
without a separate pool. Explicit source demands, placed classes, composed
native controls and ClassRef selections keep creation capacity. Existing core
classes retain their creation contract and four spare allocations.

The first regression reproduces an unused CameraRig3D factory beneath a project
ThirdPersonCamera subclass. The 14 helper regressions pass, including inherited
SpringArm/sensor factories and restoring a direct ancestor factory. Scene and
default ClassRef selections, nested value selections, Lua's statically named
class builtins and C++ UUID constants retain creation capacity. Actor/asset
reference UUIDs and comment-only constants do not become class demands. A late
regression exposed that the native dependency scanner strips string literals;
UUID selection now uses its existing comment-only filter instead. Preliminary
attempts and the failed regression are retained separately from final acceptance.

Final evidence is
`artifacts/gameplay-helpers/p8/metadata-factories/final-uuid/validation.json`.
Nine actual PSX provider executions pass. They exercise inherited casts,
controls and callbacks without an ancestor factory, plus real deferred creation
of another native camera identified only by a UUID constant. All provider
records agree; the VM-source standalone rebuild is byte-identical. The initial
UUID fixture incorrectly expected an immediate callback spawn result; the final
fixture observes the queued actor on a later simulation step.

The fresh Complete scene retains 58 actors / 151 components and zero registry
rejections. All 3,420 camera/state trace words match the old cook's frozen route.
Its 67 packages, geometry, main source and compiler values match. Linked PCDrv
RAM falls from 2,011,016 to 2,007,256 bytes (-3,760); EXE falls from 1,189,888 to
1,187,840 (-2,048). Object arena usage falls from 50,704 to 49,088 bytes. The
remaining 89,896 bytes do not establish runtime peaks or accepted Lua headroom.
Unused Basic retains its prior RAM/EXE footprint with zero delta.

Complete's standalone rebuild also matches byte-for-byte, compared against an
ordinary Build with the same default disc profile (`HOST_DATA=0`,
`SERIAL_DEBUG=0`). The first comparison used the PCDrv package (both fields 1),
which is a different program; its difference analysis is preserved. The PCDrv
memory/route receipt remains separate from this disc rebuild proof.

Guard and boss routes pass functional combat, audio, music and streaming checks.
A guard capture exposed a one-frame host-observation race: RAM was read before
the route's control request observed the final pause. The harness now takes one
more RAM snapshot after seeing that pause; both final replays match their FPS
accounting exactly, with no tolerance added. The original failed log is retained.
Performance remains unaccepted: guards have 12 slow frames / 26,752us peak, boss
5 / 25,792us. All 16 economy events, duplicate receipts, cold reload and restoration
of the original cards pass. Required checks pass 1,313 workspace tests / 88 ignored,
formatting, Clippy, debug build, API freshness and parity.

Four temporary SDK object directories identified by this turn's standalone build
logs were removed after all native jobs finished. Verified paths contain only
object files and no links. The cleanup recovered 21,254,592 file bytes; paths are
listed in `metadata-factories/sdk-temporary-cleanup.json`. Exported sources,
archives, executables and validation evidence are retained. Overall P8 and the
remaining P9/P10 requirements stay open.

## Run-flow policy extraction

The bundled complete game's intro, defeat/restart, victory and boss-entry rules
now live in a project-owned `DungeonRunFlow` component on Follow Camera.
`DungeonGame` binds the common contract by a typed component reference and
applies its output to audio, screen fade and run reset. The player retains six
components and two extension slots. The old plain policy is preserved only as a
test oracle outside the template.

The shared project base defines data and dispatch, including a typed snapshot
write. It contains no transition policy. The C++ class, ordinary Blueprint class
(12 graphs / 198 nodes) and Lua script each implement the rules independently.
Blueprint and Lua fixtures exclude the C++ policy. The ports remain subsystem
work: project creation still requires C++ until the rest of Complete is migrated.

Project records can opt into declarative property defaults using
`EPOK_VALUE(Configuration, ...)`; no game-specific type is added to the SDK
allowlist. The regression rejects unmarked records, omitted field defaults and
runtime handles, and checks Fixed/Bool/UInt32 values including UINT32_MAX.
The generic Math API now exposes positive Fixed-to-UInt32 truncation, available
through the existing operation catalog in every provider.

Current scoped evidence is under `artifacts/gameplay-helpers/p8/run-flow`.
The host oracle passes 9,600 exact legacy Q12 samples and 254 automatic resets.
Actual PSX C++, Blueprint and Lua Native C++/bytecode/source runs each pass 600
steps, with all 6,000 snapshot words identical across the five cases. Every run
observes five manual resets, seven automatic resets, ten accepted gates and
thirty invalid-delta checks. Formatting, Clippy, API freshness and the five API
inventory tool tests pass. Standalone and Complete regression validation are
recorded below; this section does not accept overall P8 performance or the
remaining complete-game migration.

All three policy standalone exports now rebuild byte-identically. The first
instrumented Complete Play build failed the unchanged 64 KiB runtime reserve.
Outlining its typed run-flow lookup and controls query recovered 6,272 bytes.
Build and Play have different linked instrumentation costs; the final trace ring
uses 32 rows instead of 60, retaining all 180 frames / 3,420 words with strict
lost-row detection. The final migrated Play passes with 2,031,184 static bytes
and 65,968 bytes unassigned. All 3,420 camera/control words match the frozen
reference. These figures include the probe, not the normal game's footprint or
measured runtime peaks. Encounter, economy and Complete export acceptance follow.

The initial default-concurrency workspace run passed 1,264 editor tests and failed
one FBX inbox worker test with a Windows access-denied error. That test passes
isolated; the complete workspace retry passes **1,314 tests / 88 ignored** with
four test threads. Both Complete template tests also pass after the final
outlining change. The first failure log is retained. No assertion or deadline
was weakened.

The normal Complete PCDrv package uses 2,012,112 static RAM bytes and a 1,191,936
byte EXE: +4,856 RAM / +4,096 EXE against the preceding factory-pruning snapshot.
Its 85,040 unassigned bytes are not measured heap/stack peaks or accepted Lua
headroom. All 67 asset packages and the geometry archive remain byte-identical.
The Complete standalone rebuild matches its disc-profile reference byte for
byte. Unused Basic remains 983,160 RAM / 436,224 EXE bytes, with zero size delta.

Guard and boss routes reach their final functional assertions with zero missing
sounds, dropped triangles or skipped/failed streaming chunks. The 25ms gate
fails and regresses against the prior recorded executable: guards have 18 slow
frames / 27,776us peak (previously 12 / 26,752us); boss has 7 / 27,520us
(previously 5 / 25,792us). This regression remains required P8 work. The typed
run-flow lookup is a candidate for profiling and reducing repeated ancestry
checks; a code-level hypothesis is not yet a measured attribution.

Fresh PocketStation installation and all 16 events of the separate full
transfer/shop/cold-reload economy circuit pass, including duplicate rejection;
both runs restore the original cards. The PCDrv footprint above is recovered
from the exact route package's ELF heap-start symbol and EXE, because the economy
runner subsequently stages a default disc build in the project's build folder.
Those profiles are recorded separately in the validation receipt. Four isolated
standalone SDK object directories were checked for containment, object-only
contents and absence of links, then removed with no build active. Cleanup
recovered 21,254,592 file bytes; paths are in `sdk-temporary-cleanup.json`.

## Run-flow access and retained HUD investigation

The next performance pass is in `artifacts/gameplay-helpers/p8/flow-performance`.
`DungeonRunFlowBinding` now reuses a successful type check while validating the
registry, handle generation and live storage on every access. Its host regression
covers replacement, destruction, deferred release, reused slots, unavailable
registries and identical handle/address values in a different registry with a
different authoritative type. The 9,600-sample legacy rule oracle still passes.

| Normal PCDrv trial | Guard slow frames / peak | Boss slow frames / peak | Static RAM / EXE |
| --- | --- | --- | --- |
| Previous extracted flow | 18 / 27,776us | 7 / 27,520us | 2,012,112 / 1,191,936 |
| Validated binding cache | 19 / 27,008us | 5 / 25,856us | 2,012,112 / 1,191,936 |
| Binding and HUD key repair | 12 / 27,584us | 4 / 26,496us | 2,012,112 / 1,191,936 |
| Rejected anchor arithmetic fast paths | 12 / 27,648us | 4 / 26,496us | 2,014,160 / 1,193,984 |

Every trial retains functional checks and the original 25ms gate. Completed-frame
records show that maxima can move between different simulation-step/HUD-update
combinations; a lower slow-frame count does not prove the gate is satisfied.
The anchor trial passes 100,000 exact Q12 reference layouts but produces no
measured speedup and costs 2 KiB. It is removed from production; its source,
oracle and measurements remain in the trial evidence.

The HUD investigation also reproduced a correctness gap: the retained PSX packet
key omitted `Text.visible_glyphs`, so an otherwise unchanged text control could
keep the previous reveal state. The runtime and host tests now share the actual
node-key function. It includes reveal state and skips fields of disabled layout
and focus controls that emission does not read. Host tests compare fresh and
cached UTF-8/markup packets through both parities, including increasing and
decreasing glyph counts. An actual PSX probe passes eight states over 48 frames
with zero glyph drops. No Text storage or widget hierarchy is added.

The workspace suite passes 1,314 tests with 88 ignored using four test threads.
That suite started with the subsequently rejected anchor trial present; final
restored-arithmetic flow/key/layout/reveal host regressions also pass. Final
PSX reveal and five flow-provider checks pass, as do their standalone exports
and the 3,420-word camera comparison. Basic retains 983,160 static RAM / 436,224
EXE bytes, with zero delta. All 16 economy events, cold reload and a separate
two-event fresh Pocket installation pass, with original cards restored. Final
workspace build, Clippy with warnings denied, formatting, API freshness and parity
pass. The frozen production hashes still match the accepted fixtures. Evidence is
`artifacts/gameplay-helpers/p8/flow-performance/validation.json`. P8, the 25ms
target, full-scenario runtime peaks, Complete Lua headroom and the remaining
P9/P10 scope stay open.

### External runtime memory observer

`tools/profile_runtime_memory.py` and `tools/runtime_memory_probe.lua` inspect
the unchanged ELF and the pinned allocator from outside the console. The Complete
verifier's opt-in `--capture-memory --deterministic-route` records allocator
entry/return snapshots, linked instructions that write SP, and a canary placed
in unused RAM at executable entry. Heap measurements distinguish live block
bytes, fragmentation/free-list footprint and the SDK's old maximum. This matters
because the pinned allocator's in-place `realloc` path does not update that
maximum. A separate optional Lua-arena record avoids counting its static BSS as
PsyQo heap.

Fourteen Python/pinned-LuaJIT tests pass, including a transient allocation freed
between frames, in-place growth, corrupt free lists, a variable stack frame and
deeper canary writes. The observer's HTTP endpoint only returns retained data:
it cannot sample an allocator interrupted halfway through mutation. A paused
final RAM image is checked independently against the Lua heap snapshot. No
target code, BSS or build flags are added; diagnostic callbacks cost host time
and cannot supply a passing frame-time acceptance result. Canary coverage shows
touched RAM, not maximum reserved stack depth in unlinked BIOS/generated code.
The initial large-executable attempt exhausted LuaJIT callback slots. The pinned
emulator's native FFI breakpoint adapter now reuses one callback across all
1,157 linked SP-write sites, composing multiple watches at the same address.
A 1,600-site regression passes. Actual BIOS callbacks also exercise linked
functions on a separate low-RAM stack near 0x83ec; these observations are retained
separately rather than included in the application heap/stack gap. Both initial
diagnostic failures remain in the evidence.

Both unchanged normal-executable routes finish 1,800 completed frames and pass
their functional checks. Their independent Python final-heap reconstruction
matches Lua exactly:

| Guard and boss observations | Bytes |
| --- | ---: |
| Live heap peak, including allocated-block headers | 1,024 |
| Heap high-water footprint, including free-list headers | 1,032 |
| Main-stack peak from linked SP writes/canary, including top reserve | 24,560 |
| Conservative gap after combining independent peaks | 59,448 |

The linked stack low-water mark is 2,072,592; the lowest changed canary word is
2,072,608. Each route performs 1,784 canary scans and 2,011 coherent allocator
samples. Their ELF heap start is 2,012,112 and their EXE SHA-256 remains
`e670404eee1601e6a40301e4a27311a8ce41a798c0e52f19fd1706e1c1c2dca2`,
identical to the preceding normal HUD-key performance trial. No target code,
BSS or build flags are added. These finite observations exclude terminal/save/
installation peaks and unobserved BIOS/generated reservations; they do not
accept Complete Lua capacity or the separate frame-time gate. Machine-readable
captures are in `artifacts/gameplay-helpers/p8/flow-performance/memory/routes.json`.

The canonical debug cache was reviewed at 24,552,800,774 bytes with more than
95 GB free; no extra Cargo cache was created. Four object-only standalone SDK
caches created by this pass were verified to be inside the workspace with no
links, then removed after native builds finished. This recovered 21,254,592 file
bytes. The complete paths are retained in `sdk-temporary-cleanup.json` alongside
the validation receipt; sources, archives, executables and evidence remain.

## Callback dispatch CPU investigation

The current runtime supports an opt-in bounded histogram window through
`EPOK_PROFILE_SAMPLE_BASE`, `EPOK_PROFILE_SAMPLE_SHIFT` and
`EPOK_PROFILE_SAMPLE_BINS`. Defaults retain the previous 32 KiB / 128-byte
histogram. A window covering physical 0x70000..0x90000 uses 2,048 64-byte bins
(8 KiB histogram) and builds at 2,022,352 static RAM bytes, retaining the ordinary
65,536-byte reserve check. The Complete verifier reads the linked layout and
cross-checks its bin count against the ELF buffer size instead of assuming 8,192.

The current guard diagnostic collects 11,274 samples over frames 301..1440:
2,493 unambiguously attributed inside the chosen window, five ambiguous and
8,776 explicitly outside. GameScene::frame contains 2,464 attributed samples;
the hottest 64-byte bin has 306 samples and candidate source locations in the
Level tick loop and registry-slot validation. Exact ELF/EXE pairs and every
aligned instruction's candidate source locations are retained. Bin counts are
not divided between source locations, and these diagnostic timings do not pass
normal performance acceptance.

| Normal PCDrv trial | Guard slow frames / peak | Boss slow frames / peak | Static RAM / EXE |
| --- | --- | --- | --- |
| Previous binding/HUD repair | 12 / 27,584us | 4 / 26,496us | 2,012,112 / 1,191,936 |
| Rejected outlined O3 dispatch | 13 / 27,072us | 5 / 25,920us | 2,018,256 / 1,198,080 |
| Rejected outlined O2 dispatch | 13 / 27,200us | 4 / 26,176us | 2,014,160 / 1,193,984 |
| Generation-safe broad callback filter | 7 / 26,944us | 4 / 25,984us | 2,012,112 / 1,191,936 |

The two outlining variants added 6 KiB / 2 KiB without reaching 25ms and are
removed. The retained change only bounds the slot index and tests the callback
mask before calling the existing dispatcher. It never dereferences an object or
authorizes a callback: the dispatcher retains the full generation, family,
lifetime and activation validation. It reads current callback masks on every
candidate, preserving same-step component attachment and stage order. No cache,
new field or runtime pool is added. Host tests inject generation-zero,
wrong-generation and out-of-range actor-list entries and verify one tick/frame
callback per live actor; lifecycle, activation, deferred destruction and component
dispatch suites also pass.

Formatting, Clippy, API generation/freshness inventory and 19 Python/pinned-LuaJIT
tool tests pass. The catalog retains 145 headers / 2,671 callables and parity retains
1,050 public / 6,441 total rows with no pending classifications. The four-thread
workspace suite passes 1,314 tests / 88 ignored in 1,031.79 seconds. Final console
reveal, its byte-identical export, five provider cases and three provider exports
pass. Camera acceptance also passes against both rebuilt projects and the previous
SDK's frozen capture. Basic and Complete build; all 16 economy events, cold reload
and a separate two-event fresh install pass. Original cards were byte-compared and
restored. The camera comparison retains all 3,420 words against the previous SDK.
External guard/boss memory observations of the exact accepted executable match
the preceding 1,024-byte live heap / 1,032-byte footprint, 24,560-byte main stack
and 59,448-byte conservative gap. A Python heap walk independently agrees.
Evidence is in `artifacts/gameplay-helpers/p8/frame-hotpaths/validation.json` and
`memory/routes.json`; 505 source hashes pin that pre-health SDK receipt.
Four verified SDK object caches were removed, recovering 21,254,592 file bytes;
their paths are in `sdk-temporary-cleanup.json`. The unchanged 25ms gate, full-scenario memory,
Complete Lua capacity and the remaining P8-P10 work remain open.

## Health mechanism integration

The next combat mechanism is an allocation-free HealthState value and HealthLibrary.
An encounter can retain this value while its render actor is inactive, so the
mechanism does not require an actor or component per encounter. Project combat
components can own the value and all eligibility, feedback and reward decisions.
There is no health registry, pool or polling callback.

The actual SDK passes 134,798 host comparisons against independent wide arithmetic,
including the game's 60/100/280 limits, negative inputs, full uint32 boundaries,
invalid state rejection, one-time depletion and revival. The library has five
stable reflected operations and two bounded records. Independently authored C++,
Blueprint and Lua policy fixtures execute on the actual console runtime;
the common fixture base only copies and records their outputs. It contains no
eligibility rule or arithmetic. All five execution cases and the following
export/unused/game-adoption acceptance now pass.

Initial C++, Blueprint and Lua Native executions each match the independent
180-word oracle. VM bytecode linking exposed a generic inheritance defect:
unused generated super-dispatch cases referenced a pure virtual parent method.
The binding generator now examines the nearest parent declaration and emits a
qualified case only when it has an implementation. Virtual self-dispatch remains.
Lua also rejects an explicit super call to an abstract method before codegen.
Two focused regressions pass, covering scalar/wide signatures and an abstract
declaration hiding an older concrete ancestor. Original failing bindings/logs
remain in `health/abstract-parent-*`.

The bundled C++ source now owns HealthState records for player, boss and six
encounters, and uses the shared operations for damage, healing and reset. Damage
eligibility, invulnerability, phase thresholds, effects, rewards and HUD policy
remain project-owned. Acceptance tools read the published health telemetry, not
private class offsets. Frozen-source acceptance passes all five cases (180 exact
oracle words each), three provider exports and the Complete standalone rebuild.
Basic retains 983,160 RAM / 436,224 EXE bytes, a zero size delta. Complete passes
the guard/boss functional routes, all 16 economy events, cold reload, two separate
fresh-install events and all 3,420 camera words against the frozen prior SDK.
All 67 packages and the geometry archive remain byte-identical. Six independent
card comparisons verify both test runs restored their originals or retained absence.

The workspace passes 1,316 tests / 88 ignored across four suites in 1,147.68 seconds.
Clippy with warnings denied, formatting, whitespace, API freshness and five parity
tool tests pass. The API has 146 headers / 2,676 callables; parity has 1,062 public /
6,456 total rows and no pending classifications. The timed PCDrv executable uses
2,014,320 static RAM / 1,193,984 EXE bytes (+2,208 / +2,048 from the pre-health
callback receipt). The 25ms gate remains failed: guards six slow frames / 26,496us,
boss four / 25,984us. These finite scenarios do not accept full runtime peaks or
the complete Lua variants.

The timed budget is independently reconstructed from the exact packaged ELF/map,
including section gaps and heap alignment. Economy Play rebuilt a different,
non-debug executable-data profile; its 2,012,080-byte report is retained separately
and is not attributed to the timed PCDrv executable. Four verified temporary SDK
object caches were removed, recovering 21,254,592 file bytes; paths are in
`health/sdk-temporary-cleanup.json`. The closed receipt is `health/validation.json`
with `health/source-snapshot.json`. P8-P10 remain open.

## Player vitality policy extraction

DungeonPlayerVitals now owns player health, hit immunity and healing eligibility.
Its C++, Blueprint and Lua implementations are project rules over a native
data/dispatch contract, using the shared health library. Five actual PSX cases
match all 6,720 oracle words; 41,204 host checkpoints match original-game
health/cooldown behavior. C++ adopts a typed component binding and the existing
run-flow cache now shares the generic DungeonRulesBinding implementation.

The fresh game passes guard/boss functionality, all 16 economy events/cold reload,
a separate two-event fresh install, resource/geometry identity and standalone
rebuild. Three provider exports reproduce both their standalone image and actual
native playback image. Basic has zero size delta. The original camera fixture
exceeded only its instrumentation memory gate; the compact fixture preserves all
180 steps / 3,420 words and the 32-row ring, matching the previous frozen capture
for both versions. Its migrated static RAM is 2,024,440 bytes. Normal full combat
acceptance uses the production scene with all guards and resources unchanged.

The workspace passes 1,316 tests / 88 ignored; Clippy, formatting and freshness
checks pass. Normal timed PCDrv RAM/EXE are 2,017,000 / 1,196,032 bytes, a 2,680 /
2,048-byte increase. Guards fail 25ms at 25,984us (seven frames), boss at 27,648us
(seven frames). Runtime peaks and Complete Lua capacity remain unaccepted.
Four verified temporary SDK caches were removed, recovering 21,254,592 file bytes.
See [the full delivery](p8-player-vitals.md) and `player-vitals/validation.json`.
