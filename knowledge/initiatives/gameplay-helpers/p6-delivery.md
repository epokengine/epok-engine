# P6 delivery and acceptance

Status: validated, 30 October 2026. P7-P10 remain open.
Machine-readable evidence: `artifacts/gameplay-helpers/p6/world-validation.json`
and the earlier `p6/catalog-validation.json`. The recorders remain beside those
reports. Runtime contracts and configuration examples are documented in
[world gameplay helpers](../../../docs/world-gameplay-helpers.md).

## Delivered behavior

The catalog projects the existing resolved class model, including user-derived
classes. It places actors and attaches components through the common operations.
Resource installation preserves user files, journals its own creations and rolls
them back after failure. Package identity/version stays in ordinary metadata.
The previous catalog milestone records real drag, undo/redo and persistence.

`world_gameplay.hpp` supplies GameplayFlag, Pickup, Door and Switch components.
ExaminableObject, DialogueNPC, Pickup, Door and Switch have 2D and 3D presets.
Existing trigger, player, camera, command and animation helpers remain part of
the same catalog. All are available in Basic independently of other templates.
Dialogue appearance belongs to ordinary editable `UIActor` subclasses of
`DialoguePresenter`, with their own controls and C++/Blueprint/Lua behavior.

Pickup state is explicit and rewards once. Flags survive owner inactivity while
typed references reject destroyed owners. Doors commit collider changes before
physics, revalidate queued requirements and restore prior collider state on
replacement/end. Reset restores authored state. Pure policy hooks and result
events extend the shared transitions. Host tests cover rejection, reentry,
destruction during events, reset, one-shot switches and collider restoration.

The C++ class factory now resolves native parents from the shared reflection
registry rather than maintaining a separate allowed-parent list. Blueprint and
Lua use the same native parents. Lua named enum constants lower to typed shared
IR literals in all execution modes and appear in generated editor definitions.
Enum-only programs retain the required header without retaining helper pools.

Catalog preview resolves the actual inherited composition, component counts,
capabilities and referenced project assets. It uses the placement resolver and
control composer without installing files. The default dialogue composition
reports 12 actors and 29 components. Capacity refusal leaves scene, files and
undo unchanged, including the shared CLI/MCP actor placement path.

Multi-selection duplication now remaps the union of selected branches once.
References between copied helper actors/components point to their copies;
external references stay external. The regression checks flag/initiator bindings,
Reset to Parent, undo/redo and save/reopen in both spatial domains.

## Validation

| Check | Result |
| --- | --- |
| Full workspace at the world/preview checkpoint | 1,292 passed; 88 intentionally ignored |
| Subsequent common duplication change | 18 focused tests passed, including one new both-domain helper regression |
| Lua regression | 84 passed; 5 ignored |
| Native world/interaction host suites | Passed |
| Native Play trigger collection | Both domains passed actual physics, once-only rewards and foreign overlap rejection |
| PSX execution | Six runs, 18 provider cases: both domains x three Lua modes; each run contains actual C++, Blueprint and Lua policies |
| Parent events, policy refusals and state records | Identical provider records; zero failed probe checks |
| Standalone export | Both domains rebuild byte-identically to the played executable |
| Clippy / formatting / whitespace | Passed; Clippy warnings denied |
| API reference and parity inventory | Fresh; 144 Epok headers / 2,633 callables; 1,035 public gameplay entries, zero pending inventory entries |
| Parity inventory tool tests | Five passed |

The PSX route covers pickup denial/foreign initiators/collection/reset,
deactivation with retained flags, switches, queued open/close/lock/reset,
requirements revoked before commit, typed enum/bool events and native parent
calls through all providers. The largest fixture allocation is 1,616,272 static
RAM bytes, leaving 480,880 unassigned bytes before runtime heap/stack usage.

The six execution runs froze editor SHA-256
`472772cf58822d2f6725a2e993656308c605846b4ba1cb01a0fd0f4bd589a6ef`.
The final editor includes the later host-only preview/duplication changes and has
SHA-256 `0a26cfbb64a673cbea5e2133783c430a4c3a2e5147988ec53da353455ceb9973`.
Runtime headers remained unchanged across those checkpoints; the evidence stores
their hashes and each run's executable hash. The final editor performs the cost
checks below.

## Optional cost

| Basic project | Static RAM | Executable | Retention |
| --- | ---: | ---: | --- |
| No helpers | 968,624 bytes | 425,984 bytes | Zero delta from the P5 Basic baseline; executable hash also unchanged |
| One shared flag | 977,560 bytes | 432,128 bytes | World gameplay enabled; interaction/control polling and pickup/door/switch pools absent |

These are real linked measurements. Pool presence and compiler flags are checked
in generated build inputs. Linked allocations are not runtime heap/stack peaks.

P6 acceptance covers the common helper library and authoring workflows. Full
template migration, the complete-game ports, hardware performance and the final
30-case template acceptance matrix remain P7-P10 work.
