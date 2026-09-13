# Lighting in Epok

The editor, exported C++ project and PSX executable now support a bounded lighting pipeline. Scene is an interactive preview. Game displays the actual emulator output, including PSX quantization and painter ordering.

## Authoring workflow

1. Use **GameObject > Light** or the Hierarchy context menu to create a Directional Light or Point Light. Existing entities can receive a **Light** through Add Component. Directional lights travel along their inherited Transform's +Z axis; rotating the entity changes the lighting direction. Light helpers can be selected in Scene.
2. Open **Window > Lighting** for ambient color, baked ambient occlusion, the realtime point-light budget, bake status and geometry/color-storage counts.
3. In Mesh Renderer, choose **Unlit**, **Baked Vertex**, or **Realtime (GTE)**. Baked Vertex enables Static. Static receivers and shadow casters must remain fixed in the game, including their parents. Unchecking Static switches a baked receiver back to Realtime.
4. Use **Subdivisions** to provide enough vertices for local illumination and shadows. The existing flat Ground primitive retains its 10×10 top grid. Keep subdivisions low elsewhere; the exporter rejects scenes above 7,000 mesh triangles.
5. Click **Bake Lighting**. This runs on a worker thread. Continue editing while it runs; results for changed bake inputs are discarded. Save Scene to persist the cache. Play, Build and Export automatically regenerate outdated lighting. Play/Build return that cache to the editor when the authored inputs still match.

Without any lights, lit materials show the environment's ambient color. The old artificial six-face shading is no longer used in the renderer. Existing scene files remain readable and existing user assets are not overwritten during migration.

| Light mode | Baked receivers | Realtime receivers |
| --- | --- | --- |
| Baked | Included in offline bake | No direct contribution |
| Realtime | No contribution | Selected within the per-object budget |
| Mixed | Included in offline bake | Selected within the per-object budget |

Mixed does not add its direct contribution twice: baked receivers use their cache; realtime receivers evaluate lights. Changing a Mixed light from C++ changes its realtime contribution only. It cannot alter shadows/colors already baked into the scene.

## Costs and limits

- Baked illumination stores RGB8 per face corner. Static shadow rays and optional eight-direction ambient occlusion run only on the PC. The PSX still pays for color data and Gouraud rasterization.
- Geometry is exported as indexed Q12 vertices, shared by matching primitive topologies. Each vertex is transformed once per object per frame. Meshes have at most 512 cached vertices.
- Realtime shading selects **one directional and one point light per visible object** from at most 32 active sources. A scene may contain at most 32 authored Light components. Runtime additions beyond the active-source capacity are ignored in entity order.
- Point lights use the object's center, squared linear falloff `(1 - distance/range)^2`, priority, selection hysteresis and a short transition filter. They approximate local illumination on small objects; use baked lighting for large surfaces.
- GTE lighting caches six normals for legacy cubes. EditableMesh uses arbitrary per-face normals. Cofactor normals account for nonuniform scale and shear. No floating-point math or per-frame heap allocation is used in target lighting.
- Realtime objects do not receive baked shadows, AO or probes. The material remains a color material; texture lightmaps, custom imported mesh normals and light probes are future work.
- Baked scenes consume additional RAM for vertex colors and indexed geometry. Triangle buffers are bounded; these capacity limits do not promise a fixed frame rate for every scene.

## Moving shadows

Add **Blob Shadow** to a moving entity. Its radius, strength and maximum distance control a soft subtractive octagon beneath the entity. It finds the highest horizontal, axis-aligned Ground mesh below the entity center and clips to that floor's bounds. It fades with height and costs eight triangles, up to 32 visible blobs. It does not follow sloped surfaces or arbitrary meshes. PSX painter-order limitations still apply. Blend commands are restored within each shadow packet so shadows do not change HUD blending.

## C++ components

```cpp
#include "epok.hpp"
using namespace epok;

void Player::start(Transform&) {
    if (auto* lamp = create_entity("Player light", &entity())) {
        lamp->transform.position[1] = 1.0;
        auto& light = lamp->add<Light>();
        light.type = LightType::Point;
        light.mode = LightMode::Realtime;
        light.color[0] = 255;
        light.color[1] = 160;
        light.color[2] = 70;
        light.intensity = 0.8;
        light.range = 5.0;
        light.priority = 10;
    }
    entity().add<BlobShadow>().radius = 0.6;
}
```

Use `get<Light>()`, `get<MeshLighting>()`, `get<BlobShadow>()` and `remove<T>()` like the HUD components. Light intensity is clamped to 0–2 and range to 0–128 on the target. Static flags and subdivisions are authoring metadata; changing them in C++ does not rebake or rebuild the exported mesh. To move an authored baked object, first switch its `lighting.receive` to `ReceiveLighting::Realtime`; its old baked shadows on other objects will remain.

`lighting_environment.ambient` and `.point_lights` are available to scripts. Ambient changes affect realtime receivers; baked receivers keep their stored ambient contribution.

`lighting_stats` exposes completed-frame active lights, lit objects, GTE normal operations, baked vertices, submitted/dropped mesh triangles, blob triangles, lighting scanlines and CPU frame scanlines. Timing reads PsyQo's existing horizontal-sync counter without reconfiguring timers. Resolution is about 64 microseconds; frame timing excludes the subsequent GPU/page-flip wait. These measurements are useful for comparing configurations, not for claiming cycle-exact hardware performance from an emulator.

## Verification

`tests/integration/verify_lighting.py` builds isolated projects, runs their native PSX executables and reads actual VRAM/RAM. It checks baked/GTE color agreement, the light budget, inverse-transpose normals, point range, C++ creation parity, static shadows, blob shadows, saved bake caches and target operation/timing counters. It leaves the user's scene untouched and writes results to `artifacts/lighting-verification.txt`.

Headless baking: `epok-editor --project <project> --bake-lighting`. This explicitly saves the prepared scene. `--screenshot-lighting --screenshot <image.png>` opens the Lighting window for an editor capture.
