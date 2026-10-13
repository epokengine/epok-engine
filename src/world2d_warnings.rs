//! Configuration warnings for 2D nodes (World2D initiative): what keeps a node from doing
//! its job, shown as a warning sign on its Hierarchy row with the reasons in a tooltip,
//! so a broken node is visible without opening it. Only cheap checks run here: the
//! Hierarchy asks every visible row, every frame.
use crate::{scene::Scene, world2d};
use uuid::Uuid;

pub const ICON_WARNING: &str = "\u{f071}";

fn has(actor: &crate::scene::Actor, class: &str) -> bool {
    world2d::component(actor, class).is_some()
}
fn solid_collider(actor: &crate::scene::Actor) -> bool {
    actor
        .components
        .iter()
        .filter(|c| c.class.class_id.as_deref() == Some(world2d::COLLIDER2D))
        .map(world2d::read::<world2d::Collider2D>)
        .any(|c| c.enabled && !c.trigger)
}
fn find(scene: &Scene, id: Option<Uuid>) -> Option<&crate::scene::Actor> {
    let id = id?;
    scene.actors.iter().find(|a| a.id == id)
}

/// The reasons the actor at `index` will not work as authored; empty when it is fine.
pub fn of(scene: &Scene, index: usize) -> Vec<String> {
    let mut out = Vec::new();
    let Some(actor) = scene.actors.get(index) else {
        return out;
    };
    if has(actor, world2d::RIGID_BODY2D) && !solid_collider(actor) {
        out.push("RigidBody2D has no solid Collider2D: it falls through everything.".into());
    }
    if has(actor, world2d::CHARACTER_BODY2D) && !solid_collider(actor) {
        out.push("CharacterBody2D has no solid Collider2D: it collides with nothing.".into());
    }
    for c in actor
        .components
        .iter()
        .filter(|c| c.class.class_id.as_deref() == Some(world2d::JOINT2D))
    {
        let joint: crate::physics2d::Joint2D = world2d::read(c);
        match find(scene, joint.body_b) {
            None => out.push("Joint2D has no Body B: pick the rigid body it holds.".into()),
            Some(b) if !has(b, world2d::RIGID_BODY2D) => {
                out.push(format!("Joint2D: Body B ({}) has no RigidBody2D.", b.name))
            }
            _ => {}
        }
    }
    if let Some(c) = world2d::component(actor, world2d::SUBVIEWPORT2D) {
        let shown = scene.actors.iter().any(|a| {
            world2d::component(a, world2d::SPRITE2D)
                .is_some_and(|s| world2d::read::<world2d::Sprite2D>(s).viewport == Some(actor.id))
        });
        if !shown {
            out.push(
                "No Sprite2D shows this SubViewport2D's picture: pick it as a sprite's Viewport."
                    .into(),
            );
        }
        let v: crate::viewport2d::SubViewport2D = world2d::read(c);
        if v.camera.is_some() && find(scene, v.camera).is_none_or(|a| !has(a, world2d::CAMERA2D)) {
            out.push("SubViewport2D: its Camera node has no Camera2D.".into());
        }
    }
    if let Some(c) = world2d::component(actor, world2d::SPLIT_SCREEN2D) {
        let split: crate::viewport2d::SplitScreen2D = world2d::read(c);
        if split
            .cameras
            .iter()
            .any(|id| find(scene, *id).is_none_or(|a| !has(a, world2d::CAMERA2D)))
        {
            out.push("SplitScreen2D needs two nodes with a Camera2D.".into());
        }
    }
    if has(actor, world2d::NAVIGATION_AGENT2D)
        && !scene
            .actors
            .iter()
            .any(|a| has(a, world2d::NAVIGATION_REGION2D))
    {
        out.push("NavigationAgent2D: the map has no NavigationRegion2D to plan in.".into());
    }
    if let Some(c) = world2d::component(actor, world2d::NAVIGATION_REGION2D) {
        let region: crate::navigation2d::NavigationRegion2D = world2d::read(c);
        match region.mode {
            crate::navigation2d::NavMode::Polygon if region.outline.len() < 3 => out.push(
                "NavigationRegion2D: the navmesh outline needs at least three points.".into(),
            ),
            crate::navigation2d::NavMode::Grid
                if region.tile_layer.is_some()
                    && find(scene, region.tile_layer)
                        .is_none_or(|a| !has(a, world2d::TILEMAP_LAYER)) =>
            {
                out.push("NavigationRegion2D: its Tile layer node has no TileMapLayer.".into())
            }
            _ => {}
        }
    }
    if let Some(c) = world2d::component(actor, world2d::SKELETON2D) {
        let s: crate::skeleton2d::Skeleton2D = world2d::read(c);
        if s.bones.is_empty() {
            out.push("Skeleton2D has no bones: use its attached parts as bones.".into());
        }
        if s.ik
            .iter()
            .any(|chain| chain.tip.is_none() || chain.target.is_none())
        {
            out.push("Skeleton2D: an IK chain has no tip or no target.".into());
        }
        if s.bones.iter().any(|b| find(scene, b.actor).is_none()) {
            out.push("Skeleton2D: a bone's node is gone from the map.".into());
        }
    }
    if let Some(c) = world2d::component(actor, world2d::PATH_FOLLOW2D) {
        let follow: world2d::PathFollow2D = world2d::read(c);
        let on_path = follow
            .path
            .and_then(|id| find(scene, Some(id)))
            .or_else(|| find(scene, actor.attach.as_ref().map(|a| a.actor)))
            .is_some_and(|a| has(a, world2d::PATH2D));
        if !on_path {
            out.push("PathFollow2D has no Path2D: pick one, or attach this node to one.".into());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world2d::{Preset, create};

    #[test]
    fn broken_nodes_explain_themselves_and_working_ones_stay_quiet() {
        let mut scene = Scene {
            actors: Vec::new(),
            ..Default::default()
        };
        let mut ball = create(Preset::RigidBody2D, "Ball", [0., 0.]);
        scene.actors.push(ball.clone());
        assert!(of(&scene, 0).is_empty(), "the preset carries its collider");
        ball.components
            .retain(|c| c.class.class_id.as_deref() != Some(world2d::COLLIDER2D));
        scene.actors[0] = ball;
        assert!(of(&scene, 0)[0].contains("no solid Collider2D"));
        scene.actors.push(create(Preset::Joint2D, "Hook", [0., 0.]));
        assert!(of(&scene, 1)[0].contains("no Body B"));
        scene
            .actors
            .push(create(Preset::SubViewport2D, "Minimap", [0., 0.]));
        assert!(of(&scene, 2)[0].contains("No Sprite2D shows"));
        scene
            .actors
            .push(create(Preset::NavigationAgent2D, "Bat", [0., 0.]));
        assert!(of(&scene, 3)[0].contains("no NavigationRegion2D"));
        scene
            .actors
            .push(create(Preset::NavigationRegion2D, "Floor", [0., 0.]));
        assert!(of(&scene, 3).is_empty() && of(&scene, 4).is_empty());
        scene
            .actors
            .push(create(Preset::Skeleton2D, "Rig", [0., 0.]));
        assert!(of(&scene, 5)[0].contains("no bones"));
        scene
            .actors
            .push(create(Preset::PathFollow2D, "Cart", [0., 0.]));
        assert!(of(&scene, 6)[0].contains("no Path2D"));
        scene
            .actors
            .push(create(Preset::Sprite2D, "Tree", [0., 0.]));
        assert!(of(&scene, 7).is_empty());
    }
}
