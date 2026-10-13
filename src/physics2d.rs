//! RigidBody2D and Joint2D records (World2D initiative, phase 4). Mirrors
//! `runtime/world2d_physics.hpp`; the solver runs on the console and in Native PC Play.
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct RigidBody2D {
    pub mass: f32,
    /// Restitution: 0 stops dead, 1 bounces back at full speed.
    pub bounce: f32,
    pub friction: f32,
    pub linear_damping: f32,
    /// Pixels per second squared.
    pub gravity: [f32; 2],
    pub collision_mask: u32,
    /// Stays put and blocks like a static body until gameplay unfreezes it.
    pub frozen: bool,
    /// Characters walking into it push it.
    pub pushable: bool,
    pub can_sleep: bool,
    /// A circle turns as it rolls.
    pub rolls: bool,
    /// Velocity when the map starts.
    pub velocity: [f32; 2],
}
impl Default for RigidBody2D {
    fn default() -> Self {
        Self {
            mass: 1.,
            bounce: 0.2,
            friction: 0.5,
            linear_damping: 0.,
            gravity: [0., 720.],
            collision_mask: u32::MAX,
            frozen: false,
            pushable: true,
            can_sleep: true,
            rolls: true,
            velocity: [0., 0.],
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum JointKind {
    /// Holds the distance exactly.
    #[default]
    Rod,
    /// Holds at most the distance.
    Rope,
    /// Pulls back towards the distance.
    Spring,
}
impl JointKind {
    pub fn cpp(self) -> &'static str {
        match self {
            JointKind::Rod => "epok::JointKind2D::Rod",
            JointKind::Rope => "epok::JointKind2D::Rope",
            JointKind::Spring => "epok::JointKind2D::Spring",
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Joint2D {
    /// The first end: a node (a rigid body moves with the joint, anything else is a
    /// fixed point); none makes the joint node itself the fixed end.
    pub body_a: Option<Uuid>,
    /// The rigid body at the other end.
    pub body_b: Option<Uuid>,
    pub kind: JointKind,
    /// Pixels; 0 keeps the distance the ends have in the map.
    pub length: f32,
    pub stiffness: f32,
    pub damping: f32,
    pub enabled: bool,
}
impl Default for Joint2D {
    fn default() -> Self {
        Self {
            body_a: None,
            body_b: None,
            kind: JointKind::Rod,
            length: 0.,
            stiffness: 40.,
            damping: 2.,
            enabled: true,
        }
    }
}

/// The joint's two ends in world pixels, when both exist in `scene`.
pub fn joint_ends(
    scene: &crate::scene::Scene,
    joint_actor: usize,
    joint: &Joint2D,
) -> Option<([f32; 2], [f32; 2])> {
    let b = scene.actor_index(joint.body_b?)?;
    let a = match joint.body_a {
        Some(id) => scene.actor_index(id)?,
        None => joint_actor,
    };
    Some((
        crate::world2d::world_position(scene, a),
        crate::world2d::world_position(scene, b),
    ))
}
/// The length a joint cooks: its own, or the distance between its ends in the map.
pub fn cooked_length(scene: &crate::scene::Scene, joint_actor: usize, joint: &Joint2D) -> f32 {
    if joint.length > 0. {
        return joint.length;
    }
    joint_ends(scene, joint_actor, joint).map_or(0., |(a, b)| (b[0] - a[0]).hypot(b[1] - a[1]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world2d::{self, Preset};

    #[test]
    fn joints_cook_the_distance_their_ends_have_in_the_map() {
        let mut scene = crate::scene::Scene {
            actors: Vec::new(),
            ..Default::default()
        };
        let anchor = world2d::create(Preset::Joint2D, "Hook", [10., 0.]);
        let ball = world2d::create(Preset::RigidBody2D, "Ball", [10., 30.]);
        let joint = Joint2D {
            body_b: Some(ball.id),
            ..Default::default()
        };
        scene.actors.push(anchor);
        scene.actors.push(ball);
        assert_eq!(cooked_length(&scene, 0, &joint), 30.);
        assert_eq!(
            cooked_length(
                &scene,
                0,
                &Joint2D {
                    length: 12.,
                    ..joint.clone()
                }
            ),
            12.
        );
        assert_eq!(
            cooked_length(
                &scene,
                0,
                &Joint2D {
                    body_b: None,
                    ..joint
                }
            ),
            0.
        );
    }
}
