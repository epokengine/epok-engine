# Gameplay helpers P0 baseline

Source: `fb542cd167fdc297425c0b15fedb901b7c4e0c08`. Measurements use the canonical debug editor, pinned Rust 1.97.1 and the existing MIPS build configuration. No template gameplay or runtime source was changed before these captures.

## Validation status

P0 is validated as the contract and baseline gate for P1. Generation, linked allocations, input/object-model tests and compiler contracts passed. Native route captures produced reproducible evidence, including the complete game's existing performance failure below. Its 25 ms performance acceptance remains failed and is a required P8/P10 issue; it does not invalidate the separately passing input contract. Runtime heap/stack peak instrumentation remains scheduled for P8/P10 and is not reported as measured.

## Reproducible evidence

- `python tools/gameplay_helpers_baseline.py --build`: 18 generation cases, 30 execution entries, ten linked measurements, no unexpected generation/build failures. Sixteen template/flavor projects generate; Dungeon Complete Blueprint and Lua reject creation as expected at this baseline.
- `python tests/runtime/verify_spatial.py spatial object_model gameplay_api`: passed all three native host executables. Existing upstream fixed-point conversion warnings remain.
- `cargo test --locked third_person::tests -- --test-threads=1`: five passed, 1,224 filtered out, zero failures.
- `python tests/integration/verify_gameplay_helpers_contract.py`: real reflection, Blueprint bool override and plain-record access from Lua Native C++, VM bytecode and VM source all compiled/staged successfully. This probe validates compiler contracts, not target runtime execution.
- `python tools/profile_runtime.py --project artifacts/gameplay-helpers/p0/third-person-cpp --output artifacts/gameplay-helpers/p0/runtime-third-person --seconds 12 --min-frames 10 --input-replay 0:0,4000:16,6000:16400,6100:16,8000:0`: 81 observed frames, median 16,832 microseconds; no dropped simulation steps, rejected actor objects or dropped triangles.

The committed contract and identity allocation live in [contract.md](contract.md) and [identities.json](identities.json). Full generated worlds, source inventories, per-mode logs, ELF reports, executable hashes, frame samples and VRAM captures are under `artifacts/gameplay-helpers/p0/`. Baseline generation reuses only projects whose editor/source provenance matches.

## Linked allocations

All sizes are bytes. The RAM column measures linked static allocation; unassigned RAM is the remaining shared budget for heap, stack and transitions. Runtime heap/stack peaks are unknown in the current profiler and remain an explicit P8/P10 instrumentation requirement.

| Template | Flavor and mode | EXE | Static RAM | Unassigned RAM |
| --- | --- | ---: | ---: | ---: |
| Basic | C++ | 421888 | 964400 | 1132752 |
| Sample | C++ | 442368 | 1014968 | 1082184 |
| Third Person | C++ | 677888 | 1471560 | 625592 |
| Third Person | Blueprint | 710656 | 1504296 | 592856 |
| Third Person | Lua Native C++ | 698368 | 1492008 | 605144 |
| Third Person | Lua VM bytecode | 782336 | 1811208 | 285944 |
| Third Person | Lua VM source | 804864 | 1866504 | 230648 |
| Platformer 2D | C++ | 843776 | 1629760 | 467392 |
| Dungeon | C++ | 868352 | 1811184 | 285968 |
| Dungeon Complete | C++ | 1122304 | 1942472 | 154680 |

Fresh template compositions use: Basic 1 actor, Sample 4, Third Person 33, Platformer 56, Dungeon 20 and Dungeon Complete 64. Maximum components on an actor are respectively 2, 3, 3, 6, 4 and 5. These are current compositions, before the proposed Basic GameRoot or new helper classes.

Dungeon Complete's 154680 unassigned bytes are already 9160 bytes short of the current 98304-byte bytecode arena plus the mandatory 65536-byte runtime margin. The source arena plus margin is short by 41928 bytes. These lower bounds exclude interpreter code and other Lua overhead, so P8 must recover additional common resources rather than disable features or raise memory limits. Third Person's measured Lua deltas provide an estimate only; they do not predict the future complete-game port exactly.

## Complete game runtime findings

The debug-symbol PCDrv export built successfully. Current guard and boss acceptance routes both fail the existing slow-frame gate:

| Route | Observed final frame counter | Slow frames | Maximum frame microseconds | Gameplay and resource observations |
| --- | ---: | ---: | ---: | --- |
| Guards | 993 | 10 | 27328 | Three active guards, 44 shots, valid geometry/texture data, no dropped triangles, skipped/failed stream chunks or missing sounds |
| Boss | 988 | 8 | 27328 | Explore, boss and victory states observed; boss health reached zero; resource checks above also passed |

The exported EXE SHA256 is `ac6e815b259dc06e4f85a24321d6c95630e1b2a14e99ce6d6aa324ac99a4c774`. Complete route evidence remains under the project's `.epok/validation/stream-pcdrv-*` directories; `result.json` retains `accepted: false`. The 25 ms threshold is unchanged.

The existing economy/PocketStation acceptance passed its full circuit on disposable generated-project cards: terminal entry, missing card, transfer from the native menu, duplicate-transfer rejection, installation preservation, sale, all three purchases, insufficient funds, full-health item preservation, medkit use and cold save reload. The runner restored the original test-project cards. Evidence is in `.epok/validation/dungeon-complete/dynamic-20261028-173526` and `complete-economy.log` under the P0 artifact root.

## Expected semantic scenarios

| Scenario | Expected contract |
| --- | --- |
| Basic and Sample | Basic contains only its current camera; Sample's selected-flavor Spinner advances rotation using its editable speed |
| Third Person | Common tuning remains max speed 6, gravity 24, jump speed 9, step height 0.6; movement, grounding, locomotion and camera obstruction agree across authoring variants |
| Platformer | Speed 96, jump speed 264 and gravity 720 use 2D pixel units; one-way/platform behavior and one-time coin collection are retained |
| Dungeon | Seed 1207, player-bound key pickup, door opening and regenerated role bindings remain valid |
| Complete game combat | Initial player health 100, initial boss health 280, at most three active guards and existing projectile pools; run transitions retain explore/boss/victory/defeat meanings |
| Economy | Ore sells for 10 credits; medkit/tank base prices 30/80 and buster price 120 times next level; current item/upgrade limits, atomic failure behavior and persistent economy survive restart |
| PocketStation | Save PSX inventory before receipt acknowledgement; duplicate/interrupted transfers cannot award ore twice; the same device program is shared across future PSX flavors |
| Future input capture | A captured confirmation cannot jump, reopen or advance twice; nested capture release and owner/level lifecycle cannot retain a lock |

Semantic role bindings and their current component properties/hashes are in the generated baseline manifest. P7 replaces generation-time name searches with explicit role construction; baseline names here are inventory labels, not new runtime lookup contracts.

## Build cache review

Before building, `CARGO_TARGET_DIR`, `RUSTFLAGS` and `CARGO_BUILD_TARGET` were unset; no applicable Cargo config redirected output. Drive D had 142.41 GiB free. The canonical debug cache occupied 18.978 GiB.

With no Cargo/Rust compiler active, 771 obsolete `epok_editor-76ecfa7c4c5078d6.*.rcgu.o` files from an old build were removed from `D:\GitProjects\GameEngines\Epok\EpokEngine\target\debug\deps`. All resolved paths were checked inside the workspace and for reparse points. This recovered 3720108921 bytes, or 3.465 GiB. Current editor/extractor binaries, reusable dependencies and validation evidence were preserved.
