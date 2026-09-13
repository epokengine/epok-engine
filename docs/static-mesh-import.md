# Static OBJ import

Copy polygonal OBJ and optional MTL files into the project's `assets/` directory. Import any referenced PNG images as Texture assets first. In the entity Inspector choose **Import static OBJ...**, enter the source and a new `.epokasset` destination, then select **Import and assign**. The imported result is an ordinary Editable Mesh: groups, UVs and material slots can be edited and overridden with the existing tools.

The command-line equivalent is:

```powershell
epok-editor --project MyGame --import-obj assets/models/room.obj --asset assets/models/room.epokasset --scale 1
```

Import supports vertices, per-corner texture coordinates, positive/negative indices, object/group names, material assignments, and concave polygon triangulation. Normals are recalculated from the resulting face geometry. OBJ V coordinates are converted to texture's top-down origin. MTL diffuse color, unlit `illum 0`, alpha blend selection and `map_Kd` texture references are transported to the PSX renderer. Dependencies must resolve inside project assets. Texture-map transforms should be baked into UVs before import.

The target accepts normalized 0–1 UVs, local coordinates within ±128 after scaling, 16,384 vertices, 3,500 faces after triangulation, 256 groups and 64 material slots. Invalid indices, nonfinite coordinates, unsupported curves/lines and degenerate polygons fail before publication. The importer writes a new asset atomically and refuses to overwrite an existing destination. Keep the original OBJ/MTL files for future imports; the `.epokasset` contains the editable converted mesh.
