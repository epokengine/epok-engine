//! Configuration warnings for 2D nodes (World2D initiative): what keeps a node from doing
//! its job, shown as a warning sign on its Hierarchy row with the reasons in a tooltip,
//! and at the top of its Inspector with a one-click fix where one is obvious, so a
//! broken node is visible without opening it. Only cheap checks run here: the
//! Hierarchy asks every visible row, every frame.
use crate::{scene::Scene, world2d};
use uuid::Uuid;

pub const ICON_WARNING: &str = "\u{f071}";
pub const COLOR: [f32; 4] = [1., 0.72, 0.3, 1.];

/// A repair the Inspector offers next to a warning (applied by `world2d_editor`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fix {
    /// Add a solid Collider2D fitted to the node's sprite.
    AddCollider,
    /// Create a Sprite2D showing the picture in a corner of the screen.
    ShowPicture,
    /// Create a NavigationRegion2D grid over the map's first tile layer.
    AddNavigationRegion,
}
impl Fix {
    pub fn label(self) -> &'static str {
        match self {
            Fix::AddCollider => "Add a Collider2D",
            Fix::ShowPicture => "Show it on screen",
            Fix::AddNavigationRegion => "Add a NavigationRegion2D",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Warning {
    pub text: String,
    pub fix: Option<Fix>,
}

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
/// Whether a Joint2D holds the actor or sits on it: a hanging body needs no collider.
fn jointed(scene: &Scene, actor: Uuid) -> bool {
    scene.actors.iter().any(|a| {
        a.components
            .iter()
            .filter(|c| c.class.class_id.as_deref() == Some(world2d::JOINT2D))
            .map(world2d::read::<crate::physics2d::Joint2D>)
            .any(|j| a.id == actor || j.body_a == Some(actor) || j.body_b == Some(actor))
    })
}
fn find(scene: &Scene, id: Option<Uuid>) -> Option<&crate::scene::Actor> {
    let id = id?;
    scene.actors.iter().find(|a| a.id == id)
}

/// The reasons the actor at `index` will not work as authored; empty when it is fine.
pub fn of(scene: &Scene, index: usize) -> Vec<Warning> {
    let Some(actor) = scene.actors.get(index) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let mut warn = |text: String, fix: Option<Fix>| out.push(Warning { text, fix });
    if has(actor, world2d::RIGID_BODY2D) && !solid_collider(actor) && !jointed(scene, actor.id) {
        warn(
            "RigidBody2D has no solid Collider2D: it falls through everything.".into(),
            Some(Fix::AddCollider),
        );
    }
    if has(actor, world2d::CHARACTER_BODY2D) && !solid_collider(actor) {
        warn(
            "CharacterBody2D has no solid Collider2D: it collides with nothing.".into(),
            Some(Fix::AddCollider),
        );
    }
    for c in actor
        .components
        .iter()
        .filter(|c| c.class.class_id.as_deref() == Some(world2d::JOINT2D))
    {
        let joint: crate::physics2d::Joint2D = world2d::read(c);
        match find(scene, joint.body_b) {
            None => warn(
                "Joint2D has no Body B: pick the rigid body it holds.".into(),
                None,
            ),
            Some(b) if !has(b, world2d::RIGID_BODY2D) => warn(
                format!("Joint2D: Body B ({}) has no RigidBody2D.", b.name),
                None,
            ),
            _ => {}
        }
    }
    if let Some(c) = world2d::component(actor, world2d::SUBVIEWPORT2D) {
        let shown = scene.actors.iter().any(|a| {
            world2d::component(a, world2d::SPRITE2D)
                .is_some_and(|s| world2d::read::<world2d::Sprite2D>(s).viewport == Some(actor.id))
        });
        if !shown {
            warn(
                "No Sprite2D shows this SubViewport2D's picture: pick it as a sprite's Viewport."
                    .into(),
                Some(Fix::ShowPicture),
            );
        }
        let v: crate::viewport2d::SubViewport2D = world2d::read(c);
        if v.camera.is_some() && find(scene, v.camera).is_none_or(|a| !has(a, world2d::CAMERA2D)) {
            warn(
                "SubViewport2D: its Camera node has no Camera2D.".into(),
                None,
            );
        }
    }
    if let Some(c) = world2d::component(actor, world2d::SPLIT_SCREEN2D) {
        let split: crate::viewport2d::SplitScreen2D = world2d::read(c);
        if split
            .cameras
            .iter()
            .any(|id| find(scene, *id).is_none_or(|a| !has(a, world2d::CAMERA2D)))
        {
            warn(
                "SplitScreen2D needs two nodes with a Camera2D.".into(),
                None,
            );
        }
    }
    if has(actor, world2d::NAVIGATION_AGENT2D)
        && !scene
            .actors
            .iter()
            .any(|a| has(a, world2d::NAVIGATION_REGION2D))
    {
        warn(
            "NavigationAgent2D: the map has no NavigationRegion2D to plan in.".into(),
            Some(Fix::AddNavigationRegion),
        );
    }
    if let Some(c) = world2d::component(actor, world2d::NAVIGATION_REGION2D) {
        let region: crate::navigation2d::NavigationRegion2D = world2d::read(c);
        match region.mode {
            crate::navigation2d::NavMode::Polygon if region.outline.len() < 3 => warn(
                "NavigationRegion2D: the navmesh outline needs at least three points.".into(),
                None,
            ),
            crate::navigation2d::NavMode::Grid
                if region.tile_layer.is_some()
                    && find(scene, region.tile_layer)
                        .is_none_or(|a| !has(a, world2d::TILEMAP_LAYER)) =>
            {
                warn(
                    "NavigationRegion2D: its Tile layer node has no TileMapLayer.".into(),
                    None,
                )
            }
            _ => {}
        }
    }
    if let Some(c) = world2d::component(actor, world2d::SKELETON2D) {
        let s: crate::skeleton2d::Skeleton2D = world2d::read(c);
        if s.bones.is_empty() {
            warn(
                "Skeleton2D has no bones: use its attached parts as bones.".into(),
                None,
            );
        }
        if s.ik
            .iter()
            .any(|chain| chain.tip.is_none() || chain.target.is_none())
        {
            warn(
                "Skeleton2D: an IK chain has no tip or no target.".into(),
                None,
            );
        }
        if s.bones.iter().any(|b| find(scene, b.actor).is_none()) {
            warn(
                "Skeleton2D: a bone's node is gone from the map.".into(),
                None,
            );
        }
    }
    if let Some(c) = world2d::component(actor, world2d::DEPTH2D) {
        let depth: world2d::Depth2D = world2d::read(c);
        if !scene.actors.iter().any(|a| a.kind == "Camera") {
            warn(
                "Depth2D: the map has no 3D camera, so these nodes draw on their canvas.".into(),
                None,
            );
        }
        if depth.anchor.is_some()
            && find(scene, depth.anchor).is_none_or(|a| {
                crate::actor_components::domain(a) != crate::reflection_schema::Domain::World3D
            })
        {
            warn(
                "Depth2D: its Anchor is gone or is not a 3D actor; Distance is used.".into(),
                None,
            );
        }
        if [world2d::LINE2D, world2d::POLYGON2D, world2d::PARTICLES2D]
            .iter()
            .any(|id| has(actor, id))
        {
            warn(
                "Depth2D places sprites and tile layers only: lines, polygons and particles stay on their canvas.".into(),
                None,
            );
        }
        if scene.actors.iter().any(|a| has(a, world2d::SPLIT_SCREEN2D)) {
            warn(
                "Depth2D is ignored while a SplitScreen2D is active.".into(),
                None,
            );
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
            warn(
                "PathFollow2D has no Path2D: pick one, or attach this node to one.".into(),
                None,
            );
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
        assert!(of(&scene, 0)[0].text.contains("no solid Collider2D"));
        assert_eq!(of(&scene, 0)[0].fix, Some(Fix::AddCollider));
        scene.actors.push(create(Preset::Joint2D, "Hook", [0., 0.]));
        assert!(of(&scene, 1)[0].text.contains("no Body B"));
        let ball = scene.actors[0].id;
        let hook = scene.actors[1]
            .components
            .iter_mut()
            .find(|c| c.class.class_id.as_deref() == Some(world2d::JOINT2D))
            .unwrap();
        world2d::write(
            hook,
            &crate::physics2d::Joint2D {
                body_b: Some(ball),
                ..world2d::read(hook)
            },
        );
        assert!(
            of(&scene, 0).is_empty() && of(&scene, 1).is_empty(),
            "a body on a joint (a lamp, a bulb) may hang without a collider"
        );
        scene
            .actors
            .push(create(Preset::SubViewport2D, "Minimap", [0., 0.]));
        assert!(of(&scene, 2)[0].text.contains("No Sprite2D shows"));
        assert_eq!(of(&scene, 2)[0].fix, Some(Fix::ShowPicture));
        scene
            .actors
            .push(create(Preset::NavigationAgent2D, "Bat", [0., 0.]));
        assert!(of(&scene, 3)[0].text.contains("no NavigationRegion2D"));
        assert_eq!(of(&scene, 3)[0].fix, Some(Fix::AddNavigationRegion));
        scene
            .actors
            .push(create(Preset::NavigationRegion2D, "Floor", [0., 0.]));
        assert!(of(&scene, 3).is_empty() && of(&scene, 4).is_empty());
        scene
            .actors
            .push(create(Preset::Skeleton2D, "Rig", [0., 0.]));
        assert!(of(&scene, 5)[0].text.contains("no bones"));
        scene
            .actors
            .push(create(Preset::PathFollow2D, "Cart", [0., 0.]));
        assert!(of(&scene, 6)[0].text.contains("no Path2D"));
        scene
            .actors
            .push(create(Preset::Sprite2D, "Tree", [0., 0.]));
        assert!(of(&scene, 7).is_empty());
    }

    #[test]
    fn depth_nodes_need_a_3d_camera_and_a_3d_anchor() {
        let mut scene = Scene {
            actors: vec![create(Preset::Depth2D, "Masks", [0., 0.])],
            ..Default::default()
        };
        assert!(of(&scene, 0)[0].text.contains("no 3D camera"));
        let mut camera = crate::scene::Actor::cube("Camera".into());
        camera.kind = "Camera".into();
        scene.actors.push(camera);
        assert!(of(&scene, 0).is_empty());
        // An anchor must be a 3D actor: a 2D node, or one gone from the map, is not.
        let tree = create(Preset::Sprite2D, "Tree", [0., 0.]);
        let tree_id = tree.id;
        scene.actors.push(tree);
        let masks = scene.actors[0]
            .components
            .iter_mut()
            .find(|c| c.class.class_id.as_deref() == Some(world2d::DEPTH2D))
            .unwrap();
        world2d::write(
            masks,
            &world2d::Depth2D {
                anchor: Some(tree_id),
                ..Default::default()
            },
        );
        assert!(of(&scene, 0)[0].text.contains("not a 3D actor"));
        scene
            .actors
            .push(create(Preset::SplitScreen2D, "Split", [0., 0.]));
        assert!(
            of(&scene, 0)
                .iter()
                .any(|w| w.text.contains("SplitScreen2D"))
        );
    }
}
