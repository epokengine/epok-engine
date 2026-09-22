# Independent game projects

Epok starts in **Projects**. Create a project with a template, a gameplay flavor, a name and a parent folder, or open an existing project folder. Recent projects are shortcuts to their folders; **Remove from list** forgets a shortcut without deleting the game. **Basic** contains a camera and an empty scene; **Sample** is a ground plane, two cubes and one Spinner behaviour. The default location is `Documents/Epok Projects`, outside the editor installation.

Use **File > Projects... (New / Open / Close)** to return to the selector. Unsaved scenes offer Save and close, Discard changes and Cancel. Closing a project stops its owned build/emulator before releasing its lock. A failed open stays in the selector and displays the error; it never substitutes a sample scene or overwrites a damaged project.

The dark Hub opens on a searchable project list. Search matches names and paths; the sort button switches between recently opened and alphabetical order. Each row shows its folder, PlayStation target and compatible editor version. Click a row to open it, or use its **...** menu to open or remove the shortcut. Unavailable projects remain visible so their errors can be inspected. **New project** opens the template browser and **Dependencies** opens the tool paths; both are panes of the same window, not dialogs over it. **Open project** accepts a folder path or the native folder browser. The Hub has its own typography and palette, preserved when returning from the editor.

## New project

The New Project view is a template browser, not a wizard. Cards on the left carry a thumbnail, a title and a one-line summary, and the whole card selects; the panel on the right shows a large 16:9 preview of the selected template, a paragraph about it and what it includes. A preview that cannot be decoded falls back to a plain tile and never stops the editor from starting.

**Project defaults** below the preview holds the two decisions a new project makes, as two lists:

- **Gameplay** — C++, Blueprint or Lua. It chooses which starter implementation the template writes, and nothing else.
- **Target** — PlayStation, the only target Epok builds today.

Project location and project name stay at the bottom with the resolved destination folder. **Create project** is unavailable while the name or the location is unusable and the reason is shown next to them; Enter in the name field creates when the fields are valid. **Cancel** returns to the project list with the selections untouched.

### Gameplay flavors

The flavor is a generation choice, never a project mode. A Lua project can gain C++ classes and Blueprints afterwards, a C++ project can gain Lua scripts, and a scene can bind all three at once. Nothing in the descriptor records the choice, because nothing in the engine acts on it after creation.

| Flavor | What the template writes | Console build |
| --- | --- | --- |
| C++ | Project-owned sources under `assets/scripts` | Compiled with the rest of the project |
| Blueprint | A project-owned graph under `assets/Blueprints` | The backend generates native code; the source you edit stays visual |
| Lua | Project-owned `.lua` under `assets/scripts` | Built with the project's Lua execution setting, which starts on the ahead-of-time native mode |

Every template accepts every flavor, and the three share one scene: the same actors, transforms, colliders and assets, with only the class the Player binds differing. Basic writes no gameplay in any flavor, and its card says so. A template that ever gains an unimplemented pair disables that flavor in the selector and explains why; creation never silently falls back to another language.

Opening a project shows a centered, borderless 640 × 400 splash with abstract
retro artwork, the original Epok logo, the project name and live loading stage.
Its activity bar does not invent a percentage. Once loading finishes, the window
restores the Hub/editor size, position and maximized state. A failed open restores
the Hub and displays the error. Startup does not add an artificial splash delay.
The artwork is embedded, so it works outside the installation's working directory.

## Ownership and format

**Third Person** adds an optimized editable arena, static blue cubes and an animated player with camera-relative movement, jumping, collision and an orbit camera. It intentionally excludes combat, NPCs and audio. See [Third Person template](third-person.md) for controls, hierarchy and rendering details. The command-line template name is `third-person`.

```text
My Game/
  My Game.epokproject              # Versioned project identity and settings
  assets/scenes/Main.epokmap      # Original scenes
  assets/scripts/                   # Original C++ and .epokscript metadata
  assets/Audio/*.epokasset          # Imported assets with UUIDs and source snapshots
  .gitignore
  .epok/                           # Regenerable build/emulator data; ignored
  UserSettings/                     # Local editor layout; ignored
  exports/                          # Standalone C++ exports; ignored
```

The root-level `.epokproject` descriptor contains `format_version`, `editor_version`, `name`, `startup_scene`, `auto_build` and `rendering`. It is the single writable owner of these settings; it is not an archive and does not contain assets. Open **Edit > Project Settings** to edit it. Rendering defaults to 640 x 480 interlaced NTSC when omitted. The startup scene is a relative path within `assets/scenes`; there is no fixed SampleScene path. Format/version mismatches are rejected. Existing `ProjectSettings/project.json` projects remain readable and migrate only through `--migrate-project <folder>`; reading never rewrites them. Projects with both active formats, or multiple root descriptors, are rejected until recovered.

Track the root `.epokproject` descriptor, `assets/`, `ProjectSettings/` and `.gitignore` in the game's own repository. Moving or copying this content preserves the game; `.epok/` can be removed while closed and regenerated on the next build. Preserve `.epok/migrations/` backups separately before deleting that cache if needed for recovery. Creation requires a new destination and never merges with existing files, and it is transactional: a creation that fails part way removes the folder it claimed, so a half-written project is never offered as a real one.

The editor executable embeds its runtime source snapshot, templates, fonts and emulator adapter. Only generated C++ builds and exports receive runtime copies. Projects do not need editor Rust sources, `runtime/`, `resources/`, `third_party/` or `.tools/`.

Tool configuration belongs to the editor installation: `Editor.epokconfig`, optionally replaced by `Local.epokconfig`. Installation discovery uses `EPOK_EDITOR_HOME`, then executable ancestors with `Editor.epokconfig`, then the development checkout recorded at compilation. It does not use the working directory. Relative tool paths resolve against that installation. An optional, untracked project `Local.epokconfig` replaces the machine configuration and resolves its paths against the project. The manifest owns the initial Auto compile setting.

The Hub stores recent folders in `%LOCALAPPDATA%/Epok/RecentProjects.epokprefs`, separate from both the editor repository and shared game files. Layout belongs to `UserSettings/editor-layout-v2.ini`. An OS-backed `.epok/project.lock` prevents simultaneous editing/building through Epok; a crash releases the lock automatically, so a leftover file is harmless.

## Command line

```powershell
cargo run --locked
cargo run --locked -- --create-project "D:/Games/My Game" --template basic
cargo run --locked -- --create-project "D:/Games/My Demo" --template sample --name "My Demo"
cargo run --locked -- --create-project "D:/Games/My Hero" --template third-person --gameplay lua
cargo run --locked -- --project "D:/Games/My Game"
cargo run --locked -- --project "D:/Games/My Game/My Game.epokproject"
cargo run --locked -- --project "D:/Games/My Game" --build-psx
cargo run --locked -- --project examples/sample-game --play-psx --stop-after 10
```

`--gameplay` takes `cpp`, `blueprint` or `lua` and defaults to `cpp`, so an existing command line keeps generating exactly what it did before. There is no target option while PlayStation is the only target.

Creation is a headless operation; open the resulting folder to edit. Build, Play, lighting bake and profiling require `--project`. Running from an arbitrary working directory works with the executable's absolute path. Game paths with spaces are tested; the upstream SDK/tool installation still needs a path without spaces.

The old repository-root sample lives in `examples/sample-game/`. It is an ordinary project and also supplies the embedded Sample game template. Rebuild the editor after changing embedded runtime/template sources.

## Runtime and asset contract

Epok ships one matched PsyQo runtime; `editor_version` pins that contract. Dependency resolution, package installation and multi-editor upgrades remain future work.

Maps can hold both the original entities and the newer [actors and components](actors.md). Opening an existing project changes nothing on disk; a map is written as scene document version 5 only once it actually contains an actor or a scene Blueprint, and an older editor refuses a version 5 map rather than opening it as empty — so keep a backup before the first such save. The project descriptor gains one optional setting, *Default Scene Blueprint Parent*, which only proposes a parent for maps that do not have a scene Blueprint yet. The actor model is host-validated; PSX build and emulator validation of actor content is pending on a machine with the PlayStation SDK. See [Migrating an existing project](migration-actors.md).

Imported AudioClips use UUID references and portable `.epokasset` packages with embedded source snapshots. Their index and conversions are disposable; moves are reconciled on opening and while running. See [Assets and audio](assets.md). Native script bindings support stable class/member IDs; old name-based bindings remain readable through the [scripting compatibility adapter](scripting.md).

## Migration and recovery

Opening never rewrites a legacy manifest. Close the project, then run `epok-editor --migrate-project <folder>`. Migration validates settings/startup scene, takes the canonical-root lock, publishes the descriptor exclusively, and preserves legacy bytes in a uniquely named `.epok/migrations/*.backup` file.

If interrupted, both formats may remain active. Choose deliberately: `epok-editor --recover-project <folder> --prefer legacy` or `--prefer descriptor`. The selected format must validate; the other is preserved as a backup. Multiple descriptors are never arbitrarily selected: move unwanted descriptors outside the root before opening. No automatic merge is performed.

The Hub's **Open project** accepts a path and offers separate **File...** and **Folder...** browsers. Folder, `--project <descriptor>`, and positional `epok-editor "My Game.epokproject"` launches use the same root, settings, lock, and recent entry.

## Optional Windows file association

After installing both host executables and the dependency distribution, run `tools/register-project.ps1 -EditorPath "C:/Epok/epok-editor.exe"` to register the quoted open command and application icon for the current user. `tools/setup.ps1 -RegisterProjectFiles -EditorPath <exe>` provides the same optional setup step. Existing non-Epok defaults are respected; choose Epok in Windows Default Apps if necessary.

Use the same command with `-Unregister` before removing that installation. It removes only registrations owned by that executable and leaves other applications' associations intact. `-WhatIf` previews registration/removal. Portable builds need no registration and never register themselves on launch. Double-clicking the descriptor opens the folder-based project, not a scene or archive.
