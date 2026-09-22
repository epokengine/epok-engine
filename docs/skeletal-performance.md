# Skeletal cooking and packet reuse

Rigid cooking merges identical `(bone, local position)` pairs before grouping
vertices by bone. Material and UV data remain on polygon corners. Authored
vertex indices are stable: optional gameplay-query tables map every portable
index to its cooked vertex, and the runtime validates against the separate
portable count rather than the smaller render count. Different bones never
merge, even if their bind positions coincide. Baked assets retain their existing
portable-order contract.

Interior skeletal geometry can use GTE RTPT batches of three vertices. Rigid
batches never cross bone ranges. Boundary geometry, projection overflow, UV
scrolling and the final one/two vertices keep the full-coordinate fallback.
`EPOK_VALIDATE_GTE=1` checks batch screen positions, depth, visibility and fog
against individual projection; validation timings are not performance results.

With Retained Packets enabled, unlit animated objects (including imported
meshes whose per-face materials are all unlit) now reuse immutable GPU
packet material/UV words just like static objects. Pose changes still project
and write every visible vertex. Model rotation does not invalidate an unlit
packet's colours. Generation, geometry, tint, textures and material changes
remain part of invalidation; fog updates each parity and scrolling faces use
the dynamic path. Lit or baked-colour animated objects are excluded. Allocation
uses the existing bounded pool and falls back to dynamic submission if full;
no additional packet RAM is reserved for this optimization.

For small NPCs that do not need runtime bone aiming, Baked Vertices can remove
hierarchy evaluation entirely. Compare the cook's actual RAM report: it is not
universally smaller than rigid animation. Keep a rigid model when scripts need
procedural bone edits. This choice does not change the source FBX or animations.
