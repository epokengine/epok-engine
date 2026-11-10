# P1 Gameplay actions and control capture

Status: P1 complete and validated. P2 is next; the recorded Complete performance failure and future VM/helper headroom gates remain open. Updated 28 October 2026.

## Delivered behavior

`runtime/gameplay_control.hpp` provides one bounded native action router, shared by C++, Blueprint and Lua. The 22 reflected operations have pinned identities in `identities.json`. InputCapture has two ABI words and GameplayActionSample has five. Lua can recover the newest matching capture through its owner identity between callbacks.

The service supports four logical pads, 22 actions including eight custom slots, per-port binding changes, and sixteen concurrent capture tokens. Priority and acquisition order select ownership of shared physical sources. Capture release, owner disable/destruction, inherited activation, level unload and service reset invalidate ownership. Token generations survive service reset and cannot wrap into a valid old token.

Frame, pending-tick and active-tick edges remain separate. A consuming read clears the same physical press across aliases and dispatch domains. Opening or closing a winning capture suppresses affected edges; a lower-priority capture cannot discard the winner's input. Raw Input keeps its existing semantics.

Console Play, Native Play and HUD preview select matching hooks through the `gameplay-control` capability. Third Person and Platformer use narrow routed adapters in C++, Blueprint and Lua; Complete uses them in its current C++ implementation. Dungeon inherits the Third Person controller. Capturing movement gates commands while physics continues; Look remains independently capturable. Full character/camera extraction remains P5.

User-facing usage and examples: [Gameplay controls](../../../docs/gameplay-controls.md).

## Validation

| Gate | Result | Evidence |
| --- | --- | --- |
| Native action/lifecycle tests | Pass | `python tests/runtime/verify_spatial.py gameplay_control spatial object_model gameplay_api` |
| Reflected identities, effects and ABI shapes | Pass | `gameplay_control_contract_is_reflected_with_pinned_ids_and_effects` |
| Native capability discovery through aliases and spaced calls | Pass | `gameplay_control_native_aliases_and_spaced_calls_select_the_runtime` |
| Native Play and HUD frame consumption before simulation | Pass | Two `gameplay_control_capture_*` tests; zero-step frame, held button, release and a later independent press |
| Full Rust workspace suite | Pass: 1,186 passed, 88 ignored across four suites | `cargo test --locked --workspace -- --test-threads=1`; 827.25 seconds. Ignored tests retain their existing environment requirements |
| Template generation/build baseline | Pass: 18 generation cases, 30 execution entries, ten linked builds, no unexpected failures | `artifacts/gameplay-helpers/p1-validated/baseline.json` |
| Platformer Blueprint and Lua adapter builds | Pass: four additional linked builds | `artifacts/gameplay-helpers/p1-validated/adapter-builds.json`; Lua native, bytecode and source |
| Final runtime revision | Pass: all fourteen builds refreshed with zero failures | `artifacts/gameplay-helpers/p1-validated/final-validation.json`; four targeted Rust tests also rerun on the final header arrangement |
| C++ / Blueprint / Lua Native C++ on PSX | Pass | `artifacts/gameplay-helpers/p1-provider/result.json` |
| C++ / Blueprint / Lua VM bytecode on PSX | Pass | Same fixture; independently authored calls in each provider |
| C++ / Blueprint / Lua VM source on PSX | Pass | `tests/integration/verify_gameplay_control.py` |
| Standalone export rebuild | Pass: byte-identical with Lua source, C++ and Blueprint | `verify_gameplay_control.py --standalone-only`; `artifacts/gameplay-helpers/p1-provider/standalone-result.json` |
| Lint / generated API reference / parity inventory | Pass | `cargo clippy --locked --all-targets -- -D warnings`; API reference `--check`; parity checker: 651 public rows, zero pending |
| Third Person runtime comparison | Pass: 83 samples, median 16,832 microseconds, zero dropped steps/triangles and rejected objects | `artifacts/gameplay-helpers/p1-validated/runtime-third-person/profile.json`; same replay as P0 (81 samples, same median) |
| Complete encounter comparison | Gameplay/resource checks pass; existing 25 ms acceptance fails | Guards: 990 frames, eight slow; boss: 986 frames, ten slow. Both peak at 27,392 microseconds versus P0's 27,328 (+64) |
| Complete economy regression | Pass: full transaction and cold-save/reload circuit; cards restored | `tests/integration/verify_dungeon_complete_economy.py` |

The provider fixture performs acquisition, recovery by owner, a blocked Jump read, an owning Confirm read, cross-domain consumption, repeated consumption rejection, release and expiry, including an immediate disable/re-enable cycle without a routing read between transitions. Native fixture methods seed input and record values; each provider owns the actual router calls. Expected eight-word rows are `[0x47504331, 0, 0, 4096, 1, 1, 0, 1]` for all three callers.

All three Lua modes passed the immediate-expiry fixture. After the final header include-order repair, all fourteen template builds, the mixed C++/Blueprint/Lua source fixture and its standalone export were repeated. The final editor SHA-256 is `f8dcb85aa7e4282c014f13980171f33c03fc1beb4e086fc220e7b2ef72ddc5b7`; the final standalone EXE SHA-256 is `27bbe7e5f65cb9e002acceb4d68d9735af0a73d077be1a5d3df88a18e584736f`.

The first integration run exposed a Lua adapter returning Fixed where the controller read an axis record. The adapter now returns the reflected sample, and the corrected ten-build baseline passes all three Lua execution modes. The earlier failed report remains under `artifacts/gameplay-helpers/p1` as diagnostic evidence.

Lifecycle expiry also runs when actor activation or parentage changes, including the inert-leaf fast path. This prevents a disabled/re-enabled owner from reviving its capture before the next routed read. Both direct and inherited transient disable cases have host regression coverage.

The economy test's deliberate pending-edge injection now resolves the routed queue from MIPS debug type information and seeds both raw and routed pending queues. It continues to test transaction rules without assuming a host struct layout. Device sampling and capture consumption are exercised separately by replay and provider fixtures. A final native-runner guard also stops simulation if a frame callback starts a scene load; its four targeted control tests pass after that change. Final editor provenance is recorded separately from the earlier ten-build executable hash.

The runtime/economy snapshots above precede the final immediate-expiry/header repair. Their recorded executable remains in Complete's certified export `p1-validated/dungeon-complete-cpp/exports/DungeonComplete-ps-exe.zip`, SHA-256 `afeb04c48a51d294236180b5bce2e47bbe02d48b2c455e970c059f2f28f4904f`. Guard and boss evidence is retained under that project's `.epok/validation/stream-pcdrv-{guards,boss}-20261028-*`. Three guards and 44 shots were observed in the guard route; the boss reached zero health and explore/boss/victory states. Both routes verified 130,912 VRAM bytes, geometry, streaming and audio, with zero dropped triangles, skipped/failed chunks or missing sounds. The strict performance test exits with failure: its 25 ms target remains open, and this snapshot's peak is one 64-microsecond clock quantum above P0. Final linked budgets below use the final source revision; lifecycle acceptance uses the host/provider regressions above.

## Cost against P0

Final measurements are recorded in rtifacts/gameplay-helpers/p1-validated/final-validation.json, refreshed with python tools/gameplay_helpers_baseline.py --output artifacts/gameplay-helpers/p1-validated --revalidate. Authored source identities are checked before reusing the fourteen canonical project build directories; the original generation/build reports are preserved. These are linked allocations, including code, static state and alignment. They are not measured peak heap/stack usage. The canonical router's MIPS symbol occupies 1,036 bytes; host storage is 1,104 bytes. No per-frame allocation or per-character capture pool is introduced.

| Template / provider | P1 static RAM bytes | P1 EXE bytes | Static RAM delta | EXE delta |
| --- | ---: | ---: | ---: | ---: |
| Basic C++ | 964,400 | 421,888 | 0 | 0 |
| Sample C++ | 1,014,968 | 442,368 | 0 | 0 |
| Third Person C++ | 1,479,752 | 686,080 | +8,192 | +8,192 |
| Third Person Blueprint | 1,512,488 | 718,848 | +8,192 | +8,192 |
| Third Person Lua native | 1,500,200 | 706,560 | +8,192 | +8,192 |
| Third Person Lua bytecode | 1,819,400 | 790,528 | +8,192 | +8,192 |
| Third Person Lua source | 1,874,696 | 813,056 | +8,192 | +8,192 |
| Platformer C++ | 1,635,904 | 849,920 | +6,144 | +6,144 |
| Dungeon C++ | 1,823,472 | 880,640 | +12,288 | +12,288 |
| Complete C++ | 1,960,904 | 1,140,736 | +18,432 | +18,432 |

Basic's EXE is byte-identical to P0: SHA-256 `bd69708d3bc1442f9828caf90b1f5c16fb068a35224810ba3312a28a682d0b0e`. Capability selection reserves no router state or frame hooks when unused. No actor or component capacity was increased.

VM arena sizes are already included in linked static RAM. The final report records them once and leaves native-to-VM arena projections unset for existing VM builds; those native projections exclude interpreter/code cost.

Complete now leaves 136,248 bytes unassigned before runtime heap/stack peaks. This reinforces the existing P8 resource-reduction gate for Lua VM support. Complete Blueprint/Lua remain explicitly unavailable until P9; P1 does not claim the final 18-flavor migration is complete.

The additional Platformer builds use 1,642,096 bytes (Blueprint), 1,646,192 (Lua native), 1,959,320 (Lua bytecode) and 2,020,760 (Lua source). Source mode leaves only 76,392 bytes unassigned, or 10,856 above the proposed 64 KiB runtime reserve. P7 must remeasure that template with interaction/dialogue enabled and reduce shared costs before adding helpers; these builds are not proof of future helper headroom.

## Scope carried forward

P2 implements interaction components and placeable trigger/object/NPC helpers. P3/P4 implement conversations and presenters. P5 adds reflected, extensible FirstPersonCamera, ThirdPersonCamera, TopDownCamera, IsometricCamera and SpringArmComponent classes over the existing camera system. The planned isometric preset uses a fixed perspective angle; true orthographic projection needs a separate renderer decision.

The P0 Complete 25 ms encounter failure and unavailable heap/stack peak measurements remain explicit P8/P10 gates. See [P0 baseline](p0-baseline.md). Camera presets and dialogue execution have not been implemented in P1.
