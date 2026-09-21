//! Lightweight animated third-person starter project.
//! Layout, controls and PSX rendering details: docs/third-person.md.
use crate::{
    assets, lighting, mesh,
    scene::{Actor, Scene},
    viewport::View,
};
use std::{path::Path, sync::Arc};
use uuid::Uuid;

const BLUE: [f32; 3] = [0.015, 0.36, 0.85];
const PLAYER_MESH: &str = "4af26725-03bf-5ebe-a790-9b37720296f0";
const IDLE_CLIP: &str = "e9a8d6a5-85b6-4a50-a556-4974152f82e5";

const LEVEL_ASSETS: &[(&str, &[u8])] = &[
    (
        "assets/Meshes/Optimized/Arena Floor Optimized.epokasset",
        include_bytes!(
            "../resources/templates/third-person/assets/Meshes/Optimized/Arena Floor Optimized.epokasset"
        ),
    ),
    (
        "assets/Meshes/Optimized/Arena Walls Optimized.epokasset",
        include_bytes!(
            "../resources/templates/third-person/assets/Meshes/Optimized/Arena Walls Optimized.epokasset"
        ),
    ),
    (
        "assets/Meshes/Optimized/Central Platform and Ramps Optimized.epokasset",
        include_bytes!(
            "../resources/templates/third-person/assets/Meshes/Optimized/Central Platform and Ramps Optimized.epokasset"
        ),
    ),
    (
        "assets/Meshes/Optimized/Long Platform Optimized.epokasset",
        include_bytes!(
            "../resources/templates/third-person/assets/Meshes/Optimized/Long Platform Optimized.epokasset"
        ),
    ),
    (
        "assets/Meshes/Optimized/Low Ramp Optimized.epokasset",
        include_bytes!(
            "../resources/templates/third-person/assets/Meshes/Optimized/Low Ramp Optimized.epokasset"
        ),
    ),
    (
        "assets/Meshes/Optimized/Round Platform Optimized.epokasset",
        include_bytes!(
            "../resources/templates/third-person/assets/Meshes/Optimized/Round Platform Optimized.epokasset"
        ),
    ),
    (
        "assets/Meshes/Optimized/Small Ramp Optimized.epokasset",
        include_bytes!(
            "../resources/templates/third-person/assets/Meshes/Optimized/Small Ramp Optimized.epokasset"
        ),
    ),
];

const PLAYER_ASSETS: &[(&str, &[u8])] = &[
    (
        "assets/Character/Player/Player.epokasset",
        include_bytes!(
            "../resources/templates/third-person/assets/Character/Player/Player.epokasset"
        ),
    ),
    (
        "assets/Character/Player/AquaAtlas.png",
        include_bytes!("../resources/templates/third-person/assets/Character/Player/AquaAtlas.png"),
    ),
    (
        "assets/Character/Player/AquaAtlas.epokasset",
        include_bytes!(
            "../resources/templates/third-person/assets/Character/Player/AquaAtlas.epokasset"
        ),
    ),
    (
        "assets/Character/Player/Material0.epokasset",
        include_bytes!(
            "../resources/templates/third-person/assets/Character/Player/Material0.epokasset"
        ),
    ),
    (
        "assets/Character/Player/Material1.epokasset",
        include_bytes!(
            "../resources/templates/third-person/assets/Character/Player/Material1.epokasset"
        ),
    ),
    (
        "assets/Character/Player/Material2.epokasset",
        include_bytes!(
            "../resources/templates/third-person/assets/Character/Player/Material2.epokasset"
        ),
    ),
    (
        "assets/Character/Player/Material3.epokasset",
        include_bytes!(
            "../resources/templates/third-person/assets/Character/Player/Material3.epokasset"
        ),
    ),
    (
        "assets/Character/Player/Animation/Skeleton.epokasset",
        include_bytes!(
            "../resources/templates/third-person/assets/Character/Player/Animation/Skeleton.epokasset"
        ),
    ),
    (
        "assets/Character/Player/Animation/Idle.epokasset",
        include_bytes!(
            "../resources/templates/third-person/assets/Character/Player/Animation/Idle.epokasset"
        ),
    ),
    (
        "assets/Character/Player/Animation/Walk.epokasset",
        include_bytes!(
            "../resources/templates/third-person/assets/Character/Player/Animation/Walk.epokasset"
        ),
    ),
    (
        "assets/Character/Player/Animation/Run.epokasset",
        include_bytes!(
            "../resources/templates/third-person/assets/Character/Player/Animation/Run.epokasset"
        ),
    ),
    (
        "assets/Character/Player/Animation/JumpUp.epokasset",
        include_bytes!(
            "../resources/templates/third-person/assets/Character/Player/Animation/JumpUp.epokasset"
        ),
    ),
    (
        "assets/Character/Player/Animation/JumpDown.epokasset",
        include_bytes!(
            "../resources/templates/third-person/assets/Character/Player/Animation/JumpDown.epokasset"
        ),
    ),
    (
        "assets/Character/Player/Animation/Land.epokasset",
        include_bytes!(
            "../resources/templates/third-person/assets/Character/Player/Animation/Land.epokasset"
        ),
    ),
];

fn empty(name: &str, parent: Option<usize>) -> Actor {
    let mut actor = Actor::cube(name.into());
    actor.kind = "Empty".into();
    actor.position = [0.; 3];
    actor.parent = parent;
    actor
}

fn install(root: &Path, path: &str, bytes: &[u8]) -> Result<(), String> {
    let destination = assets::inside(root, path)?;
    crate::project::write_changed(&destination, bytes)?;
    Ok(())
}

fn optimized_mesh(
    root: &Path,
    path: &str,
    bytes: &[u8],
    name: &str,
    parent: usize,
) -> Result<Actor, String> {
    install(root, path, bytes)?;
    let package = assets::Package::load(&assets::inside(root, path)?)?;
    if package.meta.kind != assets::Kind::EditableMesh {
        return Err(format!(
            "Third Person template mesh is not editable: {path}"
        ));
    }
    let document = mesh::Document::parse(&package.source)?;
    let mut actor = Actor::cube(name.into());
    actor.position = [0.; 3];
    actor.parent = Some(parent);
    actor.editable_mesh = Some(mesh::Component {
        document: Some(Arc::new(document)),
        ..mesh::Component::new(package.meta.id)
    });
    actor.lighting.receive = lighting::Receive::Baked;
    actor.lighting.static_geometry = true;
    Ok(actor)
}

fn collider(
    name: &str,
    center: [f32; 3],
    half_extents: [f32; 3],
    slope: Option<(f32, u8)>,
    parent: usize,
) -> Actor {
    let mut actor = empty(name, Some(parent));
    let (slope_rise, slope_axis) = slope.unwrap_or((0., 0));
    actor.position = center;
    actor.collider = Some(crate::collision::Collider {
        half_extents,
        slope_rise,
        slope_axis,
        ..Default::default()
    });
    actor
}

pub fn overview() -> View {
    let mut view = View {
        yaw: 0.,
        pitch: 0.58,
        zoom: 0.76,
        ..View::default()
    };
    let forward = view.basis()[2];
    view.center = std::array::from_fn(|i| [0., -3., 0.][i] - forward[i] * 29.);
    view
}

pub fn create(root: &Path) -> Result<Scene, String> {
    let mut scene = Scene {
        name: "Main".into(),
        actors: Vec::new(),
        ..Scene::default()
    };
    scene.environment.ambient = [0.53, 0.55, 0.59];
    scene.environment.point_lights = false;

    // Identity transforms make these empty actors editor-only hierarchy groups;
    // their children retain the authored world transforms.
    let world = scene.actors.len();
    scene.actors.push(empty("World", None));
    let environment = scene.actors.len();
    scene.actors.push(empty("Environment", Some(world)));
    let geometry = scene.actors.len();
    scene.actors.push(empty("Geometry", Some(environment)));

    let geometry_names = [
        "Arena Floor",
        "Arena Walls",
        "Central Platform and Ramps",
        "Long Platform",
        "Low Ramp",
        "Round Platform",
        "Small Ramp",
    ];
    for ((path, bytes), name) in LEVEL_ASSETS.iter().zip(geometry_names) {
        scene
            .actors
            .push(optimized_mesh(root, path, bytes, name, geometry)?);
    }

    let collision = scene.actors.len();
    scene.actors.push(empty("Collision", Some(environment)));
    for (name, center, half_extents, slope) in [
        ("Arena Floor", [0., -0.5, 0.], [16., 0.5, 16.], None),
        ("North Wall", [0., 2., 16.], [16.5, 2., 0.5], None),
        ("East Wall", [16., 2., 0.], [0.5, 2., 16.5], None),
        ("South Wall", [0., 2., -16.], [16.5, 2., 0.5], None),
        ("West Wall", [-16., 2., 0.], [0.5, 2., 16.5], None),
        ("Central Deck", [2.25, 1., 3.], [3.25, 1., 3.], None),
        ("Long Platform", [10., 1.25, 6.], [1.75, 1.25, 6.], None),
        ("Round Platform", [-10., 0.625, 9.], [2., 0.625, 2.], None),
        (
            "Small Ramp",
            [-10., 0.22, 1.5],
            [1., 0.22, 2.],
            Some((0.44, 2)),
        ),
        ("West Ramp", [-2.5, 1., 3.], [1.5, 1., 3.], Some((2., 2))),
        ("Rear Wedge", [1., 1., 7.], [2., 1., 1.], Some((2., 2))),
        (
            "Low Ramp",
            [9.5, 0.75, -9.],
            [2.5, 0.75, 2.5],
            Some((1.42, 2)),
        ),
    ] {
        scene
            .actors
            .push(collider(name, center, half_extents, slope, collision));
    }

    let props = scene.actors.len();
    scene.actors.push(empty("Props", Some(environment)));
    for (index, position, yaw) in [
        (1, [3., 0.65, -1.5], 0.),
        (2, [10., 0.65, -3.5], 18.),
        (3, [5.5, 0.65, -11.], -20.),
    ] {
        let mut cube = Actor::cube(format!("Blue Cube {index}"));
        cube.parent = Some(props);
        cube.position = position;
        cube.rotation[1] = yaw;
        cube.scale = [1.3; 3];
        cube.material.color = BLUE;
        cube.lighting.receive = lighting::Receive::Baked;
        cube.lighting.static_geometry = true;
        scene.actors.push(cube);
    }

    let lighting_group = scene.actors.len();
    scene.actors.push(empty("Lighting", Some(environment)));
    let mut sun = empty("Sun", Some(lighting_group));
    sun.rotation = [58., -32., 0.];
    sun.light = Some(lighting::Light {
        mode: lighting::LightMode::Mixed,
        intensity: 0.65,
        ..Default::default()
    });
    scene.actors.push(sun);

    for (path, bytes) in PLAYER_ASSETS {
        install(root, path, bytes)?;
    }
    let gameplay = scene.actors.len();
    scene.actors.push(empty("Gameplay", Some(world)));

    let mut camera = Actor::cube("Camera".into());
    camera.kind = "Camera".into();
    camera.parent = Some(gameplay);
    camera.position = [-7., 4.5, -13.5];
    camera.rotation = [20., 0., 0.];
    camera.camera_sky_color = [0.405, 0.624, 1.0];
    scene.actors.push(camera);

    let mut player = empty("Player", Some(gameplay));
    player.position = [-7., 0., -6.];
    player.collider = Some(crate::collision::Collider {
        center: [0., 0.92, 0.],
        half_extents: [0.28, 0.92, 0.28],
        layer: 4,
        ..Default::default()
    });
    player
        .components
        .push(crate::actor_document::ComponentInstance::new(
            Uuid::new_v4(),
            crate::actor_document::ClassReference::new(
                "ThirdPersonController",
                "9233f481-d27e-4765-a2d3-4dcfcb0cc910",
            ),
            "Third Person Controller",
        ));
    let player_index = scene.actors.len();
    scene.actors.push(player);

    let mut visual = Actor::cube("Visual".into());
    visual.parent = Some(player_index);
    visual.position = [0., 0.0078, 0.];
    visual.rotation = [0., 180., 0.];
    visual.scale = [0.6; 3];
    visual.material.unlit = true;
    visual.skeletal_mesh = Some(crate::skeletal::Component {
        clip: Some(Uuid::parse_str(IDLE_CLIP).map_err(|e| e.to_string())?),
        ..crate::skeletal::Component::new(Uuid::parse_str(PLAYER_MESH).map_err(|e| e.to_string())?)
    });
    scene.actors.push(visual);

    scene.sync_actor_components();
    scene.bake = Some(lighting::bake(&scene)?);
    scene.validate()?;

    std::fs::create_dir_all(root.join("UserSettings")).map_err(|e| e.to_string())?;
    crate::project::write_changed(
        &root.join("UserSettings/SceneView.epokprefs"),
        &crate::document::to_vec(&overview()).map_err(|e| e.to_string())?,
    )?;
    crate::project::write_changed(
        &root.join("README.md"),
        include_bytes!("../resources/templates/third-person/README.md"),
    )?;
    for (name, source) in [
        (
            "assets/scripts/ThirdPersonController.hpp",
            include_str!("../templates/ThirdPersonController.hpp"),
        ),
        (
            "assets/scripts/ThirdPersonController.cpp",
            include_str!("../templates/ThirdPersonController.cpp"),
        ),
        (
            "assets/scripts/CharacterMotion.hpp",
            include_str!("../templates/CharacterMotion.hpp"),
        ),
        (
            "assets/scripts/CharacterClips.hpp",
            include_str!("../templates/CharacterClips.hpp"),
        ),
    ] {
        let path = assets::inside(root, name)?;
        crate::project::write_changed(&path, source.as_bytes())?;
    }
    Ok(scene)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template_is_animated_grouped_and_self_contained() {
        let parent = std::env::temp_dir().join(format!("epok-third-person-{}", Uuid::new_v4()));
        let root = parent.join("Third Person Game");
        let project = crate::workspace::create(
            &root,
            "Third Person Game",
            crate::workspace::Template::ThirdPerson,
        )
        .unwrap();
        let rendering = crate::settings::rendering(&root).unwrap();
        assert_eq!((rendering.width, rendering.height), (320, 240));
        assert!(rendering.motion_interpolation);

        let scene = Scene::load(&crate::workspace::startup_scene(&root).unwrap()).unwrap();
        assert!(lighting::valid_bake(&scene));
        assert_eq!(
            scene
                .actors
                .iter()
                .filter(|actor| actor.editable_mesh.is_some())
                .count(),
            7
        );
        assert_eq!(
            scene
                .actors
                .iter()
                .filter(|actor| actor.skeletal_mesh.is_some())
                .count(),
            1
        );
        assert!(!scene.actors.iter().any(|actor| actor.audio.is_some()));
        assert!(
            !scene
                .actors
                .iter()
                .any(|actor| actor.name.contains("NPC") || actor.name.contains("Target"))
        );
        let player = scene
            .actors
            .iter()
            .position(|actor| actor.name == "Player")
            .unwrap();
        let visual = scene
            .actors
            .iter()
            .find(|actor| actor.name == "Visual")
            .unwrap();
        assert_eq!(visual.parent, Some(player));
        for group in [
            "World",
            "Environment",
            "Geometry",
            "Collision",
            "Props",
            "Lighting",
            "Gameplay",
        ] {
            assert!(
                scene
                    .actors
                    .iter()
                    .any(|actor| actor.name == group && actor.kind == "Empty")
            );
        }
        for cube in scene
            .actors
            .iter()
            .filter(|actor| actor.name.starts_with("Blue Cube"))
        {
            assert!(
                cube.components
                    .iter()
                    .all(|component| component.class.name.starts_with("epok::"))
            );
            assert!(cube.lighting.static_geometry);
            assert_eq!(cube.lighting.receive, lighting::Receive::Baked);
        }

        let index = assets::scan(&root, &mut Default::default());
        assert!(
            index.problems.is_empty(),
            "Template asset problems: {:?}",
            index.problems
        );
        assert!(index.resolve(Uuid::parse_str(PLAYER_MESH).unwrap()).is_ok());
        for forbidden in ["AttackA", "AttackB", "AttackC"] {
            assert!(
                !root
                    .join(format!(
                        "assets/Character/Player/Animation/{forbidden}.epokasset"
                    ))
                    .exists()
            );
        }

        let triangles: usize = scene
            .actors
            .iter()
            .map(|actor| lighting::quad_count(actor) * 2)
            .sum();
        assert!(
            triangles < 7000,
            "Template exceeds the PSX geometry limit: {triangles}"
        );
        let catalog = crate::scripts::catalog(&root).unwrap();
        let header = crate::project::scene_header(&scene, &catalog).unwrap();
        assert!(header.contains("ThirdPersonController"));
        assert!(root.join("UserSettings/SceneView.epokprefs").is_file());

        let editor = crate::editor::Editor::open(project).unwrap();
        assert_eq!(editor.view.center, overview().center);
        let visual = editor
            .scene
            .actors
            .iter()
            .find(|actor| actor.name == "Visual")
            .unwrap();
        let model = visual
            .skeletal_mesh
            .as_ref()
            .unwrap()
            .model
            .as_ref()
            .unwrap();
        assert_eq!(model.clips.len(), 6);
        drop(editor);
        std::fs::remove_dir_all(parent).unwrap();
    }
}
