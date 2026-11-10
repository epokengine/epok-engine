//! Journal owned file changes; retain concurrent user edits during rollback.
use std::path::{Component, Path, PathBuf};

pub struct Transaction {
    root: PathBuf,
    created: Vec<(String, String, Option<Vec<u8>>)>,
    committed: bool,
}
impl Transaction {
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.into(),
            created: Vec::new(),
            committed: false,
        }
    }
    fn path(&self, relative: &str) -> Result<PathBuf, String> {
        let relative_path = Path::new(relative);
        if !relative_path.starts_with("ProjectSettings") {
            return crate::assets::inside(&self.root, relative);
        }
        if !relative_path
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
        {
            return Err("Installation settings require a project-relative path.".into());
        }
        let path = self.root.join(relative_path);
        let canonical_root =
            std::fs::canonicalize(&self.root).map_err(|error| error.to_string())?;
        let mut existing = path.as_path();
        while !existing.exists() {
            existing = existing
                .parent()
                .ok_or("Invalid installation settings path")?;
        }
        if !std::fs::canonicalize(existing)
            .map_err(|error| error.to_string())?
            .starts_with(canonical_root)
        {
            return Err("Installation settings cannot follow links outside the project.".into());
        }
        Ok(path)
    }
    pub fn create(&mut self, relative: &str, bytes: &[u8]) -> Result<bool, String> {
        let path = self.path(relative)?;
        if path.exists() {
            return Ok(false);
        }
        crate::assets::atomic_write(&path, bytes, None)?;
        self.created
            .push((relative.into(), crate::assets::hash(bytes), None));
        Ok(true)
    }
    /// Update an installation document while retaining its exact previous bytes.
    /// Sources remain create-only. Revisions prevent overwriting concurrent edits.
    pub fn settings(&mut self, relative: &str, bytes: &[u8]) -> Result<(), String> {
        if !Path::new(relative).starts_with("ProjectSettings") {
            return Err("Installation updates require a ProjectSettings document.".into());
        }
        let path = self.path(relative)?;
        let previous = match std::fs::read(&path) {
            Ok(previous) => previous,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                if !self.create(relative, bytes)? {
                    return Err("Installation settings changed during the operation; retry.".into());
                }
                return Ok(());
            }
            Err(error) => return Err(error.to_string()),
        };
        if previous == bytes {
            return Ok(());
        }
        crate::assets::atomic_write(&path, bytes, Some(&crate::assets::hash(&previous)))?;
        self.created
            .push((relative.into(), crate::assets::hash(bytes), Some(previous)));
        Ok(())
    }
    pub fn asset(
        &mut self,
        mut candidate: crate::assets::Candidate,
        package: &str,
        version: u32,
    ) -> Result<uuid::Uuid, String> {
        if candidate.expected.is_some() {
            return Err("Helper installation cannot replace an existing asset.".into());
        }
        let relative = crate::assets::path_string(&self.root, &candidate.destination);
        if crate::assets::inside(&self.root, &relative)? != candidate.destination {
            return Err("Helper asset destination is outside its project.".into());
        }
        candidate.package.meta.extra.insert(
            "game_helper_package".into(),
            serde_json::json!({"id": package, "version": version}),
        );
        let bytes = candidate.package.bytes()?;
        let id = crate::assets::commit(candidate)?;
        self.created
            .push((relative, crate::assets::hash(&bytes), None));
        Ok(id)
    }
    pub fn commit(mut self) {
        self.committed = true;
    }
}
impl Drop for Transaction {
    fn drop(&mut self) {
        if self.committed {
            return;
        }
        for (relative, hash, previous) in self.created.iter().rev() {
            let Ok(path) = self.path(relative) else {
                continue;
            };
            let Ok(metadata) = std::fs::symlink_metadata(&path) else {
                continue;
            };
            if !metadata.is_file() || metadata.file_type().is_symlink() {
                continue;
            }
            if std::fs::read(&path).is_ok_and(|bytes| crate::assets::hash(&bytes) == *hash) {
                if let Some(previous) = previous {
                    let _ = crate::assets::atomic_write(&path, previous, Some(hash));
                } else {
                    let _ = std::fs::remove_file(path);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn settings_rollback_restores_exact_bytes_and_retains_concurrent_edits() {
        let root = crate::workspace::tests::temp("helper-settings-rollback");
        std::fs::create_dir_all(root.join("ProjectSettings")).unwrap();
        let original = b"# authored formatting\nvalue: old\n";
        for name in ["Restore", "Edited"] {
            std::fs::write(
                root.join(format!("ProjectSettings/{name}.epoksettings")),
                original,
            )
            .unwrap();
        }
        {
            let mut transaction = Transaction::new(&root);
            for name in ["Restore", "Edited", "Created"] {
                transaction
                    .settings(
                        &format!("ProjectSettings/{name}.epoksettings"),
                        b"value: generated\n",
                    )
                    .unwrap();
            }
            std::fs::write(
                root.join("ProjectSettings/Edited.epoksettings"),
                b"value: user edit\n",
            )
            .unwrap();
            assert!(
                transaction
                    .settings("assets/Source.lua", b"replace source")
                    .is_err()
            );
        }
        assert_eq!(
            std::fs::read(root.join("ProjectSettings/Restore.epoksettings")).unwrap(),
            original
        );
        assert_eq!(
            std::fs::read(root.join("ProjectSettings/Edited.epoksettings")).unwrap(),
            b"value: user edit\n"
        );
        assert!(!root.join("ProjectSettings/Created.epoksettings").exists());
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn helper_install_rolls_back_only_its_unchanged_files() {
        let root = crate::workspace::tests::temp("helper-install-rollback");
        std::fs::create_dir_all(root.join("assets")).unwrap();
        std::fs::write(root.join("assets/User.txt"), b"authored").unwrap();
        {
            let mut transaction = Transaction::new(&root);
            assert!(
                !transaction
                    .create("assets/User.txt", b"replacement")
                    .unwrap()
            );
            transaction
                .create("assets/Owned.txt", b"generated")
                .unwrap();
            transaction
                .create("assets/Edited.txt", b"generated")
                .unwrap();
            std::fs::write(root.join("assets/Edited.txt"), b"user edit").unwrap();
            assert!(transaction.create("../outside.txt", b"bad").is_err());
        }
        assert_eq!(
            std::fs::read(root.join("assets/User.txt")).unwrap(),
            b"authored"
        );
        assert_eq!(
            std::fs::read(root.join("assets/Edited.txt")).unwrap(),
            b"user edit"
        );
        assert!(!root.join("assets/Owned.txt").exists());
        let mut transaction = Transaction::new(&root);
        transaction
            .create("assets/Committed.txt", b"retained")
            .unwrap();
        transaction.commit();
        assert!(root.join("assets/Committed.txt").is_file());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn helper_composition_validation_rolls_back_scene_and_resources() {
        let root = crate::workspace::tests::temp("helper-composition-transaction");
        crate::workspace::create(&root, "Helpers", crate::workspace::Template::Basic).unwrap();
        let registry = crate::blueprint::native_registry(&root, &[]).unwrap();
        let model = registry.model().unwrap();
        let actor =
            crate::class_composition::actor(&model, crate::dialogue_layout::PRESENTER, "Widget")
                .unwrap();
        let mut invalid = actor.clone();
        invalid.id = uuid::Uuid::new_v4();
        invalid.components.clear();
        let mut scene = crate::scene::Scene {
            actors: vec![actor, invalid],
            ..Default::default()
        };
        let before = scene.clone();
        let error = crate::game_helpers::compose_dialogue_widget(
            &mut scene,
            &root,
            &registry,
            &model,
            0,
            crate::dialogue_asset::Presentation::ClassicBox,
        )
        .unwrap_err();
        assert!(error.contains("root"), "{error}");
        assert_eq!(scene, before);
        for path in ["dialogue-frame.png", "dialogue-frame.epokasset"] {
            assert!(!root.join("assets/GameHelpers/UI").join(path).exists());
        }
        std::fs::remove_dir_all(root).unwrap();
    }
}
