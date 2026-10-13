# Phase 4 — Advanced 2D (2026-10-13)

User guide: [docs/2d.md](../../../docs/2d.md) (Cut-out characters).

## 1. Cut-out skeletons (`runtime/world2d_skeleton.hpp`, `src/skeleton2d.rs`)

| Piece | Contract |
| --- | --- |
| Bones | Ordinary 2D actors (usually one Sprite2D part each) attached to each other; the skeleton writes their root transforms. Up to 24 bones, cooked as `actors[slot]` references |
| `SkeletonClip2D` | Pose keys: `times[key_count]` and `poses[key_count][bone_count][3]` (rotation, x, y), constants in the bank; `looping` blends the last key back into the first across `length` |
| Sampling | Rotation along the shorter arc, position linear; identical in `Skeleton2D::sample` (editor) and `Skeleton2DComponent::apply` (runtime) |
| `SkeletonIk2D` | Two-bone chain: upper and lower bone indices, tip and target actors, elbow side, enabled. Law of cosines on world positions, after the animation, before colliders are gathered; out of reach points straight at the target |
| Gameplay | `play`, `stop`, `is_playing`, `get_animation(_count)`, `set_speed`, `seek`, `get_time`, `take_finished`, `reset_pose`, `get_bone(_count)`, `set_ik_enabled`, `is_ik_enabled`, `set_ik_target` |
| Cost | 224 bytes per Skeleton2D component on the console; keys 12 bytes per bone per key |

Editor (`src/world2d_skeleton_editor.rs`): Skeleton2D preset (Animation group); bones
collected from the attached parts (IK tips and targets excluded), indented by depth,
selectable, removable (keys and chains follow); rest pose capture and restore;
animations with name, length, loop, plays-on-start and speed; a timeline strip with
key diamonds, click/drag scrubbing that poses the bones (one undo step per drag), Key
pose, Delete key, key time; a Play preview that animates without touching history and
restores the pose on stop; the same timeline on every bone's Inspector; IK chains
with bone pickers, tip/target pickers, elbow side and Reach target (the runtime solve,
applied once); the view draws bones as tapered shapes between joints and each chain's
tip-to-target line.

The Platformer 2D template gains a cut-out guide whose robe breathes and head nods on
a looping animation while its IK arm points at the player.

## 2. Navigation (`runtime/world2d_navigation.hpp`, `src/navigation2d.rs`)

| Piece | Contract |
| --- | --- |
| `NavGrid2D` | One bit per cell, row-major, cooked in world pixels from a tile layer's solid (not one-way) tiles, non-trigger colliders of nodes without a CharacterBody2D, dilated by the agent radius; up to 16384 cells |
| Grid search | A* with costs 2/3, octile heuristic, no corner cutting; the path to the closest cell when the target is unreachable; line-of-sight pruning (64 cells ahead) through cell centres. Scratch: `uint16` cost + `uint8` info per cell + open heap, sized by `EPOK_WORLD2D_NAV_CELLS` / `EPOK_WORLD2D_NAV_OPEN` |
| `NavMesh2D` | Ear-clipped outline (up to 64 points, 62 triangles), positive winding, neighbours across each edge; A* over centroids then the funnel algorithm; a target off the mesh clamps to the closest point |
| Agents | Follow a target actor (re-plan after `repath_interval` once it moved more than the arrive distance) or a position; one plan per step for the whole map; drive a CharacterBody2D through `move_and_slide` or the node directly; never overshoot a waypoint |
| Gameplay | Region: `set_enabled`, `is_enabled`, `is_walkable`. Agent: `set_target_position`, `set_target_actor`, `get_target_actor/x/y`, `stop`, `is_navigation_finished`, `is_target_reachable`, `take_arrived`, `get_next_x/y`, `get_velocity_x/y`, `get_path_point_count`, `set/get_max_speed` |

Editor (`src/world2d_navigation_editor.rs`): presets in a Navigation group; region
band (Grid/Navmesh, tile layer, margin or cell size and area, static bodies, agent
radius, diagonal moves, live size/blocked count or error); the navmesh outline is a
shape of the points editor; the view draws the grid (blocked cells) or triangles of
the selected region and the selected agent's predicted path (green reaching, orange
stopping short) with the editor's port of the searches.

The Platformer 2D template gains a wisp: a navigation agent on a grid over the Ground
layer that follows the player round the solid tiles with a glowing particle trail.

## Validation

| Check | Result |
| --- | --- |
| `tests/runtime/world2d_service.cpp` | pose blending, loop wrap, shorter-arc rotation, one-shot finish, rest pose; IK reach on both elbow sides, out of reach, disabled chain |
| `cargo test` | sampling, keying within a frame, normalization of new bones, bone removal, IK turns, validation, cooking (bank path included) |
| `tests/runtime/world2d_service.cpp` (navigation) | grid paths around a wall, no corner squeezing, closest cell when unreachable; navmesh path turning at an L's inner corner, off-mesh clamping; an agent walking to a position and following a moving actor, arrival reported once |
| `cargo test` (navigation) | grid building from tiles, one-way tiles, static bodies and radius; grid and navmesh searches; cooking of grids, meshes and agents; scratch capacity |
| PSX | Platformer 2D in C++, Lua and Blueprint in PCSX-Redux: the guide animates and points at the player, and the wisp follows the player as it runs and jumps |
| Sample game | static RAM unchanged (976.0 KiB): projects without 2D content compile none of it |
