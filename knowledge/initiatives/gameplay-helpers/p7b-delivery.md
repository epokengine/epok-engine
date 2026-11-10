# P7B Shared player assembly and Dungeon reuse

Status: validated. P7C, P7D and P8–P10 remain open.

Third Person now builds its player/camera composition independently of its arena.
Dungeon consumes the generated World and Player identities directly, installs the
same player resources and supplies its own world. It no longer creates an arena,
installs its resources and deletes actors found by display name.

Both templates record ordinary class references for `player`, `camera`,
`controller`, `locomotion` and `camera_input` in their versioned installation
settings. Scene references retain actor/component UUIDs. The player uses five
component slots and the camera four. Each template still installs one selected
project input extension in C++, Blueprint or Lua.

## Authored content and behavior

- Six fresh projects pass the composition audit in the three authoring flavors.
- Third Person preserves its complete 33-actor authored scene and all 21 resource
  hashes exactly against the pre-change snapshot.
- Dungeon generates 20 actors and 38 resources, with no unused arena installation.
- Three Native Play cases preserve movement, jumping and camera following after
  renaming every authored actor. Player-only assembly also validates, saves and
  reopens after renaming in all three providers.
- Fifteen PSX execution cases cover the shared-player foundation, arena surfaces
  and Dungeon. Each scenario runs C++, Blueprint and all three Lua modes. Every
  semantic record matches the frozen P5 reference exactly.
- Three Third Person standalone exports rebuild byte-identically, including the
  Lua VM source export.

| Acceptance fixture | Execution cases | Highest static Main RAM allocation |
| --- | ---: | ---: |
| Player foundation | 5 | 1,798,640 bytes |
| Arena surfaces | 5 | 1,792,008 bytes |
| Dungeon foundation | 5 | 2,026,128 bytes |

Every fixture preserves the required 65,536-byte runtime reserve. The largest
allocation leaves 71,024 bytes unassigned. These are fixture static allocations;
runtime heap/stack peaks, physical hardware results and complete-game performance
remain separate P8/P10 acceptance gates.

## Regression and provenance

The full workspace passes 1,298 tests with 88 ignored. Clippy with warnings denied,
formatting, whitespace and the canonical debug build pass. An additional test,
added after that full run, compiles the retained legacy C++, Blueprint and Lua
controller comparisons in independent Basic projects through Native Play. It
verifies compilation/loading; those classes are not starter fallback sources.

The first full run reported one FBX worker Access Denied error. Its isolated retry
and the final full regression pass; the original failure log is preserved.

The execution editor SHA-256 is
`372238958d5aad1eb62b2da8855395dc3d3cd425e6c076cbcca70ab4a6f63371`.
Runtime SDK headers are unchanged by this extraction. Later Platformer work has
its own validation scope.

Evidence:

- `artifacts/gameplay-helpers/p7/player-routes-validation.json`
- `artifacts/gameplay-helpers/p7/player-routes/result.json`
- `artifacts/gameplay-helpers/p7/third-person-before/composition.json`
- `artifacts/gameplay-helpers/p7b-workspace-final.log`
- `artifacts/gameplay-helpers/p7b-isolated-oracles.log`
- `artifacts/gameplay-helpers/p7b-clippy.log`

Reproduction uses `tests/integration/verify_player_routes.py`. It serializes
composition, execution and export checks. `record-player-routes.py` verifies the
completed evidence and frozen P5 records before writing the phase receipt.
