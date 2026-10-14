# Phase 6: Console look

Delivered on `feature/dungeon-generator`. Driven by feedback that the dungeon felt low
and open next to the Third Person character, with third-person console games of the
period as the reference: tall walls, a dark ceiling, textured panels.

## What shipped

| Area | Delivered |
| --- | --- |
| Proportions | Defaults from the Third Person character (1.84 units): cell 3, walls 6, doors 3.6 (`dungeon::sizes_for`: 1.63x, 3.26x and 1.96x its height). The 3D band's **Fit sizes to it** applies them to the Move to spawn actor's collider height. |
| Ceiling | A Ceiling spot on every floor cell at the wall height, facing down (culled from above, so the editor still sees in), with collision boxes above it. The generator's **Ceiling** toggle (on by default) removes both. With a ceiling the geometry casts no shadows, so the map's light reaches the interior and the ceiling reads dark. |
| Textures | `tools/build_dungeon_textures.py` draws five console-sized textures (floor, corridor, wall, ceiling, doorway); they are embedded in the editor and imported on first use under fixed asset identities, refreshed in place when they change. Corridor floors turn with the corridor. |
| UVs | The PSX cook clamps UVs to 0..1, so textured Blockout that repeated per cell used to smear on the console. Every built face now maps its texture once (floor and ceiling cells, wall sides floor to ceiling, beams, pillars, pedestals, the one-way arrow). |
| Lighting | A map without a directional light of its own gets the dungeon's sky light even when a style adds point lights. No light actors in the built-in style: each actor costs several KB of console RAM. |
| Budget | Navigation may use up to twelve volumes, so larger floors at the new cell size keep their navigation. |

## Verification

| Check | Result |
| --- | --- |
| Tests | Every face's UVs within 0..1 with and without a ceiling; one ceiling face per floor cell at the wall height and collision above it only with a ceiling; defaults equal `sizes_for(1.84)`; built-in textures imported under their identities; existing bake, cook and style tests updated |
| Fixture | 15 rooms, 2,461 generated quads (2,749 in the map), 224 boxes, 396 navigation nodes, 15 generated actors (21 in the map) |
| `--build-psx` | 1.89 MiB static RAM (a first attempt with a light per room overflowed by 11 KB: fifteen light actors cost about 90 KB) |
| PCSX-Redux | The character in a textured corridor: walls about three times its height, no sky, doorway beams with hazard stripes, the grated corridor with lit rails |

## Limits and follow-ups

- Large polygons show the console's affine texture warp, as on the hardware; cells
  are one quad per face.
- Lighting is one sky light plus ambient; coloured local light is up to styles, and
  each light is an actor.
