//! The curated, project-owned Dungeon game, including its native gameplay and
//! Pocket Lua miner. No caches, machine settings or saved games are bundled.
use crate::{scene::Scene, workspace::Manifest};
use include_dir::{Dir, include_dir};
use std::path::Path;

static CONTENT: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/resources/templates/dungeon-complete");

pub fn manifest() -> Result<Manifest, String> {
    crate::document::from_slice(
        CONTENT
            .get_file("DungeonComplete.epokproject")
            .ok_or("Dungeon Complete settings are missing")?
            .contents(),
    )
    .map_err(|e| e.to_string())
}

/// Only authored policy implementations vary by language. Their contracts,
/// class UUIDs and placed component identities remain shared across providers.
#[derive(Clone, Copy)]
struct Policy {
    role: &'static str,
    name: &'static str,
    id: &'static str,
}
const POLICIES: &[Policy] = &[
    Policy {
        role: "run_flow",
        name: "DungeonRunFlow",
        id: "df3f7e3e-b99c-5530-9d86-a44f87e67c5f",
    },
    Policy {
        role: "player_vitals",
        name: "DungeonPlayerVitals",
        id: "070a2ac6-e00f-5e9c-8bd2-560acb25ad2f",
    },
    Policy {
        role: "player_combat",
        name: "DungeonPlayerCombat",
        id: "e8e7c9ff-ff0e-5c38-9eec-5434e6a45354",
    },
    Policy {
        role: "encounter_policy",
        name: "DungeonEncounter",
        id: "825024e2-d9f8-5578-8198-bc52b5c217f2",
    },
    Policy {
        role: "economy_policy",
        name: "DungeonEconomyPolicy",
        id: "d3ed89cb-587b-57cb-a136-8172feb4c774",
    },
    Policy {
        role: "game_hud",
        name: "DungeonGameHUDWidget",
        id: "b220c109-6a29-53a3-bbbc-dcf880abe73d",
    },
    Policy {
        role: "terminal_widget",
        name: "DungeonTerminalWidget",
        id: "f5ddf390-875f-56c6-8efd-9265383c29ff",
    },
    Policy {
        role: "shop_widget",
        name: "DungeonShopWidget",
        id: "733195d2-87b9-5164-b675-53372b0a569c",
    },
    Policy {
        role: "inventory_widget",
        name: "DungeonInventoryWidget",
        id: "f0474611-dd51-5cea-9702-500ba3907dfd",
    },
    Policy {
        role: "economy_station",
        name: "DungeonEconomyStation",
        id: "81e70d57-1589-555f-9a42-fc6ba0683807",
    },
];
static FLAVORS: Dir<'_> =
    include_dir!("$CARGO_MANIFEST_DIR/resources/templates/dungeon-complete-flavors");
const SCRIPT_DIR: &str = "assets/scripts/DungeonCombat";
const BLUEPRINT_DIR: &str = "assets/Blueprints/DungeonCombat";
impl Policy {
    fn base_path(self) -> String {
        let base = match self.name {
            "DungeonGameHUDWidget" => "DungeonEconomyPresenterBase".into(),
            "DungeonTerminalWidget" | "DungeonShopWidget" | "DungeonInventoryWidget" => {
                "DungeonEconomyWidgetBase".into()
            }
            "DungeonEconomyStation" => "DungeonEconomyStationBase".into(),
            _ => format!("{}Base", self.name),
        };
        format!("{SCRIPT_DIR}/{base}.hpp")
    }
    fn path(self, flavor: crate::workspace::GameplayFlavor) -> String {
        use crate::workspace::GameplayFlavor;
        match flavor {
            GameplayFlavor::Cpp => format!("{SCRIPT_DIR}/{}.hpp", self.name),
            GameplayFlavor::Blueprint => format!("{BLUEPRINT_DIR}/{}.epokbp", self.name),
            GameplayFlavor::Lua => format!("{SCRIPT_DIR}/{}.lua", self.name),
        }
    }
    fn source(self, flavor: crate::workspace::GameplayFlavor) -> Option<String> {
        use crate::workspace::GameplayFlavor;
        match flavor {
            GameplayFlavor::Cpp => None,
            GameplayFlavor::Blueprint => Some(format!("blueprint/{}.epokbp", self.name)),
            GameplayFlavor::Lua => Some(format!("lua/{}.lua", self.name)),
        }
    }
}
enum RoleSource {
    Actor(&'static str),
    Component(&'static str),
}
// Extend this list for upcoming ordinary editable UIActor/widget classes.
// Copying common content does not depend on a closed list of widget files.
const COMMON_ROLES: &[(&str, RoleSource)] = &[
    ("player", RoleSource::Actor("Player Aqua")),
    ("camera", RoleSource::Actor("Follow Camera")),
    (
        "controller",
        RoleSource::Component("DungeonPlayerController"),
    ),
    ("game_root", RoleSource::Component("DungeonGame")),
];

pub fn create(root: &Path, flavor: crate::workspace::GameplayFlavor) -> Result<Scene, String> {
    use crate::workspace::GameplayFlavor;
    // Embedded overlays are mandatory. Missing/mismatched authored BP data
    // cannot silently select the bundled C++ concrete implementation instead.
    for policy in POLICIES {
        if CONTENT.get_file(policy.base_path()).is_none() {
            return Err(format!(
                "Dungeon Complete shared base for {} is missing",
                policy.name
            ));
        }
        if let Some(source) = policy.source(flavor) {
            let file = FLAVORS.get_file(&source).ok_or_else(|| {
                format!(
                    "Dungeon Complete {:?} policy {} is missing",
                    flavor, policy.name
                )
            })?;
            if flavor == GameplayFlavor::Blueprint {
                let asset: crate::blueprint_asset::BlueprintAsset =
                    crate::document::from_slice(file.contents())?;
                if asset.id != policy.id || asset.name != policy.name {
                    return Err(format!(
                        "Dungeon Complete policy {} changed its stable identity",
                        policy.name
                    ));
                }
            }
        } else if CONTENT.get_file(policy.path(flavor)).is_none() {
            return Err(format!(
                "Dungeon Complete C++ policy {} is missing",
                policy.name
            ));
        }
    }
    fn copy(dir: &Dir<'_>, root: &Path, flavor: GameplayFlavor) -> Result<(), String> {
        for file in dir.files() {
            let path = file.path();
            if path.extension().is_some_and(|ext| ext == "epokproject") {
                continue; // workspace publishes project identity last.
            }
            // The retired native protocol remains a repository differential
            // oracle. New projects use the selected economy policy and SDK IO.
            if path == Path::new("assets/scripts/DungeonCombat/DungeonPocketLink.hpp") {
                continue;
            }
            if flavor == GameplayFlavor::Blueprint
                && path.parent() == Some(Path::new("assets/Classes"))
                && path
                    .extension()
                    .is_some_and(|extension| extension == "epokclass")
                && POLICIES.iter().any(|policy| {
                    path.file_stem().and_then(|stem| stem.to_str()) == Some(policy.id)
                })
            {
                continue; // Blueprint owns its composition in the class asset.
            }
            let concrete_policy = path.parent() == Some(Path::new(SCRIPT_DIR))
                && path
                    .extension()
                    .is_some_and(|ext| ext == "hpp" || ext == "cpp")
                && POLICIES.iter().any(|policy| {
                    path.file_stem().and_then(|stem| stem.to_str()) == Some(policy.name)
                });
            if concrete_policy && flavor != GameplayFlavor::Cpp {
                continue;
            }
            crate::project::write_changed(&root.join(path), file.contents())?;
        }
        for child in dir.dirs() {
            copy(child, root, flavor)?;
        }
        Ok(())
    }
    copy(&CONTENT, root, flavor)?;
    let mut identities = crate::lua_identity::read(root)?;
    for policy in POLICIES {
        if let Some(source) = policy.source(flavor) {
            let file = FLAVORS
                .get_file(source)
                .expect("mandatory overlays checked before copying");
            let path = policy.path(flavor);
            crate::project::write_changed(&root.join(&path), file.contents())?;
            if flavor == GameplayFlavor::Lua {
                identities.classes.insert(path, policy.id.into());
            }
        }
    }
    if flavor == GameplayFlavor::Lua {
        identities.save(root)?;
    }
    let scene = Scene::load(&root.join(manifest()?.startup_scene))?;
    let mut roles = Vec::new();
    for (role, source) in COMMON_ROLES {
        let class = match source {
            RoleSource::Actor(name) => scene
                .actors
                .iter()
                .find(|actor| actor.name == *name)
                .map(|actor| actor.class.clone()),
            RoleSource::Component(name) => scene
                .actors
                .iter()
                .flat_map(|actor| &actor.components)
                .find(|component| component.class.name == *name)
                .map(|component| component.class.clone()),
        }
        .ok_or_else(|| format!("Dungeon Complete role {role} has no class in the curated scene"))?;
        roles.push((*role, class));
    }
    roles.extend(POLICIES.iter().map(|policy| {
        (
            policy.role,
            crate::actor_document::ClassReference::new(policy.name, policy.id),
        )
    }));
    crate::template_install::write(
        root,
        crate::workspace::Template::DungeonComplete,
        flavor,
        roles,
    )?;
    Ok(scene)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace::{self, CreateOptions, GameplayFlavor, Template};

    #[test]
    fn complete_game_is_portable_and_keeps_its_runtime_budget() {
        let root = workspace::tests::temp("dungeon-complete");
        let project =
            workspace::create(&root, "My Complete Game", Template::DungeonComplete).unwrap();
        assert_eq!(project.manifest.name, "My Complete Game");
        assert_eq!(project.manifest.play.target, crate::play::Target::Embedded);
        assert_eq!(project.manifest.play.data, crate::play::DataSource::Host);
        assert_eq!(project.manifest.rendering.mesh_triangle_budget, Some(2432));
        assert!(
            project
                .manifest
                .scene_storage
                .as_ref()
                .unwrap()
                .contains(&project.manifest.startup_scene)
        );
        assert_eq!(
            std::fs::read_dir(root.join("assets/scenes"))
                .unwrap()
                .count(),
            1
        );
        let scene = Scene::load(&root.join(&project.manifest.startup_scene)).unwrap();
        assert_eq!(scene.actors.len(), 58);
        let audio_root = scene
            .actors
            .iter()
            .find(|a| a.name == "Reactor Station")
            .unwrap();
        assert_eq!(audio_root.components.len(), 8);
        assert_eq!(
            audio_root
                .components
                .iter()
                .filter(|c| c.class.name == "epok::AudioComponent")
                .count(),
            7
        );
        let game = scene
            .actors
            .iter()
            .flat_map(|a| &a.components)
            .find(|c| c.class.name == "DungeonGame")
            .unwrap();
        let audio_id: uuid::Uuid = game.properties["audio_bindings"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap();
        let audio = scene
            .actors
            .iter()
            .flat_map(|actor| &actor.components)
            .find(|component| component.id == audio_id)
            .unwrap();
        assert_eq!(audio.class.name, "DungeonAudioBindings");
        let mut source_ids = std::collections::BTreeSet::new();
        for name in [
            "player_shot_source",
            "world_hit_source",
            "robot_hit_source",
            "player_damage_source",
            "enemy_shot_source",
            "robot_destroyed_source",
            "ui_source",
            "music_source",
        ] {
            let id: uuid::Uuid = audio.properties[name].as_str().unwrap().parse().unwrap();
            assert!(source_ids.insert(id), "Independent channel {name}");
            let source = scene
                .actors
                .iter()
                .flat_map(|a| &a.components)
                .find(|c| c.id == id)
                .unwrap();
            assert_eq!(source.class.name, "epok::AudioComponent");
        }
        let player = scene
            .actors
            .iter()
            .find(|actor| actor.name == "Player Aqua")
            .unwrap();
        assert!(
            player.components.len() <= 6,
            "The shipped player retains two extension component slots"
        );
        let vitals_id: uuid::Uuid = game.properties["player_vitals"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap();
        assert_eq!(game.properties["player_actor"], player.id.to_string());
        let vitals = scene
            .actors
            .iter()
            .flat_map(|actor| &actor.components)
            .find(|component| component.id == vitals_id)
            .unwrap();
        assert_eq!(vitals.class.name, "DungeonPlayerVitals");
        assert_eq!(vitals.properties["tuning"]["maximum"], 100);
        assert_eq!(vitals.properties["tuning"]["hit_immunity"], 0.7);
        assert_eq!(player.class.name, "epok::Character3D");
        let motor = player
            .components
            .iter()
            .find(|component| component.class.name == "epok::CharacterMotor3DComponent")
            .unwrap();
        assert_eq!(motor.properties["automatic_tick"], false);
        assert_eq!(motor.properties["measure_speed"], false);
        assert_eq!(motor.properties["step_height"].as_f64(), Some(0.0));
        assert_eq!(motor.properties["gravity"].as_f64(), Some(24.0));
        assert_eq!(motor.properties["jump_speed"].as_f64(), Some(9.0));
        let camera = scene
            .actors
            .iter()
            .find(|actor| actor.name == "Follow Camera")
            .unwrap();
        assert_eq!(camera.class.name, "DungeonCamera");
        assert_eq!(camera.logical_parent, Some(audio_root.id));
        assert!(camera.attach.is_none(), "The camera applies world poses");
        assert_eq!(player.properties["camera"], camera.id.to_string());
        let visual: uuid::Uuid = player.properties["visual"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap();
        assert!(
            scene
                .actors
                .iter()
                .flat_map(|actor| &actor.components)
                .any(|component| component.id == visual
                    && component.class.name == "epok::Mesh3DComponent")
        );
        assert_eq!(camera.properties["target"], player.id.to_string());
        assert_eq!(camera.properties["automatic_tick"], false);
        let flow_id: uuid::Uuid = game.properties["run_flow"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap();
        let flow = camera
            .components
            .iter()
            .find(|component| component.id == flow_id)
            .unwrap();
        assert_eq!(flow.class.name, "DungeonRunFlow");
        assert_eq!(camera.components.len(), 7);
        let combat_id: uuid::Uuid = player
            .components
            .iter()
            .find(|component| component.class.name == "DungeonPlayerController")
            .unwrap()
            .properties["combat_policy"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap();
        assert!(camera.components.iter().any(|component| {
            component.id == combat_id && component.class.name == "DungeonPlayerCombat"
        }));
        let context_id: uuid::Uuid = game.properties["collision_queries"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap();
        assert_eq!(camera.properties["query_context"], context_id.to_string());
        let context = scene
            .actors
            .iter()
            .flat_map(|actor| &actor.components)
            .find(|component| component.id == context_id)
            .unwrap();
        assert_eq!(context.class.name, "epok::SpatialQueryContextComponent");
        assert_eq!(context.properties["automatic_reset"], false);
        for (property, class) in [
            ("projectile_pool", "epok::ProjectilePool3DComponent"),
            ("encounter_policy", "DungeonEncounter"),
            ("economy_policy", "DungeonEconomyPolicy"),
        ] {
            let id: uuid::Uuid = game.properties[property].as_str().unwrap().parse().unwrap();
            assert!(
                scene
                    .actors
                    .iter()
                    .flat_map(|actor| &actor.components)
                    .any(|component| { component.id == id && component.class.name == class })
            );
        }
        for index in 0..8 {
            let visual = scene
                .actors
                .iter()
                .find(|actor| actor.name == format!("Combat FX {index}"))
                .unwrap();
            assert!(
                visual.attach.is_none(),
                "Projectile positions are world coordinates"
            );
        }
        let arm = camera
            .components
            .iter()
            .find(|component| component.class.name == "epok::SpringArmComponent")
            .unwrap();
        assert_eq!(arm.properties["adaptive_pull_in"], true);
        assert_eq!(arm.properties["collision_mask"], 1);
        assert_eq!(arm.properties["minimum_fraction"], 0);
        let index = crate::assets::scan(&root, &mut Default::default());
        assert!(index.problems.is_empty(), "{:?}", index.problems);
        assert_eq!(index.assets.len(), 68);
        assert!(
            root.join("assets/PocketStation/AsteroidCatcher.epokasset")
                .is_file()
        );
        assert!(!root.join("Tools").exists());
        assert!(!root.join("UserSettings/PocketStation.mcd").exists());
        assert!(!root.join("assets/CombatDemo").exists());
        assert!(
            !root
                .join("assets/scripts/ThirdPersonController.lua")
                .exists()
        );
        assert!(
            !root
                .join("assets/scripts/DungeonCombat/DungeonPocketLink.hpp")
                .exists()
        );
        drop(project);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn complete_game_generation_records_each_selected_authoring_flavor() {
        for flavor in GameplayFlavor::ALL {
            let root = workspace::tests::temp("dungeon-complete-flavor");
            let project = workspace::create_with_options(
                &root,
                "Complete",
                CreateOptions::new(Template::DungeonComplete).with_gameplay(flavor),
            )
            .unwrap();
            assert_eq!(
                crate::template_install::read(&root)
                    .unwrap()
                    .unwrap()
                    .gameplay,
                flavor.key()
            );
            assert_eq!(project.manifest.lua_execution, Default::default());
            drop(project);
            std::fs::remove_dir_all(root).unwrap();
        }
    }
    #[test]
    fn direct_complete_installation_selects_only_authored_policies_and_keeps_shared_identity() {
        // Inspect the exact installed sources independently of runtime evidence.
        for flavor in GameplayFlavor::ALL {
            let root = workspace::tests::temp("complete-policy-install");
            let scene = create(&root, flavor).unwrap();
            assert_eq!(
                std::fs::read(root.join(manifest().unwrap().startup_scene)).unwrap(),
                CONTENT
                    .get_file(manifest().unwrap().startup_scene)
                    .unwrap()
                    .contents()
            );
            let roles = crate::template_install::read(&root).unwrap().unwrap();
            assert_eq!(roles.template, "dungeon-complete");
            assert_eq!(roles.gameplay, flavor.key());
            let identities = crate::lua_identity::read(&root).unwrap();
            for policy in POLICIES {
                assert_eq!(roles.roles[policy.role].name, policy.name);
                assert_eq!(
                    roles.roles[policy.role].class_id.as_deref(),
                    Some(policy.id)
                );
                let placed = scene.actors.iter().any(|actor| {
                    (actor.class.name == policy.name
                        && actor.class.class_id.as_deref() == Some(policy.id))
                        || actor.components.iter().any(|component| {
                            component.class.name == policy.name
                                && component.class.class_id.as_deref() == Some(policy.id)
                        })
                });
                let composition = root.join(format!("assets/Classes/{}.epokclass", policy.id));
                let embedded = flavor == GameplayFlavor::Blueprint
                    && !crate::blueprint_asset::load(&root.join(policy.path(flavor)))
                        .unwrap()
                        .template
                        .actors
                        .is_empty();
                assert!(
                    placed || composition.is_file() || embedded,
                    "{} must be placed or available as an ordinary spawnable class composition",
                    policy.name
                );
                if flavor == GameplayFlavor::Blueprint {
                    assert!(!composition.exists());
                }
                let selected = root.join(policy.path(flavor));
                assert!(selected.is_file());
                assert_eq!(
                    root.join(policy.path(GameplayFlavor::Cpp)).exists(),
                    flavor == GameplayFlavor::Cpp
                );
                let cpp_tu = format!("{SCRIPT_DIR}/{}.cpp", policy.name);
                assert_eq!(
                    root.join(&cpp_tu).exists(),
                    flavor == GameplayFlavor::Cpp && CONTENT.get_file(&cpp_tu).is_some()
                );
                assert!(root.join(policy.base_path()).is_file());
                if let Some(source) = policy.source(flavor) {
                    assert_eq!(
                        std::fs::read(&selected).unwrap(),
                        FLAVORS.get_file(source).unwrap().contents()
                    );
                }
                match flavor {
                    GameplayFlavor::Cpp => assert!(
                        std::fs::read_to_string(&selected)
                            .unwrap()
                            .contains(policy.id)
                    ),
                    GameplayFlavor::Blueprint => {
                        let asset = crate::blueprint_asset::load(&selected).unwrap();
                        assert_eq!(asset.id, policy.id);
                        assert_eq!(asset.name, policy.name);
                        assert!(!asset.functions.is_empty());
                    }
                    GameplayFlavor::Lua => {
                        assert_eq!(identities.classes[&policy.path(flavor)], policy.id);
                        assert!(
                            std::fs::read_to_string(&selected)
                                .unwrap()
                                .contains(&format!("function {}:", policy.name))
                        );
                    }
                }
            }
            for mechanism in [
                "DungeonGame.cpp",
                "DungeonPlayerController.cpp",
                "DungeonEconomy.cpp",
                "DungeonTargeting.cpp",
            ] {
                let path = format!("{SCRIPT_DIR}/{mechanism}");
                assert_eq!(
                    std::fs::read(root.join(&path)).unwrap(),
                    CONTENT.get_file(path).unwrap().contents()
                );
            }
            assert!(
                CONTENT.get_dir(".epok").is_none(),
                "No generated caches are bundled"
            );
            assert!(!root.join(".epok/build").exists());
            assert!(!root.join("UserSettings").exists());
            assert!(!root.join("exports").exists());
            assert!(!root.read_dir().unwrap().any(|entry| {
                entry
                    .unwrap()
                    .path()
                    .extension()
                    .is_some_and(|ext| ext == "epokproject")
            }));
            // Relative Lua paths and UUID roles survive relocating the whole
            // authored project; no editor/tool/machine-specific path is stored.
            let moved = root.with_file_name(format!(
                "{}-moved",
                root.file_name().unwrap().to_string_lossy()
            ));
            std::fs::rename(&root, &moved).unwrap();
            assert_eq!(
                crate::template_install::read(&moved).unwrap().unwrap(),
                roles
            );
            assert_eq!(crate::lua_identity::read(&moved).unwrap(), identities);
            assert!(Scene::load(&moved.join(manifest().unwrap().startup_scene)).is_ok());
            std::fs::remove_dir_all(moved).unwrap();
        }
    }
}
