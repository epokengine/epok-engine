# FBX skeletal characters

The skeletal pipeline imports a real FBX on the desktop, previews its clips and compiles rigid skinning for PSX. The console never loads FBX or JSON. Skeletal texture/UV import, blended weights, animation blending, retargeting, IK and root-motion extraction remain outside this profile. Static meshes and sprites support [textures](textures.md); separate box colliders can use the [collision API](input-collision.md).

## Try it

1. Choose **Assets > Import sample character (FBX)...**, then **Import**. This copies the original Epok mannequin into the project without replacing existing files. Alternatively, copy your FBX under `assets/` and open its entry in **Imports**.
2. Select the imported **ModelSource** or **SkeletalMesh** in Project. The model window has clip selection, play/pause, restart, scrubbing, skeleton overlay and an orbitable preview.
3. Choose **Add character to scene**, select the character and choose its startup clip, looping and **Play on start (PSX)**. **Animate scene preview** previews the scene without running the console; its time is not saved.
4. Save and press **Play**. The C++ runtime evaluates the quantized skeleton and transforms each vertex with its owning bone.

The sample is original MIT-licensed geometry: 96 vertices, 144 triangles, 10 authored bones (12 nodes including transform helpers), and Idle/Walk clips. Its source is `resources/models/EpokMannequin.fbx`; the reproducible Blender authoring script is `tests/fixtures/create_skeletal_fixture.py`. Blender is not required to use or test the importer.

## Portable resources

Each import occupies `assets/<source>.imported/`:

| Asset | Contents |
| --- | --- |
| `Model.epokasset` / ModelSource | One original FBX snapshot, source hash and a manifest mapping semantic keys to subasset UUIDs. |
| Skeleton | Parent indices and quantized reference local poses, including necessary transform helpers. |
| SkeletalMesh | Triangle indices, one bone index per vertex, bone-local bind positions, material and clip UUID references. |
| AnimationClip | Skeleton UUID, name, 30 Hz local pose tracks. Constant tracks store a single pose. |
| Material | Separate flat diffuse color. Color edits in the model window survive reimport. |

Package version 2 uses typed importer settings. The package container still has the `EPOKAS01` signature; version 1 audio/editable mesh packages remain readable. The importer is pinned to ufbx 0.11.3 and does not load external FBX files or texture paths.

Reimport through **ModelSource > Reimport FBX** or **Rebuild from stored FBX**. Mesh and skeleton identity is stable; clip/material identity follows unique FBX names. Renaming a clip/material creates a new asset; retired files are retained so data is recoverable. A removed/renamed selected clip requires an explicit replacement in the character's Inspector.

The worker converts and validates all outputs before publication. Publication checks the original source and every existing output revision, stages a new directory, backs up the old one under `.epok/model-backups/`, then replaces the model directory with rollback on rename failure. A process/OS crash between the two directory renames may require restoring that backup manually. This is not a database transaction across arbitrary external processes.

Keep a model's generated files together for reimport. Individual UUID-linked assets can resolve after moves, but reimport requires restoring their original relative filenames; moving the entire `.imported` folder works. Duplicate a model by importing the FBX into a fresh `.imported` destination. Copying generated packages duplicates their UUIDs and is diagnosed as a conflict. Referenced skeletons, materials and clips cannot be deleted through the asset manager.

## First PSX profile

| Limit | Value |
| --- | --- |
| Source file | 32 MiB |
| Combined skinned model | 512 vertices, 1,024 triangles |
| Skeleton | 64 nodes including ancestors/helpers |
| Materials / clips | 64 / 16 |
| Clip duration | Up to 60 seconds, sampled at 30 Hz |
| Combined pose data | 512 KiB maximum per model |
| Animated model coordinates | Within ±7.5 meters of the model origin |
| Bone-local vertex | Signed Q12 `int16`, ±8 meters |
| Pose | Translation Q8, quaternion Q12, scale Q12; 20 bytes per sample |

These are validation limits, not guarantees of frame rate or whole-game memory fit. Unsupported shear, singular transforms, unweighted vertices, dual-quaternion skinning, blend shapes and multiple skin deformers are rejected. Multiple vertex influences are explicitly reduced to the strongest weight and reported in the model window. Export meshes with their armature and baked animation clips. The importer converts axes/units, absorbs reference scale into rigid bind positions, and preserves animated scale ratios.

Animation currently selects a sample every two 60 Hz runtime ticks; it does not interpolate between samples. Looping wraps before the terminal endpoint sample. Non-looping playback holds the final pose. Material colors initially use unlit rendering; textures and UV compilation will extend the material pipeline later.

The generated executable stores immutable geometry, skeleton and clip tables once per referenced skeletal mesh, shared by all its instances. Each entity owns only its Animator state. Skinning uses a shared scratch buffer (64 matrices and 512 positions), reused between characters, with bounds recomputed from posed vertices before frustum culling. The first compiler embeds tables into the executable; a future archive/streaming backend can package the same logical assets separately from code.

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
epok-editor --project MyGame --import-fbx assets/Hero.fbx
epok-editor --project MyGame --reimport-asset assets/Hero.imported/Model.epokasset
epok-editor --project MyGame --reimport-asset assets/Hero.imported/Model.epokasset --snapshot
epok-editor --project MyGame --preview-model assets/Hero.imported/Model.epokasset
```

`cargo test --locked` checks imported poses against independent ufbx skin evaluation, identity/material preservation, missing source recovery, stale-write rejection, invalid payloads and shared target tables. `python tests/integration/verify_skeletal.py` builds a fresh project and checks changing skeletal vertex data, position agreement and GPU frames in PCSX-Redux; it also captures the model preview and native Game view under `artifacts/`.

Importer reference: [ufbx Rust bindings](https://github.com/ufbx/ufbx-rust), [ufbx animation evaluation](https://ufbx.github.io/elements/animation/).
