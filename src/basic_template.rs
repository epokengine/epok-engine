//! Empty world with one editable, inert project entry class in the chosen flavor.
use crate::{
    actor_document::{ActorInstance, ClassReference, ComponentInstance},
    scene::Scene,
    workspace::GameplayFlavor,
};
use std::path::Path;

pub const NAME: &str = "GameRoot";
pub const CPP_ID: &str = "509247ae-8730-52eb-98eb-e0a2436e6827";
pub const BP_ID: &str = "1f370e47-2a64-5667-b6e2-fc9ca588707e";
pub const LUA_ID: &str = "8b1e9b3c-715c-585d-b19e-a6979963f499";
pub const LUA_PATH: &str = "assets/scripts/GameRoot.lua";
pub const BP_PATH: &str = "assets/Blueprints/GameRoot.epokbp";

pub fn class_id(flavor: GameplayFlavor) -> &'static str {
    match flavor {
        GameplayFlavor::Cpp => CPP_ID,
        GameplayFlavor::Blueprint => BP_ID,
        GameplayFlavor::Lua => LUA_ID,
    }
}

pub fn create(root: &Path, flavor: GameplayFlavor) -> Result<Scene, String> {
    let mut scene = Scene {
        name: "Main".into(),
        ..Default::default()
    };
    scene.actors.truncate(1);
    let class = ClassReference::new(NAME, class_id(flavor));
    let mut actor = ActorInstance::new(uuid::Uuid::new_v4(), class.clone(), NAME);
    let mut spatial = ComponentInstance::new(
        uuid::Uuid::new_v4(),
        ClassReference::new(
            "epok::SceneComponent3D",
            crate::object_model::SCENE_COMPONENT3D_ID,
        ),
        "Root",
    );
    spatial.root = true;
    spatial.inherited = true;
    actor.components.push(spatial);
    actor.refresh_components();
    scene.actors.push(actor);
    match flavor {
        GameplayFlavor::Cpp => {
            crate::project::write_changed(
                &root.join("assets/scripts/GameRoot.hpp"),
                include_bytes!("../templates/GameRoot.hpp"),
            )?;
            crate::project::write_changed(
                &root.join("assets/scripts/GameRoot.cpp"),
                b"#include \"GameRoot.hpp\"\n",
            )?;
        }
        GameplayFlavor::Blueprint => {
            let mut asset = crate::blueprint_asset::BlueprintAsset::new(
                NAME.into(),
                crate::object_model::ACTOR3D_ID.into(),
            );
            asset.id = BP_ID.into();
            asset.family = Some(crate::reflection_schema::ClassFamily::Actor);
            std::fs::create_dir_all(root.join("assets/Blueprints")).map_err(|e| e.to_string())?;
            crate::blueprint_asset::create(&root.join(BP_PATH), &asset)?;
        }
        GameplayFlavor::Lua => {
            crate::project::write_changed(
                &root.join(LUA_PATH),
                include_bytes!("../templates/GameRoot.lua"),
            )?;
            let mut identities = crate::lua_identity::Registry::default();
            identities.classes.insert(LUA_PATH.into(), LUA_ID.into());
            identities.save(root)?;
        }
    }
    crate::template_install::write(
        root,
        crate::workspace::Template::Basic,
        flavor,
        [("game_root", class)],
    )?;
    Ok(scene)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_generates_only_the_selected_inert_actor_and_keeps_helpers_optional() {
        for flavor in GameplayFlavor::ALL {
            let root = crate::workspace::tests::temp("basic-game-root");
            let project = crate::workspace::create_with_options(
                &root,
                "Basic Root",
                crate::workspace::CreateOptions::new(crate::workspace::Template::Basic)
                    .with_gameplay(flavor),
            )
            .unwrap();
            let installation = crate::template_install::read(&root).unwrap().unwrap();
            assert_eq!(installation.gameplay, flavor.key());
            assert_eq!(installation.template, "basic");
            assert_eq!(
                installation.roles["game_root"].class_id.as_deref(),
                Some(class_id(flavor))
            );
            let sources = [
                "assets/scripts/GameRoot.hpp",
                "assets/scripts/GameRoot.cpp",
                BP_PATH,
                LUA_PATH,
            ];
            let chosen: &[&str] = match flavor {
                GameplayFlavor::Cpp => &sources[..2],
                GameplayFlavor::Blueprint => &[BP_PATH],
                GameplayFlavor::Lua => &[LUA_PATH],
            };
            for source in sources {
                assert_eq!(
                    root.join(source).exists(),
                    chosen.contains(&source),
                    "{flavor:?}: {source}"
                );
            }
            let scene = Scene::load(&crate::workspace::startup_scene(&root).unwrap()).unwrap();
            assert_eq!(scene.actors.len(), 2);
            assert!(
                scene.actors[0]
                    .components
                    .iter()
                    .any(|c| c.class.class_id.as_deref()
                        == Some(crate::object_model::CAMERA3D_COMPONENT_ID))
            );
            assert_eq!(
                scene.actors[1].class.class_id.as_deref(),
                Some(class_id(flavor))
            );
            assert_eq!(scene.actors[1].components.len(), 1);
            assert!(scene.actors[1].components[0].root);
            let registry = crate::scripts::declaration_registry(&root).unwrap();
            let model = registry.model().unwrap();
            scene.validate_with_model(Some(&model)).unwrap();
            let class = installation.roles["game_root"].resolve(&model).unwrap();
            assert_eq!(class.family, crate::reflection_schema::ClassFamily::Actor);
            assert!(
                class
                    .ancestry
                    .iter()
                    .any(|id| id == crate::object_model::ACTOR3D_ID)
            );
            let declared = &registry.classes[class_id(flavor)];
            let author = match flavor {
                GameplayFlavor::Cpp => crate::reflection_schema::native_provider(),
                GameplayFlavor::Blueprint => crate::script_backend::blueprint_provider(),
                GameplayFlavor::Lua => crate::lua_asset::provider(),
            };
            assert!(
                class.derivable_by(&author),
                "{flavor:?} GameRoot must be offered as a parent in the class picker"
            );
            assert!(crate::script_backend::can_derive(&author, declared));
            assert!(declared.functions.is_empty() && declared.properties.is_empty());
            let table = crate::blueprint_spawn::object_class_table_for_scenes(
                &registry,
                std::slice::from_ref(&scene),
            )
            .unwrap();
            for capability in [
                "INTERACTION",
                "WORLD_GAMEPLAY",
                "DIALOGUE",
                "CHARACTER",
                "CAMERA_RIG",
            ] {
                assert!(
                    !table.contains(&format!("#define EPOK_{capability}_USED 1")),
                    "{flavor:?}: {capability}"
                );
            }
            let saved = std::fs::read(root.join(crate::template_install::PATH)).unwrap();
            drop(project);
            let reopened = crate::workspace::Project::open(&root).unwrap();
            assert_eq!(
                std::fs::read(root.join(crate::template_install::PATH)).unwrap(),
                saved
            );
            drop(reopened);
            std::fs::remove_dir_all(root).unwrap();
        }
    }
}
