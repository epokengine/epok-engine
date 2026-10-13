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

## 3. Rigid bodies and joints (`runtime/world2d_physics.hpp`, `src/physics2d.rs`)

| Piece | Contract |
| --- | --- |
| `RigidBody2DComponent` | Shape: the node's solid Collider2D. Integrates gravity (per body), one-step forces and damping, then sweeps through the collision world with the CharacterBody2D sweep (tiles and colliders, no tunnelling). Per blocked axis: rigid other → 1D momentum exchange with the lower restitution; static → restitution (small bounces settle) and Coulomb-style friction on the other axis. Circles roll (rotation from speed and radius). Sleeps after 0.5 s of resting contact; impulses, forces, pushes and joints wake it. Boxes do not rotate |
| Characters | `CharacterBody2D::blocked_velocity` and `wall_collider` (new) record what a move lost against a wall; the service turns that into a push on a pushable rigid body, three quarters of the speed divided by the mass above 1 |
| `Joint2DComponent` | Rod / rope: four position passes after the bodies move, split by inverse mass, plus removal of the relative speed along the joint; spring: a force before integration. Body A may be any node (fixed when not rigid) or none (the joint node). `set_length`, `break_joint` |
| `Line2DComponent::follow` | Up to 16 nodes the line runs through; the service moves its points onto them every step, after everything else moved |
| Gameplay | `apply_impulse`, `apply_force`, `set_velocity`, `get_velocity_x/y`, `get_angular_velocity`, `is_sleeping`, `wake`, `set_frozen`, `is_frozen`, `take_contact`, `get_impact`, `is_touching`; joints `set_enabled`, `is_enabled`, `set_length`, `get_length`, `break_joint`; lines `stop_following` |

Editor (`src/world2d_physics_editor.rs`): RigidBody2D and Joint2D presets in the
Physics group; bands with mass, bounce, friction, damping, gravity, start velocity,
pushable, frozen, can sleep and rolls (circles), and warnings for a missing solid
collider or an attachment; joint kind, ends, length (0 = the map distance, shown),
stiffness and damping; joints drawn while colliders are shown. Line2D gains Follow
nodes; a following line draws through its nodes in the view and leaves the points
editor.

The Platformer 2D template gains a pushable crate and a garland of light bulbs on rope
joints, its cord a Line2D following the hooks and bulbs.

## 4. Pictures and split screen (`runtime/world2d_viewports.hpp`, `src/viewport2d.rs`)

| Piece | Contract |
| --- | --- |
| `RenderPass` | The view being collected: camera (else the active one), world and/or screen canvases, one canvas layer or every layer minus hidden ones, culling rectangle. `collect_sprites`, `entry_space` and `each_tile` read it |
| `SubViewport2DComponent` | Camera (else active), picture size, background, interval, canvas filter and `hide_layer`; cooked VRAM placement `target_x/y` (64-word column, inside a 256-line page row). `view()` recentres the camera on the picture |
| PSX renderer | Per picture due this frame: FastFill, drawing area and offset at the picture, the pass, then one cache flush and the framebuffer's area restored before the views. 15-bit textures: `Texture::bits = 16`, page colour mode 2, `texture_base_u` one texel per word |
| `Sprite2DComponent::viewport` | An actor with a SubViewport2D whose picture replaces the texture (entry texture -2, component = the picture) |
| `SplitScreen2DComponent` | Two camera actors, side by side or stacked, divider width and colour; each half: drawing area clipped, background panel, world pass; then the divider and one screen-canvas pass |
| Native PC | Commands 6 (clip), 7 (begin picture n) and 8 (end picture); texture ids from 0x10000 sample picture n; the runner mirrors the console's passes |
| Texture packer | `Layout::pictures`: reserved before textures; a picture with no room fails the build |

Editor (`src/world2d_viewport_editor.rs`): presets in the Canvas and views group; picture
band (camera, size with the VRAM cost, background, redraw interval, layer filter, hide
layer, whether a sprite shows it) and split band (cameras, layout, divider); Sprite2D
gains a Viewport picker once the map has a picture; sprites showing a picture draw a
framed stand-in; the selected picture's or split's camera frames are drawn in the view.

## Validation

| Check | Result |
| --- | --- |
| `tests/runtime/world2d_service.cpp` | pose blending, loop wrap, shorter-arc rotation, one-shot finish, rest pose; IK reach on both elbow sides, out of reach, disabled chain |
| `cargo test` | sampling, keying within a frame, normalization of new bones, bone removal, IK turns, validation, cooking (bank path included) |
| `tests/runtime/world2d_service.cpp` (navigation) | grid paths around a wall, no corner squeezing, closest cell when unreachable; navmesh path turning at an L's inner corner, off-mesh clamping; an agent walking to a position and following a moving actor, arrival reported once |
| `cargo test` (navigation) | grid building from tiles, one-way tiles, static bodies and radius; grid and navmesh searches; cooking of grids, meshes and agents; scratch capacity |
| `tests/runtime/world2d_service.cpp` (physics) | falling to rest and sleeping, contact once with its impact, bouncing, friction stopping a slide, elastic exchange between equal bodies, a character pushing (and not pushing) a crate, rolling, a rod pendulum holding its length, a rope going taut, a spring settling at mg/k, breaking a joint, a line following nodes |
| `cargo test` (physics) | joint lengths from the map, cooking of bodies and joints |
| `tests/runtime/world2d_service.cpp` (views) | a pass's camera culls, world/screen canvases, one layer, hidden layers, a sprite showing a picture, picture views, split halves and views, inactive split |
| `cargo test` (views) | picture sizes and halves, VRAM reservation of pictures, cooking of pictures, splits and sprites showing them, rasterizer clipping and pictures |
| `tests/integration/verify_world2d_views.py` | PCSX-Redux: a minimap SubViewport2D shown by a HUD sprite, then a SplitScreen2D with its divider and two different halves; Native PC Play draws the same |
| PSX | Platformer 2D in C++, Lua and Blueprint in PCSX-Redux: the guide animates and points at the player, the wisp follows the player, the player pushes the crate and the garland sways |
| Sample game | static RAM unchanged (976.0 KiB): projects without 2D content compile none of it |
| Platformer 2D | static RAM 1.31 MiB with the guide, 1.35 MiB with the wisp (11 KiB of A* scratch), 1.41 MiB with the crate and garland |
