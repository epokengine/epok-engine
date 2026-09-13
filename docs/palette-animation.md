# Palette animation

Attach **Palette Animator** in the Inspector, select a Texture, and set its
inclusive first/last palette indices, steps per second and direction. The
component changes that texture's colors everywhere it is used: meshes, sprites,
particles and HUD images. It does not modify the source PNG.

Index `0` is permanently reserved for transparency. A cycle must contain at
least two existing opaque palette entries between `1` and `255`; unused padded
entries are rejected. The Inspector reports the number of available colors.
Each scene permits one enabled animator per texture. A project can use a
separate Texture asset when only one object's colors should animate.

Imported PNG colors are quantized to the PSX palette, ranked deterministically
by frequency and color. Changing or reimporting a PNG can change its palette
order, so check the selected range after reimporting. Rotation preserves the
original quantized values and transparency/semitransparency flags.

```cpp
auto& cycle = entity().add<epok::PaletteAnimator>();
// The Inspector normally assigns the Texture asset reference.
cycle.first = 1;
cycle.last = 4;
cycle.speed = 8.0;
cycle.reverse = false;
cycle.reset();
```

Animation uses simulation `dt`, so pause freezes the current palette and fixed
catch-up advances it consistently. Disabling the component or its entity
restores the original palette on the next rendered frame; re-enabling resumes
its retained phase. `reset()` returns phase to zero. Destroyed or unloaded
entities cannot keep controlling a palette, and a new bank starts with its own
component state.

The runtime updates only changed CLUTs. Each update uploads 256 16-bit palette
entries (512 bytes) after waiting for the previous GPU chain to finish, before
submitting the current frame. Uploads block until completion, so the temporary
palette buffer cannot be reused while DMA still reads it. `epok::palette_stats`
reports per-frame uploads, bytes and duplicate runtime-controller conflicts.
If scripts dynamically attach conflicting controllers, the first active entity
controls the texture and the conflict counter increases.

Scene preview reconstructs RGBA pixels from the same packed 8-bit indices and
rotated PSX palette. Its phase uses the same 60 Hz/Q12 time quantization as the
runtime. The demo's Water texture includes an animated palette.

`cargo test palette` covers serialized settings, validation, C++ initialization
and preview color rotation. `python tests/runtime/verify_spatial.py` tests the
actual PsyQo fixed-point animator, including 100 seconds without drift, reverse
rotation, transparent-index preservation, invalid inputs and large steps.
