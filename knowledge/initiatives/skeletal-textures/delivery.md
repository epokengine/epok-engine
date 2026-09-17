# Skeletal textures and expanded clip sets — delivery record (2026-09-15)

What the textured-skeletal initiative delivered, how each claim was verified, and what
was deliberately left undone. Written from the code and the measurement artifacts at
delivery time. Where a claim is not backed by an executed check it is marked as such
rather than asserted.

This file is self-contained: continuing the work needs nothing but the repository.

## 1. Scope delivered

| Milestone | State |
| --- | --- |
| Per-corner texture coordinates on skeletal triangles | Complete |
| Textured skeletal cooking through the shared page layout | Complete |
| Clip capacity raised to 32 with explicit target-layout budgets | Complete |
| Independent numerical, visual and regression validation | Complete |
| Reproducible runtime and memory measurements | Complete |
| Optional pose compression | Deferred, see section 6 |
| Optional mesh detail levels | Deferred, see section 6 |

Not in scope and not implemented: multi-weight skinning, blending between clips,
retargeting, inverse kinematics, root-motion extraction, ragdolls, facial rigs, and
skeletal streaming from disc.

## 2. Format and code changes

**Portable schema.** `skeletal::Triangle` gained
`uv: Option<[[f32; 2]; 3]>`, serialized with `#[serde(default,
skip_serializing_if = "Option::is_none")]`. One normalized pair per corner, in the
same order as `indices`. `None` marks a legacy or unmapped triangle, which keeps every
previously imported asset readable and rendering exactly as before. Absence is
therefore distinguishable from a valid coordinate pair that happens to be `[0, 0]`.

Validation runs at two levels. The payload check in `Data::parse` requires finite
values inside `0..1`; out-of-range coordinates are rejected rather than clamped, so a
model authored with repeat or wrap addressing fails with a message instead of silently
receiving a wrong mapping. The resolved check in `Model::load` fails when a triangle
uses a material slot that has a texture but carries no coordinates, naming the
triangle index and the slot. An untextured slot may legitimately have no coordinates.

**Vertical convention.** `v = 0` is the top row of the source image, so import applies
`v_epok = 1 - v_fbx` exactly once. The rule follows from two existing behaviours:
`texture::decode` walks PNG rows top-down so row 0 becomes the first row of the VRAM
page, and the runtime's page mapping scales `v` downward from the page's top row.
`mesh_compile::packed_uv` already replicates that mapping for static meshes. The flip
is asserted against the fixture rather than assumed.

**Import.** Coordinates are read by the same polygon-corner indices the triangulator
returns, with the same `[0, 2, 1]` front-face permutation applied before corners are
resolved to control points. A mesh with no coordinate set imports with `uv: None` and
a warning. Texture images are still not read from the model file; the importer stays
on bounded bytes and points the user at the model window instead.

**Cooking.** `skeletal_compile::header_with_pages` takes the same page lookup the
static editable-mesh path uses, so skeletal and static geometry pack identical page
coordinates through one helper. `rigid_order()` copies coordinates and material
assignment unchanged while remapping only position indices. Each face emits position
corners `[a, b, c, c]` and coordinate corners `[uv0, uv1, uv2, uv2]`, keeping the
degenerate second triangle suppressed. Both storage modes emit equivalent material and
coordinate data; only the position ordering differs.

**Clip capacity.** `skeletal::MAX_CLIPS = 32` is the single source of the limit, used
by payload validation, the import check and the documentation. The importer manifest
bound moved from 128 to 512 outputs: manifest entries are permanent identities, a
rename retires the old entry rather than removing it, and one import at the payload
limits already consumes 99 entries, so 128 would have failed on the first reimport
that renamed roughly a third of the subassets.

**Memory accounting.** `skeletal_compile::Budget` reports host track bytes, rigid pose
and descriptor bytes, baked frame and descriptor bytes, geometry bytes, per-instance
animator bytes and shared scratch bytes, all with checked arithmetic. Sizes come from
the target ABI, not from host `size_of` on an unrelated struct, and each one is
asserted in `runtime/skeletal.hpp` so a layout change fails the target build instead
of silently invalidating the report. Every cooked model emits its breakdown as the
header's first line. Neither the host nor the target 512 KiB limit was relaxed; an
overflow fails with the full breakdown and the available options.

**Material editing.** The model window loads the stored material, changes only the
edited field and republishes under the existing stale-revision protection. Texture,
blend mode, depth bias, scroll settings and the unlit choice survive a color edit, an
edit to another slot, a save and reload, and a storage-mode switch.

## 3. Struct sizes asserted on the target

| Type | Bytes |
| --- | --- |
| `BonePose` | 20 |
| `Bone` | 22 |
| `BoneTrack` | 8 |
| `VertexFrame` | 8 |
| `AnimationClip` | 20 |
| `Material` | 24 |
| `MeshQuad` | 72 |
| `MeshGeometry` | 72 |
| `SkeletalMesh` | 32 |
| `Animator` | 20 |
| `Affine<Fixed>` | 48 |
| Shared scratch | 6216 |

These were confirmed by a real target build, including a deliberate negative control
that made the build fail on a wrong value. Note that the runtime headers are embedded
into the editor binary at build time, so the editor must be rebuilt after editing them
or a target build will silently use a stale copy.

## 4. Verification actually performed

Host suite at delivery: 648 passed, 2 failed, 51 ignored. The two failures are
pre-existing and unrelated, caused by a missing audio encoder path in the audio
contract tests; the baseline before this work was 638 passed with the same two
failures. Ten tests were added across the two areas.

Native validation ran on the local target toolchain and emulator through
`tests/integration/verify_skeletal.py`, which was also made portable across operating
systems, and the new `tests/integration/verify_skeletal_textured.py`. Scenarios that
passed:

- Textured rigid playback keeps direct GTE submission. Assigning a texture to an
  unlit model does not fall back to the CPU path, and the seam does not duplicate a
  cooked position.
- Texture orientation is correct on the target, checked by the screen-space positions
  of identifiable atlas regions rather than by counting pixels of one color.
- A fully off-screen character records exactly zero in all four skeletal counters
  across every sample while the frame counter advances.
- Baked playback decodes the specific frame implied by the recorded animator ticks,
  not merely some frame of the clip, within the expected quantization error.
- Two instances of one model play different clips from a single shared geometry table.
- The 30-sequence model plays clips selected by identity at indices 0, 15, 16, 28
  and 29.

One counter was corrected: baked decoded-vertex counting previously incremented for a
visible baked model even when no clip was selected and the decoder returned bind
positions without copying. It now counts only genuinely decoded coordinates.

Fixtures are original and generically named. `tests/fixtures/create_textured_character_fixture.py`
regenerates them deterministically using only the Python standard library, so the
fixture geometry, the seam and the sequence count can be changed without external
authoring tools.

## 5. Measurements

Captured with `tools/benchmark_skeletal.py` at 320x240, three runs per workload with
warm-up discarded, roughly 256 to 292 warm samples per workload. Emulator timing is
evidence for that setup; it is not physical hardware timing, and host viewport frame
rate is not target performance.

The findings that change a decision:

- Baked vertex storage was faster than rigid bone ranges in every controlled pair, by
  roughly 10 percent at one instance, 16 percent at four and 18 percent at eight. At
  eight visible instances the rigid workload was the only one to exceed the frame
  budget while its baked counterpart stayed inside it.
- Lit materials, which require the compatible CPU path for posed normals, cost about
  47 percent more frame time than unlit materials. That is a larger effect than the
  storage choice, so keeping character materials unlit matters more than the storage
  mode.
- The limiting resource is frame CPU time. Animation payloads reached at most about
  9 percent of their budget, static memory stayed under half, and no primitives were
  clipped or dropped in any workload.
- Within a frame, skeletal pose and decode work is the minority of the cost. The bulk
  is per-vertex projection and per-polygon work.

Consequently the storage label was corrected from "fastest" to "smallest": rigid
storage is the smaller representation and scales gently with clip count, but no
measured workload supported calling it the faster one.

Comparisons are controlled only within an instance-count group, because the groups use
different camera distances. Timing scopes overlap and must not be summed blindly; the
counter semantics are documented in `docs/performance.md`.

## 6. Optional work, deferred with evidence

**Further pose compression is not worthwhile now.** It buys bytes, and bytes are not
the constraint. The worst measured animation payload used a small fraction of its
limit and substantial memory remained unassigned. A smaller representation would also
risk adding decode work to the path that is actually limiting. Revisit only if a real
project's clip set approaches the payload limit, and measure decoder cost on cold and
random clip switches before adopting anything.

**Mesh detail levels are plausible but not yet justified.** Detail selection is the
only lever that targets frame CPU time, which is the real limit, and the counters do
point at vertex and polygon work. But two cheaper levers were measured first and
should be applied before adding complexity: baked storage for models with several
visible instances, and unlit materials. The fixtures used here are small, at 32 and 96
positions against a 512 limit, so any decision should be re-measured with a realistic
character with those two levers already applied. If frame time still exceeds the
budget then, detail selection is the justified next step.

If detail levels are implemented, note that baked animation needs particular care:
topology variants with different position counts cannot share a baked vertex payload,
so the per-level cost must be measured before claiming a memory saving. Detail
selection also does not by itself reduce bone evaluation.

## 7. Remaining limitations

1. Texture images referenced by a model file are not read from disk. Assigning a
   Texture asset to a material slot in the model window is the complete supported
   workflow. Automatic association would need the same bounded import and dependency
   rules and is not implemented.
2. No command-line or tool-call surface assigns a texture to a material slot. Headless
   workflows currently rewrite the material package directly, which the textured
   validation harness does and documents.
3. Coordinates slightly outside `0..1` from floating-point noise are rejected rather
   than snapped. This follows the no-silent-clamping rule, but a real asset may need
   an explicit, documented tolerance.
4. The model window preview draws the shared project-browser thumbnail, so a texture
   appears flat-colored for a frame or two before it decodes, and the preview is the
   quantized thumbnail rather than the full-resolution image.
5. Lit skeletal materials remain on the compatible CPU path until a separately
   implemented and tested lighting optimization can replace it. The measured cost of
   that fallback is recorded above.
6. The instance-count benchmark groups use different cameras, so cross-group timing
   comparisons are not controlled.
7. Validation is emulator-level. Physical hardware timing and rasterization cost
   remain separate open questions.
8. The released v0.2.0 notes still describe the default storage mode with the earlier
   "fastest" wording, which the measurements do not support. The changelog was left
   unchanged because it records a shipped release.

## 8. Where things live

| Area | Path |
| --- | --- |
| Portable schema and validation | `src/skeletal.rs` |
| Import and coordinate extraction | `src/model_import.rs` |
| Cooking, budgets and target sizes | `src/skeletal_compile.rs` |
| Shared page packing reused here | `src/mesh_compile.rs` |
| Model window preview and material editing | `src/skeletal_ui.rs` |
| Target declarations and layout asserts | `runtime/epok.hpp`, `runtime/skeletal.hpp` |
| Host tests | `src/skeletal_tests.rs` |
| Native validation | `tests/integration/verify_skeletal.py`, `tests/integration/verify_skeletal_textured.py` |
| Benchmarks | `tools/benchmark_skeletal.py` |
| Fixture generator | `tests/fixtures/create_textured_character_fixture.py` |
| User documentation | `docs/skeletal.md`, `docs/textures.md`, `docs/performance.md` |

Measurement outputs and captures are written under `artifacts/`, which is ignored.
