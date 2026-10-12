# World optimization techniques

Large PSX worlds run fast when the expensive decisions are made at build time,
not on the console. Epok builds the world you author in the editor, then cooks
it with a set of techniques taken from a commercial reference renderer of the
era. Each technique is a separate option, because not every game needs every
one: a 2D game needs none of them, a guided-camera platformer benefits from
all of them, and a free-camera 3D game sits in between.

This page lists each technique, what it does, how to enable it, its cost, and
what it measured on a study scene. All figures come from the pinned PCSX-Redux
build used by the research tools, so they compare builds with each other; they
are not frame-rate claims for a particular game.

## At a glance

| Technique | What it saves | How to enable | Default |
| --- | --- | --- | --- |
| [Camera visibility tracks](#camera-visibility-tracks) | Drawing and loading what the camera cannot see | Build > Camera Path, then Bake Visibility | Off |
| [Geometry streaming](#geometry-streaming) | RAM: worlds larger than memory | Project Settings, geometry streaming | Off |
| [Static world](#static-world) | Seams between meshes; per-actor matrices | `visibility_track.static_world` | Off |
| [Prepared packet order](#prepared-packet-order) | Visibility, sorting and setup per polygon at run time | Scene `ordered_draw` program | Off |
| [Cooked GPU words](#cooked-gpu-words) | Texture lookup and word assembly per polygon | Always on for baked scenery | On |
| [Near-plane handling](#near-plane-handling) | CPU clipping at the camera | Project Settings, Exact Near Clipping | On (exact) |
| [GTE depth sorting](#gte-depth-sorting) | A CPU division per polygon | Project Settings, GTE Depth Sorting | On |
| [Retained geometry](#retained-geometry) | Rebuilding packets of static meshes | Project Settings, retained geometry | On |

The techniques in [Inside the prepared renderer](#inside-the-prepared-renderer)
are always on when that renderer is used.

### Which ones a game needs

| Kind of game | Recommended |
| --- | --- |
| 2D, or small 3D rooms | Defaults only |
| Free-camera 3D with large levels | Geometry streaming, static world for scenery that never moves |
| Guided camera (rails, corridors, chase view) | Everything: visibility tracks, streaming, static world, prepared packet order |

## Camera visibility tracks

The camera's path is sampled at build time. For each sample the build stores
which faces are visible, as changes from the previous sample, and which
streaming pages those faces need now and next. At run time the renderer only
considers the faces of the current sample, and the streamer loads exactly the
pages of the current and next sample.

- **Enable:** draw the camera's route in the editor with **Build > Camera
  Path...** (keys taken from the Scene view; see
  [Authoring a camera path](streaming.md#authoring-a-camera-path)), then bake
  the face lists with **Build > Bake Visibility...**, ideally adding cameras
  recorded while playing (Play options, *Record camera for visibility bake*);
  see [Automatic visibility bake](streaming.md#automatic-visibility-bake-experimental).
- **Cost:** the camera must stay near the recorded path. A route graph
  (`navigation`) lets a player-driven target choose among recorded routes.
- **Use when:** the camera is guided. A free camera has no path to bake.

## Geometry streaming

Mesh chunks live in 64 KiB pages on the disc and are loaded into a small pool
as they are needed. With a visibility track, the build measures the worst
current/next page union and rejects a pool that is too small, so a selected
face is never dropped to fit.

- **Enable:** Project Settings: geometry streaming, pool pages (2 to 8),
  triangle budget, prefetch. See [Geometry streaming](streaming.md).
- **Cost:** disc reads; cuts wait until the new view is resident.
- **Use when:** the level does not fit in RAM.

## Static world

Scenery that never moves is cooked in world space. Every vertex is the authored
point transformed once to world Q12 by the same routine for every actor, and
all of it is drawn with the camera's matrix on a camera-relative lattice. A
corner shared by two meshes (a wall standing on the ground) therefore lands on
exactly the same pixel from both.

Without it, each actor is drawn with its own view x model matrix rounded to
Q12, so shared corners can disagree by a pixel and the background shows
through thin seams.

- **Enable:** `visibility_track.static_world: true` for tracked baked scenery;
  the prepared packet order always cooks in world space (its own
  `static_world` flag is required).
- **Cost:** covered actors must not move at run time; their runtime transforms
  are no longer applied, and mesh vertex queries answer in world space only.
  World-space vertices may need 32-bit storage, so geometry grows somewhat.
- **Measured:** on a study scene drawn by the baked kernels, background pixels
  leaking through the ground at the start view fell from about 250 per frame
  (seam lines along walls and water edges) to about 80 (a small edge artifact
  around a face touching the camera). Geometry grew from 1.31 to 1.57 MB;
  resident RAM by 2 KB; render time was unchanged.

## Prepared packet order

The strongest technique, and the one the reference renderer is built around.
For every camera sample the build stores the exact list of polygons to draw,
already in draw order, as compact delta-coded lists. At run time the renderer
walks the list, projects each polygon with the GTE and emits its packet. There
is no visibility test, no sort and no per-polygon material setup.

- **Enable:** a scene `ordered_draw` program with `static_world`, on top of a
  visibility track. Requires unlit, untinted, non-scrolling materials, no fog
  and no backface culling. See
  [Experimental prepared packet order](streaming.md#experimental-prepared-packet-order).
- **Cost:** only for static, unlit scenery; covered actors must not move.
- **Measured:** over the 401 camera views of a recorded demo, the prepared pass
  now takes a median of **0.97x** the reference renderer's time on the same
  polygon lists (p10 0.92x, p90 1.11x); Epok is faster on 241 of the 401 views.
  At the start of this work it took 1.24x.

## Cooked GPU words

Baked faces store the three texture words a textured GPU packet takes,
resolved to VRAM at build time: UV0 with the CLUT, UV1 with the texture page
(including the blend mode), and UV2 with UV3. The renderer copies them; it no
longer looks up the texture or assembles the words per polygon. Face flags sit
in texture-page bits the GPU ignores and never reach a packet. Faces stay 32
bytes.

- **Enable:** always on for baked scenery (baked face format version 3).
- **Cost:** none at run time; geometry cooked by an older version must be
  rebuilt.
- **Measured:** 4.4% of the prepared pass (3.5% with exact near clipping).

## Near-plane handling

Faces crossing the camera's near plane can either be clipped exactly on the
CPU, or drawn as the GTE produces them, as the reference renderer does.

- **Enable:** Project Settings: Exact Near Clipping (on by default).
- **Cost:** exact clipping is CPU work; off, faces right at the camera can
  stretch or vanish.
- **Measured:** at a static study view the prepared pass took 715k cycles with
  exact clipping and 584k with it off.
- **Seams:** corners in front of the near plane keep the GTE's screen position
  in either mode, so clipped faces meet their neighbours exactly. Corners the
  CPU must project use an exact model of the GTE's own projection.

## GTE depth sorting

The ordering-table depth of prepared geometry comes from the GTE's AVSZ3/AVSZ4
instructions instead of a CPU division.

- **Enable:** Project Settings: GTE Depth Sorting (on by default).
- **Cost:** triangles may sort up to 1.6% nearer; this only changes how they
  interleave with moving objects. Quads are identical.

## Retained geometry

Static meshes keep their GPU packets across frames instead of rebuilding them.
See [Native PSX performance](performance.md).

- **Enable:** Project Settings: retained geometry (on by default).

## Inside the prepared renderer

These need no setting. They are listed so a regression can be traced to them.

| Technique | Effect on the prepared pass |
| --- | --- |
| Compact vertex blocks: 16-bit offsets from a block centre, range-checked once per block and camera | Fewer loads and checks per vertex |
| Camera-relative lattice with exact re-anchoring far from the camera | Full precision in 16-bit GTE inputs; no seams |
| Early rejection of faces entirely beside the screen, tested only when a face is not fully inside | 0.8% |
| Few values live across the polygon loop (counters derived after it, computed corner order, packed counters) | 13% combined |
| Projection blocks padded to 64 bytes | 1.2% |
| GTE latency overlap: viewport test during RTPS, fourth-corner fetch during RTPT, depth average read late | 3.6% combined |
| Free packet-slot run held in registers | 0.7% |

## Verifying a change

Every renderer change above was checked the same way; keep doing so:

- **Same output:** a diagnostic sweep build (`EPOK_VISIBILITY_SWEEP`, optionally
  with fixed cameras via `EPOK_SWEEP_CAMERAS`) visits every camera sample. Its
  packet count and every published counter must match per sample between the
  old and new build, in both near-clipping modes.
- **Same pixels:** the full VRAM at the deterministic start view must be
  byte-identical. Scenes whose camera moves from the first frame are not
  deterministic; compare cooked data instead.
- **Same projection:** a GTE validation build (`--validate-gte`, `Os`) checks
  every projected vertex against an independent model.
- **Speed:** compare per-sample cycles of the prepared pass between builds
  with the emulator's cycle counter, never across emulator builds.

## Open items

- The whole guided-camera workflow now runs in the editor: camera path,
  visibility bake and packet order. Baked from a recorded demo's cameras, the
  study scene's lists cost 1.01x the imported ones in the prepared pass (p10
  0.96x, p90 1.12x). Player-driven route graphs (`navigation`) are still
  imported.
- The views where Epok remains slowest (1.1 to 1.15x the reference) spend
  more time per polygon walking the list and emitting packets.
- A face touching the camera can leave a one-pixel edge artifact at the screen
  border in the baked kernels.
