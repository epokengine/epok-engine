# Full-controller comparison fixtures

The current Third Person starter uses `ThirdPersonInput.hpp`, `.cpp` and `.lua`,
or the shared-input Blueprint emitted by `src/third_person_blueprint.rs`.
Character, movement, camera and animation mechanisms belong to the native helpers.

The older full controllers remain isolated comparison fixtures:

| Provider | Fixture |
| --- | --- |
| C++ | `ThirdPersonController.hpp`, `.cpp`, `CharacterMotion.hpp`, `CharacterClips.hpp` |
| Lua | `ThirdPersonController.lua` |
| Blueprint | `src/third_person_blueprint_legacy.rs` under the test configuration |

Each fixture uses its own project because it retains the original controller
name and identity. Current starter generation installs only the shared-input
implementation in the selected language.

`legacy_controller_oracles_compile_independently_of_the_shared_starter` creates
separate Basic projects, installs each comparison implementation and compiles
its C++ or generated native code through Native Play. It starts the ordinary
Basic scene to verify the result loads, without installing a Third Person world
or binding the old controller to its actors. The Blueprint's graph generation
and the three source tuning/operation comparisons have their own tests.

These checks preserve compilable reference implementations for the extraction.
The current starter's movement, animation, arena, camera and cross-provider
runtime acceptance use the shared-helper routes.
