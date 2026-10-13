# Godot 2D capabilities versus Epok (October 2026)

Reference: Godot 4.7.2 stable and 4.8-dev (`scene/2d`, `scene/resources/2d`,
`modules/tilemap`, `modules/godot_physics_2d`, `modules/navigation_2d`,
`editor/scene/2d`). 193 Godot 2D features were classified against Epok v0.5.0.

| Epok before this initiative | Rows |
| --- | ---: |
| Equivalent exists (mostly UI/HUD, flipbooks, layers/masks, real-target preview) | 19 |
| Partial base (runtime contract, 3D or UI implementation to reuse) | 44 |
| Missing | 130 |

| Feasible on the PlayStation | Rows |
| --- | ---: |
| Feasible, engineering work only | 111 |
| Feasible within bounds or approximated | 38 |
| Not feasible or not applicable | 19 |
| Editor-only (host side, no hardware bound) | 25 |

The GPU of the original PlayStation is a 2D rasterizer, so the classic 2D feature
set fits it well: sprites and frame animation, tile maps with terrains, parallax,
2D cameras, platformer collision, paths, panned audio and pad-driven UI. Out of
scope on this hardware: `canvas_item` shaders, normal/specular maps, per-pixel
lights with SDF shadows, GPU particles, MSDF/dynamic font scaling, bilinear
filtering and mipmaps, HDR and MSAA, per-bus audio DSP.

Godot's runtime cannot be ported as code: it relies on floating point, heap
containers, RID servers, threads and Vulkan/GL. Its algorithms and semantics are
reused instead — terrain peering-bit rules, `Curve2D` baking, polygon
triangulation, `Camera2D` smoothing/limits/drag margins, the
`CharacterBody2D.move_and_slide` contract, `AStarGrid2D` and `Parallax2D`
scrolling — and reimplemented in Q12 fixed point under the runtime rules (no
heap, no RTTI, no exceptions, no recursion, fixed capacities).
