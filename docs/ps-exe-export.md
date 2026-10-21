# Export PS-EXE

**Build > Export PS-EXE...** creates a ZIP containing a PlayStation executable and
the files required to run it through a PCDrv-compatible loader. Exporting builds
the game; it does not need a connected console, a serial port or an emulator.

Use it to hand a game to a person or service that loads PS-X EXE programs, such
as a Unirom-based serial loader with a PCDrv host. For a bootable CD image use
**Package PSX Disc...**; for buildable C++ sources use **Export PsyQo Project...**.

## Package contents

```text
<Project>-ps-exe.zip
├── epok.ps-exe              PS-X EXE program
├── GEOMETRY.BIN             Only when Geometry Streaming produced pages
├── epok-package.json        Machine-readable description of the package
├── README.txt               Launch instructions for this package
├── LICENSE                  Epok runtime license
├── THIRD_PARTY_NOTICES.md   SDK components linked into the program
├── licenses/                License texts those notices refer to
└── debug/                   Optional: epok.elf and epok.map
```

- `epok.ps-exe` and every PCDrv file are at the archive root, which is the PCDrv
  root. The runtime opens `GEOMETRY.BIN` by that exact, upper-case name.
- A fully resident game has no `GEOMETRY.BIN`; its program reads no files at run
  time.
- Files are chosen from the validated build's metadata. The ZIP never copies the
  build folder, the project's source assets, disc images, memory cards or other
  outputs left by earlier configurations.
- Builds using a Lua VM execution mode also contain `licenses/Lua.txt`, taken
  from the pinned SDK.
- Debug symbols are off by default and are never needed to run the game.

`epok-package.json` is Epok's own description. Loaders do not need to read it.
It records the format version, project and Epok version, the executable name,
the PCDrv root (`"."`), the content scope, initial scene and included scenes, the
Lua execution mode, the display mode, loader requirements, and the path, size,
SHA-256 and role (`executable`, `pcdrv` or `debug`) of each payload file.
`omitted_content` is always empty: exports never leave authored content out.

## Exporting from the editor

Save all changes first; packages are built from saved project files, exactly as
the command line builds them. The dialog offers:

| Setting | Behavior |
| --- | --- |
| Content | **Whole Game** (default) starts at the project startup scene and includes every scene the game can load. **Current Scene** exports only the open map. **Selected Scenes** exports up to 16 chosen maps and an initial scene. |
| External data | Shows the runtime files to expect: `epok.ps-exe` alone, or with `GEOMETRY.BIN` when Geometry Streaming is enabled. |
| Debug symbols | Adds `debug/epok.elf` and `debug/epok.map`. |
| Compatibility | Lists the project's XA music and video assets, which stop the export when the exported content uses them. |
| Destination | The `.zip` path; **Browse...** opens a save dialog on Windows. The default is `exports/<Project>-ps-exe.zip`. |

Export runs as an ordinary build job: progress appears in Console and the busy
overlay offers **Cancel operation**. When it finishes, the dialog shows the
package location and **Open Folder**.

## Command line

```powershell
epok-editor --project "C:\Projects\MyGame" --export-ps-exe --output "C:\Exports\MyGame-ps-exe.zip"
```

| Option | Meaning |
| --- | --- |
| `--output <file.zip>` | Destination; defaults to `exports/<Project>-ps-exe.zip` in the project |
| `--content whole-game\|current-scene\|selected-scenes` | Content scope; default `whole-game` |
| `--scene <assets/scenes/X.epokmap>` | The current scene for `current-scene`; defaults to the startup scene |
| `--selected-scene <path>` | Repeat for each scene; implies `selected-scenes`. Without it, `selected-scenes` uses the selection saved in Play options |
| `--initial-scene <path>` | Initial scene; must be one of the selected scenes |
| `--debug-symbols` | Include `debug/epok.elf` and `debug/epok.map` |

The command prints build progress and the final archive path. Compilation,
compatibility, memory and packaging failures return a nonzero exit code. The
editor and the command line use the same backend, so equivalent exports produce
the same runtime payload. An unchanged repeat export reuses the certified build
and reproduces the ZIP byte for byte.

## Build configuration

Every export builds with a fixed hardware configuration held in memory: the
PlayStation runtime, PC-on-demand data through PCDrv, and the Serial runtime
start-up that keeps the loader's resident interrupt handler installed after
PsyQo initializes (see [Play targets](play.md#physical-psx)). The project's Play
profile, serial preferences and content are not changed.

The export uses the normal `.epok/build` directory, SDK caches, staged-input
certificates and memory-headroom validation. The request is part of the build
cache key, so a cached Play build is never reused for a package. A later Build
or Play rebuilds with the Play profile.

## Compatibility

- **XA music and video are rejected.** Serial Play omits them with a warning;
  an export stops instead and names each affected asset. Their playback needs
  the PlayStation CD decoder, which PCDrv cannot serve. Use resident
  MusicSequence assets, remove the videos from the exported scenes, or use
  **Package PSX Disc...**.
- Resident sound effects and MusicSequence music work as in other builds.
- **PCDrv reads are synchronous.** The game waits while a geometry page is read,
  so loading time depends on the host link and differs from CD playback.
- If `GEOMETRY.BIN` cannot be opened, has the wrong size, or a page fails its
  integrity check, streaming stops for the session and the failure is counted
  in the streaming statistics. Pages are not retried.
- If the BIOS interrupt queues are invalid at startup, the program stops with
  an error message rather than run without the resident handler.

## Package guarantees

Before an archive is published, Epok:

1. checks that the build used the export configuration;
2. reads the external file list from the build metadata the summary certified,
   and accepts only portable PCDrv names supported by the runtime;
3. checks each file against the hash recorded for the validated build and the
   staged-file certificate;
4. hashes every file while writing it into the ZIP, rejecting any change;
5. reopens the finished archive and checks that it holds exactly the planned
   entries with the written bytes.

The ZIP is written to a hidden `.partial` file beside the destination and
replaces it only after these checks succeed. Cancellation and failures delete
the partial file and leave a previous package untouched.

## Running a package

1. Extract the ZIP into an empty folder.
2. Set the PCDrv server's root folder to that extracted folder.
3. Upload and run `epok.ps-exe` with a loader that provides PCDrv host file access.
4. Keep the PCDrv connection active for the whole session.

No CD needs to be mounted. The packaged `README.txt` repeats these steps for the
specific package, including whether it needs PCDrv at all.

## Limits

- PCDrv payloads are limited to streamed geometry (`GEOMETRY.BIN`).
- Packages are built for NTSC timing, like all Epok builds.
- The package states requirements only. Compatibility with a particular remote
  loader or console service is confirmed only by running the package there.
