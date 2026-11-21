# Blockout and EditableMesh

Use **GameObject > 3D Object > Blockout Mesh** (or the same entry in the Hierarchy's creation menu) to create an entity, a mesh component and its editable asset under `assets/Meshes/`. The Inspector's **Mesh Renderer > Mesh** selects a project mesh or the read-only **Engine / Cube**. Add a `Mesh3DComponent` through **Add Component** when an actor has no mesh. **Edit geometry...** opens the **Geometry** panel for a project mesh; selecting a mesh package in Project opens the same tool. An asset without an instance offers **Add instance to scene**.

Import OBJ models through **Project > Import...** or drop an OBJ onto the Project browser. The importer creates a mesh package in the current project folder without changing the selected actor. Material libraries are read beside the source OBJ; referenced PNG textures must already be imported into the corresponding project folder. Assign the resulting package through **Mesh Renderer > Mesh**. Engine geometry cannot be edited.

## Build and organize a room

1. Add **Box**, **Plane**, **Ramp** or **Stairs** shapes. Origin snaps to the selected grid; size is in local scene units. Keep the initial box or select and delete its faces.
2. Click faces in Scene; Shift-click adds a face and Ctrl-click adds or removes one. Drag across the Scene to select many at once: a box takes the faces whose centre it encloses, or choose **Paint** under **Select > Drag** and sweep over the faces you want (see [Drag selection](#drag-selection)). Use **Select connected**, **Select coplanar** or **Select all visible** for larger selections. Vertex selection supports shared corners. The numeric Move, Rotate and Scale controls transform the selection about its center and snap the result to the grid.
3. In **Groups**, create `Floor`, `Walls` and `Windows`, optionally nested beneath `Room`. Choose a destination group and press **Move selected faces into group**. Choosing a group preserves the current selection; **Select group faces** explicitly selects its faces and descendants.
4. In **Materials**, create named slots, set a default color/Unlit and assign the slot to selected faces or the current group. A group may use several slots, and different groups may share one slot. Inspector material overrides affect only the selected entity.
5. Save the scene with **Ctrl+S**, then **Build PSX** or **Play**. A window opening requires removing its faces or arranging wall pieces around the opening. A group named Windows does not cut holes automatically.

Faces support individual extrusion, subdivision, flipping and deletion. Subdivision preserves the face's group/material and interpolates its stored UVs. New geometry shares exactly coincident positions while faces retain separate UVs and material assignments. Faces must remain planar and convex; split a face before moving its corners out of its plane. Negative scaling and arbitrary concave polygons are not supported.

### Geometry editing mode

**Edit geometry...** puts the editor into geometry editing for that actor's mesh, much like an element mode in other editors:

- The **Geometry** panel takes the Hierarchy's place, and the Hierarchy is hidden until the mode ends, so nothing else can be chosen meanwhile. (A Hierarchy undocked as a floating window stays visible; the panel then floats.) The panel shows the mesh and its actor, the element mode (**Faces / Edges / Vertices**), **Undo**, **Redo**, **Revert** and **Done**, then the tool categories as chips: **Create** (Box, Plane, Ramp, Stairs), **Select** (all, clear, coplanar, connected, and whether a drag selects by box or paint), **Transform**, **Model** (Extrude, Push in or Bevel, Subdivide, Flip, Delete, Extract), **Sculpt**, **Texture**, **Groups**, **Materials** and **Asset**. Picking a tool tile shows its settings and its action; the footer always shows the selection.

- A banner at the bottom of the Scene view names the mesh and its actor and shows the element mode (**Faces 1 / Edges 2 / Vertices 3**), how much is selected, **Dim others**, **Revert** and **Done (Esc)**. On a narrow panel it drops the name and title first, then wraps onto two rows.
- Clicks only select elements of the edited mesh; a click elsewhere clears the selection. Selected faces get a translucent fill and outline, and the face, edge or vertex under the cursor is highlighted before you click.
- The actor's own selection outline and light helper are hidden while you edit its geometry. **Dim others** (on by default) darkens the rest of the scene.
- **Done**, Escape, closing the Geometry panel, starting Play or choosing another actor (for example from the global search) end the mode; the Hierarchy returns with the edited actor selected. Every operation is already saved, so leaving never loses work.
- **Revert** restores the mesh as it was when editing began, as one Undo step; the panel and the banner both offer it.

### Drag selection

A left drag in the Scene view selects several elements of the current mode at once. **Select > Drag** chooses how:

- **Box** (the default) draws a rectangle and, on release, selects the faces whose centre it encloses, the edges with both ends inside it, or the vertices inside it. Only what the mesh itself shows to the camera counts, so a box over a floor never takes the ceiling or the underside behind it. **Select through** also takes elements hidden behind other faces of the mesh.
- **Paint** selects every element the cursor passes over while the button is held, which suits picking floor tiles or a strip of wall one by one.

A plain drag replaces the selection, **Shift** adds to it and **Ctrl** removes from it. Escape cancels a drag without leaving the mode (painted elements stay selected). Faces in hidden or locked groups are never selected. Moving less than three pixels before releasing is still a click; Alt + drag still orbits the camera.

### Texture correction

The PlayStation GPU interpolates texture coordinates linearly across each triangle on screen, without perspective. A large textured face seen at an angle therefore bends straight texture lines where its two triangles meet, and the editor's Scene and Game views show the same thing. Smaller faces bend less, which is how games of the era kept floors and walls straight near the camera.

**Texture correction > Subdivide large faces** cuts faces until no edge is longer than **Max edge length** (local units of the mesh; the actor's scale multiplies them). With faces selected it works on the selection, otherwise on the whole mesh. The panel previews how many faces are split and the extra PSX triangles before anything changes.

- Opposite edges of a quad, and all edges of a triangle, form edge loops. Every edge of a loop is cut into the same pieces at the same points, so a selection extends along its loops and neighbouring faces stay joined without cracks.
- UVs and vertex colors are interpolated; group and material are kept. Split faces get new IDs; untouched faces keep theirs.
- The operation is one Undo step. It refuses to change hidden or locked faces and stays within the asset limits (3,500 faces).
- It is an authoring edit: the build derives chunks, streaming pages and packet capacity from the result like any other change. More faces mean more triangles: check the scene's triangle budget in Project Settings and the dropped-triangle counter, and bake visibility again if a camera track covers the mesh.

### Keyboard editing and ramps

With Scene focused, **1** selects faces, **2** edges and **3** vertices. Click to select, Shift-click to add and Ctrl-click to add or remove; drag to select by box or paint. **E** extrudes selected faces outward; **Q** pushes them inward, using the distance in Blockout (initially 0.25 units). These are individual face extrusions, not CSG subtraction or the inverse of Undo.

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

The compiler keeps authored faces as they are: it never subdivides a polygon, so topology and the PSX's affine texture interpolation stay exactly as modelled (a face too wide for compact chunk coordinates widens its chunk instead). It quantizes positions to Q12, partitions by eight-unit spatial cells, and shares indexed positions within each chunk. A chunk contains at most 96 quads and 384 positions. To make large textured faces bend less, subdivide them while modelling; see [Texture correction](#texture-correction). Materials and baked color offsets survive partitioning. Authoring groups never force separate rendering batches.

The native renderer checks transformed chunk bounds against the camera frustum before transforming vertices. Visible chunks use triangle backface rejection and clipping against near, far and side planes. Near-plane clipping preserves interpolated vertex colors. The ordering table still cannot correctly resolve every intersecting surface.

| Limit | Current value |
| --- | --- |
| Local authored coordinates | ±128 units |
| Stored vertices per asset | 16,384 |
| Groups / material slots per asset | 256 / 64 |
| Compiled scene geometry | 7,000 base triangles, counting a triangle face conservatively as a quad |
| Extra clipping capacity | 512 triangles, in addition to runtime-object reserve |
| Geometry history | 32 commands for the current asset |

The **Asset** category displays full authored/compiled counts even while groups are hidden. These are capacity limits, not a frame-rate promise. `epok::mesh_stats` reports tested/visible chunks, transformed vertices, rejected backfaces and clipped input triangles; `lighting_stats` reports submitted/dropped triangles and frame timing. Reduce visible geometry or split a large level into separately managed sections when needed. Real-console validation remains pending.

Baked lighting uses the actual faces, including shadows between parts of the same asset. Realtime lighting uses arbitrary face normals with inverse-transpose transformation; the existing one-directional/one-point-light selection remains per entity, so baked lighting is preferable for large rooms. Blob shadows still require the legacy horizontal Ground primitive.

Materials support color, Unlit, imported PNG textures, UVs and PSX blend modes. Static OBJ/MTL import produces editable geometry; see [textures](textures.md) and [static mesh import](static-mesh-import.md). CSG, region extrusion, automatic removal of internal solid faces, automatic collider generation, portals/PVS, LOD and streaming remain outside this profile. Box colliders can be authored separately. Stairs are assembled from boxes, so remove unnecessary internal faces explicitly when optimizing.
