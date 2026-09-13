# Using geometry streaming

**Experimental: geometry streaming and preloading may lower FPS or stall frames.**
Geometry Streaming is disabled by default.

Open **Project Settings > Engine > Streaming**, enable **Geometry Streaming**,
apply the change, choose **CD on demand** or **PC on demand** in the Play profile,
and rebuild. CD launches the disc image. PC reads `GEOMETRY.BIN` from the build
directory through PCDrv, in PCSX-Redux or NOTPSXSerial. The executable alone
cannot supply that external geometry. See [Play](play.md).

| Control | Default | Effect |
| --- | --- | --- |
| Geometry Streaming | Off | Store editable-mesh geometry in external pages |
| Streaming Pool Pages | 4 | Reserve 2–8 pages of 64 KiB each in RAM |
| Per-frame Triangle Budget | 4096 | Reserve capacity for 512–8192 triangles; excess triangles are omitted and counted |
| Preload Nearby Geometry | On when streaming is enabled | Request nearby geometry when a free slot and the CD controller are available |

Visibility is configured independently under **Engine > Rendering**. See
[Project Settings](settings.md) for saved fields and reset behavior.

Streaming applies to editable-mesh render geometry. Textures, collision, scripts,
audio and skeletal meshes keep their existing storage paths. Geometry can move
with its entity. Retained packets remain available within the configured budget.

Startup loads active geometry before gameplay when its pages fit the pool.
Later required reads can pause rendering. XA music pauses during required reads
and restarts from the beginning of the track; leave geometry resident when
uninterrupted XA playback is required. A read failure prevents affected geometry
from drawing and appears in the runtime error counters.

Use matching scenes and build settings when comparing performance. Monitor
frame times, dropped steps, dropped triangles and CD stalls; reducing executable
size does not imply faster rendering.
