# Player vitality extraction

Status: closed for the vitality extraction, with the source/editor scope recorded
in `artifacts/gameplay-helpers/p8/player-vitals/validation.json`. The C++ legacy
comparison, five provider/mode executions, bundled C++ adoption, four standalone
rebuilds and that delivery's workspace suite passed. These are retained results,
not validation of later combat/UI/compiler changes. Integrated P8–P10 acceptance
continues separately across the required 18 AOT template combinations.

DungeonPlayerVitals is a project-owned component for health, hit immunity and
healing eligibility. It is distinct from attacks, aim/targeting, animation,
encounter rules and presentation, which still need further decomposition.

## Ownership and authoring

The engine's HealthState/HealthLibrary provide bounded arithmetic. The native
project DungeonPlayerVitalsBase declares data and virtual dispatch. Its `assign`
method only copies health, immunity and availability; it implements no game rule.

Each provider independently owns these rules in normal editable project files:

| Provider | Source |
| --- | --- |
| C++ | `resources/templates/dungeon-complete/assets/scripts/DungeonCombat/DungeonPlayerVitals.hpp` |
| Blueprint | `resources/templates/dungeon-complete-flavors/blueprint/DungeonPlayerVitals.epokbp` |
| Lua | `resources/templates/dungeon-complete-flavors/lua/DungeonPlayerVitals.lua` |

The Blueprint uses ordinary method graphs, operations, branches and split pins.
Its two value-returning helper graphs capture a health result on an execution
pin before state assignment; a later return therefore preserves that same result.
Lua and Blueprint fixture projects contain no authored C++ vitality implementation.
The common native fixture driver sets inputs and observes outputs only.

## Contract

Four inherited properties expose the configuration value `tuning`, the current
HealthState `health`, Fixed `immunity` and Bool `valid`. Tuning contains unsigned
`maximum` (100) and Fixed `hit_immunity` (0.7). Only tuning is editor configuration.

| Event | Project responsibility |
| --- | --- |
| `start()` | Validate tuning, fill health and clear immunity |
| `advance(Fixed dt)` | Reduce immunity during enabled simulation; clamp it to zero |
| `damage(Int32 amount, Bool enabled) -> HealthChange` | Gate the hit on availability, controls, positive amount and immunity; start immunity after applied damage |
| `heal(Int32 amount) -> HealthChange` | Gate unavailable tuning, apply healing and retain immunity |
| `can_heal() -> Bool` | Pure query for valid health below its maximum |

A maximum of zero, negative immunity or immunity above 60 seconds makes `valid`
false. The game disables controls when the required component is unavailable.
Zero, negative and greater-than-one-second deltas do not advance immunity.
The normal game continues to call advance at its previous tick location, after
intro/menu gates. Opening a terminal therefore preserves the previous pause
behavior. A full-health medkit remains in inventory; a full-health world pack
remains in the level. Default hit immunity remains exactly 2,867 Q12 units.

The coordinator consumes applied/depleted results to select feedback and run-flow
transitions. Those effects and their project rules remain in DungeonGame for the
following extractions. The HUD uses the configured maximum for fraction, crop
and percentage thresholds, with widened arithmetic. Default 100-health tuning
retains its previous arithmetic results. Guard and boss health ownership has not
moved into this player component.

DungeonGame binds the component by ObjectId. DungeonRulesBinding shares the
existing registry/identity/generation-safe type cache with the run-flow binding.
Its host regression covers null/wrong types, deferred destruction, slot reuse
and a different registry recycling the same address and handle bits.

The scene retains 58 actors. Player Aqua now has seven components and one free
extension slot. The authored scene change is exactly one component and its
binding; acceptance normalizes just those additions before comparing all other
scene data to the previous health delivery.

## Evidence and remaining work

`tests/runtime/dungeon_player_vitals.cpp` passes 41,204 exact health/cooldown
checkpoints against independent original-game arithmetic. It also covers the
one-Q12-unit expiry boundary, invalid tuning, rejected deltas, malformed state
and full unsigned limits. Damage after depletion uses the original game's
run-flow-disabled eligibility when comparing legacy behavior; an explicitly
enabled hit on empty health is separately tested as a no-op.

`tests/integration/verify_dungeon_player_vitals.py` runs 480 scripted steps in each
of C++, Blueprint, Lua Native, Lua VM bytecode and Lua VM source. All 6,720 words
per case match an independent integer oracle, including returned HealthChange
records across the virtual Lua ABI. Tuning, disabled controls, exact cooldown
boundaries, depletion/revival, invalid state and UInt32 maximum are exercised.

Evidence is under `artifacts/gameplay-helpers/p8/player-vitals/`. Provider playback
retains its actual editor hash; the final template-carrying editor must rebuild
each native provider image identically before export acceptance. Final Complete
combat, economy/cold reload, fresh install, frozen camera, resource/geometry
identity, export, linked budgets and workspace checks now pass in the scope below.

## Final acceptance

All three provider exports rebuild byte-identically, and each native image from
the final template-carrying editor equals the image actually played in its
provider case. Unused Basic retains 983,160 RAM / 436,224 EXE bytes. The source
scene comparison permits only the new component and its binding.

The fresh complete game passes functional guard/boss scenarios, all 16 economy
events, cold reload and a separate two-event fresh install. Six independent byte
checks confirm original cards were restored or retained absence. All 67 asset
packages and the geometry archive match the preceding health delivery. Complete
standalone export also rebuilds byte-identically. The scene has 58 actors / 153
components, at most eight on any actor and seven on the player.

The original camera fixture's added actors exceeded its instrumentation budget:
only 61,080 bytes remained below the unchanged 65,536-byte minimum. Its failed
log and baseline capture are preserved. The compact camera-only fixture uses an
Input-group component and reuses an inactive guard renderer as the stationary
target. It changes no controller/camera implementation. Full combat acceptance
uses the untouched production scene separately. Both compact cases match every
word of the prior frozen capture: 180 frames / 3,420 words, 55 lock-on, 35 free-aim
and 90 normal rows, including the actual jump. The ring remains 32 rows; lost rows
still fail. Its migrated static RAM is 2,024,440 bytes, leaving 72,712 bytes.
The reduction is 11,632 bytes from the failed instrumentation build.

The workspace passes 1,316 tests / 88 ignored across four suites in 1,094.71 seconds.
Clippy with warnings denied, formatting, whitespace and API freshness/parity pass.
The unchanged API inventory has 146 headers / 2,676 callables and 1,062 public /
6,456 total parity rows, with none pending. The canonical 24,630,048,546-byte debug
cache was reviewed and reused; no extra Cargo cache was created. Four verified
temporary standalone SDK object caches were removed, recovering 21,254,592 file
bytes; paths are in `player-vitals/sdk-temporary-cleanup.json`.

The exact timed PCDrv image uses 2,017,000 static RAM / 1,196,032 EXE bytes, adding
2,680 / 2,048 bytes from the preceding health image. The 25ms gate remains failed:
guards seven slow frames / 25,984us, boss seven / 27,648us. Its memory report was
captured before economy/standalone profile changes and is tied to the exact played
executable hash. This does not measure runtime heap/stack peaks. Remaining P8
mechanisms/rules, complete authored flavors, full-scenario memory/Lua capacity,
the performance gate and P10 acceptance remain open. The closed receipt is
`artifacts/gameplay-helpers/p8/player-vitals/validation.json`.
