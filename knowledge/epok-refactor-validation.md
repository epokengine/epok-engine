# Epok Engine refactor validation

Validated on Windows x64 on September 8, 2026, from upstream main `811fcb4`.
The work includes the Epok identity, YAML document IO, file/API migrations,
transparent artwork and the branded project Hub.

## Results

- `cargo test --locked`: 156 editor tests passed, 7 explicitly ignored; 2 header-tool tests passed.
- `cargo fmt --all -- --check`: passed.
- `cargo clippy --locked --all-targets -- -D warnings`: passed.
- `cargo build --locked --bins`: passed; editor and header extractor produced.
- Python tool unit tests: 14 passed, including source-preserving migration and YAML string handling.
- Runtime regression report and stream pool layout tests: 17 passed.
- Native runtime harnesses: Blueprint arithmetic/continuations/timelines/spawning,
  spatial transforms/collision, frustum/visibility, palette, streaming and sprites/particles passed.
- Blueprint headers instantiated against the pinned MIPS/PsyQo/EASTL toolchain.
- Lua bridge protocol tests passed using the pinned LuaJIT DLL and mocked emulator services.
- Isolated Windows project file association registration/unregistration tests passed.
- `verify_projects.py`: creation, YAML descriptor opening, incremental MIPS builds,
  relocation, compatible legacy descriptor migration, recovery and export passed.
- `verify_reflection.py`: real Clang extraction, annotated inheritance, typed fields,
  transitive cache invalidation, Unicode project paths, scene banks and standalone rebuild passed.
- `verify_blueprints.py --keep`: native-to-visual inheritance, events, Call Parent,
  Delay, MIPS generation, export and relocation passed (without emulator option).
- `verify_blueprint_features.py --keep`: timeline/template/resource native and debug builds passed.
- `mcp.py --emulator`: real HTTP/stdio MCP negotiation, editor operations, YAML saves,
  screenshots, PSX compilation, emulator Play/input/pause/step/stop passed.
- Migrated an untouched copy of the original RPG example with `tools/migrate_project.py`.
- Captured and visually inspected the running Hub and RPG editor. Both master PNGs
  have RGBA channels with alpha ranging from 0 to 255; packaged PNG/ICO retain alpha.

The current validation reuses installed PSX tools via ignored local paths under
`.tools` and `Local.epokconfig`. A fresh checkout provisions tools through the
existing setup scripts. These local paths and test artifacts are not published.
macOS and physical PlayStation hardware were not exercised in this run.
The seven opt-in Rust tests retain their existing explicit ignore conditions.

Historical documentation captures retain their original scene content. The current
brand, Hub and main editor images are under `resources/branding` and `docs/images`.
