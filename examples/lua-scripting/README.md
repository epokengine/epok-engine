# Lua scripting example

Three authored files — not a whole project. They show the smallest complete
`epok-lua` v1 hierarchy: a reflected C++ base, a Lua class that extends it, and
a second Lua class that extends the first.

| File | What it demonstrates |
| --- | --- |
| `EnemyBase.hpp` | A reflected `EPOK_CLASS(Blueprintable)` `Actor3D` base with a `Fixed` property, a `BlueprintCallable` method and a `BlueprintEvent`. |
| `Guard.lua` | A Lua class extending C++: own properties, a new callable, an event override declared through `overrides`, `begin_play` with `epok.super`, `tick(arg0)` using the reflected parameter name, and a call to an *inherited* native callable (`self:apply_damage`). |
| `Patrol.lua` | A Lua class extending a Lua class: `epok.super(Patrol, self):tick(arg0)`, a constant-bounded numeric `for`, and the explicit `epok.to_fixed` conversion. |

The full reference for the metadata table, the profile and the execution modes
is [`docs/lua-scripting.md`](../../docs/lua-scripting.md).

## Using them in a project

1. Create or open a project, then copy all three files into its
   `assets/scripts/` folder (a subfolder works too):

   ```sh
   cp EnemyBase.hpp Guard.lua Patrol.lua <project>/assets/scripts/
   ```

2. Build. From the editor, press **Play** or **Build**; from the command line:

   ```sh
   epok-editor --project <project> --build-psx
   ```

   `EnemyBase.hpp` is picked up by reflection, then `Guard` and `Patrol` are
   published into the same class registry and generated as real C++ subclasses.

3. Place a `Guard` or `Patrol` actor in a scene. Their `editable` properties
   (`alert_speed`, `awake`, `steps_remaining`, `stride`, `travelled`) appear in
   the Inspector next to the inherited `health`, and per-instance edits are
   ordinary overrides with **Reset to Inherited**.

## Switching execution modes

The mode is project-wide, in **Project Settings > Scripting > Lua Execution**,
stored in the project's `<Name>.epokproject` descriptor:

```yaml
lua_execution: native_cpp   # or vm_bytecode, or vm_source
```

Changing it invalidates the staged script artifacts and relinks; the `.lua`
sources themselves never change. The two VM modes additionally need Nugget's
nested `third_party/psxlua` submodule, which the repository setup scripts
initialize and verify.

## Verified in this revision

These three files were built through the production CLI in **all three execution
modes** on `develop` @ `32fc510` (working tree), macOS arm64, with
`mipsel-none-elf-gcc` from the pinned toolchain:

```sh
cargo run --locked -- --create-project <tmp> --name LuaExample3
cp EnemyBase.hpp Guard.lua Patrol.lua <tmp>/assets/scripts/
# edit `lua_execution:` in <tmp>/LuaExample3.epokproject, then:
cargo run --locked -- --project <tmp> --build-psx
```

| `lua_execution` | Static RAM reported | `epok.ps-exe` |
| --- | --- | --- |
| `native_cpp` | 785.7 KiB | **245 760 bytes** |
| `vm_bytecode` | 945.3 KiB | **309 248 bytes** |
| `vm_source` | 1.04 MiB | **337 920 bytes** |

Every build reported `PSX build ready`. The sizes are this example's, measured
with the emulator toolchain; they are not a game's and not RAM consumption
during play.

The VM-mode difference is the interpreter archive plus the chunk payload, and
the source mode additionally links the parser, lexer and code generator and
reserves a 192 KiB arena instead of 96 KiB — see
[The Lua VM runtime](../../docs/lua-vm-runtime.md#arena-budget).

Each mode emits one generated header per Lua class under
`.epok/build/scripts/generated/lua/`, named by the class UUID, containing the
native C++ subclass with `#line` directives back into the `.lua` source. In
`native_cpp` the method bodies are lowered C++; in the two VM modes the same
headers carry typed trampolines and the build additionally emits
`lua_bindings.cpp` and `lua_chunks.cpp`.

`Guard:wake_up` calls `self:apply_damage(amount)` — a `BlueprintCallable`
declared by the **native parent**, not by `Guard`. It resolves in every mode: the
generated `lua_bindings.cpp` publishes a dispatch slot for it on both `Guard` and
`Patrol` alongside their own methods.

Not validated: these files have not been run on physical PlayStation hardware.
PCSX-Redux is used for reproducible development comparison only, and this
example was built, not executed — the on-target equivalence of the three modes is
established by `tests/integration/verify_lua_modes.py` on its own fixture.
