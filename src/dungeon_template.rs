//! The Dungeon starter: the Third Person character and camera in a generated dungeon.
//!
//! The Third Person template provides the character, its controller in the chosen
//! flavor, the camera and the lighting setup; its arena is replaced by a Dungeon
//! Generator following the built-in recipe and style, and the key and door are
//! installed in the same flavor as the controller.
use crate::{
    dungeon::{Generator, Source},
    scene::{Actor, Scene},
    workspace::GameplayFlavor,
};
use std::path::Path;

/// A seed whose built-in recipe gives a varied first dungeon within the budget.
pub const SEED: u32 = 1207;

pub fn create(root: &Path, flavor: GameplayFlavor) -> Result<Scene, String> {
    let mut scene = crate::third_person::create(root, flavor)?;
    scene.sync_actor_components();
    // The arena, its props and its sun give way to the dungeon, which brings its own
    // light. Inactive actors would still be cooked, so they are deleted.
    let environment = scene
        .actors
        .iter()
        .find(|a| a.name == "Environment")
        .map(|a| a.id)
        .ok_or("The Third Person scene has no Environment to replace")?;
    scene.delete_actor_branch(environment);
    let world = scene
        .actors
        .iter()
        .find(|a| a.name == "World")
        .map(|a| a.id);
    let player = scene
        .actors
        .iter()
        .find(|a| a.name == "Player")
        .map(|a| a.id)
        .ok_or("The Third Person scene has no Player")?;

    let mut root_actor = Actor::cube("Dungeon".into());
    root_actor.kind = "Empty".into();
    root_actor.position = [0.; 3];
    root_actor.logical_parent = world;
    let mut generator = Generator::new(root_actor.id);
    generator.seed = SEED;
    generator.source = Source::Recipe;
    generator.spawn_actor = Some(player);
    let id = generator.id;
    scene.actors.push(root_actor);
    scene.dungeons.push(generator);
    scene.refresh_actor_hierarchy();

    crate::dungeon_gameplay::install(root, flavor)?;
    crate::dungeon_bake::generate(root, &mut scene, id, None)?;
    scene.sync_actor_components();
    scene.bake = Some(crate::lighting::bake(&scene)?);
    scene.validate()?;

    crate::project::write_changed(
        &root.join("UserSettings/SceneView.epokprefs"),
        &crate::document::to_vec(&overview()).map_err(|e| e.to_string())?,
    )?;
    crate::project::write_changed(&root.join("README.md"), readme(flavor).as_bytes())?;
    Ok(scene)
}

/// The Scene view the project opens with: the whole dungeon from above, at an angle.
fn overview() -> crate::viewport::View {
    let mut view = crate::viewport::View {
        yaw: 0.,
        pitch: 0.9,
        zoom: 0.76,
        ..Default::default()
    };
    let forward = view.basis()[2];
    view.center = std::array::from_fn(|i| -forward[i] * 150.);
    view
}

/// The README names the sources this flavor actually wrote.
fn readme(flavor: GameplayFlavor) -> String {
    let source = match flavor {
        GameplayFlavor::Cpp => concat!(
            "The character controller is project-owned C++ in\n",
            "`assets/scripts/ThirdPersonController.hpp` and `.cpp`. The key and the door are\n",
            "`assets/scripts/DungeonKey.hpp` and `assets/scripts/DungeonDoor.hpp`: a key\n",
            "deactivates when touched, a door when the key it names is gone."
        ),
        GameplayFlavor::Blueprint => concat!(
            "The character controller, the key and the door are project-owned visual graphs in\n",
            "`assets/Blueprints/`: `ThirdPersonController`, `DungeonKey` (deactivates when\n",
            "touched) and `DungeonDoor` (deactivates when the key it names is gone). The build\n",
            "backend turns the graphs into native console code."
        ),
        GameplayFlavor::Lua => concat!(
            "The character controller, the key and the door are project-owned Lua in\n",
            "`assets/scripts/`: `ThirdPersonController.lua`, `DungeonKey.lua` (deactivates when\n",
            "touched) and `DungeonDoor.lua` (deactivates when the key it names is gone). They\n",
            "are compiled with this project's Lua execution setting."
        ),
    };
    include_str!("../resources/templates/dungeon/README.md").replace("<!--GAMEPLAY-->", source)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace::{CreateOptions, Template};

    /// Every flavor creates a dungeon the character starts in, with a working key and
    /// door of that flavor, inside the console budget, and whose classes compile.
    #[test]
    fn every_flavor_creates_a_playable_dungeon() {
        for flavor in GameplayFlavor::ALL {
            let parent = std::env::temp_dir()
                .join(format!("epok-dungeon-template-{}", uuid::Uuid::new_v4()));
            let root = parent.join("Dungeon Game");
            crate::workspace::create_with_options(
                &root,
                "Dungeon Game",
                CreateOptions::new(Template::Dungeon).with_gameplay(flavor),
            )
            .unwrap_or_else(|e| panic!("{flavor:?} project creation: {e}"));
            let scene = Scene::load(&crate::workspace::startup_scene(&root).unwrap()).unwrap();
            assert!(!scene.actors.iter().any(|a| a.name == "Environment"));
            let g = &scene.dungeons[0];
            assert!(!g.output.is_empty() && !g.stale(), "{flavor:?}");
            // The character stands in the spawn room.
            let player = scene.actors.iter().find(|a| a.name == "Player").unwrap();
            let spawn = scene.actors.iter().find(|a| a.name == "Spawn").unwrap();
            for k in 0..3 {
                assert!(
                    (player.position[k] - spawn.position[k]).abs() < 1e-3,
                    "{flavor:?}: player at {:?}, spawn at {:?}",
                    player.position,
                    spawn.position
                );
            }
            // The key and the door are this flavor's, and the door names the key.
            let classes = crate::dungeon_gameplay::classes(flavor);
            let with = |class: &str| {
                scene
                    .actors
                    .iter()
                    .filter(|a| {
                        a.components
                            .iter()
                            .any(|c| c.class.class_id.as_deref() == Some(class))
                    })
                    .collect::<Vec<_>>()
            };
            let (keys, doors) = (with(classes.key.1), with(classes.door.1));
            assert_eq!(keys.len(), 1, "{flavor:?}");
            // Only the character picks the key up.
            let named = keys[0]
                .components
                .iter()
                .find_map(|c| c.properties.get(crate::dungeon_gameplay::PLAYER))
                .and_then(|v| v.as_str());
            assert_eq!(named, Some(player.id.to_string().as_str()), "{flavor:?}");
            assert!(!doors.is_empty(), "{flavor:?}");
            for door in doors {
                let named = door
                    .components
                    .iter()
                    .find_map(|c| c.properties.get(crate::dungeon_gameplay::KEY_REFERENCE))
                    .and_then(|v| v.as_str());
                assert_eq!(named, Some(keys[0].id.to_string().as_str()));
            }
            // Within the console budget, lit and ready.
            assert!(
                scene.actors.len() <= 64,
                "{flavor:?}: {} actors",
                scene.actors.len()
            );
            let quads: usize = scene.actors.iter().map(crate::lighting::quad_count).sum();
            assert!(
                quads <= crate::dungeon_bake::SCENE_QUADS,
                "{flavor:?}: {quads} quads"
            );
            assert!(crate::lighting::valid_bake(&scene), "{flavor:?}");
            // Every class the scene binds resolves.
            let catalog = crate::scripts::catalog(&root)
                .unwrap_or_else(|e| panic!("{flavor:?} script catalog: {e}"));
            for name in ["ThirdPersonController", classes.key.0, classes.door.0] {
                assert!(
                    catalog
                        .iter()
                        .flat_map(|s| s.classes.iter())
                        .any(|c| c.cpp_name == name),
                    "{flavor:?} catalog has no {name}"
                );
            }
            let readme = std::fs::read_to_string(root.join("README.md")).unwrap();
            assert!(readme.contains("DungeonDoor") && !readme.contains("<!--"));
            std::fs::remove_dir_all(parent).unwrap();
        }
    }
}
