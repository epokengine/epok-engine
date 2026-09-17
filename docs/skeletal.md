# FBX skeletal characters

The skeletal pipeline imports a real FBX on the desktop, previews its clips and compiles either rigid GTE skinning or compressed baked vertex animation for PSX. Both target formats use the same host-side skeleton, clips and Animator component; changing the import option does not change scene authoring. The console never loads FBX or JSON. Texture images are not read from the model file; material slots are pointed at imported Texture assets in the editor. Blended weights, animation blending, retargeting, IK and root-motion extraction remain outside this profile. Static meshes and sprites support [textures](textures.md); separate box colliders can use the [collision API](input-collision.md).

## Try it

1. Choose **Assets > Import sample character (FBX)...**, select a **PSX animation** format, then **Import**. This copies the original Epok mannequin into the project without replacing existing files. Alternatively, copy your FBX under `assets/` and open its entry in **Imports**.
2. Select the imported **ModelSource** or **SkeletalMesh** in Project. The model window has clip selection, play/pause, restart, scrubbing, skeleton overlay and an orbitable preview.
3. Choose **Add character to scene**, select the character and choose its startup clip, looping and **Play on start (PSX)**. **Animate scene preview** previews the scene without running the console; its time is not saved.
4. Save and press **Play**. The C++ runtime uses the selected target representation after rejecting off-screen characters.

The sample is original MIT-licensed geometry: 96 vertices, 144 triangles, 10 authored bones (12 nodes including transform helpers), and Idle/Walk clips. Its source is `resources/models/EpokMannequin.fbx`; the reproducible Blender authoring script is `tests/fixtures/create_skeletal_fixture.py`. Blender is not required to use or test the importer.

## Gameplay sampling

The reflected `Mesh3DComponent` API controls clip playback and samples owned
results in C++, Blueprint and Lua Gameplay profile 2. `vertex_count`,
`bone_count` and `clip_count` expose the current model. `sample_vertex` and the
four-index `sample_vertices` batch accept portable imported vertex indices,
`Bind`/`Current` pose and `Model`/`World` space. `sample_bone` returns the parent,
three affine basis vectors, translation and sampled frame. Always inspect the
result's `success`/`error`; a zero position is valid data.

Sampling is synchronous at the current animator state. Ordinary simulation
`tick` runs before the animator advances, so a query there can precede the pose
rendered later in that frame. Sampling never advances playback. Stop resets the
animator ticks and pauses the selected clip; request `Bind` explicitly when bind
pose is required.

Rigid queries preserve imported indices through a cooked remap. Baked compressed
queries use optional 16-vertex seek tables, and baked bone queries retain optional
hierarchy/track sidecars. Those tables are absent unless a reachable Blueprint,
Lua or observed native operation demands them. Explicit queries work off-screen
and their cost is reported separately from renderer pose/decode counters.

Open the model window's **Gameplay query references** section to select and copy
a portable vertex or bone index. It shows the bind Q12 position, strongest bone,
parent and topology identity, so authoring never depends on generated
`skin_vertices_*` table order. A topology-changing reimport requires references
to be reviewed against the displayed identity.

## Portable resources

Each import occupies `assets/<source>.imported/`:

| Asset | Contents |
| --- | --- |
| `Model.epokasset` / ModelSource | One original FBX snapshot, source hash and a manifest mapping semantic keys to subasset UUIDs. |
| Skeleton | Parent indices and quantized reference local poses, including necessary transform helpers. |
| SkeletalMesh | Triangle indices, one bone index per vertex, bone-local bind positions, per-corner texture coordinates, material and clip UUID references. |
| AnimationClip | Skeleton UUID, name, 30 Hz local pose tracks. Constant tracks store a single pose. |
| Material | Separate diffuse color, optional Texture reference, blend mode, depth bias and UV scroll. Edits in the model window survive reimport and preserve the fields the window does not show. |

## PSX animation format

**Rigid bones (smallest)** is the default: it stores the least animation data, and its payload grows slowly with clip count. It is not automatically the faster of the two at runtime — in the workloads measured with `tools/benchmark_skeletal.py` baked vertex frames cost less frame time. See [Native PSX performance](performance.md) for the method and measure your own model. The target compiler groups vertices into contiguous ranges by their strongest bone and remaps triangle indices. At runtime an animation envelope is culled before pose evaluation; a visible character evaluates its bone matrices, loads one model-view matrix per non-empty bone and sends each range directly through the GTE. There is no model-space skinned-vertex pass. A model with dynamic lit materials uses the compatible CPU rigid path because its face normals must currently be rebuilt from posed vertices; imported skeletal materials are unlit by default.

**Baked vertex frames** evaluates the same 30 Hz clips during compilation. Every frame stores model-space coordinates as signed one-byte Q8 deltas from the bind pose; coordinates outside that range use an absolute Q12 `int16`, and a frame switches to raw Q12 if that is smaller. Frames are independent for random access and identical payloads are shared. A visible character decodes one frame into the shared 512-vertex scratch buffer, then projects it with one object GTE matrix. This spends more executable/archive space to avoid runtime bone evaluation.

Both formats store conservative bounds covering the bind pose and all imported animation samples. Consequently a fully off-screen character costs neither bone evaluation nor baked-frame decoding. Reimport the ModelSource to switch format; placed characters and selected clip UUIDs remain valid.

Package version 2 uses typed importer settings. The package container still has the `EPOKAS01` signature; version 1 audio/editable mesh packages remain readable. The importer is pinned to ufbx 0.11.3 and does not load external FBX files or texture paths.

Reimport through **ModelSource > Reimport FBX** or **Rebuild from stored FBX**. Mesh and skeleton identity is stable; clip/material identity follows unique FBX names. Renaming a clip/material creates a new asset; retired files are retained so data is recoverable. Because retired entries are kept, the manifest holds up to 512 subasset identities: a model at the payload limits occupies 99 of them, leaving room for several later generations of renamed clips and materials. A removed/renamed selected clip requires an explicit replacement in the character's Inspector.

The worker converts and validates all outputs before publication. Publication checks the original source and every existing output revision, stages a new directory, backs up the old one under `.epok/model-backups/`, then replaces the model directory with rollback on rename failure. A process/OS crash between the two directory renames may require restoring that backup manually. This is not a database transaction across arbitrary external processes.

Keep a model's generated files together for reimport. Individual UUID-linked assets can resolve after moves, but reimport requires restoring their original relative filenames; moving the entire `.imported` folder works. Duplicate a model by importing the FBX into a fresh `.imported` destination. Copying generated packages duplicates their UUIDs and is diagnosed as a conflict. Referenced skeletons, materials and clips cannot be deleted through the asset manager.

## Textures and coordinates

Each triangle stores one normalized coordinate pair per corner, in the same order as its indices, read from the model file's first UV set. Values outside 0–1 fail the import and ask for an unwrap into atlas space; a model without a UV set imports with unmapped triangles and a warning. The vertical axis is inverted once at import: v = 0 is the top row of the source image, matching the PNG row order the [texture importer](textures.md) keeps and the page mapping the target uses.

Assign a Texture asset to each material slot under **Materials** in the model window. The window previews the assigned image on the posed triangles using the stored corner coordinates, alongside the skeleton overlay and clip scrubbing. A material slot with a Texture but no imported coordinates is rejected when the model resolves.

Corner coordinates belong to triangles, not vertices, so a texture seam does not duplicate positions and does not split the contiguous bone ranges the rigid path depends on. Compilation emits the same coordinates in both animation formats and additionally bakes 8-bit page coordinates when the texture is resident in the scene bank, so the renderer copies them instead of scaling four Q12 pairs per drawn quad. Assigning a texture alone does not move an all-unlit model off the rigid GTE path.

## Size accounting

Compilation measures the generated tables in the target's own layout rather than
in host units: 20 bytes per stored pose, 22 per bone, 8 per bone track, 8 per
baked frame descriptor, 20 per clip descriptor, 72 per face (texture coordinates
and packed page coordinates included), 6 per stored position, plus the two table
headers. The runtime header asserts each of those sizes at compile time, so a
change to a runtime structure fails the target build instead of quietly
invalidating the numbers.

Every generated model header opens with a `// skeletal budget:` line listing the
host pose tracks, the target pose payload, the bone/track/clip descriptors, the
baked frame payload and descriptors, the shared geometry, the per-character
Animator bytes and the once-per-executable pose/decode scratch, followed by the
animation total and its limit. When the animation total or the host tracks pass
the 512 KiB limit the build fails with that same breakdown and the ways to get
back under it: remove unused clips, shorten clips, switch the animation storage
mode, or reduce the model's vertex and triangle counts. The limit itself does not
move with the clip count.

## First PSX profile

| Limit | Value |
| --- | --- |
| Source file | 32 MiB |
| Combined skinned model | 512 vertices, 1,024 triangles |
| Skeleton | 64 nodes including ancestors/helpers |
| Materials / clips | 64 / 32 |
| Clip duration | Up to 60 seconds, sampled at 30 Hz |
| Combined target animation data | 512 KiB maximum per model |
| Animated model coordinates | Within ±7.5 meters of the model origin |
| Bone-local vertex | Signed Q12 `int16`, ±8 meters |
| Pose | Translation Q8, quaternion Q12, scale Q12; 20 bytes per sample |

These are validation limits, not guarantees of frame rate or whole-game memory fit. Unsupported shear, singular transforms, unweighted vertices, dual-quaternion skinning, blend shapes and multiple skin deformers are rejected. Multiple vertex influences are explicitly reduced to the strongest weight and reported in the model window. Export meshes with their armature and baked animation clips. The importer converts axes/units, absorbs reference scale into rigid bind positions, and preserves animated scale ratios.

Animation currently selects a sample every two 60 Hz runtime ticks; it does not interpolate between samples. Looping wraps before the terminal endpoint sample. Non-looping playback holds the final pose. Imported materials are unlit.

The generated executable stores immutable geometry and animation tables once per referenced skeletal mesh, shared by all its instances. Each entity owns only its Animator state. Rigid GTE instances share a 64-matrix pose scratch and baked instances share a 512-position decode scratch; the renderer reuses both serially between visible characters. The first compiler embeds tables into the executable; a future archive/streaming backend can package the same logical assets separately from code.

```cpp
if (auto* animator = entity().get<epok::Animator>()) {
    animator->play(0, true); // clip index in this imported model
    animator->pause();
    animator->resume();
    animator->stop();       // first pose of selected clip, paused
}
```

## Command line and verification

```powershell
epok-editor --project MyGame --import-fbx assets/Hero.fbx --animation-storage rigid-gte
epok-editor --project MyGame --reimport-asset assets/Hero.imported/Model.epokasset --animation-storage baked-vertices
epok-editor --project MyGame --reimport-asset assets/Hero.imported/Model.epokasset --snapshot
epok-editor --project MyGame --preview-model assets/Hero.imported/Model.epokasset
```

`cargo test --locked` checks imported poses against independent ufbx skin evaluation, identity/material preservation, missing source recovery, stale-write rejection, invalid payloads, baked-frame compression and shared target tables. `python tests/integration/verify_skeletal.py` builds both target formats, runs them in PCSX-Redux, checks animated GPU frames and compares decoded baked positions with the editor pose; it also captures the model preview and native Game view under `artifacts/`.

Importer reference: [ufbx Rust bindings](https://github.com/ufbx/ufbx-rust), [ufbx animation evaluation](https://ufbx.github.io/elements/animation/).
