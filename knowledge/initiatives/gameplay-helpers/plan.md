# Gameplay base and game helpers implementation plan

## Execution order revised 6 November 2026

### Scope correction: 18 required combinations

The user's 6 November correction supersedes every earlier 30-case/five-mode
template requirement in this plan and its historical receipts. Required delivery
is **six templates × three authoring flavors = 18 combinations**: C++, Blueprint,
and Lua using **Native C++ AOT only**. Every required combination retains its
functional, semantic, RAM, performance, Play and standalone-export requirements.
Lua contains the actual editable game rules; generated C++ is solely a compiler
artifact and must not become an authored fallback or a second implementation.

VM bytecode/source remain supported engine features with their own regression
coverage. Their existing evidence and failures are preserved separately. Do not
schedule more Dungeon Complete VM builds, capacity/arena experiments or template
acceptance runs for this delivery. Such results do not block the 18-case release.
Normal workspace/compiler regressions continue to cover VM functionality where
applicable. No other scope, budget or threshold is reduced.

### Final matrix status (9 November)

The last full matrix passes eight of nine stages for all 18 combinations: generation,
source, reflection, build, native Play, PSX Play, export and resources. The scenario
stage fails for all 18, and all of it is still-open debt:

- **Dungeon Complete closure route, original versus export (C++ and Blueprint).**
  The 14:35 run (`cases/dungeon-complete--{cpp,blueprint}--native_cpp/08060a7f9f0d0598`)
  failed with "equal-step route differs". That run used the comparator as it was
  before `step_keyed` and the 90% overlap rule (`dungeon_routes.py`, edited 15:59).
  Comparing its saved evidence again with the current `routes_match` gives a match for
  all three flavors, and no common step differs:

  | Flavor | Steps (original / export) | Common | Different |
  | --- | ---: | ---: | ---: |
  | C++ | 11,016 / 11,016 | 11,003 | 0 |
  | Blueprint | 11,013 / 11,013 | 11,005 | 0 |
  | Lua | 11,010 / 11,010 | 11,003 | 0 |

  The difference came from emulated-CD frame timing, not from gameplay.

  **Confirmed on 10 November.** The run used the `--contracts` and `--adapters`
  arguments from the main session. Before it, the relocated exports in
  `%TEMP%\epok-gameplay-final18` were reconciled: they were backed up to
  `_backup-20261109-224808`, sources were copied from the new export, and the build
  cache was kept.

  All three Complete flavors now pass 8 of 9 stages (generation through export,
  plus resources). In the scenario stage, the guards, boss and closure routes ran
  from disc, and the original and exported runs matched. The only failure left is
  the missing `terminal/install/status/transfer/ACK` evidence.
- **Next step for the interrupted ACK, paused on 10 November at the weekly usage
  limit.** The route has not been written yet. The runtime and policy already cover
  the case:
  - `DungeonEconomyPolicy.hpp`: after the save, `ack_started` puts the transaction
    in phase 4. `ack_sent` then moves it to phase 5, or returns error 23 on failure.
    Phases 5 and 6 poll the receipt until the claim arrives (output 24) or 4 s pass
    (output 23). `ack_pending` survives a failure, so a retry does not credit ore
    twice.
  - `pocketstation_async.hpp` retries unanswered requests and times out after 5 s.

  Plan: add `--interrupted-ack` to `verify_dungeon_complete_economy.py`. After the
  `ore save` wait at line 165, while `economy()[13]` (the operation) is 6 or 7, take
  the slot-2 card away or block it. Check error 23, then that ore stays 40 and is
  not credited twice. Restore the card, retry action 5, and check output 24. Find
  out first how the Redux Lua API removes or reinserts a memory card. Then call it
  from `complete_economy` in `dungeon_routes.py` and replace the fixed `False` at
  line 302.
- **Dungeon Complete, all flavors.** The required scenario
  `terminal/install/status/transfer/ACK` has no interrupted-ACK evidence from the
  PocketStation transfer.
- **The other five templates.** Their authored-edit, baseline and coverage evidence
  has not been recorded. Dungeon also fails its 25 ms gate.

### Size and CPU recovery checkpoint (7 November, handover continuation)

Dungeon Complete C++ links, exports and plays again at normal runtime `-O2`.
Measured against P0 from linker maps and identical deterministic pad schedules;
evidence is under `artifacts/gameplay-helpers/p10/size-recovery/`.

Why the working game stopped fitting (+313,968 bytes over P0):

| Cause | Delta | Kind | Resolution |
| --- | ---: | --- | --- |
| Header-defined rule classes compile in `main.o` under runtime `-O2`; `EPOK_SCRIPT_OPT` never reached them | +145 KB | accidental | scene header wraps script includes; Make passes `EPOK_SCRIPT_SIZE_OPT` |
| `project.rs` merged each prefab's unauthored `hud_budget` default into the game | +66.7 KB | cook bug | prefabs contribute measured `hud::usage`; authored 352/40/12/8/32 survives |
| Helpers linked by the game | +68 KB | real cost | retained |
| Object arena sized as the sum of every class admission bound | +25 KB | accidental | authored `spawn_reserve` sizes it for live peak |
| Prefabs kept full `ActorData` in `.data`, and a placed-only HUD kept a prototype | ~27 KB | accidental | `PrototypeSlots` builds spawns in place; `PlacedOnly` class flag |
| `objects[actors+32]` above the 64-actor level bound (already in P0) | ~47 KB | inherited | scene `spawn_reserve: 6` |

The cook fixes alone took normal O2 from -159,288 to +49,208 free bytes. Default
behaviour of other templates is unchanged: `spawn_reserve` and the size profile
are opt-in, and the sorted class lookup only applies to cooked tables.

Functional bug found on the way, now fixed: `prototype_N::initialize_components()`
ran the host scene's startup at the first spawn, clearing the visibility track,
overwriting lighting and fog, resetting navigation and repointing localization.
HEAD emits this too; Complete only reached it once modals became prefabs.

Why it ran slowly (identical pad schedule, guards route, scanlines per frame):

| Measure | P0 | Global Os | Now |
| --- | ---: | ---: | ---: |
| simulation | 40 | 128 | 56 |
| scripts | 18 | 55 | 31 |
| world | 12 | 38 | 8 to 11 |
| render | 132 | 171 | about 130 |
| `update_enemies` in a slow two-step frame | 13 | | 19 (was 59) |

Three separate causes, none of them inherent to the helper architecture:
1. `gameplay_world_affine` ran a full transform sync per helper query.
   `sync_path` already existed; `world_scanlines` fell from 36 to about 10.
2. `find_object_class` was a linear scan, repeated at every ancestry step, over a
   table the cook keeps sorted. It is now a binary search for cooked tables.
3. Rule classes spelled shared pure expressions at every use: 21 repeated
   `world_motion_goal` chains in one tick function. `tools/dungeon_player_controls.py`
   gains `hoist()`, which names work only where it is very busy, so a guarded
   division cannot move above its guard. All 67 C++ rule methods re-emitted
   byte-identically from the DSL before the change. 6 unit tests pass.

Frame results, guards / boss, identical deterministic route to 1,800 frames:

| Build | Free RAM | Slow frames | Combat max us |
| --- | ---: | ---: | ---: |
| P0 (993/988 rendered frames only) | 154,680 | 10 / 8 | 27,008 / 25,280 |
| Global Os, previous attempt | 131,272 | 1,307 / 405 | 50,048 / 48,064 |
| Cook fixes, rules Os, runtime O2 | 153,400 | 182 / 75 | 30,912 / 28,608 |
| + hoisted rules | 159,544 | 102 / 59 | 28,992 / 28,608 |
| + four per-tick files at O2 | 116,536 | 62 / 38 | 27,968 / 28,416 |

P0 ran about half as many rendered frames, so its counts are not directly
comparable: slow frames in the matching window are the fair measure.
Not closed: the 25ms gate still fails, as it did at P0. The global maximum of
about 30,000us is one startup frame (frame 84), as P0's 27,328us was.

Rejected with evidence: global Os; hot `.cpp` O2 alone (49 KB for 182 to 143);
a class lookup cache (superseded by the binary search).

User decision, 7 November: Dungeon Complete is held to parity with P0 rather than
to zero slow frames. The table is in [the contract](contract.md); every other
template keeps its gate. Reaching it needs about 33 scanlines on the guard route's
worst frames and 17 on the boss route's, plus 42 at startup.

A sampled profile of the P0 executable was abandoned: the instrumented rebuild
stayed on the loading screen, so its samples are invalid and are not used as a
baseline. `frame-trace/` under `size-recovery` replaces it with every frame's
PerformanceStats from the unmodified P0 package. The repeated `refresh_collisions`
suspicion was also dropped: `world_syncs` grew, but collision is about 5 scanlines
of a slow frame and the motor's unbatched ground query runs only on a ledge drop.

The P0 executable already carries the same subsystem timers, so both could be traced
on identical routes. Render was already better than P0. All of the regression was in
scripts, and an all-O2 build gained only about 7 scanlines on a two-step frame, so the
cause was structure, not optimization level. What was removed:

- Work repeated on catch-up steps. `epok::time.drawn_step` is false on a step the
  renderer never shows. Camera, aim pose, playback, lock marker, enemy tint, effects,
  visibility and pickups wait for the drawn step and receive the accumulated time.
- Results recomputed with unchanged inputs: resting doors, the HUD snapshot, stick
  shaping, an unmoved camera, and a projectile team with no active shot.
- Runtime scans: `Level::tick` walked 64 actors per tick group, `before_tick` captured
  a history the next step overwrote, and `ControllerPad` paid an ACK timeout on each
  empty multitap address every frame.
- Interface work moved to once per rendered frame, and interaction checks distance
  before availability and reads an unparented actor's transform directly.
- The HUD change detector multiplied for every field of every node. It now uses a
  rotate and xor.
- Actor teardown was quadratic. Closing the card modal scanned the level twice per
  actor. Leaves and unfocusable nodes now skip those scans, which took the startup
  spike from 30,144us to about 25,400us.

| Provider | Slow, guards / boss, before | After | Free RAM after |
| --- | ---: | ---: | ---: |
| C++ | 62 / 38 | 9 / 7 | 114,344 |
| Lua | 100 / 64 | 11 / 11 | 104,096 |
| Blueprint | 116 / 66 | 21 / 9 | 91,808 |

The user accepted this as technical debt; see [the contract](contract.md). Evidence
is under `size-recovery/candidate-13-*` and `size-recovery/frame-trace/`.

Validation after these changes: 1,354 workspace tests pass (88 ignored), Clippy and
formatting are clean, the 86 gameplay-helper Python tests pass, all six combat and
economy rule traces keep their baseline hashes, and the real economy circuit passes
16 events in each flavor. The runner now reports Complete's frame time without
failing on it, and ignores the editor's disc image (`epok.bin`) when it compares a
standalone rebuild, which never produces one.

Matrix on engine `a66f56fcb48430da`: generation, source, reflection, build, Native
Play, PSX Play, export and resources pass 18/18. Scenario passes 0/18:

- Basic, Sample, Third Person and Platformer still lack their authored-edit,
  baseline and coverage evidence, as before.
- Complete's scenario packages `.epok/build`, but by then the export stage has
  rebuilt it with the `--export-psx` image. That image is not the build-stage image,
  and under the PCdrv harness it stops after `runtime ready`. The adapter must run
  the build-stage snapshot and give the exported route its own transport.
- Dungeon fails its unchanged 25ms gate. Render matches P0 exactly (243 scanlines
  idle), but scripts went from 16 to 43. A sample shows the generic helpers repeat
  `refresh_collisions` through unbatched motor and camera queries (about 9 scanlines
  a frame), plus `is_active` (7) and the character command lease (4).

Complete scenario, 9 November. The adapter now runs the certified image from disc
twice per route: the original from the editor's disc, the export from a disc that
mkpsxiso authors in the relocated folder. Emulated CD reads move frame boundaries by
a few hundred microseconds, so the two routes are compared by simulation step, not
by frame, without the fields that follow frame duration. Every test run is muted
(`EPOK_MUTE_AUDIO`, plus the SPU mute in each Redux script); a muted and an audible
guards route gave identical frames and counters.

Matrix on engine `08060a7f9f0d0598` (all tests muted): every stage except scenario
passes 18/18. Complete Lua passed every route and failed only at the first missing
requirement. Complete C++ and Blueprint failed in the closure comparison, and the
cause was the adapter: the step count starts again at each restart, so a key of step
alone mixed rows from different runs, and a one-step frame shift changed which row
was compared. The key is now restart plus step, and keeps every distinct state a
step shows; the old key had silently dropped about 970 closure rows. On matrix-7's
own evidence the new comparison accepts all nine original/export pairs.

The adapter also required a MIDI loop crossing in every route. The 1,800-frame guard
and boss routes end before the first boundary, while the driver itself only asks
routes of 4,000 frames or more to cross one. The crossing is now required in the
closure, from both discs; closure crosses two in every flavor.

Direct adapter run on matrix-7's certified bindings
(`size-recovery/complete-direct-9/`):

| Complete | Routes original/export | Economy + fresh install | Slow frames | Max µs |
| --- | --- | --- | ---: | ---: |
| C++ | guards, boss, closure equal | pass | 73 | 27,968 |
| Blueprint | guards, boss, closure equal | pass | 115 | 29,696 |
| Lua (matrix-7) | guards, boss, closure equal | pass | 137 | 28,480 |

Frame time is reported, not blocking, as the contract records. Still missing for the
Complete scenario, in the runner's order: the interrupted ACK/retry/failure route;
chosen-language guard, shop and run-flow edit proofs; heap, stack, VRAM, HUD and
composition peaks (the texture VRAM count alone is not total VRAM); an equal-step
baseline comparison; peak coverage; and the exported economy circuit.

Both harness changes move the engine hash, so every ledger row reads `missing` until
the next full `--all` run; remove `%TEMP%/epok-gameplay-final18` first. Matrix-7 rows
remain in `historical_rows` and under their `08060a7f9f0d0598` case folders.

Later on 7 November all three providers reached the same point. Blueprint gained a
compiler pass, `hoist_pure_calls`, in `src/blueprint_ir.rs`, which applies the same
very-busy rule to pure functions and helps any user Blueprint. Lua regenerated from
the DSL after both sources matched it byte for byte. Speed regions use
`EPOK_SPEED_BEGIN`/`EPOK_SPEED_END`; the size profile is the project setting
`build.script_optimization: size`.

| Complete provider | Previous link | Free RAM now | Slow guards / boss | Boss defeated |
| --- | ---: | ---: | ---: | --- |
| C++ | -159,288 | 116,536 | 62 / 38 | yes |
| Lua AOT | -214,584 | 106,288 | 100 / 64 | yes |
| Blueprint | -302,648 | 94,000 | 116 / 66 | yes |

Behaviour proof, none of it inferred from a green build:
- Combat 53,040 words: C++, Blueprint and Lua hash `5c3e266d...`, byte-identical to
  the trace recorded before these changes.
- Economy/UI 77,056 words: all three hash `f1b15c16...`, likewise identical.
- Real circuit in the whole game, all three providers: 16 checkpoints including
  terminal, missing card, transfer, duplicate rejection, shop, purchases,
  insufficient funds, medkit and cold reload. HUD peak 214 of 352 glyphs, 5 of 12
  texts, zero dropped. No spawn rejected.
- Workspace 1,354 passed, 0 failed, 88 ignored. Clippy clean with warnings denied.
  API reference and parity inventory regenerated. 7 rule and launcher tests added.

`verify_dungeon_complete_economy.py` could not have passed since stations moved to
`InteractorComponent`: it injected into tick edge queues, while stations read frame
edges that only a physical pad produces. It now presses Redux's virtual pad through
`EPOK_AUTOMATION_PAD`, a bridge mask, loaded with `EPOK_PLAY_LUA`. Neither reaches
the game. The product was not at fault.

480i hypothesis refuted. `third_party/nugget/psyqo/src/gpu.cpp` carries the
field-sync proposal as an uncommitted submodule change, applied at 01:01 after the
last capture, so it had never been tested. Real capture
`p10/camera-views-480i-fieldsync/`: the even field stays without geometry in all
eight raw snapshots, and is now entirely black (153,600 against 153,358), so the
clear reaches it while triangles do not. The patch is preserved, not adopted. Whether
this predates the initiative is still unproven; P5 hashes match but P5 is not P0.

Final 18 run, editor `d2da8c27...`, engine `76bae8357d6f32b7`:

| Stage | Passed |
| --- | ---: |
| generation, source, reflection | 18/18 |
| build, resources (2 MiB and 64 KiB reserve) | 18/18 |
| Native Play, PSX Play | 18/18 |
| standalone export | 14/18 |
| scenario | 0/18 |

Not one row is accepted; `matrix.json` still reads `passed_rows: 0`, and that is
correct. Why each open stage is open, so nobody reads it as 18 broken templates:

- Export: `basic/cpp` hit a relocated folder left by an interrupted run. The three
  Complete rows failed a comparison with the `--analyze-memory` image, which uses
  another transport for a streamed project. Their relocated rebuilds are
  byte-identical to the `--export-psx` image: `p10/size-recovery/complete-export-identity.json`.
  The runner now compares with the export image. That fix has not run: changing the
  runner changes the engine hash, so it lands with the next full run. Remove
  `%TEMP%/epok-gameplay-final18` first, because relocated markers carry the old hash.
- Scenario, Basic/Sample/Third Person/Platformer: every observer ran and reports real
  metrics, but stays `missing` by design until three evidences are connected:
  chosen-language edit plus helper workflow, a matched baseline route, and reviewed
  peak coverage. Third Person and Platformer also need the capture reference.
- Scenario, Dungeon: `failed`, not missing. Key and door pass, but its route reports
  51 slow frames and 44,586us. All three flavors agree within 150us, so rules are not
  the cause. Whether this predates the initiative is unknown: there is no P0 route.
- Scenario, Complete: the routes run and the game behaves, but the runner requires zero
  slow frames, which P0 did not meet either.

Next, in order:
1. Decide the 25ms gate for Complete. Thresholds are unchanged and need the user.
2. Measure Dungeon's slow frames; compare with the P0 `dungeon-cpp` executable.
3. Connect the scenario evidence, starting with `authored_edits.py`, already written.
4. Attribute 480i against a P0-era executable or real hardware.
5. One full `--all` run on the resulting candidate.

### Current execution checkpoint (7 November)

- The fresh economy/UI conformance block passes in C++, Blueprint and Lua AOT:
  **896 rows / 77,056 exact words per provider**, normal O2, identical traces.
  Static RAM is 1,859,096 / 1,914,392 / 1,904,152 bytes respectively. This closes
  that fixture's provider comparison, not the Complete game's remaining routes,
  runtime peaks or frame-time gate. Receipt:
  `p9/economy/providers/ui896-isolated-approved/consolidated-receipt.json`.
- The first serialized 18-case real build batch is terminal. All 15 smaller cases
  compile; Complete C++ / Blueprint / Lua AOT overflow RAM by 159,288 / 302,648 /
  214,584 bytes respectively. Outputs and failed maps are preserved in
  `p8/iteration-9/first18-builds/`. Its successful builds were provenance-rejected
  because the editor generates root `.luarc.json`; the applied narrow fix ignores
  only this IntelliSense file and passes eight tests, including real-source and
  settings invalidation. Original rejected receipts remain preserved.
- Resource parity now compares actual package payloads, identity/reference edges,
  all maps and real widget compositions across companion/BP storage. All six
  groups of the 18 retained projects agree; 18 positive/negative host tests pass.
  Shared resources are not compared by provider-specific serialized UUID bytes.
- Real common-designer editing exposed a concrete regression: local widget
  controls retain resolved numeric scene parents, which the portable class
  writer rejects. The narrow UUID-parent-preserving fix and focused regression
  are applied. All twelve real Window edit/undo/redo/save/reopen cases pass,
  preserving C++/Lua source bytes and Blueprint behavior/defaults. Six focused
  editor regressions and three installer tests pass. Physical GUI replay also
  passes all twelve classes, including fresh-process reopens, with 72 captures
  and all 45 script files byte-identical. Authoring receipt:
  `p10/common-designer-real/receipt.json`. This necessary
  authoring dependency reopens only the affected common-editor path; P0–P7
  otherwise remain frozen.
- Canonical editor `1534def2…` and 1,344 product inputs are frozen in iteration 10.
  The same Game/Vitals component UUIDs move to the existing visual actor; an
  explicit player reference preserves two free player component slots (6/8),
  58 actors and 165 components. Game has 13 properties. All 18 existing projects
  were reconciled with backups and reviewed source checks; caches were retained.
  A later test-only consolidation removes redundant catalog compilation without
  changing production code/editor bytes; refresh the broad source snapshot at
  the next canonical boundary before final matrix verification.
- One controlled full-game C++ Os experiment passes the static size criterion:
  1,963,576 bytes used / 133,576 unreserved. Defaults remain O2. Exact exported
  guard/boss behavior and unchanged 25ms gates are running before any adoption.
  The separately measured exported profile uses 1,965,880 static bytes / 131,272
  unreserved; its transport image differs, while the cooked scene header agrees.
  Preserve both profiles rather than claiming identical executables. Two route
  attempts exposed observer defects (prototype/scene scope, then tuple-to-Lua
  syntax), not game performance results. Nineteen host/preflight checks and two
  real Redux LuaJIT syntax checks pass. Corrected real replay now completes both
  routes: guard/boss behavior, victory, music, streaming and zero missing sounds
  work, but performance fails (guards 1,307 slow / 50,048us maximum; boss 405 /
  48,064us). The Os candidate is **not adopted**. Phase counters separate startup
  HUD work from sustained controller/game work; one exact-image external lookup
  profile is delegated before choosing an optimization. Its first zero-frame
  instrumentation failure is preserved separately from game costs.
  Next complete that bounded cost experiment, optimize the measured common cost,
  then validate the three full providers,
  retained 18-case builds/routes/exports and the final coherent regression.
  No Dungeon Complete VM case is queued.
- The actual Basic/Sample authored-edit/helper harness is integrated (seven host
  tests), as are additional player physical routes (ten host tests). Their target
  executions remain pending. Runtime adapter source closures are now explicitly
  hashed, so changing a helper/observer cannot retain stale scenario acceptance.
- The current 640x480i capture reproduces the remaining visual failure in all
  eight stable raw snapshots: the even field remains almost empty. GPUSTAT field
  observations and unchanged raw images are retained under
  `p10/camera-views-480i-current/`. No field reconstruction or progressive image
  substitutes for this pending renderer/capture gate.

### Earlier integration checkpoints (retained evidence)

Earlier integration: ordinary HUD/modals/interactables, stable scene bindings
and the three-flavor installer are applied. Scene is 58 actors / 165 components,
maximum eight per actor; one modal reaches 64 actors. Game has 12 properties.
Real Lua combat frontend/Native PC and its AOT PSX case now pass. All three
required combat providers agree on **53,040 words per case**; C++/Blueprint
captures are reused by identical current-source executables. The receipt is
`p8/iteration-7/combat-console-attempt-2/receipt.json`; no VM case was scheduled.
UI frontend fixes and the generic AOT record-declaration dependency are applied.
All three UI providers pass real frontend/Native PC compilation and one-second
execution; this is not PSX scenario acceptance. Canonical `4a30de81…` and 1,344
product inputs are frozen under `p8/iteration-7/`. Three focused installer tests
pass, including public generation in all three flavors and source ownership.
Current Clippy passes with warnings denied. These focused checks do not replace
the pending coherent workspace regression, full routes or final acceptance.
Twelve common-designer captures cover the four new widget classes in all three
providers. Five windows were inspected directly; each class's preview pixels
match across providers, with the normal hierarchy/properties/Behavior controls
and zero observed drops. See `iteration-7/widget-designers/review.json`.
Interactive edit/save/reopen checks remain distinct and pending.
The first expanded economy/UI PSX attempt captured all 77,056 words but failed:
the fixture resolved a TextComponent as an Actor and checked deferred destruction
inside dispatch. These are verified fixture defects; game/widget rules remain
unchanged. Core protocol differences also exposed the automatic Pocket bridge
responding to the fixture's mailbox. Isolate that second responder and prove the
840-row core is unchanged before retrying; no BP/Lua replay was started.
The failed executable leaves 238,056 RAM bytes unreserved. This is a fixture
measurement, not a Complete template budget acceptance.

Final preparation generated the C++ and Blueprint Complete projects in stable
`p10/final18/projects/` paths. Its reflection verifier incorrectly used the
C++-only `--reflect` endpoint for authored providers. A read-only combined
`--class-registry` endpoint now passes the real three-provider/ill-typed-Lua test;
its canonical build is in progress. Eighteen explicit ownership contracts and
the companion-asset checker are prepared. Reconcile only README changes in the
two existing fixtures, then resume generation/reflection without new caches.

That preparation now passes **all 18 generation, source-ownership and compiled
class-registry cases** with editor `5811847c…` / 1,344 product inputs. Lua uses
Native C++ AOT throughout. Both existing Complete projects were reconciled only
for README/provenance; no cache was recreated. The receipt is
`p8/iteration-8/final18-preparation.json`. Builds, Play, scenarios and all 18
standalone exports remain separate pending gates. API reference was regenerated
from native declarations and parity inventory has zero pending classifications.
The coherent full workspace regression is running; no new global green claim.

The corrected expanded economy/UI C++ PSX case now passes **all 896 × 86 =
77,056 words**, including the unchanged 840-row core and 56 UI/interaction cases.
Its mailbox isolation has an exact host differential against the unmasked core;
it prevents the separate automatic bridge from answering this simulated device.
No selected-language rule changed. C++ evidence is preserved under
`p9/economy/providers/ui896-approved/cpp/`.
Blueprint's normal-O2 image fails the real MIPS link with RAM overflow of
306,712 bytes; Lua AOT was not started. Preserve that map/log and measure its
largest code/data costs. A single controlled Os attempt now reuses the same BP
project/cache and exact trace; defaults, scene content and all gates stay fixed.
The criterion is the full trace plus 64KiB headroom, followed by actual Complete
frame-time acceptance before adopting any compilation-profile change.
The map comparison identifies **545,616 extra text bytes** in Blueprint, with
identical `.sbss` and pools. HUD/menu methods dominate; Shop's 110 authored exec
nodes become 3,567 emitted continuations. A conservative shared-CFG-tail compiler
fix is being prepared separately, with at least 128KiB saving plus exact provider
conformance required. It must preserve captures, control flow and latent behavior;
no compiler change is applied while the controlled Os run is active.

The later workspace run finishes its main suite with **1,282 passed / 84 ignored /
one failure**. The failing Hub interaction assertion expected C++ after selecting
the last flavor and canceling Complete creation. Complete now correctly retains
Lua, as required by three-flavor support; the existing loop already verifies each
flavor can be selected. A test-only correction preserves the actual selected
flavor across Cancel. Apply/test it at the next source boundary; preserve the
failed global result and its unvisited workspace suites, not a global green claim.

The controlled BP Os image links with **1,877,528 static RAM bytes / 219,624
unreserved**, but fails the real protocol trace at `unknown-upgrade-refused`.
Keep the memory improvement separate from behavioral acceptance; no profile is
adopted. Lua AOT's same 896-row case is now running independently under normal O2
to distinguish shared fixture/runtime behavior from a Blueprint-specific defect.
The active handoff, process handles and next validation queue are recorded in
`p8/iteration-8/live-state.json`. The goal remains all pending phases, with the
18-case source/registry preparation only one completed gate.

The shared-tail compiler change is now applied and passes **134 focused Blueprint
tests / 15 ignored**, including captured getters, mutating arms, node provenance,
owner guards, loops and untouched latent functions. The corrected Hub interaction
test also passes against real ImGui input. New canonical `332d6a77…` / 1,344
product inputs / 15 validation inputs is frozen in `p8/iteration-9/`.
The fixture now renames its actual SDK mailbox in every application TU, so the
unchanged external bridge cannot discover it. The insufficient between-frame
guard is removed; two-TU identity and the complete core host differential pass.
Three fresh native UI/provider cases run serially under normal O2, requiring the
unchanged trace and at least 128KiB measured BP saving. No subsystem closes yet.
The fresh isolated C++ case passes 77,056 words with the public mailbox symbol
absent, private symbol present and no external bridge attached. Blueprint is
running; Lua follows in the same queue. Current Clippy passes with warnings
denied. Standalone export rebuild directories now use stable per-case locations;
an engine/compiler change requires reviewed reconciliation instead of another
cache. Twelve-widget edit/undo/save/reopen coverage is prepared as a test draft,
separate from the already inspected designer previews.
Blueprint now passes the fresh isolated 77,056-word case under normal O2 with
1,914,392 static RAM bytes. Its text shrinks by **490,288 bytes** versus the
pre-change O2 map, exceeding the 128KiB criterion; the core and all UI rows match
C++ exactly. The shared-tail change meets this controlled experiment's size and
behavior criteria, pending broader template/scenario acceptance. Lua is the last
active case in the same queue. Actual-project/export observers for all six
templates are integrated under `tests/integration/gameplay_helpers/`; 44 host
tests pass. They keep unmeasured mandatory gates missing until target evidence
exists. Tracked adapter and ownership contracts live beside this plan.
Next validate three-provider subsystems, affected Complete routes,
coherent regression/exports, and the final 18-case matrix. The existing O2 RAM
failure and controlled Os saving still require resolution and performance checks.

### Retained handoff before the scope correction (historical)

The next integration batch contains the 128-method Lua runtime and the corrected
Lua controller/encounter sources. Their nominal helper annotations are removed
by inlining the same algorithms into inherited overrides; the C++/Blueprint
policies and native adapters are byte-identical. Generator idempotence and
57,600 focused inline comparisons pass. These are host/source checks, not PSX
acceptance. Canonical editor `fce65cf6…` now includes this batch; its 616 product
inputs are pinned under `p8/iteration-6/`. Watcher checks pass (11 affected tests,
including the unchanged original editor assertion). The previous global failure
is retained; this focused result does not make that workspace run green.

The serialized console queue resumed the missing economy VM cases. Bytecode
now passes the method-count guard but **fails at boot**: a 380-byte allocation
with 94,360 live payload bytes exhausts the fixed 98,304-byte arena. Preserve
this failure (`iteration-6/economy-vm-bytecode-failure.json`); investigate actual
chunk/allocation costs without increasing budgets. VM source and Lua combat
remain queued. Preserve the three
native economy captures (72,240 words each) and C++/Blueprint combat captures
(53,040 words each), with their original editor hashes. Reuse requires an
explicit unchanged-dependency or executable-identity proof; do not relabel old
captures as current-source results. All five native rebuilds now prove exact
executable identity and their retained traces pass check-only validation;
`iteration-6/native-validation.json` preserves original playback hashes. No
accepted native playback is repeated.

VM source also fails economy boot (1,280 requested / 124,592 live / 131,072 arena).
Combat Lua passes declaration preparation but still fails real body typing on
helper purity, generated local names and untyped literal comparisons. Those
authoring corrections resume with a real frontend check before another PSX
attempt. Four full-scene jobs stopped at that same frontend prerequisite;
their old cached map values are explicitly rejected, not current RAM evidence.
The collector now checks map freshness and whether MIPS compilation was reached.

Current Complete C++ misses the 64KiB reserve by 9,968 bytes under normal O2.
A controlled existing Os override saves **241,664 bytes**, leaving **297,232**
unreserved; its static criterion passes. Defaults remain unchanged, and this
does not establish gameplay/frame-time acceptance or final UI capacity. The
unvisited Pocket workspace suite passes 40 tests (one ignored). Clippy found
two test-only redundant clones; both are corrected for the next quality batch.

Independent work now completes the ordinary HUD/modal/interactable draft and
the three-flavor Complete installer. No class is a separate delivery. Integrate
the full UI block once, retaining editable common-designer classes and the
64-actor/eight-components limits. Then run affected Complete scenarios, current
full-scene VM measurements, coherent regression/exports and finally all P10.
Estimate: one provider cycle for the applied A/B rules, followed by one coherent
UI/interaction integration cycle; remaining memory and frame-budget failures can
require measured follow-up and are not included as completed work.

This section is the current execution authority. Older entries below are retained
as dated evidence, not instructions to repeat accepted work. The objective still
includes all remaining P0-P10 work under the 18-combination scope above.

P0-P7 are **frozen and accepted**. Reopen a specific contract only for an observed
regression or a dependency required by the remaining implementation. Record the
reason and affected tests; do not restart a phase or its full acceptance matrix.

Player vitality is **closed as an implementation prerequisite**, not as a new
subsystem release. Its completed receipt is
`artifacts/gameplay-helpers/p8/player-vitals/validation.json`: 41,204 host
checkpoints, five PSX cases with 6,720 words each, affected Complete scenarios,
exports and workspace regression passed. No outstanding validation job was
reported by its final progress record. The recorded 25ms failure remains open.
Preserve the receipt and logs. Reuse these results while their source and tool
dependencies remain unchanged; a historical receipt is never validation of a
subsequent SDK/compiler or game change.

### Complete subsystem delivery queue

P8 mechanisms and C++ migration now advance together with P9 authored rules.
Finish a functional subsystem in all providers before declaring it delivered.
Individual classes and helpers are implementation checkpoints only.

Current iteration after the coherent checkpoint: the prepared six enemy-shot
methods and nine-word tuning are now applied to the existing encounter policy in
all three providers, and Game consumes their results. All 10,264 focused shot
checks pass using `verify_spatial.py dungeon_encounter -- --shots-only`; accepted
guard/boss host checks were not repeated. The existing combined fixture is being
extended to retain its accepted 56-word prefix and add an independently checked
14-word shot tail. The generic HUD appearance-reuse experiment passes its focused
host checks and all 40 same-image PSX VRAM comparisons, including both parities,
focus/reveal and stale-key transitions. Its 532-byte state meets the memory
criterion, but repeated appearance updates cost 16 scanlines versus 38 with the
optimization disabled: the unchanged at-most-12 criterion **fails**. Preserve the
failed experiment; this is not performance acceptance. Current five-mode combat
playback **passes all 16,800 words per case**, preserving the accepted 13,440-word
prefix and matching the independent 3,360-word shot oracle. Its current receipt
is `p8/iteration-2/combat-validation.json`; the prior consolidated receipt is
preserved alongside it. The single economy fixture **passes all five modes**:
181 cases / 15,566 exact words per mode. See `p8/iteration-2/economy-validation.json`.
Two unsupported Lua helper return annotations were replaced by inline record
construction; only the three affected Lua modes were rerun. The authoring-only
source adjustment is pinned separately and did not change the compiled SDK or
running Cargo inputs. No prior vitality acceptance is repeated.
The previous 622-input receipt is historical for these changed dependencies;
unchanged results remain available for scoped reuse. No subsystem or phase closes
at this implementation checkpoint.

| Block | Required implementation and original requirements | Completion evidence | State / next estimate |
| --- | --- | --- | --- |
| A. Player combat | Shared health, projectile storage/query batches and targeting mechanisms; editable vitality, attack eligibility, cadence, damage, lock/aim and feedback decisions in C++, Blueprint and Lua AOT. Preserve camera/control and run-flow contracts. Covers P8B health/projectiles/target queries, P8C combat and P9A/player P9B. | Three-provider native conformance, player attack/aim/damage/death/restart PSX scenarios, changed-rule checks, regression and exports. | Current three-provider conformance passes 53,040 words/case; C++/BP reuse identical executables, Lua AOT has fresh PSX playback. Full Complete routes, camera trace and coherent resource/export gates remain. Next: one combined scenario/resource cycle after C's conformance. |
| B. Encounters | Bounded activation/target mechanisms; selected-language guard perception/attacks, boss phases, rewards, victory and transitions. Covers remaining P8B, encounter/game-flow P8C and P9A/P9B. | Combined guard/boss/victory/restart scenarios in the three native providers, editable-rule mutation, identity/resource checks and exports. | Rules and explicit scene bindings applied; combined mechanism/rule conformance passes in all three providers. Full Complete routes and changed-rule observations remain. Validate with A in one shared scenario cycle. |
| C. Economy and interfaces | Editable inventory, prices, purchase/terminal/save/progression and PocketStation orchestration; bounded SDK card/Pocket mechanisms. Ordinary common-designer UIActor widgets with children and their own C++/Blueprint/Lua rules. Covers P8B save/devices, P8C economy/UI/data and P9C/P9D. | Economy circuit, fresh install/cold reload, disconnect/interruption/duplicate/restart/exit/save-before-ack, UI edits and rules in three native providers. | Fresh normal-O2 conformance passes all 77,056 words/provider, including 840 core rows and 56 UI/interaction rows. Real frontends and preview parity pass. Fix the observed common-designer edit regression, then complete full persistence/device routes and the twelve-class authoring workflow. Estimate: one editor rebuild/focused test cycle, followed by one combined three-provider route cycle; target runtime failures may add measured optimization work. |
| D. Packaging and final acceptance | Finish P9E generation, source inventories and authored-rule ownership; P10 documentation, Native/PSX views and 480i field checks. | All 18 generated/executed combinations and 18 corresponding standalone exports, semantic/visual/resource checks, runtime budgets and recorded hardware status. | Three-flavor installer and 18-case runner applied; all generation/ownership/registry cases pass. Actual 18-build batch running, then reviewed metadata fix, routes, Play and 18 relocated exports in the same queue. Estimate: one cached build verification pass, 18 export rebuilds and six shared scenario families; final global and 480i checks remain explicit. |

Shared encounter tables, item definitions, authored assets and schemas remain
data. Blueprint/Lua variants must contain real editable rules, with no concealed
authored C++ game-policy fallback. Native mechanisms expose parameters/results,
not template-specific decisions. The earlier phase requirement lists below remain
the exhaustive checklist; this mapping changes delivery order, not scope.

### Validation and integration policy

1. During implementation, run compile checks and focused host/provider tests for
   changed contracts. Consolidate reusable fixtures where coverage is preserved.
2. At a subsystem boundary, run C++, Blueprint and Lua AOT conformance and affected PSX routes.
   A changed SDK/compiler contract may require earlier cross-provider/global tests.
3. At a coherent integration boundary, run the workspace regression, quality/API
   checks and affected standalone exports once against frozen integration sources.
4. At the end, run the complete P10 matrix. Never relax the 25ms, memory, identity,
   feature, visual or provider-ownership requirements to obtain a green result.

Use one integration owner and **one serialized queue for MIPS builds, emulator
runs and standalone exports**. Independent agents have explicit file ownership
and stable contracts; they may run host checks but must request that queue before
console work. Reuse `target/`, existing projects and valid receipts/caches. Record
whether each result is current-source, scoped/unaffected reuse, or historical.
Do not rerun vitality or frozen P0-P7 acceptance merely to close a helper class.

### Performance and memory workstream

Measure full-scene Lua AOT capacity with all features and the normal 64KiB
runtime reserve. VM capacity experiments below are retained historical research,
outside template delivery; do not resume Complete VM attempts for this release.
Measure the full guard/boss and terminal/save/install paths as they become ready.

Prioritize measured dominant costs. Every experiment records its hypothesis,
baseline, affected route, success criterion and keep/reject decision. Current
reference: vitality PCDrv image uses 2,017,000 static RAM / 1,196,032 EXE bytes;
guards peak at 25,984us and boss at 27,648us, so the 25ms gate **fails**. Earlier
guard/boss external observations do not establish current full-scenario peaks.
Reject experiments promptly when they fail their stated criterion. Do not add
new per-frame instrumentation or repeat accepted routes without a concrete need.

At each block closure update this queue with delivered behavior, exact evidence,
remaining gates and an estimate for the next block based on measured work.

The stream-pool sizing experiment was rejected without another build: current
interchangeable stream slots require the existing 65,536 bytes each, so the
conservative saving is zero. See `p8/execution-reorg/stream-sizing/result.json`.
The next bounded memory experiment packs the seven generated initial actor fields
and reconstructs unchanged `ActorData{}` defaults before components initialize.
It preserves all 90 actor slots and 32 spawn reservations. Host reconstruction
passes 256 cases; estimated gross saving is 77,024 bytes, with **net at least
65,536 bytes in both VM maps** required. A fresh baseline uses the current
58-actor/157-component scene and all five migrated Lua policies, still honestly a
hybrid. No memory gate is accepted from estimates. The HUD follow-up and remaining
impact/feedback rules are prepared separately; their validation is consolidated
into the existing fixtures after the controlled memory comparison.

The current five-policy hybrid baseline is now measured: bytecode/source failed
links at **2,412,056 / 2,432,536 bytes**, requiring **380,440 / 400,920 bytes**
of reduction for the unchanged 64KiB reserve. The 23-word save uses nine physical
wire slots (13,952 scratch bytes); planned arenas remain 98,304 / 131,072 bytes.
These current 58-actor/157-component results supersede the older two-policy
capacity estimate for scheduling, while retaining its isolated optimization
evidence. Maps and source inventory are in
`p8/execution-reorg/actor-initial-packing/current-before/`. The compact-state
experiment alone cannot close Lua capacity; investigate the largest measured
code/data growth next, preserving features and limits. The frozen SDK/compiler
checkpoint passes the full workspace again (**1,323 passed / 88 ignored / four
suites**) and Clippy with warnings denied; see `p8/iteration-2/quality-validation.json`.
This receipt predates the subsequent controlled compiler and HUD/game changes.

The isolated compact-state implementation now passes 16 bank tests (one ignored)
and its normal editor build. Controlled identical five-policy scene maps save
**77,824 / 79,872 bytes** in bytecode/source, exceeding the unchanged 65,536-byte
experiment criterion. Lua still needs **302,616 / 321,048 bytes** for its reserve;
no VM image booted. `actor-initial-packing/controlled-comparison.json` pins both
maps and the single changed engine input. Provider/default PSX behavior remains
required before final retention. The next coherent batch is applied: six impact,
shot-style, FX, flash and rumble policy methods in all three providers, expanded
existing combat fixture (21,600 words), generic HUD lookup reuse, and neutral
UInt32 XOR/Fixed floor APIs. Identical staged host/graph proofs are reused by
hash/text checks; only the new math contract was tested anew. Canonical build,
current five-mode execution and the updated HUD visual/timing probe are next.

Subsequent focused results: updated HUD captures match all 40 old goldens and all
40 off/on states in both parities, with 596 bytes of actual MIPS state and no
drops/errors. Peak cost is **15 scanlines**, so the unchanged <=12 experiment
criterion still **fails**. A concrete provider-preparation failure exposed custom
Blueprint helpers classified as impure when called by PureEvents. A conservative
compiler purity fix and focused tests are in progress; this necessary dependency
reopens only that compiler contract, not any frozen phase's gameplay acceptance.

The existing MIPS `-Os` override is now a controlled experiment, with normal
`-O2` defaults unchanged. In the identical retained five-policy hybrid it saves
**260,096 bytes in each VM** (criterion at least 65,536), leaving **42,584 / 61,016
bytes** still needed for the runtime reserve. See
`p8/execution-reorg/size-optimization/comparison.json`. Flags are process-local,
the same project/cache is reused, and current normal exports are not relabeled as
validated with different flags. Exact HUD visuals/timing with `-Os` are running;
affected Complete routes must pass the unchanged frame gate before adoption.
Stream sizing, archive compression (geometry already external), and the isolated
Lua scalar-member proposal were rejected early against their stated savings
criteria; their evidence remains under `p8/execution-reorg/`.

The complete Pocket protocol/coordinator port is now applied in all providers;
C++ shell dispatches the chosen implementation. The scene is 58 actors / 158
components, Station Collision exactly eight; no limit increased. Its 2,375-packet
host differential is reused by exact source identity, and the new shell adapter
passes save/ACK/heal/rebind failure tests. The existing economy fixture is being
extended; frontend/provider/real Pocket acceptance and ordinary editable terminal
widgets remain open. The remaining controller port is a separate complete draft,
not a finished template flavor. No subsystem/phase closes at this checkpoint.

### Active integration queue after the implementation batch

The controller and the remaining Game combat/encounter/door/pickup decisions are
now applied in all three providers. Their unchanged staged host/graph evidence
is 104,173 + 104,173 and 105,257 + 105,257 checks, respectively. This is code
integration, not closure of A/B. The combined fixture retains its accepted
70-word historical trace and independent feedback oracle, and is being expanded
in place to cover the complete rules. No 90/147-word PSX capture is claimed.

The expanded economy fixture passes C++ **840 rows / 72,240 exact words**.
Its full capture is retained. Two actual provider defects are being resolved:
Blueprint needed to preserve a getter's explicit execution-point capture after
purity inference (45 compiler tests now pass); FNV must use modulo-2^32
multiplication rather than the ordinary saturating UInt32 operation in BP/Lua.
The latter uses the explicit generic `MathLibrary::wrapping_multiply`, with its
hash vectors passing. All other arithmetic and all budgets remain unchanged.
Only affected provider cases resume; accepted C++ behavior is not discarded.

The longer circuit required a longer emulator watchdog. A headless cooperative
Stop now closes playback immediately after the validated summary, through the
normal Stop lifecycle, retaining the zero-exit check. This avoids waiting out
the watchdog and changes no frame-time acceptance requirement. Earlier partial
captures and harness failures remain archived.

Editor `9ce618f4d23a71d24ab95ac0ce1716b09bc4a947fdbfb550891d882e5c64c6c5`
and 616 product inputs are frozen under `p8/iteration-5/`. Ordered work:

1. Finish the bounded HUD POD-scratch O2/Os comparison; the previous best remains
   Os **13 > 12** (failed), with exact visuals. No threshold change.
2. Resume BP/Lua economy and the complete combined combat circuit using retained
   projects/caches and exact historical prefixes. The current workspace run is
   justified by the coherent compiler/SDK/game changes; the preceding 1,323-test
   receipt remains historical for those changed inputs.
3. Finish ordinary terminal/shop/inventory widgets and Game HUD presentation.
   The draft reuses the Combat HUD root and budgets one six-actor active modal,
   for at most 64 actors. It uses the common designer and chosen-language logic.
4. Validate current Complete routes, frozen camera trace, integration exports and
   revised full-scene VM capacity. The Os memory experiment is promising but
   has not been adopted; full-scenario timing must pass first.
5. Complete provider packaging and the unchanged full P10 matrix/checklist.

Next estimate: one consolidated provider/scenario validation cycle for the
applied A/B code, with fixes driven by actual failures; C still needs widget
adoption and a combined persistence/device/UI cycle. P9 packaging and every P10
requirement remain in scope. No block closes before its functional/provider/
scenario evidence and outstanding runtime budgets are assessed.

Latest validation state: the 840-row protocol now agrees in **all three native
providers** (72,240 words each); Lua native was checked separately after the
multi-mode runner stopped. The bytecode VM boots but aborts at the documented
64-method capacity because the real economy class has 72 methods. A bounded
128-method SDK extension is applied with safe MIPS32 masks, class/slot isolation
and real Frame host checks; its explicit cost is 1,024 static bytes for 64 class
slots. This is a capability extension pending integration, not a passed memory
budget. Arenas, call depth, live values, wire width and runtime reserve stay fixed.

The complete combined fixture is now frozen at **221 words / 240 frames**
(53,040 per case), including selected RunFlow consequences; its current C++ PSX
case and BP native case pass. Lua native preparation exposed unsupported custom
return type annotations; their source correction is in progress. Historical 70-word
captures remain the only previously accepted prefix; 90/147 were host/source
checkpoints, not accepted PSX deliveries.

The later workspace run retained **1,278 passed / 84 ignored** in its main suite
but failed one editor idle-read/stale-output timing assertion. That concrete
failure is under isolated diagnosis; the run is not green and its remaining
workspace suites were not reached. Do not repeat the passed cases without a
changed dependency. The preceding full green receipt remains historical.
The failing test passes in isolation. Its watcher subscribes recursively to the
repository when only two configuration files are needed. Watcher saturation is
a hypothesis for the failure, not a proven cause: 35,218 irrelevant writes did
not reproduce it. A deterministic subscription-scope regression and narrowly
scoped correction are in progress, retaining the original idle-read assertions.
The HUD scratch experiment is terminal: O2 **14**, Os **13**, still above 12;
both have 40 old-golden and 40 off/on matches, two parities, 596 bytes and no
errors/drops. ASM confirms the 768-byte clear was removed; further HUD micro
experiments stop until current Complete routes identify the next measured cost.

Current-source implementation checkpoint: editor SHA
`20118b7e02257f4c1ff9d6728be60c5ce5eaac8ee38c133ad94be80961520890`
and 616 frozen product inputs are recorded under `p8/iteration-2/`. The Complete
scene now has 58 actors and 157 components, still at most eight per actor and
seven on the player. Economy policy owns inventory/trade/save/receipt decisions;
the shell uses bounded generic named-card and owned Pocket packet mechanisms.
The focused adapter proof covers the actual applied methods, 92-byte saves,
save-before-ack, failed/stale writes, duplicate reconnect, corrupt load and
saturating medkit healing. These checks do not establish current PSX scenario or
full-template Lua acceptance. The old coherent checkpoint below remains dated
evidence; its changed dependencies require the queued current checks. No global
regression or export sweep is repeated before the next coherent boundary.

Execution checkpoint (6 November, not a subsystem closure): recovery found no
remaining build/emulator/validation jobs; Python processes were unrelated MCP
servers. Vitality's receipt pins its editor and 509 source hashes. The retained
audit already lists three changed dependencies (`world_gameplay.hpp`,
`game_helpers.rs`, `project.rs`) and classifies that acceptance as historical
(`artifacts/gameplay-helpers/p8/execution-reorg/state-audit.json`). Preserve that
distinction as further combat/SDK/compiler changes land.
Current focused checks cover 6,180 projectile motion checkpoints, 37,040 player
combat checks and 245,924 encounter checks. C++ adopts these mechanisms/policies
in the scene (58 actors, 156 components, maximum eight, player seven); actual
combined provider execution and affected Complete scenarios are still pending.
The new `verify_dungeon_combat.py` consolidates all three authored policies and
projectile lifecycle into one five-mode fixture instead of repeating vitality.

The first full-scene hybrid VM experiment fails to link with static extents
2,231,568 bytes (bytecode) and 2,252,048 bytes (source). Required reductions to
retain the existing 64KiB reserve are at least 199,952 / 220,432 bytes. These are
pre-projectile baseline capacity results, not full-Lua gameplay acceptance. Exact
maps, source inventory and logs are retained under
`artifacts/gameplay-helpers/p8/execution-reorg/full-scene-vm/`. One bounded compiler
experiment now reuses unrooted immutable value temporaries while retaining the
16-depth/64-live-value/32-word limits. Its host allocator and 13 targeted compiler
tests pass; PSX re-entry/alias/loop tests and measured full-scene savings must pass
before retaining the experiment. No overall performance/memory gate is accepted.

The first combined C++ PSX case passes all 13,440 words (25 player shots, two
guard shots, three boss shots). Its capture was reused after proving that a
provenance-only Blueprint fix and the Lua-only compiler follow-up regenerate the
exact same C++ executable; the paired fingerprints and explicit reuse reason are
in `combat/providers/cpp/scoped-provenance-reuse.json`. The Blueprint fix makes a
shared resource-demand signature independent of the operation consuming it; the
new cross-class staging regression passes. This is a necessary compiler
dependency repair, not reopening accepted P0-P7 gameplay. The broader Lua host
regression passes 92 tests with five existing ignores. Actual 32-word lifetime,
alias, loop and native re-entry checks pass both VMs on the first optimization.
The initial measured gain was 32,592 bytes, **176 bytes below** the stated 32KiB
experiment criterion. The short lexical helper alias now passes both PSX lifetime
cases and saves **36,688 bytes in both full-scene maps**, meeting that unchanged
criterion. Its receipt is `execution-reorg/wire-scratch/validation.json`. All five
combined provider cases now pass the same 13,440 words; see
`combat/providers/consolidated-validation.json`. These are scoped rule/mechanism
conformance results, not completion of combat/encounter gameplay ownership.
The isolated hybrid scene still needs at least 163,264 / 183,744 additional bytes
(bytecode/source) for the required reserve; it intentionally retains the old
native game and is not acceptance of the new Complete scene or a full Lua port.
The proposed direct ActorData-capacity reduction was rejected because authored
slot quarantine permits legal spawns that a 64-slot clamp would prevent.
The next measured experiment excludes the existing 36,104-byte particle pool
only when cooked scenes, capabilities and observed source provenance prove it
unused. Unknown includes, macros, dynamic class creation and unproven native code
retain the default behavior. Fourteen focused backend tests pass. Console memory,
active-particle behavior, global regression and exports remain required before
retaining this change; see `execution-reorg/particles/host-validation.json`.

### Coherent integration checkpoint

`artifacts/gameplay-helpers/p8/combat/integration/validation.json` closes the
current implementation checkpoint against 622 frozen source/validation inputs.
The workspace passes **1,323 tests / 88 ignored / four suites**; Clippy with
warnings denied, formatting, API freshness and parity pass. The five provider
traces agree on all 13,440 words. Fresh Complete passes both 1,800-frame functional
guard/boss routes, all 16 economy events, cold reload and the separate two-event
fresh install; six independent comparisons confirm card restoration. Its camera
matches all 3,420 frozen words, reusing the old expected trace without replaying
that accepted baseline. All 67 packages and geometry remain identical. Complete
and the three native provider standalone exports rebuild byte-identically;
provider rebuilds also match their actual accepted playback images.

The current normal PCDrv image uses **1,997,264 static RAM bytes**, leaving
99,888 unassigned and 34,352 beyond the unchanged 65,536-byte runtime reserve.
Basic uses 936,824 RAM / 425,984 EXE bytes, a 46,336-byte RAM reduction.
In an identical current Complete scene, retaining the particle pool costs
**44,288 bytes** and fails the reserve by 9,936 bytes; the active-particle PSX
probe passes 1,200 captured words. This controlled experiment validates the
exclusion mechanism. The separately stated *coherent net reduction versus the
vitality baseline* is only **19,736 bytes PCDrv / 21,720 embedded**, so its 32KiB
experiment criterion **fails** and remains recorded as failed. The optimization
is retained for its measured controlled saving and preserved behavior; no phase
budget or experimental result is relabeled green.

The **25ms gate still fails**: guards 15 slow frames / 27,008us; boss five /
25,856us. Equal-step samples identify about 4.5-5.3 additional script scanlines
per tick; most slow frames have two simulation steps. Damage-flash HUD rebuilds
also cost 26-33 scanlines against roughly 7-8 normally. The next performance
experiment is bounded appearance-only image reuse, with repeated flash HUD cost
at most 12 scanlines, exact visual/focus/reveal behavior and at most 1KiB extra
state; see `combat/next-performance/measured-priority.json`. No runtime peak or
full-game Lua capacity acceptance is implied by the static RAM results.

Blocks A and B remain active: the shared mechanism and authored policy
prerequisites are integrated, but remaining impact/feedback/controller and
complete encounter ownership must move out of the native monolith before either
block closes. The next ready patch adds enemy projectile/aim/fan/size/damage rules
to the existing encounter policy in all three providers; its 10,264 independent
host checks pass and it remains unapplied during this freeze. Expect one focused
implementation/validation iteration for that prepared patch, followed by the
remaining combat ownership work; reuse existing scenarios and fixtures.

Block C has an independent artifacts-only draft with real C++, Blueprint and Lua
inventory/trade/eight-receipt and save-before-ack rules. Its 1,380,000 host
comparisons and failure/duplicate checks pass; frontend checking and integration
are still pending. Generic transports and ordinary editable terminal widgets
remain required. This block will need several integration iterations because
transport errors, persistence and UI capture must be validated together. P9 full
template packaging and every P10 combination/view/export remain pending.

Status: P0–P7 validated. P8 is in progress; P9–P10 remain pending. The pre-existing Dungeon Complete frame-time failure and full-scenario runtime memory acceptance remain open P8/P10 gates; guard/boss heap/stack observations are now available. Platformer and Dungeon pass their scoped five-mode template acceptance; the complete-game migration and final execution matrix remain required. See [P0 evidence](p0-baseline.md), [P1 delivery and tests](p1-delivery.md), [P6 delivery and tests](p6-delivery.md), [P7D delivery](p7d-delivery.md), the results below and the [implementation contract](contract.md).

Planning baseline: `fb542cd167fdc297425c0b15fedb901b7c4e0c08`, 28 October 2026.

Latest validated delivery: [player vitality extraction](p8-player-vitals.md). Independently
authored C++, Blueprint and Lua policies pass five PSX executions (6,720 oracle
words each) and 41,204 exact legacy host checkpoints. The new typed component
binding and template generation check pass. Fresh Complete combat, 16 economy
events/cold reload, two fresh-install events, three provider exports and the
Complete export pass. Both compact camera cases match all 3,420 frozen words.
All 67 packages and geometry are unchanged. The workspace passes 1,316 tests /
88 ignored; Clippy, formatting and API checks pass. The timed PCDrv build uses
2,017,000 RAM / 1,196,032 EXE bytes (+2,680 / +2,048). Its 25ms gate still fails:
guards seven slow frames / 25,984us, boss seven / 27,648us. The unchanged-size
unused Basic and the original camera-instrumentation memory failure are retained
in `artifacts/gameplay-helpers/p8/player-vitals/validation.json` and adjacent logs.
P8-P10 remain open. Four verified temporary SDK object caches were removed,
recovering 21,254,592 file bytes; paths are in `player-vitals/sdk-temporary-cleanup.json`.

Preceding P8 delivery: shared health values/library and C++ game adoption.
Five actual PSX provider/mode cases match the independent 180-word oracle, three
provider exports and the complete-game export rebuild byte-identically, and
unused Basic has zero size delta. Complete passes guard/boss functional routes,
all 16 economy events, cold reload, a separate two-event fresh install, all 3,420
frozen camera words and resource/geometry identity. The workspace passes 1,316
tests / 88 ignored; Clippy, formatting and API checks pass. The timed PCDrv image
uses 2,014,320 RAM / 1,193,984 EXE bytes (+2,208 / +2,048 from the callback receipt).
Its 25ms gate still fails at 26,496us / 25,984us. Full-scenario memory, remaining
P8 mechanisms/project rules and P9-P10 are open. See `artifacts/gameplay-helpers/p8/health/validation.json`.

Earlier P8 extraction: the complete game's run-flow policy has independent C++,
Blueprint and Lua implementations over a project data/dispatch contract. Five
actual PSX executions match all 6,000 Q12/state words, and all three standalone
exports rebuild byte-identically. The complete game retains all 3,420 camera
trace words, passes its 16-event economy circuit and standalone rebuild, and the
workspace retry passes 1,314 tests / 88 ignored. Its normal RAM/EXE grow by
4,856 / 4,096 bytes. The frame-time gate regresses to 27,776us (guards) /
27,520us (boss), leading to the performance investigations below.
This is not acceptance of P8 or availability of the complete Blueprint/Lua
templates. See [run-flow results](p8-design.md#run-flow-policy-extraction)
and `artifacts/gameplay-helpers/p8/run-flow/validation.json`.

The subsequent performance pass adds lifecycle-safe run-flow type-check reuse
and repairs a retained-HUD defect that could freeze text reveal. The latter
passes an actual 48-frame/two-parity PSX test and byte-identical standalone
rebuild. A trial anchor optimization is rejected: exact results, no speedup,
and +2 KiB. Final current-source provider/export/camera/economy validation passes:
five provider cases, three provider exports, 3,420 camera words, all 16 economy
events and a separate two-event fresh install. The workspace passes 1,314 tests /
88 ignored; final restored-arithmetic host checks, build, Clippy, formatting and
API checks pass. Basic retains its previous RAM/EXE sizes. The normal build still fails
25ms (12 guard frames / 27,584us; four boss frames / 26,496us). See the
[performance investigation](p8-design.md#run-flow-access-and-retained-hud-investigation)
and `artifacts/gameplay-helpers/p8/flow-performance/validation.json`.

External observation of the same normal executable over both 1,800-frame routes
records 1,024 live heap bytes / 1,032 footprint bytes and 24,560 main-stack bytes,
including the initial 256-byte top reserve. Combining independent observed peaks
leaves a conservative 59,448-byte gap. Python independently reconstructs the final
heap from paused RAM. The observer adds no console code/BSS and uses one shared
FFI callback for 1,157 linked SP-write sites; BIOS-owned low-RAM stack calls are
recorded separately. Fourteen observer tests and five parity-tool tests pass.
These finite guard/boss observations do not accept terminal/save/install peaks,
all BIOS/generated stack reservations or Complete Lua capacity, and cannot pass
the frame-time gate. Four verified standalone SDK object caches were removed,
recovering 21,254,592 file bytes; paths are in the adjacent cleanup receipt.

The current CPU follow-up adds bounded configurable sampling windows and removes
redundant handle validation from Level's callback prefilter; actual dispatch still
checks generation, class family, lifetime and activation. Host lifecycle and
stale/out-of-range tick/frame tests pass. In the normal unchanged-size Complete
build, guards improve to 7 slow frames / 26,944us and boss to 4 / 25,984us, with
2,012,112 static RAM / 1,191,936 EXE bytes. Two outlining variants are rejected
for 6 KiB / 2 KiB extra RAM and no passing performance gate. Final provider,
export, frozen-SDK camera, 16-event economy, fresh-install and workspace regression
pass on the pinned pre-health SDK. External memory observation of that exact
executable confirms the same 59,448-byte conservative observed gap on both routes.
This does not accept P8 or the 25ms gate. The completed receipt is
`artifacts/gameplay-helpers/p8/frame-hotpaths/validation.json`; four verified
standalone SDK object caches were removed, recovering 21,254,592 file bytes.
Paths are retained in `frame-hotpaths/sdk-temporary-cleanup.json`. See
[the CPU investigation](p8-design.md#callback-dispatch-cpu-investigation).

Combat preparation now includes allocation-free SDK health values and a function
library. The actual SDK passes 134,798 independent host comparisons over the game's
60/100/280 ranges and integer boundaries. It leaves damage eligibility,
invulnerability, feedback and rewards in project rules. Initial C++, Blueprint and
Lua Native PSX executions each match all 180 oracle words. The VM fixture exposed
an unconditional qualified call to a pure virtual parent; the compiler repair and
two regressions pass, including a nearer abstract declaration hiding an older
concrete method. The frozen repaired SDK passes the full five-case matrix and all
three byte-identical standalone exports. Unused Basic remains at 983,160 RAM /
436,224 EXE bytes, with zero size delta. Player, guard and boss health now use
the shared values in source. Fresh Complete acceptance and the full workspace
pass as recorded above. Six independent checks confirm the economy and fresh-install
cards match their saved originals or remain absent. Geometry and all 67 packages
remain byte-identical. Four verified temporary SDK object caches were removed,
recovering 21,254,592 file bytes; paths are in `health/sdk-temporary-cleanup.json`.
Evidence is under `artifacts/gameplay-helpers/p8/health/`; the arithmetic
prototype remains preserved in the adjacent `health-contract/` directory.

Build a shared gameplay foundation and a library of configurable actors and components for rapid prototyping. Deliver interaction and dialogue first, then refactor every shipped template to use the foundation. All six templates must offer C++, Blueprint and Lua gameplay, including Dungeon Complete.

The final user workflow is to place an examinable object or NPC, assign a conversation and an editable dialogue widget class instance, and interact with it using a reusable player. Users can change defaults, derive classes or replace components without copying the underlying systems.

## Required outcomes

1. Interaction, dialogue, character movement and common helpers have one engine implementation and a reflected API available to all three authoring providers.
2. Each template writes its project gameplay in the selected flavor. Blueprint and Lua versions contain editable game rules and do not delegate those rules to an undisclosed copy of the original C++ game.
3. Shared assets, scene composition, tuning and behavior agree across flavors. Authoring format and its generated artifacts may differ.
4. Helpers can be placed in Basic projects and existing current-format projects independently of the template originally selected.
5. Templates remain useful starting points: characters, interaction and dialogue are configurable, references survive renaming and duplication, and component budgets leave room for extension.
6. Play, Build and standalone export use the same source contracts, resource discovery and failure behavior.

### Template completion matrix

Every cell is required before the initiative is complete. The table describes the target, not current availability.

| Template | C++ | Blueprint | Lua | Project gameplay owned by the chosen flavor |
| --- | --- | --- | --- | --- |
| Basic | Required | Required | Required | An inert `GameRoot` entry class for future rules; the scene remains visually empty apart from its camera |
| Sample | Required | Required | Required | The small Spinner behavior |
| Third Person | Required | Required | Required | Player rules and extensions over shared movement, animation and camera helpers |
| Platformer 2D | Required | Required | Required | Platformer rules, collection and restart behavior over the existing 2D body |
| Dungeon | Required | Required | Required | Key ownership, lock requirements and dungeon progression over shared helpers |
| Dungeon Complete | Required | Required | Required | Combat, encounters, game flow, economy, inventory decisions, terminal flow and progression |

This produces exactly 18 required authoring/execution combinations: six templates multiplied by C++, Blueprint and Lua Native C++ AOT. Lua VM bytecode/source remain engine features with separate regressions; they are not template acceptance requirements for this delivery.

Basic gains an ordinary, inert `GameRoot` actor class authored in the chosen flavor. It does not gain a player, an active dialogue or a demonstration level. Helper acceptance scenes remain separate fixtures.

The PocketStation miner remains the same Pocket Lua asset in every Dungeon Complete variant. Pocket Lua targets a separate device and is independent of the PSX game's chosen authoring flavor. The PSX rules that start a transfer, award ore, purchase items and advance progression belong to the selected flavor.

## Existing implementation and constraints

The table records the P0 starting point. The phase results below describe the
delivered changes and current availability.

| Current source | Consequence for this initiative |
| --- | --- |
| [`src/workspace.rs`](../../../src/workspace.rs), [`src/project_templates.rs`](../../../src/project_templates.rs) | Six templates; Dungeon Complete currently rejects Blueprint and Lua, and Basic currently emits no gameplay source |
| [`src/third_person.rs`](../../../src/third_person.rs), [`templates/ThirdPersonController.cpp`](../../../templates/ThirdPersonController.cpp) | One controller combines input, movement, animation and camera; its C++ variant resolves Visual and Camera by display name |
| [`src/platformer_template.rs`](../../../src/platformer_template.rs), [`runtime/world2d_components.hpp`](../../../runtime/world2d_components.hpp) | The 2D character body already exists and must be reused rather than reimplemented |
| [`src/dungeon_template.rs`](../../../src/dungeon_template.rs), [`src/dungeon_gameplay.rs`](../../../src/dungeon_gameplay.rs) | Dungeon builds on Third Person; keys and doors are copied project classes in the selected flavor |
| [`src/dungeon_complete_template.rs`](../../../src/dungeon_complete_template.rs), [`resources/templates/dungeon-complete`](../../../resources/templates/dungeon-complete) | The complete game is bundled as a project copy with tightly connected native gameplay |
| [`runtime/object_model.hpp`](../../../runtime/object_model.hpp), [`src/scene_bank.rs`](../../../src/scene_bank.rs) | Runtime bounds are 64 actors per level and eight components per actor; the complete game starts at 64 actors in P0. P8 sound consolidation reduces fresh templates to 58 without raising either limit |
| [`runtime/input.hpp`](../../../runtime/input.hpp), [`src/controls.rs`](../../../src/controls.rs) | Runtime button state exists; host keyboard and mouse mappings already feed virtual pads; gameplay action routing is a separate concern |
| [`runtime/text.hpp`](../../../runtime/text.hpp), [`runtime/hud_core.hpp`](../../../runtime/hud_core.hpp) | Text has a 511-byte payload and UTF-8 rendering, but no reveal counter; HUD has explicit layout and primitive budgets |
| [`src/localization.rs`](../../../src/localization.rs), [`src/hud.rs`](../../../src/hud.rs) | Resource discovery currently follows authored text and font references; conversation-only dependencies must be added |

Follow the [actor-only contract](../actor-architecture/actor-only.md) and [gameplay API parity contract](../gameplay-api-parity/contract.md). Actors and ActorComponents remain the authoritative model. Do not introduce another entity collection, class catalog, script lifecycle or runtime graph VM.

At the planning baseline, maps use version 6, Blueprint assets version 5, reflection schema 10 and Lua Gameplay profile 2. Use the source constants as authority during implementation. Add new asset kinds with their own versions; change existing schemas only when their serialized contract actually changes.

## Architecture and ownership

| Layer | Owns | Extension boundary |
| --- | --- | --- |
| Engine core | Actor identity, transforms, collision, rendering, input sampling, HUD, assets, reflection and script compilation | Existing engine APIs |
| Gameplay base | Action routing, control capture, interaction contracts, dialogue execution, character motors, camera and animation mechanisms | Native reflected classes, methods, values and events |
| Game helpers | Ready-to-place triggers, examinable objects, speakers, player presets, pickups, doors and dialogue presenters | Derived classes, component composition and editable configuration |
| Project gameplay | Game-specific conditions, rewards, combat rules, item prices, encounter transitions, objectives and authored content | C++, Blueprint or Lua selected for that template |

Native services may perform expensive or device-specific operations, including collision queries, projectile storage, save transport and PocketStation transactions. Their parameters and observable results must be generic. A service that contains Reactor Station's encounter sequence, item prices or reward decisions is still project gameplay and must be expressed in each selected flavor.

Project data such as item definitions, encounter tables, save schemas and dialogue assets may be shared across flavors. Generated C++ data and engine runtime code are normal build artifacts. A Blueprint or Lua project must not ship an authored `DungeonGame.cpp` that owns its gameplay decisions.

### Proposed class and service contracts

Names below are proposed API names. Phase 0 fixes their signatures and assigns stable identities before implementation.

| Surface | Responsibility |
| --- | --- |
| Gameplay action service | Map logical actions to the existing pad input, select contexts and expose consumption consistently |
| Control capture token | Capture selected actions for a particular owner, level and input port; release safely on every lifecycle path |
| `InteractorComponent` | Maintain bounded candidates, select a focus target and request interaction |
| `InteractableComponent` | Availability, prompt key, priority, interaction request and reflected extension events |
| `TriggerBox3D` and `TriggerArea2D` | Placeable presets over existing collider triggers with typed target bindings |
| `DialogueInteractionComponent` | Derive from the interaction contract and start an assigned conversation |
| `DialogueAsset` | Stable conversation nodes, participants, localized text keys, branches and typed action bindings |
| `DialogueStyle` | Optional shared timing and sounds; widget controls own all appearance |
| Dialogue session service | Execute a conversation, own its state and report completion or cancellation |
| `DialoguePresenter` | Derivable dialogue widget over UIActor, with a class-owned editable control hierarchy and dialogue API |
| `Character3D` and `Character2D` | Minimal character compositions with compatible body and movement contracts |
| Player and NPC presets | Convenient character configurations that select input or AI command sources |
| Camera rig presets | First person, third person, top down and isometric camera actors over a shared rig base |
| `SpringArmComponent` | Configurable camera boom with collision retraction, target offsets and optional smoothing |

Character3D derives from Actor3D, Character2D from Actor2D, and DialoguePresenter from UIActor. Shared gameplay components use the appropriate owner domains. No class inherits both spatial domains. Player control is a role; changing the command source must not require replacing the character motor.

### UI class authoring decision (user correction, 30 October 2026)

Presentation is authored as a reusable widget class with child controls, exposed properties and its own C++, Blueprint or Lua behavior. Build this on UIActor and the existing actor/component/template model. Extend the generic class/UI editor with a hierarchy, canvas preview and normal inspectors; do not create a dialogue-specific visual designer, separate widget runtime hierarchy or second class catalog. Generalize composition authoring across the three providers rather than tying editable UI trees exclusively to Blueprint assets.

The dialogue widget base reuses the session/handle API and the DialoguePresenter implementation. It supplies bound text, reveal/choice state, navigation/advance/cancel operations and extension events. Authored positions, anchors, sizes, fonts, panels and child-control composition remain authoritative during Play. The implemented parent present() updates content, visibility and focus while preserving the user's control geometry, fonts, colors and panel material. Cook pagination and HUD budgets measure the actual widget controls and selected font, including class defaults and supported instance overrides.

Classic Box, Portrait Conversation, Subtitles and Speech Bubble ship as normal editable widget classes/compositions over the common dialogue base. Optional shared timing/sound assets use the ordinary inspector. All appearance belongs to the widget controls. Conversation assets retain the separate content editor for lines, choices, conditions, actions and localization. The dedicated Dialogue Style window added during P4 exploration was removed after this correction; its screenshot is historical, superseded evidence.

`UIActor` is also the common widget base for UI unrelated to dialogue. It owns child controls and class behavior in the selected provider. `DialoguePresenter` specializes that base with dialogue bindings and events; its project subclasses use the same hierarchy, preview, inspectors and Behavior action. The four starters are editable starting compositions, and do not restrict custom widget classes.

Rechecked 6 November 2026: four focused `dialogue_widget` editor tests and the three native host suites (`dialogue_widget`, `dialogue`, `hud_reveal`) pass against the current source. See the [scoped architecture recheck](p4-acceptance.md). P8 remains in progress; this recheck does not change the remaining phase gates.

Generic UI class preview resolves authored defaults without running arbitrary project code. Dialogue sample states use the same binding/layout contract in that preview. Explicit Play validates the selected provider's real behavior. Inheritance, reusable child widgets, duplication, typed control references and save/reopen must work across C++, Blueprint and Lua AOT. Existing VM widget coverage remains engine regression evidence. This replaces the earlier assumption that style-specific geometry plus a specialized property window was sufficient.

Native bases are Blueprintable and extensible unless a specific invariant requires a final member. Public configuration uses reflected typed properties. Events, condition queries, methods and Call Parent behavior must work through the existing provider rules. Use native parents for APIs shared by all three providers; arbitrary cross-provider inheritance is not a requirement of this initiative.

References use stable actor, component, asset, class and member identities. Shared helpers cannot resolve required bindings using display names, source filenames or hard-coded scene array positions. Class defaults, derived defaults and placement overrides keep the existing inheritance rules.

### Initial bounded runtime profile

These are proposed starting defaults to validate in Phase 0, with limits visible in host configuration and cook diagnostics:

- One active modal dialogue per level, recording the initiating input port.
- Eight interaction candidates per interactor, with deterministic selection and explicit overflow statistics.
- Four visible choices per page, with pagination when authored content exceeds that display capacity.
- A maximum of 32 immediate dialogue node transitions per advance operation, with an explicit error on an invalid non-yielding loop.
- Shared presenter storage and fixed pools; no heap allocation per NPC, line or interaction request.

Keep the existing 64-actor and eight-component bounds for the initial implementation. Account for roots, render components, sensors, UI children and runtime spawn reserves. Target at least two available component slots in shipped player presets; resolve excessive bookkeeping through shared services or tighter composition before considering a capacity change.

Unused helpers must not register polling callbacks or reserve dialogue, camera or projectile pools. Code may be staged with the runtime, but linked state and resources must follow actual use. Report the unused-feature delta against Basic.

## Phase sequence and milestones

| Phase | Deliverable | Depends on | Milestone |
| --- | --- | --- | --- |
| P0 | Baseline, ownership contracts and acceptance matrix | Current repository | Scope and budgets fixed |
| P1 | Gameplay actions and scoped control capture | P0 | Shared controls usable from all providers |
| P2 | Interaction and trigger helpers | P1 | Place an object and interact in 2D or 3D |
| P3 | Dialogue assets, compiler and session runtime | P1, P2 | One conversation works end to end |
| P4 | Dialogue editor, four styles and complete authoring workflow | P3 | First usable interaction and dialogue release |
| P5 | Shared character, camera and locomotion mechanisms | P1; after P4 in delivery order | Reusable player and NPC foundation |
| P6 | Helper catalog, pickups, doors and switches | P2, P4, P5 | Helpers available in every project |
| P7 | Five starter templates on the common foundation in three flavors | P5, P6 | Basic, Sample, Third Person, Platformer and Dungeon migrated |
| P8 | Dungeon Complete decomposed and refactored in C++ | P7 | Complete game's common mechanisms separated from rules |
| P9 | Dungeon Complete Blueprint and Lua implementations | P8 | All 18 primary combinations implemented |
| P10 | Full execution matrix, exports, budgets and documentation | All previous phases | All 18 execution combinations accepted |

P5 does not block the first dialogue release. P1 adds narrow integration points to the current controllers so movement and camera obey input capture while the full controller extraction remains scheduled for P5.

## Phase 0 Baseline and implementation contracts

Progress: completed as the baseline/contract gate. Eighteen generation cases, ten linked builds, real compiler contract probes, native host tests and representative runtime captures are recorded in [p0-baseline.md](p0-baseline.md). The existing complete-game performance failure is preserved, not marked as passing.

**Work**

1. Record current project generation, role bindings, assets, controller tuning, runtime bounds and source hashes. Separate actor/component source versions from historical documentation.
2. Produce a subsystem ownership table for Dungeon Complete. Mark each item as an engine mechanism, shared authored data or selected-flavor game rule. Include targeting, combat, inventory arithmetic, save serialization, UI, PocketStation transport and reward confirmation.
3. Define the reflected action, interaction and dialogue signatures, lifecycle order, cancellation reasons, asset schemas, limits and stable IDs. Check bool condition overrides and typed records against both the Blueprint compiler and Lua ABI before fixing their shapes.
4. Define semantic roles in template manifests so the generators can bind Player, Visual, Camera, dialogue participants and door targets without name searches. Keep role identity separate from each flavor's class identity.
5. Establish deterministic scenarios and event traces for the current starters and complete game. Normalize intentionally different class IDs and authoring artifacts when comparing flavors.
6. Capture baseline executable size, static and peak RAM, stack headroom, VRAM, HUD counts, actors, components, native frame time and dropped/rejected counters. Include explore, combat, boss, terminal and restart states.
7. Verify Dungeon Complete gameplay RAM for all three required authoring flavors, including Lua AOT. Preserve the full game; VM arena investigations are separate engine work following the 6 November scope correction.

**Primary touchpoints:** `workspace`, template generators, `object_model`, `reflection_schema`, `gameplay_api`, `script_backend`, existing gameplay parity and Dungeon Complete integration tools.

**Acceptance**

- The 18 required authoring/execution cases have named scenarios and expected results. Earlier additional VM cases remain preserved engine evidence.
- Every planned reusable system has a clear ownership boundary and public contract.
- Baseline measurements have matching source, executable, configuration and instrumentation identifiers; failures are recorded separately from passing evidence.
- Template generation and helper contracts account for all current actor and component limits.

## Phase 1 Gameplay actions and control capture

Progress: complete and validated. The native router, 22 reflected operations, runner integration and current-controller adapters are delivered. Host lifecycle tests, Native Play/HUD consumption, 1,186 workspace tests, fourteen final template builds, all five provider surfaces and the byte-identical standalone rebuild pass. Runtime snapshots preserve the existing Complete 25 ms failure as an open performance gate. See [P1 delivery and costs](p1-delivery.md). Platformer Lua source has limited headroom; P7 must remeasure it with helpers before migration is accepted.

**Work**

1. Add a native gameplay action layer above the existing pad state. Start with Move, Look, Jump, Interact, Confirm, Cancel and Pause; allow project action definitions for combat and menus.
2. Keep host keyboard, mouse and controller mapping in `controls.rs`. Gameplay action bindings consume the resulting pad state and do not duplicate host device handling.
3. Resolve context priority before gameplay dispatch. Define how press edges are consumed across a rendered frame with zero, one or several simulation steps; one physical press must not confirm, skip and jump together.
4. Add bounded capture tokens associated with a live owner, level generation and input port. Support nested captures and out-of-order owner destruction. Release on close, cancellation, disable, destruction, restart and scene unload.
5. Separate voluntary control from physics. Capturing movement stops new movement commands while gravity and collision remain active. Camera capture is an independent option. A modal dialogue does not pause the world by default.
6. Define UI progression on the existing frame callback when simulation is paused, without advancing the same session through both frame and simulation callbacks.
7. Add narrow action-routing integration to current Third Person, Platformer and Dungeon Complete control entry points. Keep their movement algorithms intact until extraction.
8. Publish the same operations and typed token values to C++, Blueprint and Lua Gameplay v2, including VM modes, through native reflection.

**Primary touchpoints:** new `runtime/gameplay_control.hpp`, `runtime/input.hpp`, `runtime/gameplay_api.hpp`, runtime lifecycle, current controllers, registry generation and Lua definitions.

**Acceptance**

- A menu capture prevents the same Cross press from also jumping or reopening an interaction.
- Releasing one capture does not release another owner's capture.
- Destroying a captured object or switching scenes cannot leave input locked.
- Paused and catch-up frames preserve the input contract in all providers.
- Raw input remains available; capture guarantees apply to routed actions, and shipped helpers use routed actions consistently.

## Phase 2 Interaction and trigger helpers

Progress: complete and validated as the interaction foundation. The native kernel, bounded candidate selection, 2D/3D sensor adapters, frame-input routing, typed actor/component references and demand-loaded helper factories are implemented. Native Trigger/Prompt compositions are available through existing placement/component authoring. Object/NPC role fixtures exercise the same extensible target contract in C++, Blueprint and Lua in both dimensions. The complete conversation authoring/presentation release remains P4.

Validation findings (29 October 2026): an unused Basic build initially activated World2D through the new SDK classes. Correcting SDK demand detection reduced static RAM from 1,206,104 to 982,832 bytes. Gating the unused declarations also removed speculative virtual-call linkage: Basic now matches P1 byte for byte, with 964,400 static RAM bytes, a 421,888-byte executable and SHA-256 `bd69708d3bc1442f9828caf90b1f5c16fb068a35224810ba3312a28a682d0b0e`. Logs: `artifacts/gameplay-helpers/p2/basic-build.log`, `basic-build-after-demand-fix.log` and `basic-build-after-header-gate.log`.

The first real provider PSX build exposed a compiler defect: Blueprint/Lua generated a non-const override for the const pure condition. Both emitters now preserve const qualification, and both frontends reject mutation/effectful calls in pure bodies. Two targeted compiler tests pass. The provider fixture now executes successfully on PSX in all five surfaces (C++, Blueprint, Lua Native C++, VM bytecode and VM source), in both spatial domains, proving Boolean parent queries, denied requests, parent start events and exactly-once completion. Static RAM for this six-target, twelve-actor instrumented fixture is 1,292,296 / 1,599,224 / 1,660,664 bytes for the three Lua execution modes respectively; these are whole-fixture costs, not isolated helper deltas. Exact executable/editor/runtime hashes and probe values are recorded in `artifacts/gameplay-helpers/p2/providers/result.json`. The earlier export-only P0 probe did not establish target execution of this case.

The final `interaction_` Rust run passed 13 tests (three existing ignored tests), including actual 2D/3D trigger physics: Enter registers focus, Cross interacts once, moving away clears focus and a later press cannot interact. The expanded editor-path test passes placement of native Trigger/Prompt compositions, typed references after duplicate/rename/reparent/save/reopen, prompt-only localization dependencies and HUD budget rejection. Native Play draws the shared prompt, switches between seven- and eight-glyph translations and reports zero dropped primitives while preserving input consumption.

Standalone export rebuilt byte-identically in Lua VM source mode (`artifacts/gameplay-helpers/p2/providers/standalone-result.json`). The trace-enabled pure Blueprint subclass also passes MIPS syntax checking. Additional checks pass: 41 Blueprint compiler tests; 76 Lua tests (five existing ignored); two pure-override regression tests; host interaction/control tests; Clippy with warnings denied; formatting; API-reference freshness; parity inventory; forbidden-term and whitespace checks. These targeted results do not replace the required final P10 execution matrix or the P1 workspace baseline.

Sensor binding diagnostics, monitoring/disable behavior, binding replacement, pure-condition reentry and destruction have passing host tests. The API catalog and parity inventory include the new interaction module; usage is documented in `docs/input-collision.md`. The provider project under `artifacts/gameplay-helpers/p2/providers/project` is an instrumented acceptance fixture; polished conversation examples and styles are P4 work.

Build-cache review: removed 20 obsolete editor dependency artifacts from the canonical `target/debug/deps` directory after verifying their paths and that no matching build/process was running. Recovered 2,697,241,714 bytes (2.512 GiB); retained the current editor, tool and test binaries. No additional Cargo cache was created.

**Work**

1. Implement InteractorComponent and InteractableComponent with editable prompt key, enabled state, priority, range and domain-compatible references.
2. Add candidate registration from trigger events and bounded proximity queries. Select by explicit priority, distance and a stable tie breaker. Revalidate availability at confirmation time.
3. Define synchronous conditions and interaction results: accepted, unavailable, out of range, busy, missing binding and invalid owner. Provide reflected events for focus gained, focus lost, interaction accepted and completion where applicable.
4. Implement TriggerBox3D and TriggerArea2D as ordinary actor compositions using existing Enter, Stay and Exit dispatch. Filter initiators explicitly so a projectile or unrelated actor does not activate a player-only helper.
5. Keep physical collision independent from detection. For a solid chest or NPC, use a range query by default or an explicit sensor actor linked to its target; report any sensor's actor cost.
6. Add a reusable prompt presenter with localization and controller-port ownership. A prompt is shared presentation state rather than an actor created for every interactable.
7. Add placeable examinable-object and interactable-NPC fixtures in both dimensions. Expose attachment through Add Component and placement through the actor picker before the full catalog arrives.

**Primary touchpoints:** new `runtime/interaction.hpp`, reflected classes, collider adapters, actor defaults, scene authoring, HUD and asset bindings.

**Acceptance**

- Two nearby interactables select consistently, and a disabled or destroyed target immediately loses focus.
- Rename, duplicate, reparent, save/reopen and two independent instances preserve the correct bindings.
- C++, Blueprint and Lua can override interaction behavior and call the parent implementation.
- Candidate overflow, trigger exits, scene transitions and destruction during an interaction produce defined results without stale callbacks.
- The same helper workflow works on Actor3D and Actor2D owners with domain validation.

## Phase 3 Dialogue assets and session runtime

Progress: complete and validated as the asset/session/helper foundation (30 October 2026). Version-1 Dialogue/DialogueStyle packages are registered in the existing asset system. The schema covers Line/Choice/Branch/Event/End nodes, participant roles, symbolic text keys with explicit String Table UUIDs, four presentation identities and reflected condition/action bindings. Internal-ID duplication preserves external resources; atomic saves check the existing asset revision. The host compiler canonicalizes IDs, detects compact-ID collisions and validates real reflected callback signatures before producing typed node/role/binding tables.

The bounded native session kernel is implemented with modal capture, reveal, language switching, four visible choices per page, condition revalidation, exactly-once entry/actions and a 32-transition limit. Destruction, deactivation, scene reset and lost captures cancel safely; opening and closing input edges are consumed. Frame updates use unscaled microseconds in PSX, Native Play and HUD runners. The session measures 256 bytes on the host and 240 bytes with the MIPS compiler. Optional text reveal uses existing alignment padding: Text remains 528 bytes on both targets. These are structure sizes, not measured total or peak runtime costs.

The host cook now emits immutable C++ conversation/style/page tables and typed reflected callback glue. Play, PSX staging and standalone export select these through typed asset references. Discovery follows all node text tables, role portraits, style fonts/panels/sounds and records their revisions, including dependencies absent from the first displayed line. Long UTF-8/color text is paginated with the selected font's advances; choices and participant names must fit their labels or cooking fails explicitly. Pageable String Tables permit 65,535-byte entries and preserve CSV/JSON round trips without replacing the legacy HUD table. Locales are matched by trimmed, case-insensitive names; missing, extra or ambiguous columns produce diagnostics. The project HUD table, or the project's lowest-UUID String Table when no HUD table exists, defines indices consistently across scene selections. A language-only table supplies the runtime language selector when the scene has no HUD keys.

Seventeen targeted dialogue tests pass, including asset/schema/callback validation, long UTF-8/color pagination against the actual HUD compiler, table round trips/stale saves, resource discovery/invalidation, stable language indices across scene selections, capability selection and Native Play execution from real Dialogue/Style assets. Native Play confirms progressive text continues while game time is paused, typed Branch/Event callbacks execute and closing does not trigger Jump. Additional runner regressions pass: eight HUD simulation tests (one ignored) and eight Native Play tests (four ignored). The native dialogue/reveal/interaction/control suites, Clippy with warnings denied, formatting, three repository-policy tests, compiler-contract probes, generated API freshness and parity inventory also pass. The kernel MIPS syntax/size probe remains in `artifacts/gameplay-helpers/p3/kernel-mips-syntax.log` and `kernel-mips-sizes.s`.

The earlier workspace regression passed 1,208 tests, with 88 ignored (144.83 seconds reported). The newer final regression is recorded below. Neither replaces the final template execution matrix or runtime budget gates. Consolidated delivery source hashes, test counts and remaining initiative scope are in `artifacts/gameplay-helpers/p3/validation.json`.

The PSX cooked-native acceptance fixture executes all five node kinds from packaged assets: a long localized/color line over five pages, six choices with a second choice page, a pure condition, one action, exactly-once entry and capture cleanup. It reports a 240-byte MIPS session and 528-byte Text. Its whole-fixture static RAM is 1,035,376 bytes; executable SHA-256 is `6c2aa8a4ccb6824706c9ffd46ce4ebdd67a00774b8f216d29434d8e18a2cab2f`. Standalone export rebuilds byte-identically. The instrumented driver is native C++; this is not provider-helper acceptance. Commands, probe words, source/executable hashes and logs: `tests/integration/verify_dialogue.py` and `artifacts/gameplay-helpers/p3/cooked-native/`. The harness checks exclusive ownership of its emulator port and cleans its own process tree on failure.

A fresh unused Basic build still matches P1 byte for byte: 964,400 static RAM bytes, 421,888 executable bytes and SHA-256 `bd69708d3bc1442f9828caf90b1f5c16fb068a35224810ba3312a28a682d0b0e` (`artifacts/gameplay-helpers/p3/basic-unused-result.json`). The canonical debug Cargo cache was reviewed at 18.4 GiB with 139.77 GiB free; current reusable outputs were retained and no additional cache was created.

Continuation (30 October): the reflected `DialogueSessionLibrary`, generation-checked handles, `DialogueInteractionComponent`, `DialogueRoleBindingsComponent` and reusable `DialoguePresenter` are implemented. The eight session operations expose registered values of one, two and sixteen wire words. A stale handle cannot advance or cancel a newer session. Roles use a separate component with eight fixed bindings, preserving the existing sixteen-property class limit and extension room on the interaction helper. Placing a presenter through the actor picker materializes ordinary UI children with typed references; its default composition costs nine actors and nineteen components (three on the root, two per child). Duplication remaps bindings; placement that would exceed the actor capacity rolls back atomically.

Native Play acceptance now uses those public helpers: Cross opens the conversation, reveal completes before advance, six options span two pages, completion/cancellation/disable release capture, and a later Jump works. The actual reusable UI reports zero dropped primitives. The cook runs the same native HUD compiler over conservative line/choice content and rejects an explicitly undersized glyph budget. An initial test exposed a stats-index mistake (images versus dropped primitives); the checker and text pagination assertion now inspect the dropped counter. Screenshots and the focused workflow transcript are in `artifacts/gameplay-helpers/p3/native-ui/` and `native-helper-workflow.log`.

Cooking now selects each helper instance's actual conversation/style pair, including a built-in default alongside explicit styles. A regression proves an unrelated portrait style no longer invalidates a classic-only choice label; opaque consumers conservatively retain both default and selected explicit resources. The latest focused Rust run passes twenty-two dialogue tests; host C++ dialogue tests also pass with stale handles, all eight role slots, immediate-End event ordering and source deactivation during opening. The earlier seventeen-test/workspace/PSX measurements above remain historical evidence; a fresh final regression and new target hashes are still required for these public headers.

The public provider fixture now executes successfully on PSX through C++, Blueprint, Lua Native C++, VM bytecode and VM source in both spatial domains: independent mappings of one conversation, pure condition/action parent calls, start/snapshot/advance/cancel operations, pagination, stale handles and presenter loss. The first VM compilation exposed an ABI defect: a one-word record was treated as a scalar. Compiler dispatch now preserves aggregate shape independently of word count; runtime value metadata uses existing alignment padding, and nested one-word members remain records. The focused compiler regression and both VM target runs pass. The fixture's manually authored Canvas/Audio initially duplicated the presenter's inherited components; it now references the class defaults correctly.

Final acceptance passes the expanded target fixture in all five provider surfaces and both dimensions, including a second String Table referenced only after a Branch/Event path. Each of its six independent targets completes one conversation, cancels a newer session while rejecting stale handles, then loses a required presenter label and recovers capture. Whole-fixture static RAM is 1,377,688 bytes for Lua Native C++, 1,696,904 for VM bytecode and 1,756,296 for VM source; the fixture contains 22 actors, 51 components and all three authoring providers. These are acceptance-fixture costs, not isolated dialogue costs. Exact probe words, runtime/editor/executable hashes and logs are in `artifacts/gameplay-helpers/p3/providers/result.json`.

The VM source standalone export rebuilds byte-identically (SHA-256 `a0c12ae430ec8fca545c5b6457806b6c042c2917b32d05e4e3835cf6777c13d7`). A fresh unused Basic build remains identical to P1: 964,400 static RAM bytes, 421,888 executable bytes, SHA-256 `bd69708d3bc1442f9828caf90b1f5c16fb068a35224810ba3312a28a682d0b0e`. Both were built with editor SHA-256 `335368c44f1f0e7e98d619ab364ebe315f19187774366010a30c8a08b8bb1d52`. Evidence: `providers/standalone-result.json` and `basic-unused-result.json`.

The final workspace regression passes 1,213 tests across three nonempty suites, with 88 ignored (177.44 seconds reported). The four host runtime suites, 22 focused dialogue tests, Clippy with warnings denied, formatting, generated API freshness, parity inventory and five parity-tool tests pass. The API inventory contains 769 public gameplay declarations and zero pending classifications. Logs: `workspace-tests-current.log` and `host-runtime-current.log`; screenshots: `native-ui/`. The canonical target/debug cache was reviewed at 18.54 GiB with 141.34 GiB free; current reusable outputs were retained, with no additional build cache.

Historical pre-P4 handoff: P4 still owned the complete authoring UI and four finished presentation styles; cooked identities/layout data alone were not a finished style release. P4/P5 were subsequently accepted below. Remaining template work now follows the 18-case scope correction above.

**Work**

1. Add versioned DialogueAsset and DialogueStyle packages to the existing asset registry. Conversation nodes initially cover Line, Choice, Branch, Event and End; automatic timing is a line policy.
2. Give conversations, nodes, options, participants and condition/action bindings stable identities. Resolve participant roles to scene actors or components when starting a session, so a reusable conversation contains no fixed scene-object identity.
3. Store text as localization keys and speaker metadata as data. Validate every destination, condition signature, event binding, translation, font glyph and portrait reference. Diagnose missing or ambiguous resources with asset and node context.
4. Cook conversations into bounded native tables and typed dispatch glue. Conditions and events use declared reflected signatures and IDs; no arbitrary expression evaluator or runtime name lookup is added.
5. Implement session states for revealing, waiting for advance, selecting, executing, completed, cancelled and failed. Define reentry policy, busy responses, exactly-once node entry events and safe cancellation when callbacks destroy their owner.
6. Integrate action capture. Opening a conversation consumes the initiating edge; the first confirm while revealing completes the line, and a later press advances it. Closing consumes the final edge before returning control.
7. Add a reveal count to text drawing while preserving layout from the complete string. Count the supported Unicode scalar/glyph boundary without splitting UTF-8 sequences or color tags. Use glyph advances and actual font metrics for page breaks.
8. Enforce the 511-byte text payload per displayed page in every translation. Pagination must report or handle a single overlong token and cannot silently truncate a localized line.
9. Add dependency traversal from dialogue and style assets to strings, fonts, portraits and audio. Include their signatures in preview, script capability selection, scene banks, invalidation and standalone export.
10. Deliver a minimal classic presenter and DialogueInteractionComponent so the asset path works end to end before building the full editor.

**Primary touchpoints:** new `src/dialogue_asset.rs`, `src/dialogue_compile.rs`, `runtime/dialogue.hpp`, `assets`, `localization`, `hud`, `text`, dependency tracking, `project`, `scene_bank` and export staging.

**Acceptance**

- A chest and an NPC can start the same conversation with independent role bindings.
- C++, Blueprint and Lua start, inspect, advance and cancel sessions and implement condition/action hooks.
- A resource used only by a later dialogue branch is present in a fresh PSX build and standalone export.
- Accented text, imported fonts, markup, long translations, choices with no available options and malformed loops have tested behavior.
- End Play, scene unload, source/presenter destruction and reentrant callbacks cannot retain capture tokens or pointers into an unloaded bank.

## Phase 4 Dialogue editor and presentation styles

Progress: complete and validated. The [requirement audit](p4-acceptance.md) records all eight implementation requirements and six acceptance gates. Final evidence is `artifacts/gameplay-helpers/p4/release-validation.json`: 1,258 workspace tests / 88 ignored, nine widget-provider/Lua-execution cases, 162 sessions in both spatial domains, twenty console 2D-anchor checks, byte-identical exports and generic usage reporting. P5 is enabled. Earlier implementation snapshots follow for traceability.

Progress: in progress, with the UI-class correction above now authoritative. Conversation content editing includes a graphical canvas with pan/zoom/framing and mouse-driven links, the five node kinds, localization/participant/callback pickers, undo/redo, protected removal and independent duplication through Project. The graph and details edit the same document; callback choices use the cook's signature predicate. Save All saves source/Blueprint edits before validating a dirty conversation. Thirty focused tests passed before removing the superseded style window (that window contributed one test). A real conversation-editor screenshot exists in `artifacts/gameplay-helpers/p4/conversation-editor.png`. Generic class authoring, initial editable presets and authored-layout binding are implemented below. Anchored bubbles, complete conversation preview and final authoring/style acceptance remain pending.

The [widget class composition implementation](p4-widget-classes.md) shares the existing Template across C++, Blueprint and Lua, with source-owned composition companions retaining the registered class UUID. The resolver, placement, linked refresh, scene reopening, Blueprint validation/reset and console template preparation use those same layers; the generic UI class designer shares component tools and renders authored defaults through the HUD compiler. The preceding foundation regression passed 1,226 tests with 88 ignored across three nonempty suites (editor suite 167.31 seconds). Provider tests cover independent instances, typed control bindings, inheritance, native defaults, save/reopen, normal catalog recompilation and cooking. A pixel regression verifies authored foreground/background order independently of UUID ordering. Three focused tests also pass with canonical Windows project paths. Nested reusable composition is now implemented and undergoing its own acceptance; full authoring/style acceptance remains required.

The generic designer previews the same authored class used by placement and spawning. Project > Create > Dialogue Widgets generates all four starters in C++, Blueprint or Lua; their controls use ordinary anchors, properties and Focusable components. Empty custom widget hierarchies remain empty on placement. Source and graph editing share the class identity and composition. The updated starters cost 12 actors / 29 components; Speech Bubble costs 13 / 31 with its normal container and anchor. Their separate choice popup is an ordinary Image control. Timing assets contain timing and sounds only; all appearance belongs to the widget controls.

The presenter updates content, transient visibility, portraits and focus while retaining authored geometry, fonts, colors, wrapping and materials. Body and optional prompt controls paginate independently; an optional continuation Text retains its authored content and appearance. `dialogue_opened` and `dialogue_closed` expose the 16-word session snapshot to derived classes. Cleanup releases capture, preserves terminal identities, closes once through invalidation/destruction and permits a derived close animation. Unbound optional names impose no invisible font requirement. Native Play stages spawn compositions/resources and includes their component density in registry capacity. Render-only line/prompt/choice/closed samples use the generic designer; complete localized runtime preview remains pending.

The preceding widget-state milestone passed 1,231 tests and 162 console sessions across nine provider/execution combinations. It also covered 12 CLI-created classes, 32 inspected placeholder previews and a byte-identical standalone rebuild. Its exact sources and executable remain recorded in `artifacts/gameplay-helpers/p4/widget-state-validation.json`; the prior authored-layout evidence remains in `widget-layout-validation.json`.

The preceding focus/projection milestone passed 1,233 tests with 88 ignored, native host regressions, Clippy, formatting, the debug build and API freshness/parity checks. Ordinary Focusable controls gained an editable outline; eight inspected captures showed choice focus at both resolutions while retaining text colours. Bound inactive prompts preserve their layout. Shared Q12 camera projection aligns console/Native Play clipping, verified through 16 API cases. A Lua VM-source widget passed 18 sessions, actual contour rendering and cleanup. Its fixture now authors spatial attachment and hidden Text data explicitly. The standalone rebuild was byte-identical. Basic measured 966,448 static bytes and a 423,936-byte executable after the earlier shared-core increase. Its source provenance is preserved in `artifacts/gameplay-helpers/p4/widget-focus-validation.json`; the nine-case provider matrix remains historical evidence for its own recorded sources.

The preceding anchoring milestone passed **1,235 workspace tests / 88 ignored** (editor suite 209.40 seconds), host regressions, formatting, Clippy and the debug build. An independent optional UI component moves a normal child container while preserving controls; host tests pass 2D/3D cameras, rotated bounds, fallback, lifecycle and capture release. Native Play spawns the bubble in all three providers with authored pagination. The console fixture passes 18 sessions, eight independent 3D anchor checks and direct Lua VM-source calls to all four anchor methods, including enum returns. Existing projection/focus/cleanup probes pass. The instrumented fixture uses 1,828,288 static bytes and its standalone rebuild is byte-identical. Unused Basic remains byte-identical to the preceding image. API parity now includes the anchor surface (789 public / 5,849 total / zero pending classifications). All 12 CLI-created classes preserve their documents and reject duplicate creation. Eight inspected choice captures cover all starters at both resolutions, with 58 glyphs and zero dropped primitives; the bubble's authored container centers correctly at both widths. Its provenance is `artifacts/gameplay-helpers/p4/widget-anchor-validation.json`. Complete finished-bubble/2D-console/provider acceptance remains open.

Nested widget milestone (30 October): child class expansion, stable namespace identities, inherited overrides, capture and refresh now use the same resolver in all providers. Native Play and PSX execute six independently bound C++/Blueprint/Lua children in two placed containers. All three Lua execution modes pass the console probe, with 46 glyphs and no dropped primitives. VM source exports rebuild byte-identically. Real ImGui input selects and drags rotated inherited controls, with one undo step, redo and save/reopen in each provider. Canvas insertion also preserves an existing UIActor RectTransform.

The final workspace passes **1,240 tests / 88 ignored** (editor suite 211.70 seconds), formatting, Clippy with warnings denied and the debug build. API freshness and five tool tests pass; parity records 791 public / 5,852 total capabilities with zero pending classifications. Blueprint 6 and companion 2 accept their immediate previous formats in memory without rewriting source. The independent instrumented widget fixture uses 1,117,392 / 1,422,256 / 1,483,696 static bytes for native / bytecode / source execution. Basic uses 966,576 static bytes, a measured **+128-byte** delta, with the same 423,936-byte EXE size; its image hash changes. These are whole-project measurements, not an isolated per-widget estimate. Ten screenshots were inspected: the generic designer at both resolutions and eight starter choice previews (58 glyphs / zero dropped primitives). Placeholder portrait, subtitle and panel styling remain unfinished. Its provenance is `artifacts/gameplay-helpers/p4/widget-nesting-validation.json`; the preceding anchor manifest remains preserved.

Component authoring milestone: the scene Inspector and class designer share component choices, initialization, the eight-component limit and model validation. Components are selected by UUID in the existing hierarchy and use the normal reflected property editors. Real ImGui add/edit/reset/reference/remove, undo/redo and save/reopen pass in C++, Blueprint and Lua; data tests cover explicit None, incompatible and dangling references, inherited removal, budgets and future parent-value propagation. Blueprint 7 / companion 3 preserve compatible older sources in memory and distinguish literal None from member deletion.

The final workspace passes **1,244 tests / 88 ignored** (editor suite 192.78 seconds), formatting, Clippy and the debug build. Native Play and PSX execute six independently bound components with their own C++/Blueprint/Lua behavior and configured fields. All three Lua modes return the expected probe and 18 glyphs with zero drops; static whole-fixture RAM is 1,083,608 / 1,392,648 / 1,454,088 bytes. VM source export rebuilds byte-identically, and unused Basic stays byte-identical to the nesting milestone. API freshness and five tool tests pass with 791 public capabilities / 5,852 total / zero pending classifications. Its provenance is `artifacts/gameplay-helpers/p4/widget-components-validation.json`, with the prior nesting manifest preserved; all 11 captures were inspected; fixture reuse now resolves existing Blueprint/Lua identities and the preset verifier passes. The final audit must also cover nullable placement overrides and class refresh, resource choices for built-in controls, and inherited/derived component behavior across execution modes.

Placement inheritance acceptance passes: **1,247 workspace tests / 88 ignored**, formatting, Clippy with warnings denied, debug build, API freshness and five tool tests. The editor-path fixture covers C++, Blueprint and Lua actor/component None values, save/reopen, duplicated identities, inherited reset and subsequent class-default changes. Three inspected scene Inspector captures, each isolating one provider's branch, show only the placement's Target property override, with inherited Amount/Enabled fields unmarked. PSX passes native, VM bytecode and VM source with probe `[0, 6, 0, 84, 0]`: six empty component references, unchanged authored labels, 84 glyphs and zero drops. Whole-fixture static RAM is 1,083,608 / 1,392,648 / 1,454,088 bytes. VM source standalone rebuilds byte-identically. Unused Basic remains byte-identical at 966,576 static bytes and a 423,936-byte EXE (zero delta). Current provenance is `artifacts/gameplay-helpers/p4/widget-placement-validation.json`; all preceding manifests retain their own source/editor hashes. Scene 7 distinguishes literal None from deletion; actor-format scene 6 upgrades in memory without rewriting the source.

Project resource authoring acceptance passes: **1,248 workspace tests / 88 ignored**, formatting, Clippy, debug build, API freshness and five tool tests. Scene, generic class and Blueprint composition inspectors share project asset selectors and HUD string keys. Real ImGui input selects a Font, Texture and localized key in C++, Blueprint and Lua; undo/redo, save/reopen and the normal Strings editor request pass. All providers render English/Spanish with seven/six glyphs and zero drops using the imported font, retaining the authored text. The test exposed and fixed a stale-language preview cache. Three captured designer previews were inspected. Unused Basic remains byte-identical at 966,576 static bytes / 423,936 EXE bytes, with zero delta. Runtime headers are unchanged; no new console/export claim is made for this editor-only milestone. Its provenance is `artifacts/gameplay-helpers/p4/widget-resources-validation.json`.

P4 remains open for complete built-in control authoring (component-specific inspection and persistent focus links), localized conversation preview, finished bubble/provider/target acceptance, subtitle/panel polish, obsolete style-field cleanup and final acceptance of every finished widget/provider/target combination. See [widget implementation and evidence](p4-widget-classes.md) and [public workflow](../../../docs/dialogue-widgets.md). P5–P10 retain their complete scope.

**Work**

1. Add conversation creation, node/line editing, branch and event binding, participant configuration, validation and navigation to the offending node. A flow view and detail view edit the same asset data.
2. Support undo/redo, atomic save, external-edit conflict handling and stable node identities. Duplicating a conversation remaps its internal identities and preserves external asset references deliberately.
3. Complete generic widget class composition/editing/preview over UIActor for C++, Blueprint and Lua. Reuse the existing template resolver, typed references, inheritance and component inspectors. A widget owns its child controls and behavior; the visual class editor and behavior/source editor operate on the same class.
4. Ship Classic Box, Portrait Conversation, Subtitles and Speech Bubble as editable widget classes over a common dialogue widget base. Expose fonts, nine-slice panels, colors, padding, portrait positions, timing, continuation and sounds through normal class/control properties. Provide bound-state updates and dialogue API/events without reapplying a fixed layout each frame. Measure pagination and HUD budgets from the authored widget. Choices remain conversation behavior shared by the presenters.
5. Implement bubble anchoring through existing 3D camera projection or 2D world-to-screen operations. Define off-screen, behind-camera and missing-speaker behavior without introducing another world-space UI system.
6. Use one shared presenter pool per level. Show the actual actors, components, text storage, glyphs, rectangles, textures and fonts required by each style.
7. Add deterministic dialogue sample states to the generic widget class preview using the runtime binding/layout contract and synthetic input. Select language, resolution, widget preset, participant and branch without executing project code for discovery. Conditions requiring live gameplay use explicit preview values; Play validates real callbacks.
8. Expose the workflow through the existing editor automation surface where practical: create/edit assets, assign bindings, inspect validation and capture UI using existing commands. Do not create a parallel automation state model.

**Acceptance**

- The user can place an object, assign a conversation and dialogue widget class, edit that class's control hierarchy in the generic UI editor and Play without editing engine code.
- All four styles display correctly at 320 by 240 and 640 by 480, including long supported translations and choice pagination.
- Replacing the style or presenter leaves the conversation, branch state and game events unchanged.
- A derived presenter in each flavor changes presentation while retaining parent behavior.
- C++, Blueprint and Lua widget classes preserve child-control layout and typed bindings after inheritance, duplication, save/reopen and Play. Preview and runtime agree; the parent dialogue implementation does not overwrite authored layout.
- The first release includes complete 2D and 3D fixtures in all three flavors, with no permanently blocked controls and no dropped HUD primitives within configured budgets.

This is the first independently usable milestone. Later character and template work builds on this interaction and dialogue surface.

## Phase 5 Shared characters cameras and locomotion

Status: complete and validated. Shared character, command, locomotion and camera
mechanisms, derivable presets and production starter adoption pass their scoped
acceptance. The final fifteen real starter routes cover 4,200 steps across C++,
Blueprint and all three Lua execution modes with matching outcomes. Dungeon's
unused sprite-pool tuning saves 126,976 bytes; bytecode/source retain 130,416 /
71,024 bytes free against the unchanged 65,536-byte reserve. The fresh workspace
passes 1,284 tests / 88 ignored. See [P5 evidence](p5-progress.md) and
`artifacts/gameplay-helpers/p5/starter-routes-validation.json`. Complete-game
adoption, installed-helper budgets, runtime peaks and the open 480i capture remain
the explicit P7–P10 gates.

Initial motor milestone: the reflected native motor is staged with the SDK and
excluded from unused-project pools. The focused reflection test passes; native
collision comparison preserves 1,440 Q12 frames and the component lifecycle test
passes. Console acceptance passes C++, Blueprint and Lua in all three Lua modes:
1,080 steps have identical motion records and actual displacement. The workspace
suite passes 1,259 tests / 88 ignored. Unused Basic retains its P4 static RAM and
EXE sizes. VM-source standalone export rebuilds byte-identically. See [P5 evidence](p5-progress.md) and
[the motor API and current limits](../../../docs/character-motor.md). Character
compositions, input/animation mechanisms, camera rigs and template adoption
remained required at that milestone; the motor extraction alone did not validate P5.

Camera milestone: native CameraRig3D, four derivable presets, SpringArm and routed
orbit input are validated independently of the remaining character work. The
fresh suite passes 1,264 tests / 88 ignored; 36 derived provider cases pass 504
camera checks across 12 PSX runs, and four standalone exports rebuild identically.
Unused Basic is byte-identical to the motor baseline, with zero RAM/EXE delta.
Default-view images are accepted at 320x240 progressive; the 640x480 behavior and
export checks pass, while raw 480i capture/field synchronization remains a P10
visual check. See [current P5 evidence](p5-progress.md) and
`artifacts/gameplay-helpers/p5/camera-validation.json`. Character compositions,
command producers, animation and production template adoption remain required.

Character milestone: Character3D and Character2D now compose their existing
collider/body mechanisms and native command motors, with published 3/4-component
budgets and typed visual/camera references. Global Input, Gameplay, Physics and
PostPhysics stages remove placement-order dependencies and retain same-step
commands. The final workspace passes 1,266 tests / 88 ignored; 18 provider cases
agree over 2,160 automatic steps across both domains and all Lua modes. Two
VM-source standalone exports rebuild identically; Native Play passes both
domains, and unused Basic retains its RAM/EXE sizes. See
`artifacts/gameplay-helpers/p5/character-classes-validation.json` and
[the current evidence](p5-progress.md). Local-player/NPC/scripted producers,
animation and production template adoption remain required to close P5.

Command producers are validated. The workspace passes
1,269 tests / 88 ignored, including the nine helper/reflection tests and Native
Play capture/physics in both domains. Six native suites, Clippy, formatting and
API freshness pass. Shared sources, local-player input, point/target steering,
Player/NPC compositions, command ownership and extension callback lifetimes are
covered. Unused Basic remains byte-identical with no RAM/EXE delta. All 18
provider cases agree over 2,160 steps and callbacks in both domains and all Lua
modes; both VM-source standalone exports rebuild identically. An additional
focused Native Play test passes authored command/destination properties in both
domains after the full suite. Acceptance:
`artifacts/gameplay-helpers/p5/command-producers-validation.json`.
Animation and production template adoption remain required to close P5;
see [the current P5 evidence](p5-progress.md).

Animation drivers pass native, Native Play and six console domain/mode runs:
18 provider cases, 2,160 steps/callbacks and 10,800 exactly compared trace words.
The final workspace passes 1,275 tests / 88 ignored. Local record edits preserve
copy semantics in every backend; VM boolean parameters are normalized at entry,
with the discovered landing-state regression covered. Unused Basic remains
byte-identical with zero RAM/EXE delta. Production template adoption is underway
and remains required before closing P5; see [the current evidence](p5-progress.md).

The Third Person starter now generates the shared Character3D/input/animation
and ThirdPersonCamera composition in every flavor. All five execution cases
agree over 1,200 automatic route steps, including renamed identity bindings,
six animation states, two jumps and gravity during input capture. The workspace
passes 1,276 tests / 88 ignored. All three flavor exports rebuild byte-identically.
Scoped evidence: `artifacts/gameplay-helpers/p5/third-person-validation.json`.
Platformer adoption, remaining starter routes and later template/helper phases
remain open.

Platformer now generates Character2D, project input and the sprite driver, with
an attached camera actor and one available player component slot. Generation,
compilation/staging and the five-case console route pass; the fresh workspace
passes 1,278 tests / 88 ignored. VM source's static headroom failure is resolved
in the route by sizing Lua scratch values from typed program demand, retaining
the 32-word limit and frame/value capacities. Four scalar/full-width VM cases
pass. Twelve callback-provider cases and all three byte-identical exports pass,
including the generated capacity header. Scoped evidence:
`artifacts/gameplay-helpers/p5/platformer-validation.json`. Remaining template
routes and later helper/template phases remain open; see [the current evidence](p5-progress.md).

**Work**

1. Extract Third Person's movement kernel into a reflected CharacterMotor3D mechanism: desired motion, velocity, grounding, jump, gravity, turning, step-up and step-down behavior. Preserve Q12 tuning and collision behavior before adding new options.
2. Keep command production separate. A local-player source reads routed actions; an NPC or scripted source submits the same desired movement and jump requests without reading a pad.
3. Extract orbit camera behavior, including boom collision, pitch limits and smoothing, into a separate camera actor/helper with an explicit target. Provide extension points for camera modes used by Dungeon Complete.
4. Extract locomotion animation selection and speed-driven playback into a reusable driver with configured clip references and thresholds. Remove character-specific clip names from the engine mechanism.
5. Build Character3D from a compatible root, collider and motor. A skeletal visual and animation driver are configurable; combat and dialogue are optional capabilities.
6. Build Character2D over the existing CharacterBody2DComponent. Add a shared command contract, sprite animation/facing driver and platformer input preset without replacing move_and_slide, one-way platforms or existing collision services.
7. Keep 3D world units and 2D pixel units explicit in profiles, APIs and the Inspector. Share lifecycle and control semantics rather than numeric assumptions about units or gravity direction.
8. Provide player and NPC presets for each domain and typed references to visual, body and camera. Ensure class defaults and derived component templates can replace supported bindings without changing unrelated placements.
9. Replace the temporary routing adapters from P1 with these shared mechanisms. A movement capture continues physics; interaction and camera locks remain independent.
10. Ship native, derivable FirstPersonCamera, ThirdPersonCamera, TopDownCamera and IsometricCamera actor classes over CameraRig3D. Expose typed targets, offsets, pitch/yaw, distance, follow and input behavior consistently to C++, Blueprint and Lua.
11. Implement a reusable SpringArmComponent for third-person and optional elevated camera rigs. Preserve the current obstruction query semantics initially; add independently configurable collision retraction and smoothing. First-person rigs do not require a boom.
12. Define isometric as a fixed-angle perspective preset on the current renderer. A true orthographic projection requires its own renderer contract and is not silently implied by the preset name. Include a fixed-camera example and rig-switch behavior in camera acceptance.

**Primary touchpoints:** proposed `runtime/character3d.hpp`, character profile assets, existing `world2d` services, `third_person`, `third_person_blueprint`, platformer generators, character template headers and reflected component defaults.

**Acceptance**

- The same Character3D moves under player, NPC and scripted command sources.
- Two players or an NPC and player have independent visuals, motion and camera bindings after rename and duplication.
- Existing movement routes preserve jump height, step handling, grounding, wall response, animation transitions and camera obstruction behavior.
- The 2D player preserves one-way and moving-platform behavior and works with the existing physics objects.
- Each flavor can extend a player or animation policy through reflected hooks without copying a motor.
- Shipped compositions fit the component budget and publish their available extension slots; moving optional cameras to another actor includes that actor in the map budget.

## Phase 6 Helper catalog pickups doors and switches

Status: validated. The catalog, resource transaction and resolved placement preview
use the common authoring model. Pickup, Door, Switch, GameplayFlag,
ExaminableObject and DialogueNPC are available in both domains alongside the
existing helpers. Native host and Native Play physics checks pass. Six PSX runs
contain 18 matching C++/Blueprint/Lua provider cases, and both standalone exports
rebuild byte-identically. The workspace passes 1,292 tests / 88 ignored; a later
common duplication correction passes 18 focused regressions, including copied
helper bindings, reset, undo/redo and persistence in both domains. Clippy, API
freshness and parity checks pass. Basic with no helpers retains identical RAM,
executable size and executable hash; flag-only builds retain no interaction
polling or pickup/door/switch pools. See [P6 delivery and acceptance](p6-delivery.md),
`artifacts/gameplay-helpers/p6/world-validation.json` and the earlier
`p6/catalog-validation.json`. The initiative now proceeds to P7.

**Work**

1. Add a Game Helpers category to placement and content workflows, using existing class metadata and actor composition. Initial groups are Interaction, Dialogue, Characters and World Gameplay.
2. Support dragging actor helpers into the matching scene view and dragging compatible helper components onto existing actors. Use the same domain and dependency validation as Instantiate Actor and Add Component.
3. Materialize required style, font, texture, conversation and configuration assets into the project on first use. Record package identity and version, reuse unchanged resources and remap references transactionally. Do not overwrite user-edited resources during a later placement.
4. Ship TriggerBox3D, TriggerArea2D, ExaminableObject, DialogueNPC, player presets, Pickup, Door and Switch. A chest composition may use an interaction, a visual state and a reward event without requiring a full inventory subsystem.
5. Give pickups an explicit accepted initiator and collected result. Distinguish collected state from visibility, enabled state or destruction.
6. Give doors explicit open/closed/locked states, a requirement query and open/close commands. Apply collider changes at a defined lifecycle point and emit results that game rules can consume.
7. Add reusable bounded flag/requirement data only where needed for key and switch scenarios. Keep inventory quantities, quests and game-specific progression in the selected project flavor or authored game data.
8. Expose editable properties, derived-class creation, Reset to Parent and valid replacement points. Show dependencies and estimated runtime cost before placement; rejection of an over-budget composition must leave the scene unchanged.
9. Implement helper installation and placement through a shared transaction that also serves CLI/MCP operations, with undo/redo and save/reopen support.

**Primary touchpoints:** `src/game_helpers.rs`, `src/game_helper_catalog.rs`, `src/helper_install.rs`, `runtime/world_gameplay.hpp`, helper resources, `actor_workflow`, `actor_scripts`, `project_browser`, scene drop handling, component contracts, asset management and editor operation batches.

**Acceptance**

- Every helper installs into Basic with no prior Third Person or Dungeon content.
- Repeated placements reuse common resources while retaining independent actor, component and conversation state.
- A pickup can drive a door requirement without relying on an inactive key actor.
- Existing user assets are retained, reference remapping is correct and failed installation rolls back its own changes.
- C++, Blueprint and Lua projects can derive and customize the same native helper classes using existing provider rules.

## Phase 7 Starter template refactor in three flavors

Refactor one template at a time and complete its three flavors before proceeding to the next. Scene/assets are built from one composition path; flavor emitters produce project classes and explicit role bindings. Avoid a universal generator that introduces a second class or asset schema.

### P7A Basic and Sample

Status: validated. Basic emits an inert `GameRoot : Actor3D` in the selected
flavor alongside its camera. Both starters record semantic class roles using
existing ClassReference identities. The compiler and class picker share the same
derivation rule, including Lua parents. Source ownership, inherited properties,
optional-helper exclusion, reset/reference handling and reopen checks pass.
Native Play passes all six template/flavor cases. Ten PSX executions produce
matching per-template traces; all six standalone exports rebuild byte-identically.
Authored compositions also match after normalizing local identities and the
selected starter class UUID. The full workspace passes 1,296 tests / 88 ignored;
Clippy, format and whitespace checks pass. See [P7A delivery and acceptance](p7a-delivery.md)
and `artifacts/gameplay-helpers/p7/small-starters-validation.json`. P7B-P10 remain open.

1. Generate GameRoot in the selected flavor for Basic, with an inert instance and camera. Keep helpers opt-in and verify no dialogue or movement pool is linked solely because the library exists.
2. Keep Sample's Spinner behavior genuinely authored in each selected flavor. Use it to validate inherited defaults, component ownership and the smallest project build.
3. Make Hub and CLI describe exactly which files are emitted, with consistent language labels and no unavailable flavor options for these templates.

### P7B Third Person

Progress: construction is split into shared player/camera assembly and arena
assembly. Dungeon consumes generated World/Player handles directly and no longer
creates or installs an unused arena before deleting actors by display name.
Third Person and Dungeon record player, camera, controller, locomotion and camera
input class roles in their installation data. The five existing Third Person
tests and the new three-provider composition/renamed Native Play checks pass.
Six fresh generated projects pass the role/tuning audit; Third Person retains
all 33 actors and all 21 resource hashes exactly against the pre-change snapshot,
and Dungeon generates 20 actors / 38 resources without arena installation.
The first full regression reported one FBX import Access Denied error; its isolated
retry and final full regression pass (1,298 tests, 88 ignored). An additional
isolated test compiles the three legacy comparison controllers independently;
Clippy with warnings denied also passes. All fifteen PSX routes pass and match the
frozen P5 records exactly. Three standalone exports rebuild byte-identically.
P7B is validated; see [the delivery and acceptance](p7b-delivery.md) and
`artifacts/gameplay-helpers/p7/player-routes-validation.json`.

1. Replace controller implementation copies with shared character, input, animation and orbit-camera mechanisms.
2. Emit a small project-owned ThirdPersonRules or player extension in C++, Blueprint or Lua. Its extension points and configurable properties should be useful, and common movement stays inherited.
3. Replace global Visual/Camera name lookup with generated typed role bindings in every flavor. Share movement profiles and animation role data.
4. Keep the original arena and basic movement scope. Store the interaction/dialogue demonstration in a separate current-format example or acceptance scene rather than expanding every starter by default.

### P7C Platformer 2D

Status: validated. See [P7C delivery and acceptance](p7c-delivery.md) and
`artifacts/gameplay-helpers/p7/platformer-validation.json`. Historical progress
and failures below are retained; the checklist records the final outcome.

Current acceptance checklist:

- [x] Shared Pickup inheritance and selected-language collection policy.
- [x] Original world/resources comparison in all three providers.
- [x] Full ordinary widget placement, Native Play smoke and host runtime suite.
- [x] Twenty console cases, including ten exact P5 movement/surface replays.
- [x] Three byte-identical standalone exports.
- [x] Four scalar/full-width VM cases and all 21 Lua conformance checks.
- [x] Current Basic footprint and optional-helper exclusion: 983,160 static RAM
  bytes and 436,224 executable bytes; no optional gameplay capabilities/pools.
- [x] Final workspace regression: 1,301 passed, 88 ignored, no failures.
- [x] Final evidence receipt and delivery.

Progress: CoinPickup now derives from the common PickupComponent in all three
providers. Each project owns the collected callback that hides the coin; explicit
player bindings and auto-collection defaults remain scene data. The template
records semantic player/controller/locomotion/pickup/camera roles. A pre-change
snapshot preserves the 57-actor scene and 23 resource hashes for comparison.
The three focused composition/compiler tests pass. The Native Play scenario
passes eleven checks in each provider, including invalid initiators, once-only
collection and repeated collection after reset, with every authored actor renamed.
The C++ scene and all resource payloads/settings match the pre-change snapshot
after normalizing generated identities and the two new pickup bindings. The full
regression is running. A serial acceptance runner covers the remaining five-mode
movement, surfaces, pickup and ordinary dialogue-widget scenarios plus three
exports. Console, export, helper headroom and final regression gates remain open;
P7C is not accepted. Fixture compilation failures and their passing recheck are
preserved under `artifacts/gameplay-helpers/p7c-*`.

The first full regression passes 1,300 tests / 88 ignored. Equal inherited owner
contracts are now checked in all three providers; the four focused tests pass,
and the final warning-free debug build and Clippy pass. A final full regression
and the serial console matrix are running with editor hash `9032de79...`.
The three-provider original-world audit passes. The first C++ console movement
record matches P5 exactly at 1,700,920 static bytes. A separate Native Play
preflight of the full level plus the dialogue widget fails the generated 64 KiB
object-pool cook assertion. Its target-console result, allocation diagnosis and
repair remain acceptance work; no pool limit or game feature has been changed.
The preflight initially assembled only the widget root. The fixture now uses
the normal `--add-actor` class factory, including all authored controls. That
placement correctly rejects the default widget: it needs 12 actors while the
unchanged Platformer already contains 57 (level capacity 64). The final full
regression passes 1,300 tests / 88 ignored. The real widget placement/headroom
failure remains open; neither the root-only preflight nor a smaller replacement
widget certifies that gate. Preserve this evidence while resolving capacity.

Capacity repair in progress: the six empty organizational actors are consolidated
into one `World2D` root. All rendered/physical actors, assets, tuning and attached
structures are retained. The new starter has 52 actors; the full 12-actor widget
can use the remaining slots, with dialogue on the existing Guide and observation
on the existing world root. The comparison verifier explicitly normalizes only
that organizational-parent consolidation and the two pickup bindings. Focused
tests are running; a fresh factory placement, native/console budgets and replay
comparisons are required before accepting this repair.

The full 64-actor widget fixture now runs in Native Play. Cooked classes share a
bounded 64 KiB object arena while preserving their typed handle/admission limits;
host allocation, alignment, reuse, destruction and lifecycle checks pass. Four
console dialogue cases pass at the first shared-storage revision. Lua source
fails its mandatory reserve (30,896 bytes free), so that failure is retained.
The editor now plans the Lua heap from normalized program demand within the
existing mode ceilings, with unchanged call/argument/value limits. Ten focused
Lua tests, formatting, Clippy and API freshness/parity pass. The fresh Lua-source
dialogue run passes with a 32 KiB arena (5,408 bytes retained at initialization),
1,967,952 static RAM bytes and 129,200 bytes unassigned, preserving the mandatory
65,536-byte runtime reserve. Its dialogue/controls record matches the other modes.
All five current-revision full-widget dialogue cases now pass with identical
records. All ten movement/surface cases also pass and match their frozen P5
records exactly. All five pickup cases pass as well, completing the 20-case
console matrix. Three exports and Lua capacity/conformance checks are running
serially. The three exports now rebuild byte-identically. The first additional
Lua-value fixture failed before runtime because it replaced the project's Lua
identity registry, dropping P7A's GameRoot. It now merges its own entry and reruns
in a fresh directory; all four scalar/full-width VM cases pass. The broader
conformance run first found that its size-tool lookup required PATH despite the
pinned SDK being installed. The verifier now prefers that SDK and reruns; both
original failures are retained. P7C remains unaccepted until
their results and the final regression are recorded.

1. Compose Character2D, the shared platformer command source, sprite driver and existing following camera.
2. Replace CoinPickup copies with the shared pickup helper and selected-flavor collection policy. Keep the current level, visual assets and 2D demonstrations intact.
3. Provide actual project-owned rules in the chosen flavor, including the collection outcome and restart/respawn policy established in P0.
4. Recheck VM source memory with installed interaction/dialogue helpers. P1's routed-input build already uses 2,020,760 static bytes and leaves 76,392 unassigned; preserve runtime reserve through shared-cost reductions rather than removing a flavor or game behavior.

### P7D Dungeon

Status: validated; the final workspace run passes 1,310 tests / 88 ignored.
See [migration details](p7d-design.md) and [delivery evidence](p7d-delivery.md).

Fresh keys and doors derive from PickupComponent and DoorComponent in all three
providers. Explicit GameplayFlag components represent key ownership. Generation
consumes typed key/door/key_state role UUIDs, remaps stable component references,
and rejects incompatible world domains. Renamed and replaced classes retain
bindings across save/reopen/regeneration. Existing legacy scripts and Lua class
identities remain preserved. Failed installation rolls back its own source and
settings writes, retaining concurrent user edits.

Completed checks: ten console ownership cases across both domains, five complete
Dungeon routes, three Native Play providers and three byte-identical standalone
exports. Five 180-frame replays match all 1,080 trace words, including door
visibility; no boundary difference was observed. The final debug editor builds,
Clippy and formatting pass, and fresh projects in all three providers preserve
20 actors and 38 original resources. All 145 runtime headers match the exports.
The final source snapshot is recorded separately from the frozen console-matrix
editor so installation hardening is traceable. The earlier workspace snapshot
was superseded, and the test-only telemetry constructor failure is preserved
alongside its passing three-provider Native Play recheck. The final full
workspace run passes and the acceptance receipt is
`artifacts/gameplay-helpers/p7/dungeon-validation.json`. P7 is complete; P8-P10
remain required.

1. Build its player from the same character composition as Third Person without generating an arena and then searching for display names to delete it. Extract explicit player/camera and arena/world generation steps.
2. Keep dungeon recipe, style, generated geometry, handmade rooms, lighting and navigation generation as shared data/services.
3. Place the generic pickup and door helpers using generated typed bindings. Express key ownership and door requirements through DungeonRules in each selected flavor.
4. Replace hard-coded flavor detection and key/door class lists with a semantic-role mapping in the template installation data. The dungeon generator must continue placing valid helpers after regeneration.
5. Preserve the current key auto-pickup behavior and explicit interaction settings; adopting the interaction base does not require changing every pickup into a button prompt.

**Primary touchpoints:** `workspace`, `project_templates`, `sample_template`, `third_person`, `platformer_template`, `dungeon_template`, `dungeon_gameplay`, template resources and source generators.

**Acceptance for each template**

- Generate and compile C++, Blueprint and Lua projects with the same authored world and tuning. Blueprint and Lua sources are editable and implement their declared project rules.
- Generate only the chosen project gameplay implementation. Engine-native services, shared content and compiler-generated native output are allowed; alternate authored gameplay implementations are not bundled as fallback.
- Lua Native C++, VM bytecode and VM source run the same role bindings and rules.
- Required class/component/asset references survive save/reopen and regeneration. Opening the project cannot silently rewrite its selected flavor.
- Existing template tests are updated to assert shared-base behavior and source ownership. Preserve educational full-controller comparisons separately as examples if retained; they must compile independently of the starter path.

## Phase 8 Dungeon Complete decomposition and C++ migration

This phase preserves a working complete C++ game while creating the mechanisms and data needed by the other flavors. Do not merely rename DungeonGame into a native service and call it from empty visual or Lua classes.

Validated kernel adoption: the bundled C++ template uses the validated shared
character kernel, SpatialQueryContextComponent and project DungeonCamera derived
from ThirdPersonCamera. Its fresh CLI generation produces the validated
`8f533159...` executable byte-for-byte, using 1,988,296 linked RAM bytes. Both
guard/boss replays, final counters and slow-frame records exactly match the
validated prototype. All 67 resource packages match. The full 16-event economy
circuit, cold reload and restoration of disposable cards pass. Required checks
pass 1,311 workspace tests / 88 ignored, formatting, Clippy and the debug build.
Evidence is `artifacts/gameplay-helpers/p8/bundled-foundation/validation.json`.
The matched pre-adoption source build under this same runtime uses 1,949,704 RAM
bytes; adoption adds 38,592 RAM bytes and 30,720 EXE bytes, with identical geometry.
The resulting 108,856 unassigned bytes are not a measured heap/stack allowance
or accepted Lua headroom. See `adoption-footprint.json` in the same evidence folder.
This source adoption does not close overall P8 acceptance or the 25ms gate.

Validated component migration (6 November 2026): the reflected motor now exposes
shared-context simulation, explicit facing, contact-preserving velocity clearing
and optional speed measurement. The final host oracle passes 1,440 exact Q12
steps and 524 idle query skips, including context/body identity invalidation.
C++, Blueprint and Lua pass nine actual provider executions across the three Lua
modes; the VM-source standalone rebuild is byte-identical. Evidence is
`artifacts/gameplay-helpers/p8/motor-shared-api/providers-final/` and
`identity-host.log` in its parent directory. Player Aqua now uses Character3D
with its inherited Motor/Collider and native visual/camera references; project
movement state and its private query adapter were removed. Fresh public-CLI
generation/export, 3,420 exact camera/state trace words, nine ordinary-motor
provider regressions, zero-cost unused Basic, all 16 economy events and cold
reload pass. Runtime composition is 58 actors / 151 components, with two free
player component slots and zero registry rejections. All 67 packages and the
geometry archive match. The same-runtime source comparison adds 22,720 RAM
bytes and 18,432 EXE bytes, for 2,011,016 linked RAM bytes; runtime peaks and Lua
headroom remain unaccepted. The camera fixture now captures all 180 frames with
a 60-frame ring; its baseline agrees with the prior full buffer in every word
and lost/overwritten frames fail the harness. Required checks pass 1,311 workspace
tests / 88 ignored, formatting, Clippy, debug build, API freshness and parity.
Evidence is `artifacts/gameplay-helpers/p8/character3d-adoption/validation.json`.
The unchanged 25ms gate still fails: guards 11 slow frames / 26,752us peak, boss
5 / 25,792us. Functional combat, music, audio and streaming checks pass. The prior
kernel receipt above remains historical evidence; neither closes overall P8.

### P8A Resource and composition budget

Validated resource work: optional helper ancestry metadata is now separate from
creation capacity. Native controls, explicit source demands and ClassRef
selections/defaults retain their factories. The 14 helper regressions cover
inherited controls, nested selections, Lua class strings and C++ UUID literals
while excluding comment-only UUIDs. Nine real provider executions pass, including
deferred native creation by UUID and the VM-source standalone rebuild. Complete
matches all 3,420 trace words from the old cook and its standalone rebuild matches
an ordinary Build under the same disc profile. Unused Basic retains zero delta.
The matched PCDrv comparison saves 3,760 linked RAM bytes and 2,048 EXE bytes:
2,007,256 RAM / 1,187,840 EXE bytes; object arena 50,704 -> 49,088 bytes. All 67
packages, geometry and compiler values match. Guard/boss functional routes and
all 16 economy events, cold reload and card restoration pass. The route harness
now re-reads paused RAM before final accounting, preserving exact counter/replay
equality. Required checks pass 1,313 workspace tests / 88 ignored, formatting,
Clippy, build, API freshness and parity. Evidence is
`artifacts/gameplay-helpers/p8/metadata-factories/final-uuid/validation.json`.
The 25ms gate still fails: guards 12 slow frames / 26,752us peak, boss 5 /
25,792us. Runtime peaks, Lua headroom and overall P8 acceptance remain open.

A disposable 58-actor sound-composition prototype has passed its scoped checks; see
[the capacity notes](p8-design.md). Its reflection and exact original-content /
67-resource-package comparisons, PSX build and full economy/PocketStation circuit
pass. Guard/boss behavior, audio and streaming execute, but the existing 25ms
performance gate still fails (27,264us maximum; nine guard-route and six boss-route
slow frames). The same-editor untouched baseline also fails (27,456us maximum,
13 slow frames). The sound composition is now applied to the bundled template:
58 actors, six free slots, 22,608 fewer linked RAM bytes and 8,192 fewer EXE bytes.
Fresh generation matches the tested prototype's scene and gameplay sources.
Evidence is `artifacts/gameplay-helpers/p8/sound-capacity-validation.json`.
Performance and the rest of P8 remain open; no overall P8 acceptance is claimed.

1. Resolve the map's existing 64-actor occupancy before adding sensors, presenters or game roots. Inventory UI, decorative actors, geometry batching and camera composition must be evaluated with the current editor/runtime semantics.
2. Reuse existing UI actors or consolidate compatible content while preserving appearance and editable source assets. Record the final actor/component cost and remaining extension capacity.
3. Reserve measured headroom for Lua source and bytecode arenas, generated bindings, game state, dialogue and save buffers. Preserve the same assets and complete feature set across modes; reduce shared cost instead of downgrading a flavor.
4. Preserve current streaming configuration, MIDI instrument budget, audio resources and collision query batching until matched-route measurements justify a change.

### P8B Generic mechanisms

Health values/library and their Complete C++ adoption are validated. Five
provider/mode executions, 134,798 host comparisons, four byte-identical standalone
rebuilds, zero unused Basic size delta, functional combat/economy/camera and
1,316 workspace tests pass. The public [health contract](../../../docs/health.md)
keeps damage eligibility and consequences in project code. See the latest
`artifacts/gameplay-helpers/p8/health/validation.json`; the full P8 gates remain open.

The shared SpatialQueryContextComponent is implemented over SpatialQueryBatch,
with typed publication, movement, single/pair raycasts and an explicit reset
boundary. Its adapter/lifecycle host checks and six affected native runtime
fixtures pass. Real C++, Blueprint and Lua policies pass Native C++ and both VM
modes (nine executions), and the source-mode standalone rebuild is byte-identical.
API reference regeneration passes. The first unused-Basic measurement exposed a
368-byte retained default pool; adding the class to the existing optional-helper
retention path removes that cost. The recheck matches the previous 983,160 RAM /
436,224 EXE bytes and contains no unused context pool. After the repair, all nine
used-context executions and the byte-identical standalone rebuild pass again;
see `artifacts/gameplay-helpers/p8/query-context-validation.json`. The disposable
Complete integration preserves all 67 resource packages and changes only one
component plus its typed binding in the scene. Guard/boss scenarios reach their
expected combat/victory states with zero missing audio, dropped triangles or
failed/skipped streaming work. The full economy circuit and cold save reload
pass. Both routes still fail the 25ms gate at 27,648us (15 guard / eight boss slow
frames). Static RAM is 1,956,304 bytes; no heap/stack peak is claimed. This remains
a prototype; see `artifacts/gameplay-helpers/p8/query-complete/validation.json`.

The common 3D motor kernel now accepts an internal query adapter, so a native
consumer can reuse synchronized collision queries without copying locomotion.
Zero stair height disables lift and snap-down probes. Four native host regressions
and the adapter/zero-stair check pass, including the existing 1,440-frame movement
reference. Another 1,440 exact steps compare the Complete flat-floor/idle-query
policy against its old integration, with 524 identical idle-query skips and
wall, ceiling, jump, teleport, rotation and revision scenarios. This kernel
change passes the disposable Complete guard/boss functional scenarios and full
economy/PocketStation circuit including cold reload; both routes still fail the
25ms gate at 27,648us. Its scene and 67 resources are unchanged from the query
prototype, and no component was added. Evidence is
`artifacts/gameplay-helpers/p8/character-complete-recheck/validation.json`.
That prototype receipt predates the bundled controller adoption recorded above.

SpringArm's new reflected shared-context solve and adaptive close clearance pass
host checks alongside the original 720-frame camera reference. The query context
owns one recent native-ray cache; changes of consumer, inputs, revision or ignore
identity re-query, and publication/movement/reset invalidate it. A different
consumer replaces the cache without dropping a query. All nine provider executions,
the byte-identical standalone rebuild and zero unused Basic cost pass. The required
root-package check passes 1,270 tests / 87 ignored, with Clippy and 29 focused
helper/model tests; this command did not include the PocketStation workspace
member. See `artifacts/gameplay-helpers/p8/shared-camera-validation.json`.

The Complete project-camera prototype derives from ThirdPersonCamera and retains
its orbit/aim policies in project source. Its first console run exposed a native
composition gap: unlinked source classes did not adopt existing singleton controls
as inherited defaults. The runtime rejected a duplicate Camera component and
rolled back the bank (one registry rejection, zero arena rejections, zero actors).
The failed runs are preserved. A host reproduction fails with two components
instead of three. The common refresh path now hydrates native defaults before
cooking. The regression now passes: controls retain identities/values, the native
indices are correct, repeated refresh is stable, the saved source is unchanged
and an unaffected lighting bake remains valid. Native PC preparation and scene
loading now run that refresh for unlinked source actors too. The rebuilt editor
generates indices 0/1/2 for the unchanged camera authoring inputs. The full
workspace, including PocketStation, passes 1,311 tests / 88 ignored; Clippy passes
after a test-only initializer style repair. The fresh console bank loads all 58
actors and 150 components with zero registry rejections. Guard/boss functional
routes and the economy/PocketStation circuit, including cold reload, pass; both
routes still fail the unchanged 25ms gate at 28,032us (13 slow frames each).
All 67 resource packages remain byte-identical. The initial lobby framing was
visually inspected, but matched normal/lock-on/free-aim trajectories remain open.
See `artifacts/gameplay-helpers/p8/native-default-refresh-validation.json` and
`artifacts/gameplay-helpers/p8/camera-complete-fixed/validation.json`.
Character/camera migration and the failed P8 performance gate remain open.

Optional native HUD phase counters now publish complete records with their draw
identity last. The detailed recheck validates 476 complete records; native
HUD/text/widget fixtures, 15 profiler-tool tests, API generation and debug
binaries pass. The ordinary executable remains byte-identical to the earlier
camera prototype, has no profiling symbols and preserves all 67 resource
packages. It still fails the frame gate at 28,032us (15 slow frames in this
repeat). The exploration peak spends only 10 scanlines in the enclosing HUD
phase, so no additional layout cache is introduced. See
`artifacts/gameplay-helpers/p8/hud-phase-recheck/validation.json` and
[the P8 notes](p8-design.md).

The deterministic Complete camera comparison now passes 180 logical steps and
3,420 exact Q12/state words: 90 normal, 55 lock-on and 35 free-aim steps, including
simultaneous L1/R1, release, reacquisition, movement and an actual jump. The first
matching trace had opened a nearby terminal on Cross; the fresh fixture moves
outside interaction range and asserts a 7,210-Q12 jump rise. Both original
projects remain unchanged and all 67 resource packages match. The common camera
adds 27,544 linked RAM bytes in this matched fixture; its totals include the test
actors and trace buffer and are not production RAM. Evidence is
`artifacts/gameplay-helpers/p8/camera-mode-trace-jump/validation.json`.
Every-view visual acceptance, the failed performance gate, runtime heap/stack
peaks and the remaining decomposition still prevent overall P8 acceptance.

Native controller transition logging is now opt-in through `EPOK_PAD_TRACE=1`,
with native build-cache tracking. A same-input off/on comparison recreates the
original executable exactly when enabled; default-off removes five early slow
intervals and saves 2,048 PCDrv RAM bytes. Guard/boss functional checks, audio and
streaming pass, but the unchanged 25ms gate still fails at 28,928us / 27,648us.
The workspace passes 1,311 tests / 88 ignored, with Clippy and formatting. See
`artifacts/gameplay-helpers/p8/pad-trace-complete/validation.json` and the P8 notes
for corrected CD/PCDrv RAM comparison scope. The subsequent internal HUD cache
change passes native packet/budget regressions, console functional routes,
the full economy/PocketStation/cold-reload circuit and serial startup ordering.
It adds no cache storage or Complete linked RAM; unused Basic retains 983,160 RAM /
436,224 EXE bytes and no query pool. All 67 resource packages match. Guard/boss
frame acceptance still fails at 28,736us / 25,920us (eight / six slow frames).
See `artifacts/gameplay-helpers/p8/hud-independent-pools-complete/validation.json`.
An optional deterministic replay now applies setup before physics, pad changes
at native polling and the exact stop at completed frame 1,800. Eight frozen-binary
runs cover both routes before/after the HUD change, twice each. Each repeated
case has identical setup, work aggregates, final counters and slow-frame records.
Guard slow frames change from ten to nine (both peaks 27,776us); boss slow frames
change from five to four (27,328us / 25,664us). Every run still fails the unchanged
25ms gate. Across different binaries, simulation-step counts and combat trajectories
can differ, so the grouped CPU means are descriptive rather than identical-work
speedup claims. See `artifacts/gameplay-helpers/p8/deterministic-routes/comparison.json`
and its repeatability receipt.

Native PC sampling now works alongside native-only sequence music: Timer0 starts
after sequence-clock setup, while music retains Timer2. A corrected 29-bit
physical address mask keeps BIOS addresses outside RAM code bins; the first
capture using the old mask is marked superseded. The validated frame-301..1,440
window contains 11,191 samples: 9,746 wholly attributed to one linked function,
1,445 ambiguous and zero outside the covered code range. Retained interior
triangle emission accounts for at least 1,833 samples (16.4%). The diagnostic
passes combat/music/audio/streaming checks and still fails the unchanged 25ms
gate at 27,136us. It adds 34,816 RAM bytes only when enabled. Normal RAM remains
1,988,296 bytes with no sampler symbols; its loaded image differs from the prior
normal image only in one source-location diagnostic byte. All 67 packages and
the geometry archive match. See
`artifacts/gameplay-helpers/p8/native-music-sampler-physical/validation.json`.
A retained-triangle NCLIP experiment passes 809,659 target CPU/GTE area
comparisons with zero differences. Its unchecked guard/boss peaks are worse:
27,968us / 26,112us against 27,776us / 25,664us, with unchanged linked RAM and
negligible grouped frame-work changes. The experiment and its validation counter
were removed; the previous runtime source is restored byte-for-byte. See
`artifacts/gameplay-helpers/p8/retained-nclip/rejection.json`. P8 acceptance,
runtime peaks and the remaining Complete decomposition remain open. The current
bundled source adoption is recorded at the start of this phase.

1. Move player locomotion and the orbit camera to P5 mechanisms, including typed references and generic extension points for normal, lock-on and free-aim camera modes.
2. Extract bounded health/damage, target selection, projectile pool operations and encounter actor activation only to the extent that the complete game requires reusable mechanisms.
3. Expose efficient batched queries and typed snapshots so Blueprint and Lua do not repeat per-vertex or per-projectile foreign calls every frame.
4. Replace terminal proximity and interface lock handling with interaction and control capture. Use shared UI/dialogue presentation mechanisms for messages and prompts where appropriate; retain a proper menu for inventory and shop operations.
5. Expose Memory Card and PocketStation transaction operations, status and bounded payloads through the common reflected service boundary. Generalize byte transport, installation and validation independently of Reactor Station's rewards and receipt policy.

### P8C Project rules and data

Player vitality is now a focused project component with independent C++,
Blueprint and Lua policies over a data/dispatch-only base. Damage eligibility,
immunity timing and healing eligibility have moved out of DungeonGame; effects,
attack/targeting and encounter/economy decomposition remain required. C++ adopts
the component by identity, with 58 actors / 153 components and one remaining
player extension slot. The host/oracle/provider/export/Complete acceptance above
passes. The cheaper camera fixture preserves all 180 rows, the 32-row ring and
exact equality to the earlier frozen capture. See [the delivery](p8-player-vitals.md).

1. Split DungeonGame into focused project-owned rules for run flow, player combat, encounters/boss behavior, inventory/economy and terminal operations. Keep these C++ classes as the first implementation of the shared game contract.
2. Put tuning such as item prices, damage, cooldowns and encounter definitions into shared authored data where that makes the rules clearer. Keep the decisions that use the data in project gameplay.
3. Define a stable save schema and generic serialization path accessible to all three providers. Preserve the current valid save contents, field bounds and receipt accounting unless an explicit versioned change is needed.
4. Preserve the ordering of operations: save PSX inventory before acknowledging PocketStation receipts; a failed save must not consume an item, complete a purchase or award the same ore twice.
5. Keep transient run state separate from persistent economy. Restart resets the same state as the current game and releases all interaction/camera captures.
6. Add observer hooks or normalized telemetry for acceptance tools without making gameplay depend on tests reading a C++ class layout.

**Primary touchpoints:** `DungeonPlayerController`, `DungeonGame`, `DungeonEconomy`, `DungeonInventory`, `DungeonPocketLink`, `DungeonTargeting`, shared native services, template scene and packaging tools.

**Acceptance**

- The C++ template retains the playable level, guards, boss, targeting, camera modes, attacks, damage feedback, healing, inventory, trading, persistence, PocketStation miner, music and streaming.
- Existing guard/boss replay, economy and packaging validations pass against fresh generated projects.
- Save failure, disconnected card, interrupted transfer, duplicate receipt, restart and scene exit have explicit expected behavior.
- All APIs needed by P9 are reflected, bounded and covered by C++, Blueprint and Lua conformance fixtures before the large game port begins.
- Actor, component, RAM, stack, HUD, VRAM and native frame budgets are measured in the refactored C++ game; no required feature is disabled to pass.

## Phase 9 Dungeon Complete Blueprint and Lua parity

Port each subsystem into both providers against the P8 C++ contract. Complete a subsystem's three-way scenario before adding the next one. Shared mechanics and data remain common; editable project rules use the selected language.

### P9A Game flow and controls

- Author game initialization, intro timing, fade/music coordination, input unlock, death, victory and restart rules.
- Bind roles and component references without actor-name searches or dependency on C++ static globals.
- Confirm normal movement, jump, free aim, lock-on, target cycling, inventory and interaction contexts use the shared action contract.

### P9B Combat encounters and boss

- Author attack requests, combo state, cooldowns, damage decisions, invulnerability, healing and audiovisual event selection.
- Author guard and boss state transitions and encounter activation using shared query/pool mechanisms and shared data.
- Preserve projectile limits, line of sight, target priority, lock marker behavior, skeletal aim and movement/camera independence.
- Keep feedback and sound decisions editable in the selected flavor; underlying audio, rumble and effects remain native services.

### P9C Inventory shop and save rules

- Author item use, upgrade decisions, pricing lookup, ore sale, limits and commit/rollback behavior.
- Build inventory and shop event handling with the shared presentation and control-capture contracts.
- Serialize the same save schema, validate it and preserve persistent economy across run restarts.

### P9D PocketStation orchestration

- Author the PSX terminal flow for install/start, status inspection, transfer and receipt acknowledgement.
- Keep the same Pocket Lua miner asset and low-level device implementation in all flavors.
- Preserve interrupted-transfer recovery, slot selection, timeouts, save-before-acknowledgement and duplicate-receipt rejection through selected-flavor rules.

### P9E Generation and packaging

- Change `dungeon_complete_template::create` to accept GameplayFlavor and use a common content/composition builder plus flavor-specific project sources.
- Update `workspace` to preserve the chosen flavor, Lua profile and execution mode when loading the complete template's manifest defaults.
- Update `project_templates` descriptions, generated READMEs, CLI help, packaging and source filters. Remove the C++-only availability restriction only when both new variants pass their complete acceptance scenarios.
- Generalize `tools/package_dungeon_complete.py` resource traversal so it follows class and asset dependencies for Blueprint and Lua rather than only C++ include relationships.
- Exclude the original authored C++ gameplay from Blueprint/Lua packages. Validate the generated source inventory and the ownership table, not only executable presence.

**Acceptance**

- C++, Blueprint and Lua complete the same playable game routes, shop transactions, item use, saves and PocketStation transfers.
- Semantic state/event traces agree under the same seeded scenarios and logical input timeline; normalize flavor-specific identities while preserving every gameplay outcome.
- Lua Native C++ AOT executes the actual editable Lua rules and fits the same console budgets. Generated C++ is a compilation artifact, never a second authored implementation or hidden fallback. Compiler/ABI gaps use the existing registry/IR/ABI and conformance coverage. VM execution remains separate engine coverage.
- A user can change a guard behavior, a shop decision and a run-flow event directly in each selected authoring format and observe the corresponding change in Play.
- Generated projects and standalone exports contain the expected selected-flavor source and all required shared resources, with no paths back into the original project.

## Phase 10 Full validation documentation and delivery

**Work**

1. Run the complete 18-case generation/build/Play matrix: six templates in C++, Blueprint and Lua AOT. Reuse the same scene scenarios and validation protocol across providers. Do not add Dungeon Complete VM cases to this release matrix.
2. Verify standalone PsyQo exports rebuild outside the editor and original project directory for every execution combination, with matching feature scenarios and resource manifests.
3. Test helper placement, component attachment, derived classes, inherited defaults, reset, instance overrides, duplicate/remap, undo/redo, save/reopen, missing assets, external source changes and transactional failure.
4. Validate scene and template dependency footprints. Editing a conversation-only translation, portrait, style, helper base or game rule must invalidate the correct build inputs. Unrelated content must not certify a stale build.
5. Measure native frame times, CPU work by equal simulation-step count, Main RAM, stack/arena peaks, VRAM, actor/component costs and HUD drops. Compare matching routes, settings and instrumentation against P0 and the P8 refactor.
6. Keep the existing Dungeon Complete slow-frame acceptance threshold, streaming/music checks and missing-sound checks. Any change to a performance gate is a separate explicit design change with measured justification, not a way to pass a new flavor.
7. Validate embedded Play and standalone PCDrv/CD paths used by the templates. General helper APIs remain transport-independent. Physical device results are recorded separately; emulator evidence does not certify hardware behavior.
8. Update public documentation for Game Helpers, interaction, dialogue, characters, extension points and all template flavors. Regenerate the API reference from native declarations using the repository's existing tool.
9. Update maintainer architecture and initiative records with final ownership, accepted limits, schema changes, removal of copied implementations and validation evidence.
10. Deliver an explicit completion report with the 18-case matrix, source hashes, budgets and any failed or unrun required scenarios. Required cases that are unrun keep the initiative incomplete. Record engine VM regression results separately.

Open capture check from P5: direct 640x480 interlaced VRAM captures in the default
camera-view fixture contain alternating old/current fields. Diagnose capture and
field synchronization before claiming final 480i visual acceptance. The preserved
evidence is `artifacts/gameplay-helpers/p5/camera-views/raw-480i/`; reproduce with
`tests/integration/verify_camera_views.py --width 640` in a separate output folder.
Progressive camera-view captures and passing 640x480 behavior/export checks do not
close this visual check.

**Acceptance**

- All six template cards and CLI template names support all three gameplay flavors.
- Every generated game uses the common gameplay base and retains editable selected-flavor rules.
- The interaction/dialogue workflow works from a fresh Basic project in both spatial domains.
- No required combination fails generation, build, Play, standalone export, behavior parity or console resource gates.
- Documentation describes actual behavior, and obsolete claims that complete games are C++-only are removed.

## Validation scenarios and evidence

Each phase records implementation commits, commands, generated project manifests, source ownership, executable hashes and results under the existing ignored `artifacts/` workflow. A successful compile does not replace behavioral acceptance.

| Area | Required scenarios |
| --- | --- |
| Input and lifecycle | Simultaneous jump/interact binding, held confirm, rapid press/release, zero-step and catch-up frames, nested captures, pause, disable, destruction and scene unload |
| Interaction | Two candidates, priority ties, out-of-range confirmation, unavailable target, sensor exit, invalid initiator, overflow and destruction during a callback |
| Dialogue | Linear lines, conditional branches, disabled/all-unavailable options, choice pagination, events, loop rejection, cancellation, busy/reentry and participant replacement |
| Text and styles | Accents, imported glyphs, markup, long translations, 511-byte boundary, page breaks, every style, both supported example resolutions and off-screen bubble targets |
| Authoring | Place, attach, derive in each flavor, Call Parent, inherited properties, replace presenter, duplicate, rename, reparent, undo/redo, save/reopen and source conflict |
| Cooking | Late-branch resource, dialogue-only string/font/portrait/audio, missing asset, invalid signature, changed source, bank unload/reload and standalone relocation |
| Characters | Grounded run, walk/run transition, jump/fall/land, step up/down, ceiling/wall collision, moving/one-way 2D platforms, camera obstruction and input source replacement |
| Pickups and doors | Correct initiator, explicit collection, one-time reward, unmet/met requirement, opening collision, switch activation and reset |
| Complete game | Intro, guards, boss, lock-on/free aim, attack flow, death/retry, victory, damage feedback, inventory, shop, successful/failed save, restart persistence and all terminal operations |
| Persistence and devices | Same save schema across flavors, card missing/full/busy, interrupted transfer, repeated receipt, device timeout, installer restart and save-before-acknowledgement |
| Resource use | Unused-feature cost in Basic, maximum conversation/choice load, worst complete-game phase, Lua arena/stack peaks, actor/component headroom and no dropped/rejected runtime work |

Use focused host/runtime tests for observable transitions, bounds and failures. Exercise the same native kernels from host acceptance where possible rather than rewriting them in Rust as an oracle. Reuse the gameplay API parity machinery to prove operation availability and calling semantics across providers.

Extend existing `verify_dungeon_complete.py`, `verify_dungeon_complete_economy.py`, Lua mode verification and packaging tests. Add focused helper/dialogue and template-flavor harnesses where existing tools do not cover the scenario. Make editor paths, generated project paths and telemetry queries explicit so the harnesses can use canonical debug binaries and any selected flavor. Use dedicated test projects and disposable card copies; never run destructive persistence scenarios against a user's working saves or physical device by default.

The template matrix runner should produce one machine-readable row per execution case with generation, source inventory, reflection, build, Play, scenario, export and resource results. Compare deterministic semantic outputs rather than source text or provider-specific memory layouts. Record missing, failed and skipped results distinctly.

### Build and disk discipline

Follow [local testing](../../maintainers/testing.md) and the session's build artifact policy:

1. Before substantial builds, inspect repository Cargo configuration, `CARGO_TARGET_DIR`, free space and canonical cache size. Reuse `target/` unless the repository already configures another directory.
2. Keep the pinned toolchain, debug profile and compiler flags consistent. Do not create per-phase Cargo caches or use release builds for normal development.
3. Treat 15 GiB as a cache review threshold. Inspect obsolete generated outputs before adding another cache; clean only verified build paths within scope, with no running build or links outside the workspace. Preserve evidence and report recovered space.
4. Build the standalone reflection extractor before tests that create reflected Actor projects. Rebuild the editor when its embedded runtime headers change, and serialize emulator runs.
5. Run relevant focused checks per change, then the repository's required full checks at integration milestones. Avoid rebuilding the editor while an acceptance run is using it.

The existing integration check sequence, with RTK as required by this workspace, is:

```powershell
rtk cargo fmt --all -- --check
rtk cargo build --locked --bin epok-header-tool
rtk cargo test --locked --workspace
rtk cargo clippy --locked --all-targets -- -D warnings
rtk cargo build --locked
```

Native/MIPS and emulator checks remain additional explicit acceptance steps. Use the existing API-reference generator after reflected runtime changes; generated reference files must not be edited by hand.

## Versioning and project ownership

- Introduce new native class/member IDs once and preserve them through later refactors. Keep semantic template roles independent of source names and flavor-specific class IDs.
- Version dialogue, style and helper installation data explicitly. If text reveal or new bindings change a serialized layout, update each authoritative codec, generated initializer, preview adapter and dependency signature together.
- Newly generated templates use the shared base. Existing project-owned scripts and assets remain owned by that project and are not overwritten on editor update or helper placement.
- Offer an explicit, reviewable replacement workflow for a current-format actor when useful: create a derived helper composition, map supported properties/references and retain the original source until the user removes it. Do not add Entity/Behaviour compatibility or restore retired document formats.
- Reusing or cloning a helper preserves declared external asset references and remaps internal actor/component references. Conflicting packages or user modifications require a clear diagnostic and transactional handling.
- Source samples describing complete independent controllers can remain educational examples. They must not become a second production implementation that new templates silently depend on.

## Main risks and planned resolution

| Risk | Resolution and gate |
| --- | --- |
| Dungeon Complete is already at its actor limit | Inventory composition in P0 and reclaim measured capacity in P8A before new placements; do not silently raise the global limit |
| Required AOT game exceeds RAM or frame budget | Measure all three native providers, reduce measured common/resource costs and retain the same game features and existing gates across the 18 cases |
| Blueprint/Lua expose signatures but cannot express actual rules efficiently | Use subsystem fixtures before ports, extend the common reflection/IR/ABI as needed, batch mechanism operations and validate editable game decisions in P9 |
| A native facade hides the original C++ game | Enforce the ownership table, source inventory and representative user edits; native services own generic mechanisms only |
| Input capture releases incorrectly or double-consumes a press | Set the event timeline in P1 and test nested ownership, reentry, frame/tick boundaries and every exit path |
| Helpers consume all component slots | Publish composition costs, move shared bookkeeping to services and keep character presets minimal; record remaining extension slots |
| Conversation-only resources disappear from builds | Add typed reachability to the existing dependency/cook path in P3 and verify late-branch content in fresh exports |
| Shared code changes alter template gameplay | Preserve baseline routes and tuning, compare semantic traces and native budgets after each template/subsystem migration |
| Packaging carries only native include dependencies | Update packaging to follow the shared class and asset dependency graph before enabling complete-game flavors |
| Receipt/save behavior changes during porting | Keep the shared save schema and test interrupted and repeated transactions before accepting each flavor |

## Definition of completion

The initiative is complete when a user can create any of the six templates in C++, Blueprint or Lua AOT, place and extend an interaction/dialogue helper without copying engine systems, and build, play and export the same supported game behavior within console budgets. All 18 required authoring/execution combinations and their standalone exports must pass. Lua rules remain editable source; generated C++ is only a build artifact. VM bytecode/source functionality and engine regressions are preserved separately and do not require Dungeon Complete execution for this delivery.

The component-inspection/focus-authoring milestone passed 1,251 workspace tests / 88 ignored, Native Play and three console modes with nine independent widgets, plus nine inspected component panels. Scene 8 / Blueprint 8 / companion 4 preserve persistent authored focus identities. Its provenance remains `artifacts/gameplay-helpers/p4/widget-focus-links-validation.json`.

The focus lifecycle repair passes **1,252 workspace tests / 88 ignored**, formatting, Clippy, debug build, API freshness, five tool tests, the full native Blueprint runtime suite and pinned MIPS syntax. Native Play and all three console modes prove target-only destruction and slot reuse safe in the C++/Blueprint/Lua widget classes, with 15 glyphs and zero drops. Failed spawn rollback and level unload also pass. The original alias reproduction now passes; prior failing evidence is preserved. VM-source standalone export rebuilds byte-identically, including a fresh build from relocated exported inputs outside the source project. Unused Basic uses 968,624 static bytes / 425,984 EXE bytes, a +2,048 / +2,048 byte delta from the focus-authoring snapshot. Provenance: `artifacts/gameplay-helpers/p4/widget-focus-lifetime-validation.json`.

The real-conversation preview milestone passes **1,255 workspace tests / 88 ignored**, formatting, Clippy, debug build, API freshness and five tool tests. The common UI class designer runs the actual native Session, input router, DialoguePresenter and HUD compiler against authored controls and localized pages, with explicit condition values and an event trace. Project callbacks are not linked. Forty-eight cases cover three providers, four starters, two languages and two resolutions; all report zero drops. Real ImGui input, held confirm, cancellation, unavailable options, timing, context restart, separate logical/physical hierarchy, unchanged class data and save/reopen pass. Twenty-four native renders, a full 640x480 portrait render and six designer captures were inspected. Unused Basic remains byte-identical at 968,624 static bytes / 425,984 EXE bytes. Runtime headers are unchanged; no new console/export claim is made. Provenance: `artifacts/gameplay-helpers/p4/widget-conversation-preview-validation.json`.

The widget-owned appearance implementation passes 1,258 workspace tests / 88 ignored, Clippy, debug builds, API freshness and native host checks. Dialogue Style v2 stores timing and sounds only; v1 reads without rewriting and explicit migration saves the complete original package. All four starters use ordinary project-owned controls/textures. DialoguePresenter uses 13 exposed properties, including one two-word DialogueTiming configuration value, leaving three project properties within the unchanged hierarchy limit of 16. The starter test adds those three properties in every style/provider. New Lua class skeletons keep their sample speed assignment commented, preserving the same extension room as C++ and Blueprint.

The console dialogue fixture now derives from the widget base and uses its authored text areas. It passes long text, four pages, six choices, typed callbacks and capture release, and its standalone rebuild is byte-identical. The shared helper fixture passes Native C++, VM bytecode and VM source; the last mode's export also rebuilds byte-identically. Unused Basic remains byte-identical at 968,624 static bytes / 425,984 EXE bytes. Ninety-six native renders cover 48 provider/style/language/resolution cases with zero drops and 32 pixel-identical provider groups. Twelve CLI-created classes preserve identity/composition, and sixteen generic designer captures were inspected. Provenance: `artifacts/gameplay-helpers/p4/widget-style-ownership-validation.json`.

P4 acceptance is complete; see [the audit](p4-acceptance.md) and `release-validation.json`. P5 includes shared characters and four camera rigs, including SpringArm. The remaining P8–P10 work retains every template in three authoring flavors, complete-game ports and the 18-case AOT acceptance matrix established above.
