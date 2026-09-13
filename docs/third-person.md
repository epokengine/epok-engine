# Third Person arena template

Choose **New project > Third Person** in the Hub, or use:

```powershell
cargo run --locked -- --create-project "D:/Games/Third Person" --template third-person
```

This template is an editable **level placeholder**. Play renders the arena through a static overview camera on PSX. The mannequin and blue cubes in this template are static. The engine's [input and collision APIs](input-collision.md) are available, but this template does not attach a character controller or follow-camera Behaviour. The separate [FBX skeletal pipeline](skeletal.md) can import and animate characters, but is not wired into this template yet. See the [included example](../examples/rpg-2-5d-demo/README.md) for input, collision and scene transitions in use.

## Arena layout

The square grey enclosure contains a central raised deck with rounded corners and access ramps, a long platform on the right, a round platform at the back left, a small wedge on the left, a low ramp at the front right and three blue cubes.

The arena has a 32 × 32 unit interior, 4-unit perimeter walls, a 2-unit central deck, a 2.5-unit long platform and a 1.25-unit round platform. Curves use twelve sides for the cylinder and four segments for rounded corners. The mannequin is built from six rigid boxes.

All geometry is generated locally by `src/third_person.rs`. Project creation publishes seven independent editable mesh packages under `assets/Meshes`, assigns fresh asset UUIDs, and saves `assets/scenes/ThirdPersonArena.epokmap` with baked vertex lighting. Materials and named face groups remain editable through Blockout.

## PSX adaptation

The floor and wall grid uses adjacent colored polygons, with no overlapping decal surfaces. The template retains this geometry-based grid, although imported textures are now supported. Thin joints are widened for the native resolution. The usual geometry compiler subdivides and partitions meshes; a template test verifies the generated scene remains below the 7,000 triangle-slot capacity. This capacity check is not a frame-rate guarantee.

Creation also seeds `UserSettings/SceneView.epokprefs` with an overview for the editor. This optional local starting view is validated when a project opens; a missing or invalid file falls back to the normal editor view. It is independent of the scene's runtime camera and is not currently saved automatically during navigation.

Validation includes project creation, asset resolution, mesh validation, baked-light validity, scene-header generation and editor opening. A development preview is also compiled to PS-X EXE and captured in PCSX-Redux. Physical-console validation remains pending.
