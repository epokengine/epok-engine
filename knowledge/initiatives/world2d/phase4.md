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

## Validation

| Check | Result |
| --- | --- |
| `tests/runtime/world2d_service.cpp` | pose blending, loop wrap, shorter-arc rotation, one-shot finish, rest pose; IK reach on both elbow sides, out of reach, disabled chain |
| `cargo test` | sampling, keying within a frame, normalization of new bones, bone removal, IK turns, validation, cooking (bank path included) |
| PSX | Platformer 2D in C++, Lua and Blueprint in PCSX-Redux: the guide animates and points at the player as it runs and jumps |
| Sample game | static RAM unchanged (976.0 KiB): projects without 2D content compile none of it |
