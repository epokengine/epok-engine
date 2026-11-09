# P7D Dungeon shared-rule migration

Status: validated. The final workspace run passes 1,310 tests / 88 ignored.
The receipt is `artifacts/gameplay-helpers/p7/dungeon-validation.json`.

Fresh DungeonKey and DungeonDoor classes derive from the common PickupComponent
and DoorComponent in C++, Blueprint and Lua. Project callbacks hide collected
keys, request automatic opening and hide open doors. Native mechanisms admit the
player, commit one-time collection, maintain explicit GameplayFlag ownership and
apply queued collider changes. Visibility alone no longer implies possession.

Template installation records contain typed key, door and key_state roles.
Generation resolves class UUIDs through the common declaration model, preserving
renames and compatible replacement classes. Stable component assignment remaps
the whole generated batch, including cross-actor flag and collider references.
Both domains retain endpoint identities across save/reopen/regeneration.

Existing project sources retain their exact bytes and legacy lock/player/key
contract. Missing-source installation is not an implicit behavior migration.
Lua identities are preserved and invalid identity documents reject installation.
A common journal rolls back newly created sources and changed registry settings
on rejected resolution while retaining concurrent user edits. Settings publication
is atomic. Recorded roles narrowed to the wrong world domain reject generation.

## Completed evidence

- Ten console policy cases cover both spatial domains and all five execution
  choices: C++, Blueprint, Lua Native C++, VM bytecode and VM source, with zero
  object-arena rejections.
- Five Dungeon foundation routes pass with actual selected-language project
  callbacks, player-only pickup and linked doors.
- All five 180-frame replays match the old rules exactly across 1,080 trace words
  each, including positions, key/door visibility and camera blockage. No door
  visibility boundary difference was observed.
- Three standalone exports rebuild byte-identically.
- Three Native Play providers pass ownership independent of visibility, foreign
  actor rejection, one-time collection and explicit reset.
- Three journal tests pass, including exact-byte settings restoration and
  preservation of edits made after an installation write.
- World/resource comparison passes all three providers: 20 actors and 38 resource
  payloads retain their original content after documented identity normalization.
- Source inventory finds exactly the selected authored key/door implementation
  in each provider. Generated Blueprint/Lua C++ remains compiler output.

Detailed evidence is under `artifacts/gameplay-helpers/p7/dungeon-shared/`.
Runtime matrix evidence uses frozen editor
`4e2dcc8f96c356565fd0c3842431b88409a59960e618d9b1de8a5cd0dad47a08`.
Final installer hardening is separately covered by the final source snapshot,
focused tests, debug build, fresh generation checks and the full workspace run.
Its editor is `c124854190b15486fc778e98c77066368eb3f33b426fdbb42cea233d45414d79`.
All 145 runtime headers and the six authored key/door sources match the frozen
matrix and final generation outputs. Clippy, formatting and whitespace pass.

## Preserved failures and limits

The first shared-rule preparation exposed component UUID replacement without
reference remapping. Its failing project is retained; the repaired generator
passes 2D/3D target-resolution and regeneration tests. The first journal settings
check exposed its original assets-only path restriction; explicit contained
ProjectSettings support fixes that case. A Native Play test-only telemetry
constructor error was corrected and the three-provider recheck passes.

The old full workspace snapshot was intentionally interrupted after installer
changes and is not a completed pass. The final run supersedes it. Legacy
preservation tests cover the supported actor-reference contract;
they do not claim arbitrary historical source-schema migration.

Complete-game refactoring, all complete-game providers, runtime heap/stack peaks,
visual/performance/hardware acceptance and the final template matrix remain
P8-P10 work in the main plan.
