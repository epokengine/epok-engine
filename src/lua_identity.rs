//! Class identity for `.lua` scripts, kept outside the script.
//!
//! A Lua class is named by its file and declared by its source text, so the
//! source itself carries no identity: a renamed file would otherwise be a new
//! class and every placed instance would lose its binding. The editor therefore
//! records one UUID per script path in a project settings document, exactly as
//! `scene_bank` records the registered maps.
//!
//! The document is a plain text map — one `path: uuid` line per class, sorted
//! by path — so a version control merge of two authors who each added a class
//! is a two-line, conflict-free merge, and a conflict is readable when it does
//! happen. Nothing in the engine, the compiler or the cooked build reads it
//! other than to answer "which class is this file?".
use crate::lua_asset::LuaFile;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path},
};

pub const REGISTRY: &str = "ProjectSettings/LuaClasses.epoksettings";
/// Every recorded script lives here, so a path outside it is a mistake.
pub const SCRIPTS: &str = "assets/scripts";

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct Registry {
    /// Project-relative `.lua` path to the class UUID it declares.
    pub classes: BTreeMap<String, String>,
}

/// Project-relative, forward-slashed spelling of a script path. The path may
/// already be resolved — `lua_asset::create_in` canonicalizes the scripts
/// folder — so the project root is resolved the same way before comparing.
pub fn relative(root: &Path, path: &Path) -> Option<String> {
    let strip = |root: &Path| Some(path.strip_prefix(root).ok()?.to_owned());
    let relative = strip(root).or_else(|| strip(&fs::canonicalize(root).ok()?))?;
    Some(relative.to_string_lossy().replace('\\', "/"))
}

pub fn read(root: &Path) -> Result<Registry, String> {
    let path = root.join(REGISTRY);
    if !path.exists() {
        return Ok(Registry::default());
    }
    let registry: Registry =
        crate::document::from_slice(&fs::read(path).map_err(|e| e.to_string())?)
            .map_err(|e| format!("Lua class registry: {e}"))?;
    registry.validate()?;
    Ok(registry)
}

impl Registry {
    /// Canonical UUIDs, unique across the project, on distinct `.lua` paths
    /// under `assets/scripts`. The file is hand-editable and hand-merged, so
    /// every one of those is checked on read rather than assumed.
    pub fn validate(&self) -> Result<(), String> {
        let mut ids = BTreeSet::new();
        let mut paths = BTreeSet::new();
        for (path, id) in &self.classes {
            let relative = Path::new(path);
            if relative
                .components()
                .any(|c| !matches!(c, Component::Normal(_)))
                || !relative.starts_with(SCRIPTS)
                || !path.ends_with(".lua")
                || !paths.insert(path.to_lowercase())
            {
                return Err(format!(
                    "Lua class registry requires unique relative {SCRIPTS}/*.lua paths ({path})"
                ));
            }
            if !crate::lua_asset::canonical(id) || !ids.insert(id.clone()) {
                return Err(format!(
                    "Lua class registry requires a unique canonical UUID per class ({path})"
                ));
            }
        }
        Ok(())
    }
    pub fn save(&self, root: &Path) -> Result<(), String> {
        self.validate()?;
        crate::project::write_changed(
            &root.join(REGISTRY),
            &crate::document::to_vec(self).map_err(|e| e.to_string())?,
        )
    }
}

/// Record a fresh identity for a script that is about to be created. Returns
/// the UUID the class will carry for the rest of its life.
pub fn assign(root: &Path, path: &Path) -> Result<String, String> {
    let relative =
        relative(root, path).ok_or_else(|| format!("{} is outside the project", path.display()))?;
    let mut registry = read(root)?;
    let id = uuid::Uuid::new_v4().to_string();
    registry.classes.insert(relative, id.clone());
    registry.save(root)?;
    Ok(id)
}

/// Follow a Content Browser rename or move: the class keeps its UUID because
/// the entry follows the file. `moves` are project-relative `(from, to)` pairs
/// naming files or the folders that contain them.
pub fn moved(root: &Path, moves: &[(String, String)]) -> Result<(), String> {
    let mut registry = read(root)?;
    let mut changed = false;
    for (from, to) in moves {
        let mut moved = BTreeMap::new();
        registry.classes.retain(|path, id| {
            let Some(tail) = follows(path, from) else {
                return true;
            };
            moved.insert(format!("{to}{tail}"), id.clone());
            false
        });
        changed |= !moved.is_empty();
        registry.classes.extend(moved);
    }
    if changed { registry.save(root) } else { Ok(()) }
}

/// Forget the classes a Content Browser deletion removed.
pub fn removed(root: &Path, paths: &[String]) -> Result<(), String> {
    let mut registry = read(root)?;
    let before = registry.classes.len();
    registry
        .classes
        .retain(|path, _| paths.iter().all(|gone| follows(path, gone).is_none()));
    if registry.classes.len() == before {
        return Ok(());
    }
    registry.save(root)
}

/// The part of `path` below `moved`, when `path` is `moved` itself or lies
/// inside it. Comparison is by whole path components, so `assets/scripts/AB`
/// is never treated as a child of `assets/scripts/A`.
fn follows(path: &str, moved: &str) -> Option<String> {
    if path == moved {
        return Some(String::new());
    }
    path.strip_prefix(moved)
        .filter(|tail| tail.starts_with('/'))
        .map(str::to_owned)
}

/// Stamp every discovered script with the identity the project records for it.
pub fn resolve(root: &Path, files: &mut [LuaFile]) -> Result<(), String> {
    let registry = read(root)?;
    for file in files {
        file.id = relative(root, &file.path).and_then(|path| registry.classes.get(&path).cloned());
    }
    Ok(())
}

/// Give every script that still has no entry one, so it is rename-safe from the
/// next catalog refresh on. A script copied in by hand therefore compiles under
/// its derived `lua:<Name>` identity once and is pinned to a UUID from then on.
/// The author's `.lua` is never rewritten: only the settings document changes.
pub fn adopt(root: &Path, files: &[LuaFile]) -> Result<(), String> {
    let mut registry = read(root)?;
    let mut changed = false;
    for file in files {
        let Some(path) = relative(root, &file.path) else {
            continue;
        };
        if registry.classes.contains_key(&path) {
            continue;
        }
        registry
            .classes
            .insert(path, uuid::Uuid::new_v4().to_string());
        changed = true;
    }
    if changed { registry.save(root) } else { Ok(()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project() -> std::path::PathBuf {
        let root = crate::workspace::tests::temp("lua-identity");
        fs::create_dir_all(root.join(SCRIPTS)).unwrap();
        fs::create_dir_all(root.join("ProjectSettings")).unwrap();
        root
    }

    #[test]
    fn lua_identity_survives_creation_rename_and_deletion() {
        let root = project();
        let path = root.join("assets/scripts/Cube.lua");
        fs::write(&path, "-- cube").unwrap();
        let id = assign(&root, &path).unwrap();
        assert!(crate::lua_asset::canonical(&id));
        assert_eq!(read(&root).unwrap().classes["assets/scripts/Cube.lua"], id);

        // A rename through the Content Browser keeps the identity.
        moved(
            &root,
            &[(
                "assets/scripts/Cube.lua".into(),
                "assets/scripts/Block.lua".into(),
            )],
        )
        .unwrap();
        let registry = read(&root).unwrap();
        assert_eq!(registry.classes.get("assets/scripts/Cube.lua"), None);
        assert_eq!(registry.classes["assets/scripts/Block.lua"], id);

        // So does a move of the folder that holds it.
        moved(
            &root,
            &[("assets/scripts".into(), "assets/scripts/enemies".into())],
        )
        .unwrap();
        assert_eq!(
            read(&root).unwrap().classes["assets/scripts/enemies/Block.lua"],
            id
        );

        // Deleting the folder forgets the class.
        removed(&root, &["assets/scripts/enemies".into()]).unwrap();
        assert!(read(&root).unwrap().classes.is_empty());
    }

    #[test]
    fn lua_identity_adopts_files_that_have_none_and_leaves_the_rest() {
        let root = project();
        let hand_copied = root.join("assets/scripts/Copied.lua");
        fs::write(&hand_copied, "-- copied").unwrap();
        let mut files = vec![LuaFile {
            path: hand_copied.clone(),
            source: "-- copied".into(),
            id: None,
        }];
        // Before adoption the file has no recorded identity at all.
        resolve(&root, &mut files).unwrap();
        assert_eq!(files[0].id, None);

        adopt(&root, &files).unwrap();
        resolve(&root, &mut files).unwrap();
        let assigned = files[0].id.clone().unwrap();
        assert!(crate::lua_asset::canonical(&assigned));

        // Adoption is idempotent: a second refresh keeps the same identity.
        adopt(&root, &files).unwrap();
        resolve(&root, &mut files).unwrap();
        assert_eq!(files[0].id.as_deref(), Some(assigned.as_str()));
    }

    #[test]
    fn lua_identity_rejects_a_hand_edited_registry() {
        let root = project();
        let write = |text: &str| {
            fs::write(root.join(REGISTRY), text).unwrap();
            read(&root).map(|_| ())
        };
        assert!(write("classes:\n  assets/scripts/A.lua: not-a-uuid\n").is_err());
        assert!(
            write(
                "classes:\n  assets/scripts/A.lua: 0f4a6d18-5b7e-4c92-8a30-1d2e3f4a5b6c\n  assets/scripts/B.lua: 0f4a6d18-5b7e-4c92-8a30-1d2e3f4a5b6c\n"
            )
            .is_err(),
            "two classes may not share one identity"
        );
        assert!(write("classes:\n  assets/A.lua: 0f4a6d18-5b7e-4c92-8a30-1d2e3f4a5b6c\n").is_err());
        assert!(
            write("classes:\n  ../outside/A.lua: 0f4a6d18-5b7e-4c92-8a30-1d2e3f4a5b6c\n").is_err()
        );
        assert!(
            write("classes:\n  assets/scripts/A.lua: 0f4a6d18-5b7e-4c92-8a30-1d2e3f4a5b6c\n")
                .is_ok()
        );
    }
}
