//! Game project identity and lifetime. No editor sources belong in this directory.
use crate::{project::write_changed, scene::Scene};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Component, Path, PathBuf},
};

/// The entry file is the only writable owner of project-wide settings in new projects.
pub const DESCRIPTOR_EXTENSION: &str = "epokproject";
pub const LEGACY_MANIFEST: &str = "ProjectSettings/project.json";
/// Kept as a compatibility name for callers which only need to name the old format.
#[allow(dead_code)]
pub const MANIFEST: &str = LEGACY_MANIFEST;
pub const FORMAT: u32 = 1;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub format_version: u32,
    pub editor_version: String,
    pub name: String,
    pub startup_scene: String,
    pub auto_build: bool,
    #[serde(default)]
    pub build: crate::build_report::Options,
    #[serde(default)]
    pub play: crate::play::Profile,
    #[serde(default)]
    pub transition: crate::play::Transition,
    #[serde(default)]
    pub rendering: crate::settings::Rendering,
    #[serde(default)]
    pub debug: crate::settings::DebugHud,
    #[serde(default)]
    pub default_sound_bank: Option<uuid::Uuid>,
    /// `cpp_name` of the `SceneScriptActor` subclass proposed when a map's
    /// scene Blueprint is created. Changing it never touches existing maps.
    #[serde(default)]
    pub default_scene_script_parent: Option<String>,
}

pub struct Project {
    pub root: PathBuf,
    pub manifest: Manifest,
    // OS lock releases on normal exit AND crashes; the file itself is disposable.
    _lock: fs::File,
}

impl Project {
    pub fn open(path: &Path) -> Result<Self, String> {
        let (root, manifest_path) = resolve(path)?;
        let manifest = read_manifest_at(&root, &manifest_path)?;
        // Validate the document here; the project loader indexes and decodes
        // its resources once, after acquiring the project lock.
        Scene::load_unresolved(&scene_path(&root, &manifest)?)?;
        fs::create_dir_all(root.join(".epok")).map_err(|e| e.to_string())?;
        let lock = lock_root(&root)?;
        // Invalid visual graphs must remain openable for repair in the canvas.
        // Native discovery still gates unavailable SDK/reflection prerequisites.
        crate::scripts::native_catalog(&root).inspect_err(|error| {
            let _ = crate::timeline_compile::invalidate_all(&root, error);
        })?;
        fs::create_dir_all(root.join("UserSettings")).map_err(|e| e.to_string())?;
        Ok(Self {
            root,
            manifest,
            _lock: lock,
        })
    }
}

fn lock_root(root: &Path) -> Result<fs::File, String> {
    fs::create_dir_all(root.join(".epok")).map_err(|e| e.to_string())?;
    let lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(root.join(".epok/project.lock"))
        .map_err(|e| e.to_string())?;
    lock.try_lock()
        .map_err(|e| format!("Project is already open or cannot be locked: {e}"))?;
    Ok(lock)
}
fn is_descriptor(path: &Path) -> bool {
    path.extension()
        .and_then(|v| v.to_str())
        .is_some_and(|v| v.eq_ignore_ascii_case(DESCRIPTOR_EXTENSION))
}
fn descriptors(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut paths = vec![];
    for entry in fs::read_dir(root).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if !is_descriptor(&path) {
            continue;
        }
        let metadata = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            return Err("Project descriptors must be regular files directly inside the project root, not links or directories.".into());
        }
        paths.push(path);
    }
    paths.sort();
    Ok(paths)
}

/// Resolve either a project directory or its `.epokproject` entry file.
/// The returned root is canonical so file and directory aliases share locks and recents.
pub fn resolve(path: &Path) -> Result<(PathBuf, PathBuf), String> {
    let input = fs::canonicalize(path).map_err(|e| format!("Project {}: {e}", path.display()))?;
    let root = if input.is_file() {
        if !is_descriptor(&input) {
            return Err("Choose a project folder or an .epokproject descriptor.".into());
        }
        input
            .clone()
            .parent()
            .ok_or("Project descriptor has no parent folder")?
            .to_path_buf()
    } else if input.is_dir() {
        input.clone()
    } else {
        return Err("Choose a project folder or an .epokproject descriptor.".into());
    };
    let descriptors = descriptors(&root)?;
    let legacy = root.join(LEGACY_MANIFEST);
    if descriptors.len() > 1 {
        return Err(format!(
            "Project {} has multiple .epokproject descriptors; keep exactly one.",
            root.display()
        ));
    }
    if let Some(descriptor) = descriptors.into_iter().next() {
        if legacy.is_file() {
            return Err("Project has both an .epokproject descriptor and legacy ProjectSettings/project.json. Use --recover-project <folder> --prefer descriptor (or legacy); the other manifest is preserved in .epok/migrations.".into());
        }
        if path.is_file() && descriptor != input {
            return Err("The selected descriptor is not the project's active descriptor.".into());
        }
        Ok((root, descriptor))
    } else if path.is_file() {
        Err("The selected .epokproject descriptor does not exist.".into())
    } else if legacy.is_file() {
        Ok((root, legacy))
    } else {
        Err("Not an Epok project: expected one .epokproject descriptor or ProjectSettings/project.json.".into())
    }
}

pub fn manifest_path(root: &Path) -> Result<PathBuf, String> {
    resolve(root).map(|(_, manifest)| manifest)
}
/// Only descriptor-free scratch/test directories may use default settings.
/// Ambiguous or broken project settings must not silently select defaults.
pub fn optional_manifest(path: &Path) -> Result<Option<Manifest>, String> {
    if !path.exists() {
        return Ok(None);
    }
    if path.is_file() || path.join(LEGACY_MANIFEST).exists() || !descriptors(path)?.is_empty() {
        read_manifest(path).map(Some)
    } else {
        Ok(None)
    }
}

pub fn read_manifest(root: &Path) -> Result<Manifest, String> {
    let (root, path) = resolve(root)?;
    read_manifest_at(&root, &path)
}

fn read_manifest_at(root: &Path, path: &Path) -> Result<Manifest, String> {
    let manifest: Manifest = crate::document::from_slice(
        &fs::read(path).map_err(|e| format!("Not an Epok project ({}): {e}", path.display()))?,
    )
    .map_err(|e| format!("Invalid project settings: {e}"))?;
    if manifest.format_version != FORMAT {
        return Err(format!(
            "Unsupported project format {} (this editor supports {FORMAT}).",
            manifest.format_version
        ));
    }
    // No silent upgrades while Epok's serialization/runtime API is experimental.
    if manifest.editor_version != env!("CARGO_PKG_VERSION") {
        return Err(format!(
            "Project requires Epok {}; this editor is {}.",
            manifest.editor_version,
            env!("CARGO_PKG_VERSION")
        ));
    }
    validate_name(&manifest.name)?;
    manifest.rendering.validate()?;
    manifest.play.validate()?;
    manifest.transition.validate()?;
    if manifest.default_sound_bank.is_some_and(|id| id.is_nil()) { return Err("Project Default SoundBank UUID cannot be nil".into()); }
    scene_path(root, &manifest)?;
    Ok(manifest)
}

pub fn save_manifest(root: &Path, manifest: &Manifest) -> Result<(), String> {
    crate::settings::save_document(&manifest_path(root)?, manifest)
}

/// Explicit, recoverable migration. Reading a legacy project never invokes this.
pub fn migrate_legacy(root: &Path) -> Result<PathBuf, String> {
    let root = fs::canonicalize(root).map_err(|e| e.to_string())?;
    let _lock = lock_root(&root)?;
    let legacy = root.join(LEGACY_MANIFEST);
    if !legacy.is_file() {
        return Err("Legacy ProjectSettings/project.json was not found.".into());
    }
    if !descriptors(&root)?.is_empty() {
        return Err(
            "A .epokproject descriptor already exists; resolve the ambiguity before migrating."
                .into(),
        );
    }
    let manifest = read_manifest_at(&root, &legacy)?;
    Scene::load(&scene_path(&root, &manifest)?)?;
    let file_name = format!("{}.{}", manifest.name, DESCRIPTOR_EXTENSION);
    let descriptor = root.join(file_name);
    // Exclusive publication: an interrupted write leaves both active formats and
    // is recoverable explicitly. Reading never guesses which one wins.
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&descriptor)
        .map_err(|e| e.to_string())?;
    file.write_all(&crate::document::to_vec(&manifest)?)
        .map_err(|e| format!("Migration interrupted: {e}; recover with --prefer legacy"))?;
    file.sync_all().map_err(|e| e.to_string())?;
    drop(file);
    preserve_manifest(&root, &legacy)?;
    Ok(descriptor)
}

fn preserve_manifest(root: &Path, path: &Path) -> Result<PathBuf, String> {
    let directory = root.join(".epok/migrations");
    fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
    let backup = directory.join(format!(
        "{}-{}.backup",
        path.file_name().unwrap().to_string_lossy(),
        uuid::Uuid::new_v4()
    ));
    fs::rename(path, &backup).map_err(|e| {
        format!(
            "Could not preserve {}: {e}. Both formats remain; use --recover-project.",
            path.display()
        )
    })?;
    Ok(backup)
}
/// An explicit choice is required; retain the losing format byte-for-byte.
pub fn recover_migration(root: &Path, prefer: &str) -> Result<PathBuf, String> {
    let root = fs::canonicalize(root).map_err(|e| e.to_string())?;
    let _lock = lock_root(&root)?;
    let files = descriptors(&root)?;
    if files.len() != 1 || !root.join(LEGACY_MANIFEST).is_file() {
        return Err("Recovery requires exactly one descriptor and one legacy manifest. For multiple descriptors, move the unwanted files out of the project root explicitly.".into());
    }
    let (keep, other) = match prefer {
        "descriptor" => (files[0].clone(), root.join(LEGACY_MANIFEST)),
        "legacy" => (root.join(LEGACY_MANIFEST), files[0].clone()),
        _ => return Err("Choose --prefer descriptor or --prefer legacy.".into()),
    };
    let manifest = read_manifest_at(&root, &keep)?;
    Scene::load(&scene_path(&root, &manifest)?)?;
    preserve_manifest(&root, &other)?;
    Ok(keep)
}

pub fn scene_path(root: &Path, manifest: &Manifest) -> Result<PathBuf, String> {
    let relative = Path::new(&manifest.startup_scene);
    if relative
        .components()
        .any(|c| !matches!(c, Component::Normal(_)))
        || !relative.starts_with("assets/scenes")
        || !manifest.startup_scene.ends_with(".epokmap")
    {
        return Err("Startup scene must be a relative .epokmap path inside assets/scenes.".into());
    }
    let path = root.join(relative);
    if path.exists() {
        let resolved = fs::canonicalize(&path).map_err(|e| e.to_string())?;
        let assets = fs::canonicalize(root.join("assets")).map_err(|e| e.to_string())?;
        let canonical_root = fs::canonicalize(root).map_err(|e| e.to_string())?;
        if !assets.starts_with(&canonical_root) || !resolved.starts_with(assets) {
            return Err("Scene links must remain inside the project assets.".into());
        }
    }
    Ok(path)
}

pub fn startup_scene(root: &Path) -> Result<PathBuf, String> {
    let (root, path) = resolve(root)?;
    scene_path(&root, &read_manifest_at(&root, &path)?)
}

pub fn validate_name(name: &str) -> Result<(), String> {
    let reserved = name.split('.').next().unwrap_or("").to_ascii_uppercase();
    if name.trim().is_empty()
        || name != name.trim()
        || name.chars().count() > 80
        || name.ends_with('.')
        || name
            .chars()
            .any(|c| c.is_control() || "<>:\"/\\|?*".contains(c))
        || [
            "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7",
            "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
        ]
        .contains(&reserved.as_str())
    {
        return Err("Use a project name of 1-80 characters without path separators or reserved filename characters.".into());
    }
    Ok(())
}

#[derive(Clone, Copy, PartialEq)]
pub enum Template {
    Basic,
    Sample,
    ThirdPerson,
}

pub fn create(destination: &Path, name: &str, template: Template) -> Result<Project, String> {
    validate_name(name)?;
    // Claim a NEW directory exclusively. Never merge into or overwrite existing content.
    let parent = destination
        .parent()
        .ok_or("Choose a parent folder for the project.")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    fs::create_dir(destination).map_err(|e| {
        format!(
            "Choose a new project folder ({}): {e}",
            destination.display()
        )
    })?;
    for directory in ["assets/scenes", "assets/scripts", "ProjectSettings"] {
        fs::create_dir_all(destination.join(directory)).map_err(|e| e.to_string())?;
    }
    let scene = match template {
        Template::ThirdPerson => crate::third_person::create(destination)?,
        Template::Basic => {
            let mut scene = Scene {
                name: "Main".into(),
                ..Scene::default()
            };
            scene.entities.truncate(1);
            scene
        }
        Template::Sample => crate::document::from_str(include_str!(
            "../examples/sample-game/assets/scenes/SampleScene.epokmap"
        ))
        .map_err(|e| e.to_string())?,
    };
    if template == Template::Sample {
        for (name, contents) in [
            (
                "Spinner.cpp",
                include_str!("../examples/sample-game/assets/scripts/Spinner.cpp"),
            ),
            (
                "Spinner.hpp",
                include_str!("../examples/sample-game/assets/scripts/Spinner.hpp"),
            ),
            (
                "Spinner.epokscript",
                include_str!("../examples/sample-game/assets/scripts/Spinner.epokscript"),
            ),
            (
                "Behaviour001.cpp",
                include_str!("../examples/sample-game/assets/scripts/Behaviour001.cpp"),
            ),
            (
                "Behaviour001.hpp",
                include_str!("../examples/sample-game/assets/scripts/Behaviour001.hpp"),
            ),
            (
                "Behaviour001.epokscript",
                include_str!("../examples/sample-game/assets/scripts/Behaviour001.epokscript"),
            ),
        ] {
            write_changed(
                &destination.join("assets/scripts").join(name),
                contents.as_bytes(),
            )?;
        }
    }
    let manifest = Manifest {
        format_version: FORMAT,
        editor_version: env!("CARGO_PKG_VERSION").into(),
        name: name.into(),
        startup_scene: format!("assets/scenes/{}.epokmap", scene.name),
        auto_build: false,
        build: Default::default(),
        play: Default::default(),
        transition: Default::default(),
        rendering: Default::default(),
        debug: Default::default(),
        default_sound_bank: None,
        default_scene_script_parent: None,
    };
    scene.save(&destination.join(&manifest.startup_scene))?;
    write_changed(
        &destination.join(".gitignore"),
        b"/.epok/\n/UserSettings/\n/exports/\n/artifacts/\n/Local.epokconfig\n",
    )?;
    write_changed(&destination.join("assets/scripts/.gitkeep"), b"")?;
    // Publish identity last; a failed creation is never mistaken for a valid project.
    write_changed(
        &destination.join(format!("{name}.{DESCRIPTOR_EXTENSION}")),
        &crate::document::to_vec(&manifest).map_err(|e| e.to_string())?,
    )?;
    Project::open(destination)
}

/// Editor installation/tool configuration; deliberately independent of the game and CWD.
pub fn editor_home() -> PathBuf {
    if let Some(path) = std::env::var_os("EPOK_EDITOR_HOME") {
        return path.into();
    }
    if let Ok(exe) = std::env::current_exe() {
        for parent in exe.ancestors().skip(1) {
            if parent.join("Editor.epokconfig").is_file() {
                return parent.to_path_buf();
            }
            // macOS bundles keep distribution resources below Contents. This
            // preserves the normal configuration-relative layout without
            // placing arbitrary files at the root of the .app bundle.
            let bundled = parent.join("Resources/Epok");
            if bundled.join("Editor.epokconfig").is_file() {
                return bundled;
            }
        }
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn user_data() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .or_else(|| std::env::var_os("XDG_DATA_HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|| user_home().join(".local/share"))
        .join("Epok")
}
pub fn user_home() -> PathBuf {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Recent {
    pub name: String,
    pub path: PathBuf,
}
pub fn recent_projects() -> Result<Vec<Recent>, String> {
    read_recent(&user_data().join("RecentProjects.epokprefs"))
}
pub fn read_recent(path: &Path) -> Result<Vec<Recent>, String> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    crate::document::from_slice(&fs::read(path).map_err(|e| e.to_string())?)
        .map_err(|e| format!("Recent projects: {e}"))
}
pub fn remember(project: &Project) -> Result<(), String> {
    update_recent(
        &user_data().join("RecentProjects.epokprefs"),
        &project.root,
        Some(&project.manifest.name),
    )
}
pub fn update_recent(path: &Path, project: &Path, name: Option<&str>) -> Result<(), String> {
    use std::io::Write;
    fs::create_dir_all(
        path.parent()
            .ok_or("Recent-project registry needs a parent directory")?,
    )
    .map_err(|e| e.to_string())?;
    let lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path.with_extension("lock"))
        .map_err(|e| e.to_string())?;
    lock.try_lock()
        .map_err(|e| format!("Recent projects are being updated: {e}"))?;
    let project = resolve(project)
        .map(|(root, _)| root)
        .or_else(|_| fs::canonicalize(project).map_err(|e| e.to_string()))
        .unwrap_or_else(|_| project.to_path_buf());
    let mut entries = read_recent(path)?;
    for entry in &mut entries {
        entry.path = resolve(&entry.path)
            .map(|(root, _)| root)
            .or_else(|_| fs::canonicalize(&entry.path).map_err(|e| e.to_string()))
            .unwrap_or_else(|_| entry.path.clone());
    }
    entries.retain(|e| e.path != project);
    if let Some(name) = name {
        entries.insert(
            0,
            Recent {
                name: name.into(),
                path: project,
            },
        );
    }
    entries.truncate(20);
    let temporary = path.with_extension("tmp");
    let mut file = fs::File::create(&temporary).map_err(|e| e.to_string())?;
    file.write_all(&crate::document::to_vec(&entries).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    file.sync_all().map_err(|e| e.to_string())?;
    drop(file);
    fs::rename(temporary, path).map_err(|e| e.to_string())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    pub fn temp(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "epok-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }
    #[test]
    fn independent_projects_lock_relocate_and_regenerate() {
        let a = temp("project-a");
        let b = temp("project-b");
        let project = create(&a, "My Game", Template::Sample).unwrap();
        assert!(Project::open(&a).is_err());
        assert!(!a.join("runtime").exists());
        assert!(!a.join("src").exists());
        let scene = Scene::load(&startup_scene(&a).unwrap()).unwrap();
        crate::project::stage(&a, &scene).unwrap();
        assert!(a.join(".epok/build/scripts/Spinner.cpp").is_file());
        drop(project);
        fs::remove_dir_all(a.join(".epok")).unwrap();
        fs::rename(&a, &b).unwrap();
        let project = Project::open(&b).unwrap();
        crate::project::stage(&b, &scene).unwrap();
        assert!(create(&b, "Other", Template::Basic).is_err());
        assert_eq!(project.manifest.name, "My Game");
        drop(project);
        fs::remove_dir_all(b).unwrap();
    }
    #[test]
    fn two_projects_keep_sources_and_tool_configuration_independent() {
        let a = temp("isolated-a");
        let b = temp("isolated-b");
        let first = create(&a, "First", Template::Basic).unwrap();
        let second = create(&b, "Second", Template::Sample).unwrap();
        let original = fs::read(startup_scene(&b).unwrap()).unwrap();
        let mut editor = crate::editor::Editor::open(first).unwrap();
        editor.scene.name = "First level".into();
        editor.changed();
        assert!(editor.save());
        // Project isolation is independent of the optional host reflection tools.
        // The native creation path is verified by verify_reflection.py.
        for ext in ["hpp", "cpp", "epokscript"] {
            fs::copy(
                b.join(format!("assets/scripts/Behaviour001.{ext}")),
                a.join(format!("assets/scripts/Behaviour001.{ext}")),
            )
            .unwrap();
        }
        assert_eq!(crate::scripts::catalog(&a).unwrap().len(), 2);
        assert_eq!(crate::scripts::catalog(&b).unwrap().len(), 2);
        assert!(!b.join("assets/scripts/Behaviour002.cpp").exists());
        assert_eq!(fs::read(startup_scene(&b).unwrap()).unwrap(), original);
        let config_a = crate::project::Config::load(&a).unwrap();
        let config_b = crate::project::Config::load(&b).unwrap();
        assert_eq!(config_a.nugget, config_b.nugget);
        assert_eq!(config_a.make, config_b.make);
        assert!(!Path::new(&config_a.nugget).starts_with(&a));
        // The game cannot accidentally replace the editor-owned runtime snapshot.
        fs::create_dir_all(a.join("runtime")).unwrap();
        fs::write(a.join("runtime/epok.hpp"), "wrong runtime").unwrap();
        crate::project::stage(&a, &editor.scene).unwrap();
        assert_eq!(
            fs::read(a.join(".epok/build/epok.hpp")).unwrap(),
            include_bytes!("../runtime/epok.hpp")
        );
        drop(editor);
        assert!(Project::open(&a).is_ok());
        assert!(Project::open(&b).is_err());
        drop(second);
        fs::remove_dir_all(a).unwrap();
        fs::remove_dir_all(b).unwrap();
    }
    #[test]
    fn invalid_settings_and_missing_scenes_never_open_or_overwrite() {
        let root = temp("invalid-project");
        assert!(Project::open(&root).is_err());
        let project = create(&root, "Game", Template::Basic).unwrap();
        let mut manifest = project.manifest.clone();
        drop(project);
        let original = fs::read(root.join(&manifest.startup_scene)).unwrap();
        for scene in [
            "../outside.epokmap",
            "assets/scenes/../../../outside.epokmap",
            "C:/outside.epokmap",
        ] {
            manifest.startup_scene = scene.into();
            write_changed(
                &root.join("Game.epokproject"),
                &serde_json::to_vec(&manifest).unwrap(),
            )
            .unwrap();
            assert!(Project::open(&root).is_err());
        }
        manifest.startup_scene = "assets/scenes/Main.epokmap".into();
        manifest.format_version = FORMAT + 1;
        write_changed(
            &root.join("Game.epokproject"),
            &serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        assert!(Project::open(&root).is_err());
        assert_eq!(
            fs::read(root.join(&manifest.startup_scene)).unwrap(),
            original
        );
        manifest.format_version = FORMAT;
        manifest.editor_version = "999.0.0".into();
        write_changed(
            &root.join("Game.epokproject"),
            &serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        assert!(Project::open(&root).is_err());
        manifest.editor_version = env!("CARGO_PKG_VERSION").into();
        fs::remove_file(root.join(&manifest.startup_scene)).unwrap();
        write_changed(
            &root.join("Game.epokproject"),
            &serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        assert!(Project::open(&root).is_err());
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn invalid_names_are_rejected() {
        for name in [
            "", "../Game", "a/b", "a\\b", "CON", "NUL.txt", "a:", "Game.", " Game",
        ] {
            assert!(validate_name(name).is_err(), "{name}");
        }
        assert!(validate_name("My Game").is_ok());
    }

    #[test]
    fn recovery_preserves_bytes_rejects_ambiguity_and_honors_alias_lock() {
        let root = temp("recovery Unicode ゲーム");
        let project = create(&root, "Recovery Game", Template::Basic).unwrap();
        let descriptor = manifest_path(&root).unwrap();
        assert_eq!(
            read_manifest(&descriptor).unwrap().name,
            project.manifest.name
        );
        assert_eq!(
            startup_scene(&descriptor).unwrap(),
            startup_scene(&root).unwrap()
        );
        assert!(Project::open(&descriptor).is_err());
        let original = fs::read(&descriptor).unwrap();
        fs::write(root.join(LEGACY_MANIFEST), &original).unwrap();
        assert!(
            recover_migration(&root, "descriptor")
                .unwrap_err()
                .contains("locked")
        );
        drop(project);
        assert!(read_manifest(&root).unwrap_err().contains("both"));
        assert!(crate::settings::rendering(&root).is_err());
        assert!(crate::project::Config::load(&root).is_err());
        assert!(recover_migration(&root, "automatic").is_err());
        fs::write(&descriptor, b"{interrupted publication").unwrap();
        assert!(recover_migration(&root, "descriptor").is_err());
        assert_eq!(
            recover_migration(&root, "legacy").unwrap(),
            fs::canonicalize(&root).unwrap().join(LEGACY_MANIFEST)
        );
        assert_eq!(fs::read(root.join(LEGACY_MANIFEST)).unwrap(), original);
        assert!(Project::open(&root).is_ok());
        let descriptor = migrate_legacy(&root).unwrap();
        assert_eq!(fs::read(&descriptor).unwrap(), original);
        let backups = fs::read_dir(root.join(".epok/migrations"))
            .unwrap()
            .map(|e| fs::read(e.unwrap().path()).unwrap())
            .collect::<Vec<_>>();
        assert!(backups.contains(&original));
        assert!(backups.contains(&b"{interrupted publication".to_vec()));
        fs::write(root.join("Second.EPOKPROJECT"), &original).unwrap();
        assert!(resolve(&descriptor).unwrap_err().contains("multiple"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn descriptor_aliases_migration_and_recents_share_a_canonical_root() {
        let root = temp("descriptor aliases");
        let project = create(&root, "Non ASCII ゲーム", Template::Basic).unwrap();
        let descriptor = manifest_path(&root).unwrap();
        assert_eq!(resolve(&root).unwrap().0, resolve(&descriptor).unwrap().0);
        drop(project);
        fs::remove_dir_all(root.join(".epok")).unwrap();
        let manifest = fs::read(&descriptor).unwrap();
        fs::remove_file(&descriptor).unwrap();
        crate::project::write_changed(&root.join(LEGACY_MANIFEST), &manifest).unwrap();
        let legacy_project = Project::open(&root).unwrap();
        drop(legacy_project);
        let descriptor = migrate_legacy(&root).unwrap();
        assert!(descriptor.is_file());
        assert_eq!(
            fs::read_dir(root.join(".epok/migrations")).unwrap().count(),
            1
        );
        let registry = root.join("recent.json");
        update_recent(&registry, &root, Some("Game")).unwrap();
        update_recent(&registry, &descriptor, Some("Game")).unwrap();
        assert_eq!(read_recent(&registry).unwrap().len(), 1);
        fs::remove_dir_all(root).unwrap();
    }
}
