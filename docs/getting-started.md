# Getting started

## Requirements

Use Windows x64, macOS 11+ on Apple Silicon, or experimental Linux x86_64. Install Git and rustup on any host. Windows additionally needs Visual Studio Build Tools with the Desktop development with C++ workload and a Windows SDK. macOS additionally needs Xcode Command Line Tools and Homebrew. Linux setup builds the MIPS compiler locally and needs a native C/C++ compiler, Make, curl, Python 3, tar, xz and bzip2. Python 3.10 or newer is needed for local verification scripts. VS Code is optional.

Use a checkout path without spaces. The upstream MIPS Makefiles do not escape every path. Other desktop platforms are not yet supported.

## Install and run

```powershell
git clone https://github.com/franadoriv/epok-engine.git Epok
cd Epok
powershell -ExecutionPolicy Bypass -File tools/setup.ps1
cargo build --locked --bins
cargo run --locked
```

Use a normal clone. Setup initializes only `third_party/nugget`; do not use recursive submodule initialization for this pinned SDK. Nugget's mirror contains an unrelated xmake submodule with an inconsistent path.

On macOS Apple Silicon, run:

```sh
xcode-select --install
./tools/setup-macos.sh
make run
```

The macOS script installs Rust and the MIPS compiler through Homebrew, builds `psxavenc` and mkpsxiso from their pinned source revisions, downloads PCSX-Redux into `.tools/macos/redux`, and writes an ignored `Local.epokconfig` containing the native executable paths. It does not change the shell PATH: use `make run` or `./tools/run-macos.sh` to start the editor.

On Linux x86_64, run:

```sh
./tools/setup-linux.sh
make run
```

The Linux installer initializes the pinned Nugget SDK, builds the pinned MIPS
compiler under `.tools/linux/mips`, and downloads verified PCSX-Redux,
psxavenc, mkpsxiso and libclang packages. The compiler build can take several
minutes. It does not change the system PATH or require `sudo`.

Exact SDK and download versions are recorded in [dependencies.json](../tools/dependencies.json). Setup verifies archive SHA-256 hashes and checks installed distribution files against those archives, including DLLs, headers and licenses. Existing source changes or a different Nugget revision produce an error rather than being overwritten.

Build/distribute both `epok-editor` and `epok-header-tool`; native reflection uses pinned host libclang from setup. It is not included in PSX games. New projects have a root `.epokproject` descriptor. See [Projects](projects.md) for folder/file opening, explicit legacy migration and optional Windows file association. No registration is required for portable use.

If an installation was interrupted or its distribution files changed, preserve any intentional edits and run:

```powershell
powershell -ExecutionPolicy Bypass -File tools/setup.ps1 -Repair
```

Repair restores distribution files from verified archives and leaves extra files alone. It does not reset SDK source changes. If an archive checksum is wrong, move that archive aside and rerun setup to download it again. Keep the archives in `.tools/` so later validation does not need to download them.

Source ZIP downloads omit submodule content. Setup can fetch the same pinned Nugget revision when run from an extracted source archive.

## Common development commands

The root `Makefile` provides the same Cargo-based commands across host systems. GNU Make is a convenience for developers; direct Cargo commands remain supported. `make setup` provisions Windows, macOS, or Linux x86_64. macOS support targets Apple Silicon and uses Metal through wgpu; Linux editor and Play support remain experimental.

After setup, Windows already has GNU Make under `.tools/mips/bin/`. Use it directly, or add a session-local PowerShell alias:

```powershell
Set-Alias make (Resolve-Path .tools/mips/bin/make.exe).Path
make help
make run
make run PROJECT=examples/sample-game
make release
make app
make check
```

The alias only affects the current PowerShell session. Before the first setup, use the PowerShell setup command above, or `make setup` if GNU Make is already installed. On macOS, `make setup` runs `tools/setup-macos.sh`; common targets find Cargo through Homebrew when it is not in PATH.

| Command | Result |
| --- | --- |
| `make run` | Build and open the debug editor Hub |
| `make run-release` | Build and open the optimized editor |
| `make build` | Compile the debug editor |
| `make release` | Compile the optimized editor binary |
| `make app` | On macOS, create `target/release/Epok Engine.app` and ask whether to copy/replace it on the Desktop |
| `make check` | Check formatting, run default tests, strict Clippy and a debug build, stopping on failure |
| `make test` / `make lint` | Run default tests or strict Clippy separately |
| `make fmt` / `make fmt-check` | Apply formatting or check it without edits |
| `make setup` / `make setup-repair` | Verify/install host PSX dependencies; on Linux the pinned MIPS compiler is built locally |
| `make build-psx` | Compile the sample game for PSX; override with `PROJECT=...` |

`PROJECT` is optional for `run` and `run-release`; quote the assignment for a project path containing spaces. `ARGS` forwards additional editor arguments. For example:

```powershell
make run "PROJECT=D:/Games/My Game"
make run PROJECT=examples/sample-game "ARGS=--window-size 1024x720 --screenshot artifacts/editor.png"
make build-psx PROJECT=examples/sample-game
```

Run Make from the repository root, or use `make -C <checkout> ...`. Relative project and capture paths resolve from that root. Cargo's normal environment configuration remains available, including `CARGO_TARGET_DIR`.

With the default Cargo target directory, `make release` writes `target/release/epok-editor.exe` on Windows. Unix host builds use `target/release/epok-editor` without `.exe`. On Apple Silicon macOS, `make app` builds both host binaries and assembles `target/release/Epok Engine.app`, a single Finder item containing Epok's executable, reflection extractor, Nugget SDK and any locally built audio/disc tools. It asks before replacing `~/Desktop/Epok Engine.app`. Run `make setup` first for full Play support; it installs the Homebrew MIPS compiler and stores PCSX-Redux inside the app's local tools folder.

## First session

1. In **Projects**, choose a name/location, select **Sample game**, and click **Create project**. Then select **Cube** in Hierarchy. The sample attaches **Spinner** to it.
2. Use **Edit Script** in Inspector to open its original C++ source in VS Code.
3. Save a change. With Auto compile enabled, the editor waits 700 ms after a write and compiles in the background.
4. Double-click a compiler diagnostic in Console to open the corresponding original script and line.
5. Press **Play** to compile and run in PCSX-Redux. Click Game to send keyboard input to the emulated controller.
6. Use Pause/Resume, Step and Stop, then save scene edits with Ctrl+S.

A failed build does not launch an older executable. Changes made to C++ during Play wait until that session stops. Play does not save over the authored scene.

## Configuration

The checked-in `Editor.epokconfig` points to the portable tools. To use machine-specific paths, copy it to `Local.epokconfig` and edit the copy. That file is ignored by Git.

The Hub and editor check dependency paths at startup and warn about missing tools,
showing which features need them. Open **Dependencies** in the Hub or
**Edit > Editor Preferences > Dependencies** to inspect the results, edit paths,
or browse for an executable or directory on Windows. **Install / Repair** is one
non-overlapping operation that downloads or repairs all missing bundled packages
on Windows and Linux; on macOS it runs the complete supported setup in the
background. Its live log and completed dependency checks are visible in the
editor. **Apply** saves the effective
installation or project `Local.epokconfig`; changes apply to subsequent builds and
imports without restarting. The first replaced local configuration is backed up
under `.epok/dependencies/Local.epokconfig.bak`.

On Windows and Linux, **Use bundled paths** selects tools in the current editor
installation and repairs stale path settings after moving a checkout.
On macOS, **Install / Repair** runs the supported host setup, which repairs the
shared Homebrew and local tools together and downloads PCSX-Redux into the local
Epok tools folder when needed. Windows and Linux downloads verify the
complete bundled setup in the background using the pinned manifest and SHA-256
checks (Nugget uses its pinned Git revision).
It restores package distribution files, preserves extra local files, and reports
progress or failure in the installation log. Apply the proposed path after a
successful install. Build/Play and dependency changes wait for installation to
finish. On macOS, run `tools/setup-macos.sh`, then **Reload saved paths**.
Repair preserves a package directory junction or symbolic link as a
`.link-backup-*` sibling and installs a real local directory, so links left behind
by a moved installation do not break extraction. Its previous target is untouched.
Installing the MIPS package updates both the GNU Make and compiler path fields.
Discovery checks file presence (and executable permissions on Unix), not binary
compatibility; the build still validates tools when using them.

The local file **replaces** the editor installation configuration; the two JSON files are not merged. Missing fields use built-in defaults.

| Field | Purpose |
| --- | --- |
| `make` | Make executable |
| `toolchain_bin` | MIPS tools directory, prepended to the child process PATH |
| `nugget` | Nugget SDK directory |
| `emulator` | PCSX-Redux executable |
| `psxavenc` | XA audio encoder executable |
| `libclang` | C++ reflection library directory; empty uses the bundled library |
| `mkpsxiso` | CD image builder executable |
| `code` | VS Code executable; empty enables discovery |
| `web_port` | Emulator HTTP port, normally 8077 |
| `auto_build` | Fallback; the game manifest owns the initial Auto compile setting |

Relative paths resolve from the editor installation. An optional game-local `Local.epokconfig` instead resolves its overrides from that game folder. Executable names without separators can be resolved through PATH. Port 8077 must be available; Epok refuses to control a pre-existing emulator session.

Start without arguments to create/open projects, or use `--project <directory>`. Games contain assets and a root `.epokproject` YAML descriptor; runtime and tools stay with the editor. See [Projects](projects.md) for command-line creation, project versioning and portability.

## Build without opening the editor

```powershell
cargo run --locked -- --project examples/sample-game --build-psx
```

Output is written to the selected project's `.epok/build/epok.ps-exe`, with ELF and map files beside it. Edit `assets/scripts/`, not the staged copies under `.epok/build/`.

For a bounded emulator run:

```powershell
cargo run --locked -- --project examples/sample-game --play-psx --stop-after 10
```

## Troubleshooting

- **Host linker missing:** install the Visual Studio C++ workload and Windows SDK.
- **MIPS compiler or SDK missing:** rerun setup and inspect the configuration paths.
- **SDK revision mismatch:** preserve local changes, then update only the Nugget submodule to the recorded revision.
- **Port busy:** stop the other session or choose a free port in the local configuration. Integration scripts currently require 8077.
- **Layout unusable:** use Layout > Default or Window > Reset Layout.
- **Executable locked during compilation:** close the running editor before rebuilding it.
- **No suitable graphics adapter:** on Windows, verify DirectX 12 support and graphics drivers; on macOS, verify Metal support and macOS updates; on Linux, install current Vulkan-capable drivers for your GPU. GPU-free unit tests can still run.

Python validation and migration tools require `python -m pip install -r tools/requirements.txt`. See [document formats](formats.md) for YAML and the UniQo-to-Epok migration.
