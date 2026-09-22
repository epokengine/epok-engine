//! Lightweight animated third-person starter project.
//! Layout, controls and PSX rendering details: docs/third-person.md.
use crate::{
    assets, lighting, mesh,
    scene::{Actor, Scene},
    viewport::View,
    workspace::GameplayFlavor,
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

/// Every generated flavor binds a class with this name to the Player actor; the
/// identity differs because a C++ class, a Blueprint class and a Lua class are
/// three different classes, not three spellings of one.
pub const CONTROLLER_NAME: &str = "ThirdPersonController";
pub const CPP_CONTROLLER_ID: &str = "9233f481-d27e-4765-a2d3-4dcfcb0cc910";
pub const LUA_CONTROLLER_ID: &str = "5d5b3e5e-604d-4b2a-bd1d-313ce3bd2598";
pub const LUA_CONTROLLER_PATH: &str = "assets/scripts/ThirdPersonController.lua";

/// The six locomotion clips, in the order the controller indexes them. The C++
/// flavor looks them up by name at begin_play; the Blueprint and Lua flavors are
/// handed the resolved indices as typed properties, so neither depends on a
/// display name surviving a rename.
const CLIP_NAMES: [&str; 6] = [
    "CharacterRig|Sequence_00_Wait",
    "CharacterRig|Sequence_01_Sequence",
    "CharacterRig|Sequence_02_Run",
    "CharacterRig|Sequence_24_Jump_Up",
    "CharacterRig|Sequence_25_Jump_Down",
    "CharacterRig|Sequence_26_Land",
];
const CLIP_PROPERTIES: [&str; 6] = [
    "idle_clip",
    "walk_clip",
    "run_clip",
    "jump_up_clip",
    "jump_down_clip",
    "land_clip",
];

/// Where the shared scene keeps the actors a flavor has to bind to.
struct Gameplay {
    player: usize,
    camera: usize,
    visual: usize,
}

pub fn create(root: &Path, flavor: GameplayFlavor) -> Result<Scene, String> {
    install_shared_assets(root)?;
    let (mut scene, gameplay) = build_shared_scene(root)?;
    install_gameplay(root, &mut scene, &gameplay, flavor)?;
    finalize(root, &mut scene, flavor)
}

fn install_shared_assets(root: &Path) -> Result<(), String> {
    for (path, bytes) in PLAYER_ASSETS {
        install(root, path, bytes)?;
    }
    Ok(())
}

/// The level, the player and the camera. Identical for every flavor: one copy of
/// the assets, one hierarchy, one set of transforms and colliders.
fn build_shared_scene(root: &Path) -> Result<(Scene, Gameplay), String> {
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

    let gameplay = scene.actors.len();
    scene.actors.push(empty("Gameplay", Some(world)));

    let mut camera = Actor::cube("Camera".into());
    camera.kind = "Camera".into();
    camera.parent = Some(gameplay);
    camera.position = [-7., 4.5, -13.5];
    camera.rotation = [20., 0., 0.];
    camera.camera_sky_color = [0.405, 0.624, 1.0];
    let camera_index = scene.actors.len();
    scene.actors.push(camera);

    let mut player = empty("Player", Some(gameplay));
    player.position = [-7., 0., -6.];
    player.collider = Some(crate::collision::Collider {
        center: [0., 0.92, 0.],
        half_extents: [0.28, 0.92, 0.28],
        layer: 4,
        ..Default::default()
    });
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
    let visual_index = scene.actors.len();
    scene.actors.push(visual);

    scene.sync_actor_components();
    Ok((
        scene,
        Gameplay {
            player: player_index,
            camera: camera_index,
            visual: visual_index,
        },
    ))
}

/// The only part of the template a flavor owns: the controller source and the
/// class the Player actor binds. The scene above is untouched.
fn install_gameplay(
    root: &Path,
    scene: &mut Scene,
    gameplay: &Gameplay,
    flavor: GameplayFlavor,
) -> Result<(), String> {
    let class_id = match flavor {
        GameplayFlavor::Cpp => CPP_CONTROLLER_ID,
        GameplayFlavor::Blueprint => crate::third_person_blueprint::CLASS_ID,
        GameplayFlavor::Lua => LUA_CONTROLLER_ID,
    };
    let mut controller = crate::actor_document::ComponentInstance::new(
        Uuid::new_v4(),
        crate::actor_document::ClassReference::new(CONTROLLER_NAME, class_id),
        "Third Person Controller",
    );
    if flavor != GameplayFlavor::Cpp {
        // Typed references, resolved once here. Neither generated controller
        // looks an actor up by display name, so renaming one is safe. The Lua
        // flavor binds the camera's transform rather than the camera, because
        // writing a transform component takes three scalars and a Lua class
        // that never holds a whole vector compiles in every execution mode.
        let camera = match flavor {
            GameplayFlavor::Lua => {
                scene.actors[gameplay.camera]
                    .root()
                    .ok_or("Third Person template Camera actor has no transform")?
                    .id
            }
            _ => scene.actors[gameplay.camera].id,
        };
        let visual = scene.actors[gameplay.visual]
            .components
            .iter()
            .find(|c| c.class.class_id.as_deref() == Some(crate::object_model::MESH3D_COMPONENT_ID))
            .ok_or("Third Person template Visual actor has no mesh component")?
            .id;
        controller.properties.insert(
            "camera".into(),
            serde_json::Value::String(camera.to_string()),
        );
        controller.properties.insert(
            "visual".into(),
            serde_json::Value::String(visual.to_string()),
        );
        for (name, slot) in CLIP_PROPERTIES.iter().zip(clip_slots(root)?) {
            controller
                .properties
                .insert((*name).into(), serde_json::Value::from(slot));
        }
        controller
            .overrides
            .extend(controller.properties.keys().cloned());
    }
    scene.actors[gameplay.player].components.push(controller);

    match flavor {
        GameplayFlavor::Cpp => {
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
                crate::project::write_changed(&assets::inside(root, name)?, source.as_bytes())?;
            }
        }
        GameplayFlavor::Lua => {
            crate::project::write_changed(
                &assets::inside(root, LUA_CONTROLLER_PATH)?,
                include_bytes!("../templates/ThirdPersonController.lua"),
            )?;
            let mut registry = crate::lua_identity::Registry::default();
            registry
                .classes
                .insert(LUA_CONTROLLER_PATH.into(), LUA_CONTROLLER_ID.into());
            registry.save(root)?;
        }
        GameplayFlavor::Blueprint => {
            let asset = assets::inside(root, crate::third_person_blueprint::ASSET_PATH)?;
            if let Some(directory) = asset.parent() {
                std::fs::create_dir_all(directory).map_err(|e| e.to_string())?;
            }
            crate::blueprint_asset::create(&asset, &crate::third_person_blueprint::asset())?;
        }
    }
    Ok(())
}

/// The clip index of each locomotion name, in the order the bundled model
/// stores them. Reading the installed packages keeps this honest: a reordered
/// or renamed clip fails here instead of silently animating the wrong state.
fn clip_slots(root: &Path) -> Result<[u32; 6], String> {
    let package = assets::Package::load(&assets::inside(
        root,
        "assets/Character/Player/Player.epokasset",
    )?)?;
    let crate::skeletal::Data::SkeletalMesh(mesh) = crate::skeletal::Data::parse(&package.source)?
    else {
        return Err("Third Person template player asset is not a skeletal mesh".into());
    };
    let mut names = std::collections::BTreeMap::new();
    for (path, _) in PLAYER_ASSETS {
        if !path.contains("/Animation/") || path.ends_with("Skeleton.epokasset") {
            continue;
        }
        let package = assets::Package::load(&assets::inside(root, path)?)?;
        if let crate::skeletal::Data::AnimationClip(clip) =
            crate::skeletal::Data::parse(&package.source)?
        {
            names.insert(package.meta.id, clip.name);
        }
    }
    let mut slots = [0u32; 6];
    for (slot, wanted) in CLIP_NAMES.iter().enumerate() {
        let index = mesh
            .clips
            .iter()
            .position(|id| names.get(id).is_some_and(|name| name == wanted))
            .ok_or_else(|| format!("Third Person template is missing the {wanted} clip"))?;
        slots[slot] = index as u32;
    }
    Ok(slots)
}

fn finalize(root: &Path, scene: &mut Scene, flavor: GameplayFlavor) -> Result<Scene, String> {
    scene.sync_actor_components();
    scene.bake = Some(lighting::bake(scene)?);
    scene.validate()?;

    std::fs::create_dir_all(root.join("UserSettings")).map_err(|e| e.to_string())?;
    crate::project::write_changed(
        &root.join("UserSettings/SceneView.epokprefs"),
        &crate::document::to_vec(&overview()).map_err(|e| e.to_string())?,
    )?;
    crate::project::write_changed(&root.join("README.md"), readme(flavor).as_bytes())?;
    Ok(scene.clone())
}

/// The generated README names the source that was actually written, never the
/// files of a flavor this project does not have.
fn readme(flavor: GameplayFlavor) -> String {
    let source = match flavor {
        GameplayFlavor::Cpp => concat!(
            "The controller is project-owned C++ in `assets/scripts/ThirdPersonController.hpp`\n",
            "and `assets/scripts/ThirdPersonController.cpp`, with\n",
            "`assets/scripts/CharacterMotion.hpp` for the locomotion state machine and\n",
            "`assets/scripts/CharacterClips.hpp` for the clip names."
        ),
        GameplayFlavor::Blueprint => concat!(
            "The controller is a project-owned visual graph in\n",
            "`assets/Blueprints/ThirdPersonController.epokbp`. Open it from the Project browser\n",
            "to edit movement, the camera boom or the animation states. The build backend turns\n",
            "the graph into native console code; the source you edit stays visual."
        ),
        GameplayFlavor::Lua => concat!(
            "The controller is project-owned Lua in `assets/scripts/ThirdPersonController.lua`.\n",
            "It is compiled with this project's Lua execution setting, which starts on the\n",
            "ahead-of-time native mode so the console build stays small; Project Settings can\n",
            "switch it to the virtual machine at any time."
        ),
    };
    include_str!("../resources/templates/third-person/README.md").replace("<!--GAMEPLAY-->", source)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn disposable(flavor: GameplayFlavor) -> (std::path::PathBuf, std::path::PathBuf) {
        let parent = std::env::temp_dir().join(format!("epok-third-person-{}", Uuid::new_v4()));
        let root = parent.join("Third Person Game");
        crate::workspace::create_with_options(
            &root,
            "Third Person Game",
            crate::workspace::CreateOptions::new(crate::workspace::Template::ThirdPerson)
                .with_gameplay(flavor),
        )
        .unwrap_or_else(|e| panic!("{flavor:?} project creation: {e}"));
        (parent, root)
    }

    /// Every flavor writes its own gameplay and nothing else: a Lua project has
    /// no generated C++ controller to fall back on, and neither does a
    /// Blueprint one. Silence here would be the worst outcome of all.
    #[test]
    fn each_flavor_generates_only_its_own_gameplay_source() {
        let cpp = [
            "assets/scripts/ThirdPersonController.hpp",
            "assets/scripts/ThirdPersonController.cpp",
            "assets/scripts/CharacterMotion.hpp",
            "assets/scripts/CharacterClips.hpp",
        ];
        let lua = [LUA_CONTROLLER_PATH];
        let blueprint = [crate::third_person_blueprint::ASSET_PATH];
        for (flavor, mine) in [
            (GameplayFlavor::Cpp, &cpp[..]),
            (GameplayFlavor::Lua, &lua[..]),
            (GameplayFlavor::Blueprint, &blueprint[..]),
        ] {
            let (parent, root) = disposable(flavor);
            for path in cpp.iter().chain(&lua).chain(&blueprint) {
                assert_eq!(
                    root.join(path).is_file(),
                    mine.contains(path),
                    "{flavor:?} generated {path}"
                );
            }
            let readme = std::fs::read_to_string(root.join("README.md")).unwrap();
            assert!(!readme.contains("<!--GAMEPLAY-->"));
            for path in cpp.iter().chain(&lua).chain(&blueprint) {
                let named = readme.contains(path.rsplit('/').next().unwrap());
                assert_eq!(named, mine.contains(path), "{flavor:?} README names {path}");
            }
            // Build output is never template content, and the caches the editor
            // does create on open stay out of version control.
            for excluded in ["exports", "artifacts"] {
                assert!(
                    !root.join(excluded).exists(),
                    "{flavor:?} bundled {excluded}"
                );
            }
            let ignore = std::fs::read_to_string(root.join(".gitignore")).unwrap();
            for excluded in ["/.epok/", "/UserSettings/", "/exports/", "/artifacts/"] {
                assert!(
                    ignore.contains(excluded),
                    "{flavor:?} .gitignore misses {excluded}"
                );
            }
            std::fs::remove_dir_all(parent).unwrap();
        }
    }

    /// The scene, the assets and the transforms are the template; the flavor is
    /// only the class the Player binds. Anything else drifting between them
    /// would make the three impossible to compare.
    #[test]
    fn every_flavor_shares_one_scene() {
        let mut shape = None;
        for flavor in [
            GameplayFlavor::Cpp,
            GameplayFlavor::Lua,
            GameplayFlavor::Blueprint,
        ] {
            let (parent, root) = disposable(flavor);
            let scene = Scene::load(&crate::workspace::startup_scene(&root).unwrap()).unwrap();
            let described = scene
                .actors
                .iter()
                .map(|actor| {
                    (
                        actor.name.clone(),
                        actor.kind.clone(),
                        actor.parent,
                        actor.position,
                        actor.rotation,
                        actor.scale,
                        format!("{:?}", actor.collider),
                        format!("{:?}", actor.editable_mesh.as_ref().map(|m| m.asset)),
                        format!(
                            "{:?}",
                            actor.skeletal_mesh.as_ref().map(|m| (m.asset, m.clip))
                        ),
                    )
                })
                .collect::<Vec<_>>();
            let files = walk(&root.join("assets/Character"));
            match &shape {
                None => shape = Some((described, files)),
                Some(first) => {
                    assert_eq!(first.0, described, "{flavor:?} changed the shared scene");
                    assert_eq!(first.1, files, "{flavor:?} changed the shared assets");
                }
            }
            let player = scene
                .actors
                .iter()
                .find(|actor| actor.name == "Player")
                .unwrap();
            let controller = player
                .components
                .iter()
                .find(|c| c.class.name == CONTROLLER_NAME)
                .unwrap_or_else(|| panic!("{flavor:?} Player has no controller"));
            let expected = match flavor {
                GameplayFlavor::Cpp => CPP_CONTROLLER_ID,
                GameplayFlavor::Lua => LUA_CONTROLLER_ID,
                GameplayFlavor::Blueprint => crate::third_person_blueprint::CLASS_ID,
            };
            assert_eq!(controller.class.class_id.as_deref(), Some(expected));
            if flavor != GameplayFlavor::Cpp {
                // Typed references, not a display-name lookup at begin_play.
                let camera = scene
                    .actors
                    .iter()
                    .find(|actor| actor.name == "Camera")
                    .unwrap();
                let bound = match flavor {
                    GameplayFlavor::Lua => camera.root().unwrap().id,
                    _ => camera.id,
                };
                assert_eq!(
                    controller.properties.get("camera").and_then(|v| v.as_str()),
                    Some(bound.to_string().as_str())
                );
                let visual = scene
                    .actors
                    .iter()
                    .find(|actor| actor.name == "Visual")
                    .unwrap();
                let mesh = visual
                    .components
                    .iter()
                    .find(|c| {
                        c.class.class_id.as_deref()
                            == Some(crate::object_model::MESH3D_COMPONENT_ID)
                    })
                    .unwrap();
                assert_eq!(
                    controller.properties.get("visual").and_then(|v| v.as_str()),
                    Some(mesh.id.to_string().as_str())
                );
                let mut slots: Vec<u64> = CLIP_PROPERTIES
                    .iter()
                    .map(|name| controller.properties[*name].as_u64().unwrap())
                    .collect();
                slots.sort_unstable();
                assert_eq!(slots, vec![0, 1, 2, 3, 4, 5], "{flavor:?} clip indices");
            }
            std::fs::remove_dir_all(parent).unwrap();
        }
    }

    /// The compiler is the only honest answer to "does this flavor work?": it
    /// runs the real Lua frontend and the real Blueprint backend against the
    /// generated sources and resolves the class the scene binds.
    #[test]
    fn every_flavor_compiles_and_resolves_its_controller() {
        for flavor in [
            GameplayFlavor::Cpp,
            GameplayFlavor::Lua,
            GameplayFlavor::Blueprint,
        ] {
            let (parent, root) = disposable(flavor);
            let index = assets::scan(&root, &mut Default::default());
            assert!(
                index.problems.is_empty(),
                "{flavor:?} asset problems: {:?}",
                index.problems
            );
            let catalog = crate::scripts::catalog(&root)
                .unwrap_or_else(|e| panic!("{flavor:?} script catalog: {e}"));
            let controller = catalog
                .iter()
                .flat_map(|script| script.classes.iter())
                .find(|class| class.cpp_name == CONTROLLER_NAME)
                .unwrap_or_else(|| panic!("{flavor:?} catalog has no {CONTROLLER_NAME}"));
            let expected = match flavor {
                GameplayFlavor::Cpp => CPP_CONTROLLER_ID,
                GameplayFlavor::Lua => LUA_CONTROLLER_ID,
                GameplayFlavor::Blueprint => crate::third_person_blueprint::CLASS_ID,
            };
            assert_eq!(controller.id, expected);
            let scene = Scene::load(&crate::workspace::startup_scene(&root).unwrap()).unwrap();
            let header = crate::project::scene_header(&scene, &catalog).unwrap();
            assert!(header.contains(CONTROLLER_NAME), "{flavor:?} scene header");
            std::fs::remove_dir_all(parent).unwrap();
        }
    }

    /// Every numeric literal in a source, so the three flavors can be compared
    /// on the values they actually contain rather than on a claim in a comment.
    fn literals(source: &str) -> std::collections::BTreeSet<String> {
        let bytes: Vec<char> = source.chars().collect();
        let mut out = std::collections::BTreeSet::new();
        let mut index = 0;
        while index < bytes.len() {
            if !bytes[index].is_ascii_digit() {
                index += 1;
                continue;
            }
            let start = index;
            while index < bytes.len() && (bytes[index].is_ascii_digit() || bytes[index] == '.') {
                index += 1;
            }
            let run: String = bytes[start..index].iter().collect();
            // A decimal point is what makes a tuning value; indices, ports and
            // button numbers are integers and are compared by behaviour, not here.
            if !run.contains('.') || run.ends_with('.') {
                continue;
            }
            let signed = start > 0
                && bytes[start - 1] == '-'
                && (start < 2 || !bytes[start - 2].is_alphanumeric());
            let Ok(value) = run.parse::<f64>() else {
                continue;
            };
            out.insert(format!("{:.6}", if signed { -value } else { value }));
        }
        out
    }

    /// The flavors are only comparable if they are built from the same values
    /// and the same engine operations. Both are checked on the sources that are
    /// actually generated, so a change to one of the three has to be made to
    /// all three or this fails.
    #[test]
    fn the_three_flavors_share_their_tuning_and_their_operations() {
        let cpp = include_str!("../templates/ThirdPersonController.cpp");
        let lua = include_str!("../templates/ThirdPersonController.lua");
        let blueprint = String::from_utf8(
            crate::document::to_vec(&crate::third_person_blueprint::asset()).unwrap(),
        )
        .unwrap();
        let tuning = [
            6.0, 2.6, 540.0, 140.0, 70.0, -10.0, 55.0, 5.4, 1.1, 24.0, 9.0, -0.05, 0.6, 12.0, 3.0,
            0.2, 1.6, 60.0, 0.04, 0.12, 0.001,
        ];
        for (flavor, source) in [("C++", cpp), ("Lua", lua), ("Blueprint", &blueprint[..])] {
            let found = literals(source);
            for value in tuning {
                assert!(
                    found.contains(&format!("{value:.6}")),
                    "the {flavor} controller is missing the tuning value {value}"
                );
            }
        }

        // Lua and Blueprint reach the engine through the same reflected
        // operations. The Blueprint asset names them by UUID and the Lua source
        // by their script spelling, so the pairs are checked together.
        for (uuid, spelling) in [
            (
                "eeee7d31-a8c6-437a-95db-10e163771b24",
                "epok.math.sine_degrees",
            ),
            (
                "565552a8-869a-4ca7-a1d8-a7ad6e1508a7",
                "epok.math.cosine_degrees",
            ),
            ("57b625c6-06d7-4635-b551-17a75ceffa3b", "epok.math.length2"),
            (
                "9b882fef-4a7b-4463-81aa-65b2f7e0dcd0",
                "epok.math.wrap_degrees",
            ),
            (
                "bacfcfc1-e38c-44d2-b47d-19ec3ae37f8d",
                "epok.math.move_toward_degrees",
            ),
            (
                "1057a007-0420-4b95-8f48-140be81f9ba5",
                "epok.math.heading_degrees",
            ),
            (
                "5b0a28df-53d9-40e4-b7ca-c5381f473771",
                "epok.math.stick_intent",
            ),
            ("6b6ba8bd-8446-4fef-a147-e41b61f88a61", "epok.math.clamp"),
            ("4fe0f030-091c-4d55-984d-769077622c33", "epok.math.vector3"),
            (
                "da298163-8f45-4b47-81c9-e2e70c89a566",
                "epok.collision.move",
            ),
            (
                "acebf3f3-9592-481f-820e-99e44b834f5d",
                "epok.collision.raycast_segment",
            ),
            (
                "e36ec48c-b2fe-49a2-8935-894d346882f7",
                "epok.scene.set_camera",
            ),
            ("047cbafb-5024-44d8-92a3-4939b31de81d", ":play_clip("),
            ("74c0cd75-108d-486c-bccb-68b0a85b1a9a", ":pause_animation("),
            ("3ec24d35-4613-4ffb-b50a-0ea205e901b1", ":playback_state("),
            ("3ea752b4-6bf6-4e45-831c-b05524ecdc46", ":clip_frames("),
            ("857927dd-9229-44d3-8556-b392f18f3567", ":clip_loop_ticks("),
            (
                "b0b30f1a-a82b-4806-bb2a-eb7392bb6607",
                ":set_animation_position(",
            ),
        ] {
            assert!(lua.contains(spelling), "the Lua controller lost {spelling}");
            assert!(
                blueprint.contains(uuid),
                "the Blueprint controller lost {spelling} ({uuid})"
            );
        }

        // The C++ oracle uses the same math rather than a private copy of it,
        // which is what makes the comparison meaningful at all.
        for operation in [
            "MathLibrary::sine_degrees",
            "MathLibrary::cosine_degrees",
            "MathLibrary::length2",
            "MathLibrary::wrap_degrees",
            "MathLibrary::move_toward_degrees",
            "MathLibrary::heading_degrees",
            "MathLibrary::stick_intent",
            "MathLibrary::clamp",
        ] {
            assert!(
                cpp.contains(operation),
                "the C++ controller lost {operation}"
            );
        }
        // No flavor reaches another's implementation.
        assert!(!lua.contains("ThirdPersonController.hpp"));
        assert!(!blueprint.contains("CharacterMotion"));
    }

    fn walk(root: &Path) -> Vec<String> {
        let mut out = vec![];
        let mut pending = vec![root.to_path_buf()];
        while let Some(directory) = pending.pop() {
            let Ok(entries) = std::fs::read_dir(&directory) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    pending.push(path);
                } else {
                    out.push(
                        path.strip_prefix(root)
                            .unwrap()
                            .to_string_lossy()
                            .replace('\\', "/"),
                    );
                }
            }
        }
        out.sort();
        out
    }

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
