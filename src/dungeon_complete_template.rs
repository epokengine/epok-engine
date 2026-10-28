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

pub fn create(root: &Path) -> Result<Scene, String> {
    fn copy(dir: &Dir<'_>, root: &Path) -> Result<(), String> {
        for file in dir.files() {
            // workspace publishes the new project's identity last, with the
            // author's chosen name and the current editor revision.
            if file
                .path()
                .extension()
                .is_some_and(|ext| ext == "epokproject")
            {
                continue;
            }
            crate::project::write_changed(&root.join(file.path()), file.contents())?;
        }
        for child in dir.dirs() {
            copy(child, root)?;
        }
        Ok(())
    }
    copy(&CONTENT, root)?;
    Scene::load(&root.join(manifest()?.startup_scene))
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
        assert_eq!(scene.actors.len(), 64);
        let index = crate::assets::scan(&root, &mut Default::default());
        assert!(index.problems.is_empty(), "{:?}", index.problems);
        assert_eq!(index.assets.len(), 67);
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
            root.join("assets/scripts/DungeonCombat/DungeonPocketLink.hpp")
                .is_file()
        );
        drop(project);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn unsupported_gameplay_does_not_claim_a_project_folder() {
        for flavor in [GameplayFlavor::Lua, GameplayFlavor::Blueprint] {
            let root = workspace::tests::temp("dungeon-complete-flavor");
            assert!(
                workspace::create_with_options(
                    &root,
                    "Complete",
                    CreateOptions::new(Template::DungeonComplete).with_gameplay(flavor)
                )
                .is_err()
            );
            assert!(!root.exists());
        }
    }
}
