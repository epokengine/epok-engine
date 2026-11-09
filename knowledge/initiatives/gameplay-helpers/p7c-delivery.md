# P7C Platformer delivery and acceptance

Status: validated. P7D and P8–P10 remain pending. Authoritative receipt:
`artifacts/gameplay-helpers/p7/platformer-validation.json`.
The accepted editor SHA-256 is `b70a9d9f745df06b31e2460445f3d6289064ad4356cd09a2556f9087a856e5a1`.

## Shared mechanisms and project rules

CoinPickup derives from PickupComponent in C++, Blueprint and Lua. The common
base admits the configured player and commits collection once; each project's
editable collected callback hides its coin. Generated typed bindings enable
automatic collection and retain semantic player, controller, locomotion, pickup
and camera roles. Reactivation alone does not collect twice. Reactivation plus
reset_collection permits a fresh collection.

All rendered/physical actors, tuning and 23 resource payloads/settings retain
their original values. Six empty organizational actors become one World2D root,
reducing the starter from 57 to 52 actors. The comparison normalizes only that
organizational parent consolidation, generated identities and pickup bindings.
It checks all three providers.

The interaction fixture places the complete 12-actor Classic Box class through
the ordinary class factory, reaching 64 actors. The existing Guide carries the
conversation and World2D carries the observation component. The player's eighth
component is its interactor. Presentation remains an editable UIActor-derived
project widget with the shared DialoguePresenter API.

## Storage and Lua planning

Cooked class factories share a bounded object arena within the existing 64 KiB
budget. Typed admission bounds, object handles, class IDs and lifecycle rules
remain unchanged. Host tests cover alignment, allocation failure, invalid
releases, reuse, multiple inheritance, destruction and service quarantine.
Allocation remains bounded and can report exhaustion; the arena is not a
guarantee that every class's individual maximum can coexist.

Small Lua programs use a heap plan from their normalized source and frame demand,
rounded to 4 KiB within 32–96 KiB for bytecode and 32–128 KiB for source. Existing
argument, frame and 32-word value limits are retained. Explicit arena overrides
still take precedence. This is a planning estimate, checked against real runtime
behavior and allocator telemetry rather than a formal upper-bound proof.

The full-widget console fixture reports these linked allocations. Its static
figures include the object and Lua arenas; unassigned memory is additional.

| Provider | Execution | Static RAM bytes | Unassigned bytes |
| --- | --- | ---: | ---: |
| cpp | native_cpp | 1,808,336 | 288,816 |
| blueprint | native_cpp | 1,808,336 | 288,816 |
| lua | native_cpp | 1,814,480 | 282,672 |
| lua | vm_bytecode | 1,941,328 | 155,824 |
| lua | vm_source | 1,967,952 | 129,200 |

Every case preserves the mandatory 65,536-byte runtime reserve. The five dialogue
records agree on opening, visible text, input capture and release, and report no
object-arena rejection. These measurements do not establish whole-runtime
heap/stack peaks, physical-hardware performance or final visual acceptance.

## Validation and preserved failures

- Twenty console cases pass. All ten movement/surface records match frozen P5
  records exactly; pickup and dialogue records agree across all five routes.
- Three standalone template exports rebuild byte-identically and contain the
  current object-storage/model headers.
- Four scalar/full-width Lua value cases pass in both VM modes. The broader Lua
  conformance report passes numeric, lifecycle, identity, mode-switch and export
  checks. Its report is `artifacts/gameplay-helpers/p7/lua-heap-conformance.json`.
- Native Play covers all three pickup policies, renamed bindings and reset.
  The complete widget has a fresh Native Play smoke. The full host runtime suite
  passes. The workspace passes 1,301 tests / 88 ignored; Clippy, format, API
  freshness/parity and whitespace checks pass.
- Fresh Basic links no optional interaction, dialogue, character or world-helper
  capabilities/pools. Its measured cost and comparison scope are recorded in
  `artifacts/gameplay-helpers/p7/shared-storage-footprint/result.json`. The P7
  GameRoot starter differs intentionally from the historical camera-only P5
  starter, so their raw RAM/EXE difference is not an unused-helper delta.

Original failures remain in artifacts: the 57-actor starter could not place the
full widget; the first corrected placement exceeded fixed typed-pool storage;
the initial shared-storage source build left only 30,896 bytes unassigned.
Additional harness failures dropped GameRoot from LuaClasses and required an SDK
size tool on PATH. Their passing rechecks preserve the production paths and
the original evidence. None is recorded as a passing runtime result.

The next phase is Dungeon's shared pickup/flag/door adoption and semantic
regeneration roles. Dungeon Complete, all requested project-language variants,
the final full-template matrix and unresolved performance/telemetry/visual gates
remain required in P7D–P10.
