# Project Settings and Editor Preferences

Open **Edit > Project Settings...** or **Edit > Editor Preferences...**. Both windows use a dark category sidebar, searchable property sections and an **Apply** button. Search spans all categories. Changes remain drafts until applied; close the window and reopen it to discard a draft. Apply is disabled while compiling or playing.

## Project Settings

These settings live in the root `.epokproject` YAML descriptor and travel with the game project.

| Category | Options |
| --- | --- |
| Project / Description | Project name shown in the Hub, the default SoundBank and the default scene Blueprint parent |
| Project / Maps & Build | Startup scene, automatic asset reports (on by default), [Build/Play profile, scene selection and loading transitions](play.md) |
| Engine / Rendering | Native output resolution, retained packets and precomputed visibility |
| Engine / Streaming | Geometry streaming, page pool, RAM budget and disc music behavior |
| Engine / Debug | Independent runtime FPS, CPU, Geometry/GTE, GPU DMA and SPU RAM overlays |

**Default Scene Blueprint Parent** names the class proposed in Map Settings when a map's
scene Blueprint is created. It is a suggestion only: changing it never modifies a map that
already has one, and each map can choose a different parent. Leaving it unset proposes
`epok::SceneScriptActor`.

The default is **640 x 480 interlaced NTSC**, including existing projects that do not yet have a `rendering` entry. Widths of 256, 320, 368, 512 and 640 are available at 240 progressive or 480 interlaced lines. PAL output is not exposed by this runtime yet.

Debug overlays are off by default, persist in the descriptor's `debug` section,
and apply on the next build. See [counter meanings and cost](performance.md#optional-on-screen-debug-hud).

**Apply saves project settings without reopening the scene, refreshing script
declarations or starting a build.** Existing compiled consumers are marked stale;
the next Play/Build uses the new values. Compilation is always requested explicitly
through Build, Play or memory analysis. Legacy `auto_build` fields are accepted for
compatibility but never schedule compilation. Scene View updates independently.

The Game view toolbar has **Fit (4:3)**, **Stretch**, and **Integer (4:3)**.
Fit is the default: it fills the available width or height without cropping,
leaving bars only on the other axis. Stretch fills both axes and may distort
the image. Integer keeps whole scale factors when possible. These modes also
appear under **Editor Preferences > Play > Game View** and are local display
preferences: they work during Play and never change native resolution or build
the game. Existing preference files start in Fit; Integer remains selectable.

```yaml
rendering:
  width: 640
  height: 480
  retained_geometry: true
  precomputed_visibility: false
  streaming_geometry: false
  streaming_pool_pages: 4
  streaming_triangle_budget: 4096
  streaming_prefetch: true
```

**Retained Packets** (`retained_geometry`, default on) keeps the GPU packets of
static meshes alive between frames: corner colours, page UVs and texture words
are written once and rebuilt only when the material, lighting, texture bank,
transform basis or entity generation changes. Each frame writes screen
coordinates, links the packet into the ordering table and refreshes colours only
where fog applies. Animated (skeletal) meshes and materials with UV scrolling
always use the per-frame path. The switch reaches the runtime through
`display.hh` and changes nothing in the image; see
[performance](performance.md) for the measured effect. Turn it off to compare
against the per-frame path or when RAM is tight: retention uses one record per
allocated quad, bounded by the per-frame triangle budget in streaming builds.

**Precomputed Visibility — Experimental** (`precomputed_visibility`, default off)
adds conservative selection of editable-mesh chunks. Camera, object and parent
transforms remain supported. It does not add occlusion or increase the visible
range, and it may lower FPS.

**Position Interpolation** (`motion_interpolation`, default on) under
**Engine > Rendering > Movement** smooths entity and camera translations between
the fixed 60 Hz simulation steps. It does not change movement speed, collision
positions or input sampling. It adds up to one simulation tick (~16.7 ms) of
visual delay, plus fixed RAM and CPU work; compare it on/off for your project.
It does not interpolate rotation, scale, skeletal poses or particle positions.
Camera cuts, scene changes and pauses reset the history. For a scripted teleport,
call `epok::reset_motion_interpolation()` after changing the transform to snap
immediately. The setting applies on the next build, including C++ exports.

**Geometry Streaming — Experimental** (`streaming_geometry`, default off) stores
immutable editable-mesh geometry in 64 KiB CD pages. It may lower FPS or stall
frames. A generated CD image is required. Textures, collision, scripts and other
resources retain their existing storage paths. Visibility and streaming can be
enabled independently.

**Streaming Pool Pages** (`streaming_pool_pages`, default 4, range 2–8) reserves
128–512 KiB for resident page payloads. Active pages load before gameplay when
they fit the pool; later required reads may stall rendering. XA music pauses
during required reads and restarts at the beginning of the track.

**Per-frame Triangle Budget** (`streaming_triangle_budget`, default 4096, range
512–8192) caps triangle storage in streamed builds, including resident geometry
and clipped triangles. Lower values save RAM. Excess triangles are omitted and
counted as dropped. Retained packets share this bounded capacity; objects that
do not fit use per-frame packets.

**Preload Nearby Geometry — Experimental** (`streaming_prefetch`, default on,
inactive while streaming is off) requests a nearby page when a free slot and
the CD controller are available. Additional work may lower FPS. Disable it to
compare demand-only loading. It does not interrupt XA playback or evict loaded
pages for speculative reads.

These controls appear under **Engine > Streaming** and apply on rebuild. See
[Using geometry streaming](streaming.md) for the CD launch requirements and
runtime behavior.

Resolution changes apply on the next build. The generated `display.hh` travels with standalone C++ exports; runtime GPU setup, projection, clipping and blob-shadow projection use that configuration. The image keeps the same camera field of view and is presented at 4:3, including modes with nonsquare pixels.

The output mode currently applies to every scene in the build. Per-scene modes
and runtime resolution changes are not implemented; choosing 512 x 240 here
also changes the title and gameplay scenes, not just menus.

HUD coordinates are native output pixels. Anchors follow the selected canvas size; text remains an 8 x 16 bitmap font. A centered panel stays centered when resolution changes, but an authored 100-pixel width remains 100 pixels. The editor's 2D HUD canvas follows the selected resolution too.

Analog displays and video converters may crop the outer lines. Keep important
HUD content inside margins and verify it on the intended display; backgrounds
can still reach the edges. Changing the GPU display range changes the visible
region, not the pixel size, so it does not automatically scale a HUD to fit.
See the [PSX display range reference](https://psx-spx.consoledev.net/graphicsprocessingunitgpu/#display-startend).

640 x 480 contains four times the pixels of 320 x 240. The higher mode costs more GPU fill work; it does not increase geometry or animation budgets. Interlaced modes can flicker on a CRT. Measured elapsed time drives fixed 60 Hz simulation with at most eight catch-up steps per rendered frame; see [input and time](input-collision.md). Hardware validation is pending.

**Reset Rendering** restores the rendering defaults without changing streaming,
project identity or the startup scene. **Reset Streaming** restores streaming
to off, its pool to four pages, its triangle budget to 4096 and preloading to on
without changing rendering. A startup-scene
change takes effect when the project is reopened.

## Editor Preferences

Preferences are stored in the user's Epok data directory (`%LOCALAPPDATA%/Epok/Editor.epokprefs` on Windows). They apply across projects and are not included in game exports.

| Category | Options |
| --- | --- |
| General / Viewports | Default grid visibility and flight speed |
| General / Play | Integer scaling in Game and smoothing in the external emulator window |
| General / Integrations / MCP | Enable the local MCP server, choose its port, rotate the access key and copy HTTP/stdio client configurations |

Applying viewport preferences updates the current view. A template's saved starting view can override the navigation defaults on opening. Integer scaling uses whole display-scale steps when space allows; smaller Game panels fit the image instead. Game always uses point filtering.

**Smooth Image** controls PCSX-Redux's external debugger window and takes effect on the next Play. It is off by default to keep pixels crisp. Other emulator settings are preserved.

**Enable MCP Server** is off by default. Apply starts or stops the server for the open project; MCP preferences can be applied while Play is running. The access key stays in local editor preferences. See [Integrations / MCP](mcp.md) for available tools, screenshots and connection instructions.
