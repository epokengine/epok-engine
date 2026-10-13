# World2D initiative

Turns the `Actor2D` runtime contract (`runtime/world2d.hpp`, actor-architecture P8)
into a complete 2D game workflow: authored components, a native PSX 2D renderer,
2D physics and navigation, tile worlds and a complete visual 2D editor. Work happens on
`features/2d-improvements`, one pushed checkpoint per phase.

| Phase | Scope | Status |
| --- | --- | --- |
| 0 | Foundations: target validation of the 2D/actor runtime, measured MIPS sizes, resolution-aware VRAM reservation | delivered ([phase0.md](phase0.md)) |
| 1 | Playable 2D: Sprite2D/AnimatedSprite2D, Camera2D, Collider2D, CharacterBody2D, 2D render path, 2D gameplay API, 2D editor tools, Platformer 2D template | delivered ([phase1.md](phase1.md)) |
| 2 | Tile worlds: TileSet asset, TileMap layers, tile painting, terrains, tile collision, Parallax2D, canvas layers | delivered ([phase2.md](phase2.md)) |
| 3 | Content and polish: Path2D/PathFollow2D, Line2D, Polygon2D, 2D particles, 2D audio, notifiers, remote transforms, UI gaps | delivered ([phase3.md](phase3.md)); scroll containers, tabs, themes, interactive XA music, timeline 2D tracks and scene import deferred |
| 4 | Advanced: Skeleton2D cut-out animation and IK, grid/navmesh navigation, bounded rigid bodies and joints, render-to-VRAM groups | cut-out skeletons, IK and navigation delivered ([phase4.md](phase4.md)); physics and render-to-VRAM in progress |
| 5 | Showcase template: the Platformer 2D template made more attractive (more parallax layers, effects, ambience) while staying simple to start a game from | planned |
