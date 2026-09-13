# Blockout and EditableMesh

Use **GameObject > Blockout mesh** to create an entity, an `EditableMesh` component and its editable asset under `assets/Meshes/`. The Inspector's **Add Component > EditableMesh** attaches one to an existing entity. **Edit geometry...** opens the dockable Blockout panel; selecting a mesh package in Project opens the same tool. An asset without an instance offers **Add instance to scene**.

## Build and organize a room

1. Add **Box**, **Plane**, **Ramp** or **Stairs** shapes. Origin snaps to the selected grid; size is in local scene units. Keep the initial box or select and delete its faces.
2. Click faces in Scene; Ctrl-click extends or removes selection. Use **Select connected**, **Select coplanar** or **Select all visible** for larger selections. Vertex selection supports shared corners. The numeric Move, Rotate and Scale controls transform the selection about its center and snap the result to the grid.
3. In **Groups**, create `Floor`, `Walls` and `Windows`, optionally nested beneath `Room`. Choose a destination group and press **Move selected faces into group**. Choosing a group preserves the current selection; **Select group faces** explicitly selects its faces and descendants.
4. In **Materials**, create named slots, set a default color/Unlit and assign the slot to selected faces or the current group. A group may use several slots, and different groups may share one slot. Inspector material overrides affect only the selected entity.
5. Save the scene with **Ctrl+S**, then **Build PSX** or **Play**. A window opening requires removing its faces or arranging wall pieces around the opening. A group named Windows does not cut holes automatically.

Faces support individual extrusion, subdivision, flipping and deletion. Subdivision preserves the face's group/material and interpolates its stored UVs. New geometry shares exactly coincident positions while faces retain separate UVs and material assignments. Faces must remain planar and convex; split a face before moving its corners out of its plane. Negative scaling and arbitrary concave polygons are not supported.

### Keyboard editing and ramps

With Scene focused, **1** selects faces, **2** edges and **3** vertices. Click to select and Ctrl-click to extend selection. **E** extrudes selected faces outward; **Q** pushes them inward, using the distance in Blockout (initially 0.25 units). These are individual face extrusions, not CSG subtraction or the inverse of Undo.

In edge mode, **Q** or **Bevel / ramp** cuts off the selected corner and closes it with a sloped face. Existing faces retain their materials and UVs; the new slope inherits a neighboring face's material/group and gets planar UVs. The distance measures perpendicular depth of the cut. A sufficiently deep cut across a cube corner produces a triangular ramp. The operation participates in Undo/Redo and rejects changes to hidden/locked geometry.

The first bevel implementation requires a closed convex solid with outward winding and an edge shared by two quad faces. Triangle edges, open/non-manifold shells, coplanar edges and cuts that remove the solid are rejected without saving partial changes. Multiple edges can be beveled together when earlier cuts leave the later edges valid; incompatible selections fail atomically. Select the new edges again after a topology operation.

Hold **right mouse button** to look around from the camera's position and fly with **WASD**; **Q/E** then mean down/up and do not edit geometry. **Shift** accelerates, and the wheel while holding right mouse changes flight speed. Release the button or press Escape to release capture. **Alt + left drag** orbits; middle drag pans. Wheel alone moves the camera forward/backward without changing its field of view.

Hide, isolate and lock belong to the editing session. They do not remove geometry from builds. A vertex shared with hidden or locked geometry cannot be moved through another face. All faces are available for editing from either side in Scene; the PSX renderer rejects backfaces, so use **Flip faces** when needed.

## Entities, groups and identity

```text
Scene
└── Building (entity: Transform + EditableMesh)
    │   asset UUID → assets/Meshes/Building.epokasset
    │   material overrides: slot UUID → color / Unlit
    └── Door (separate entity when it needs movement or behavior)

Building asset
├── vertices and faces, including disconnected pieces
├── groups: Room → Floor, Walls, Windows
└── material slots: Concrete, Brick, Frame
```

Groups are named face sets, not entities or runtime draw units. Group, face and material-slot UUIDs remain stable through renaming and reordering. Each face belongs to one group and references one material slot. Group parents organize editing; entity parents control transforms and behaviors.

Duplicating an entity shares its asset. Edits to the asset affect its instances, including references in other scenes. The panel counts instances in the current scene. Use **Make independent copy** in the Inspector to create a new asset UUID before changing only one instance, then reopen its geometry.

**Extract to child entity** creates another editable asset and a child entity with an identity local transform, preserving the source's world transform, lighting settings and material overrides. It requires one instance in the current scene and no references from other saved scenes; otherwise make an independent copy first. The original asset loses the extracted faces. Extraction participates in geometry Undo/Redo, with a scene-change guard to avoid replacing later unrelated scene edits. An undone extraction retains its newly created asset for recovery.

## Saving, external moves and recovery

Every successful geometry command saves atomically to the `.epokasset` package. **Ctrl+Z**, **Ctrl+Y** or **Ctrl+Shift+Z**, and the Undo/Redo buttons, restore up to 32 recent geometry edits. This is geometry history for the open asset, not global scene history. It resets when reopening the asset or accepting an external source change. Closing without saving the scene does not discard already saved geometry.

The package contains the asset UUID and editable JSON source together. It has no external import recording or sidecar to keep synchronized. **Asset / Budget > Move / Rename asset**, or an external move while the editor is closed, preserves references. An external copy retaining the same UUID creates a reported conflict; **Make independent copy** deliberately assigns a new one. Missing/conflicting references retain their UUID and prevent a build instead of silently substituting a cube.

The watcher updates paths and reloads external geometry changes. A revision check prevents a stale editor command from overwriting a replacement made on disk. Restore the package to recover missing references. Authored meshes never enter the audio conversion/reimport path.

## PSX compilation and limits

The source remains editable. Builds resolve the package by UUID and generate derived `MeshGeometry`/`MeshQuad` arrays in `.epok/build/scene.hh`; exports include those arrays. Regeneration does not rewrite the source mesh or scene references.

The compiler subdivides surfaces spanning more than four local units, quantizes positions to Q12, partitions by four-unit spatial cells, and shares indexed positions within each chunk. A chunk contains at most 96 quads and 384 positions. Materials and baked color offsets survive partitioning. Authoring groups never force separate rendering batches.

The native renderer checks transformed chunk bounds against the camera frustum before transforming vertices. Visible chunks use triangle backface rejection and clipping against near, far and side planes. Near-plane clipping preserves interpolated vertex colors. The ordering table still cannot correctly resolve every intersecting surface.

| Limit | Current value |
| --- | --- |
| Local authored coordinates | ±128 units |
| Stored vertices per asset | 16,384 |
| Groups / material slots per asset | 256 / 64 |
| Compiled scene geometry | 7,000 base triangles, counting a triangle face conservatively as a quad |
| Extra clipping capacity | 512 triangles, in addition to runtime-object reserve |
| Geometry history | 32 commands for the current asset |

**Asset / Budget** displays full authored/compiled counts even while groups are hidden. These are capacity limits, not a frame-rate promise. `epok::mesh_stats` reports tested/visible chunks, transformed vertices, rejected backfaces and clipped input triangles; `lighting_stats` reports submitted/dropped triangles and frame timing. Reduce visible geometry or split a large level into separately managed sections when needed. Real-console validation remains pending.

Baked lighting uses the actual faces, including shadows between parts of the same asset. Realtime lighting uses arbitrary face normals with inverse-transpose transformation; the existing one-directional/one-point-light selection remains per entity, so baked lighting is preferable for large rooms. Blob shadows still require the legacy horizontal Ground primitive.

Materials support color, Unlit, imported PNG textures, UVs and PSX blend modes. Static OBJ/MTL import produces editable geometry; see [textures](textures.md) and [static mesh import](static-mesh-import.md). CSG, region extrusion, automatic removal of internal solid faces, automatic collider generation, portals/PVS, LOD and streaming remain outside this profile. Box colliders can be authored separately. Stairs are assembled from boxes, so remove unnecessary internal faces explicitly when optimizing.
