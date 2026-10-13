# Scene View resources and camera

The Windows authoring view is not constrained by the PSX resident texture or
geometry pool. Baked visibility simulation is opt-in and resets on scene switch.

## Textures

All referenced texture identities receive a slot, including temporarily unresolved
ones. GPU arrays contain at most 256 layers (the default WebGPU device limit):
one white layer and up to 255 texture layers. Additional banks are created as
needed; a scene has no artificial 32-texture limit. A bank's allocation matches
its actual layer count and is replaced/released when that count changes.

Draw ranges retain their existing order and split at bank or pipeline boundaries.
In particular, transparent surfaces are not regrouped by bank. Each bank has a
white layer for untextured overlays. Resource identity and palette offsets drive
updates; only changed layers are uploaded when allocation is unchanged.

This is a host renderer change, not an increase in PSX VRAM, palette budgets, or
runtime residency. Large scenes still need an appropriate PSX resource cook.

## Camera

Unvisited maps frame active source geometry. Bounds use source vertices rather
than expanded PSX subdivisions and ignore the baked visibility selection. No
actor, vertex or gameplay camera transform is changed by framing.

Scene cameras are saved separately under `UserSettings/SceneViews/`, keyed by the
project-relative map path. Switching maps and normal GUI layout saves remember
the current view. Invalid/missing preferences fall back to framing. Legacy
`UserSettings/SceneView.epokprefs` remains a startup-only fallback for templates,
not a view applied to every subsequently opened map. Reset View still explicitly
returns to the origin view.

## Verification

- `scene_view_state::tests`: far-origin framing, hidden geometry exclusion,
  independent preferences, corrupt settings, no authored changes.
- `editor::tests::scene_switch_frames_new_map_and_restores_each_authoring_camera`:
  actual map switching, per-map restoration, unchanged map bytes, no automatic
  build, streaming simulation reset.
- `scene_gpu::tests::texture_banks_cover_every_slot_and_preserve_draw_order`:
  511 referenced textures, unresolved identities, bank boundaries and interleaved
  transparent draw ranges.
- Ignored `scene_gpu::tests::gpu_samples_textures_beyond_32_and_across_banks`:
  real GPU pixel assertions for slots 33, 255 and 256, warm reuse, texture
  replacement with unchanged material identity, and bank release after switching
  to a smaller scene.
- Ignored `scene_gpu::tests::capture_authoring_preview` writes an actual GPU
  capture. Set `EPOK_PREVIEW_SCENE` and `EPOK_PREVIEW_OUTPUT`. It frames the map
  unless `EPOK_PREVIEW_VIEW` supplies JSON. `EPOK_PREVIEW_CAMERA_SAMPLE` selects
  a track camera pose without filtering geometry; `EPOK_PREVIEW_BAKED_SAMPLE`
  explicitly enables baked selection. Flat/no-cull diagnostic flags are test-only.
