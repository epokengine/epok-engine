//! Project-owned Play choices, independent of machine-local transport settings.
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Target {
    #[default]
    Embedded,
    Window,
    Serial,
}
impl Target {
    pub const ALL: [Self; 3] = [Self::Embedded, Self::Window, Self::Serial];
    pub fn label(self) -> &'static str {
        match self {
            Self::Embedded => "Embedded emulator",
            Self::Window => "Emulator window",
            Self::Serial => "PSX via serial",
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Content {
    #[default]
    CurrentScene,
    WholeGame,
    SelectedScenes,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataSource {
    #[default]
    Executable,
    Disc,
    Host,
}
impl DataSource {
    pub const ALL: [Self; 3] = [Self::Executable, Self::Disc, Self::Host];
    pub fn label(self) -> &'static str {
        match self {
            Self::Executable => "In executable",
            Self::Disc => "CD on demand",
            Self::Host => "PC on demand",
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Profile {
    pub target: Target,
    pub content: Content,
    pub data: DataSource,
    pub selected_scenes: Vec<String>,
    pub initial_scene: Option<String>,
    /// Opt-in analog first controller for emulator Play; physical hardware is unchanged.
    #[serde(skip_serializing_if = "is_false")]
    pub analog_controller: bool,
}
fn is_false(value: &bool) -> bool {
    !*value
}
impl Profile {
    pub fn normalize(&mut self) {
        if self.target == Target::Serial && self.data == DataSource::Disc {
            self.data = DataSource::Host;
        }
        if !self
            .initial_scene
            .as_ref()
            .is_some_and(|p| self.selected_scenes.contains(p))
        {
            self.initial_scene = self.selected_scenes.first().cloned();
        }
    }
    pub fn set_scene_included(&mut self, path: &str, included: bool) {
        if included {
            if !self.selected_scenes.iter().any(|p| p == path) {
                self.selected_scenes.push(path.into());
                self.selected_scenes.sort();
            }
        } else if let Some(index) = self.selected_scenes.iter().position(|p| p == path) {
            self.selected_scenes.remove(index);
            if self.initial_scene.as_deref() == Some(path) {
                self.initial_scene = self
                    .selected_scenes
                    .get(index)
                    .or_else(|| self.selected_scenes.first())
                    .cloned();
            }
        }
        self.normalize();
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.target == Target::Serial && self.data == DataSource::Disc {
            return Err(
                "Serial Play cannot use CD data. Choose PC on demand or In executable.".into(),
            );
        }
        let mut unique = std::collections::BTreeSet::new();
        for path in &self.selected_scenes {
            let p = Path::new(path);
            if p.components()
                .any(|c| !matches!(c, std::path::Component::Normal(_)))
                || !p.starts_with("assets/scenes")
                || !path.ends_with(".epokmap")
                || !unique.insert(path.to_lowercase())
            {
                return Err(
                    "Selected scenes require unique relative assets/scenes/*.epokmap paths.".into(),
                );
            }
        }
        Ok(())
    }
    pub fn validate_build(&self, root: &Path) -> Result<(), String> {
        self.validate()?;
        if self.content == Content::SelectedScenes {
            if self.selected_scenes.is_empty() {
                return Err("Selected Scenes cannot be used because no scenes are selected. Select at least one scene in Play options.".into());
            }
            if self.selected_scenes.len() > 16 {
                return Err("Selected Scenes supports at most 16 scenes.".into());
            }
            let base =
                std::fs::canonicalize(root.join("assets/scenes")).map_err(|e| e.to_string())?;
            for path in &self.selected_scenes {
                let resolved = std::fs::canonicalize(root.join(path))
                    .map_err(|e| format!("Selected scene {path}: {e}"))?;
                if !resolved.starts_with(&base) || !resolved.is_file() {
                    return Err(format!(
                        "Selected scene {path} must be a file inside assets/scenes."
                    ));
                }
            }
        }
        Ok(())
    }
    pub fn load(root: &Path) -> Result<Self, String> {
        Ok(crate::workspace::optional_manifest(root)?
            .map(|m| m.play)
            .unwrap_or_default())
    }
}

fn initial_path(
    root: &Path,
    current: &Path,
    profile: &Profile,
) -> Result<std::path::PathBuf, String> {
    match profile.content {
        Content::CurrentScene => Ok(current.to_owned()),
        Content::WholeGame => crate::workspace::startup_scene(root),
        Content::SelectedScenes => profile
            .initial_scene
            .as_ref()
            .map(|p| root.join(p))
            .ok_or_else(|| "Selected Scenes has no initial scene.".into()),
    }
}

/// Headless requests use saved provenance while resolving the same scene scope as the editor.
pub fn saved_input(
    root: &Path,
    current: &Path,
    mut profile: Profile,
) -> Result<crate::scene_dependencies::Input, String> {
    profile.normalize();
    profile.validate_build(root)?;
    let mut input =
        crate::scene_dependencies::Input::load(&initial_path(root, current, &profile)?)?;
    input.play = Some(profile);
    input.play_settings_signature = Some(settings_signature(root)?);
    Ok(input)
}

pub fn input(
    root: &Path,
    path: std::path::PathBuf,
    scene: crate::scene::Scene,
    mut profile: Profile,
    physical_disc: bool,
) -> Result<crate::scene_dependencies::Input, String> {
    if physical_disc {
        profile.target = Target::Embedded;
        profile.content = Content::WholeGame;
        profile.data = DataSource::Disc;
    }
    profile.normalize();
    profile.validate_build(root)?;
    let settings_signature = settings_signature(root)?;
    let mut input = crate::scene_dependencies::Input::editor(path.clone(), scene.clone());
    if profile.content != Content::CurrentScene {
        let startup = initial_path(root, &path, &profile)?;
        if std::fs::canonicalize(&startup).ok() != std::fs::canonicalize(&path).ok()
            || !path.exists()
        {
            input = crate::scene_dependencies::Input::load(&startup)?;
            let included = profile.content != Content::SelectedScenes
                || std::fs::canonicalize(&path).ok().is_some_and(|open| {
                    profile.selected_scenes.iter().any(|entry| {
                        std::fs::canonicalize(root.join(entry)).is_ok_and(|entry| entry == open)
                    })
                });
            if included {
                input.editor_override = Some((path, scene));
            }
        }
    }
    input.play = Some(profile);
    input.play_settings_signature = Some(settings_signature);
    Ok(input)
}

pub fn settings_signature(root: &Path) -> Result<String, String> {
    Ok(crate::scene_dependencies::hash(
        crate::workspace::optional_manifest(root)?.map(|m| (m.play, m.startup_scene)),
    ))
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Transition {
    pub fade_out_ms: u16,
    pub fade_in_ms: u16,
    pub text: String,
    /// Optional imported texture UUID. Kept outside scene lifetime.
    pub image: Option<uuid::Uuid>,
}
impl Default for Transition {
    fn default() -> Self {
        Self {
            fade_out_ms: 300,
            fade_in_ms: 300,
            text: "Now loading...".into(),
            image: None,
        }
    }
}
impl Transition {
    pub fn validate(&self) -> Result<(), String> {
        if self.fade_out_ms > 10000 || self.fade_in_ms > 10000 {
            return Err("Transition fades must be between 0 and 10000 milliseconds.".into());
        }
        if self.text.len() > 95 || !self.text.bytes().all(|b| (32..=126).contains(&b)) {
            return Err("Loading text must contain at most 95 printable ASCII characters.".into());
        }
        Ok(())
    }
    pub fn stage_image(
        &self,
        _root: &Path,
        build: &Path,
        index: &crate::assets::Index,
    ) -> Result<crate::playback_staging::ResourceOutput, String> {
        let mut inputs = std::collections::BTreeMap::from([(
            "scene-transition-settings".into(),
            crate::scene_dependencies::hash(self),
        )]);
        let mut header =
            String::from("#pragma once\n#include \"transition.hpp\"\nnamespace epok {\n");
        if let Some(id) = self.image {
            let record = index.resolve(id)?;
            let package = crate::assets::Package::load(&record.path)?;
            if package.meta.id != id || package.meta.kind != crate::assets::Kind::Texture {
                return Err("Loading image must reference a Texture asset.".into());
            }
            let data = crate::texture::decode(&package.source)?;
            if data.width > 64 || data.height > 64 || data.width % 2 != 0 {
                return Err(
                    "Loading image must be at most 64 x 64 pixels with an even width.".into(),
                );
            }
            inputs.insert(
                format!("asset:{id}"),
                crate::assets::cache_key(&package.meta),
            );
            let pixels = data
                .rgba
                .chunks_exact(4)
                .map(|p| {
                    if p[3] < 128 {
                        0
                    } else {
                        0x8000u16
                            | u16::from(p[0] >> 3)
                            | (u16::from(p[1] >> 3) << 5)
                            | (u16::from(p[2] >> 3) << 10)
                    }
                })
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join(",");
            header += &format!(
                "alignas(4) inline constexpr uint16_t loading_pixels[]={{{pixels}}};\ninline constexpr LoadingImage default_loading_image{{loading_pixels,{},{}}};\n",
                data.width, data.height
            );
        } else {
            header += "inline constexpr LoadingImage default_loading_image{};\n";
        }
        header += "}\n";
        crate::project::write_changed(&build.join("loading-image.hh"), header.as_bytes())?;
        Ok(crate::playback_staging::ResourceOutput {
            path: "loading-image.hh".into(),
            signature: crate::assets::hash(header.as_bytes()),
            inputs,
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Serial {
    /// Legacy import hint. Verified components are copied into this editor's installation.
    pub executable: String,
    pub port: String,
    pub fast: bool,
    /// USB identity when the adapter supplies a serial number. Never shared with the project.
    pub device_id: Option<String>,
}
impl Serial {
    pub fn validate(&self) -> Result<(), String> {
        if self.port.trim().is_empty() {
            return Err(
                "Choose your adapter in PSX connection, then start the Unirom loader.".into(),
            );
        }
        if self.port.chars().any(char::is_control) {
            return Err("The serial port cannot contain control characters.".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn analog_controller_is_opt_in_and_round_trips() {
        let mut profile: Profile = serde_json::from_str("{}").unwrap();
        assert!(!profile.analog_controller);
        assert!(
            serde_json::to_value(&profile)
                .unwrap()
                .get("analog_controller")
                .is_none()
        );
        profile.analog_controller = true;
        let restored: Profile =
            serde_json::from_value(serde_json::to_value(&profile).unwrap()).unwrap();
        assert_eq!(restored, profile);
    }
    #[test]
    fn selected_scene_initial_follows_next_included_and_empty_is_persistable() {
        let mut profile = Profile {
            content: Content::SelectedScenes,
            ..Default::default()
        };
        assert!(profile.validate().is_ok());
        assert!(
            profile
                .validate_build(Path::new("unused"))
                .unwrap_err()
                .contains("no scenes")
        );
        for path in [
            "assets/scenes/A.epokmap",
            "assets/scenes/B.epokmap",
            "assets/scenes/C.epokmap",
        ] {
            profile.set_scene_included(path, true);
        }
        profile.initial_scene = Some("assets/scenes/B.epokmap".into());
        profile.set_scene_included("assets/scenes/B.epokmap", false);
        assert_eq!(
            profile.initial_scene.as_deref(),
            Some("assets/scenes/C.epokmap")
        );
        profile.set_scene_included("assets/scenes/C.epokmap", false);
        assert_eq!(
            profile.initial_scene.as_deref(),
            Some("assets/scenes/A.epokmap")
        );
        profile.set_scene_included("assets/scenes/A.epokmap", false);
        assert_eq!(profile.initial_scene, None);
        let restored: Profile =
            serde_json::from_slice(&serde_json::to_vec(&profile).unwrap()).unwrap();
        assert_eq!(restored, profile);
    }
    #[test]
    fn selected_build_uses_initial_and_unsaved_included_map_without_unselected_maps() {
        let root = crate::workspace::tests::temp("selected-scenes");
        let project =
            crate::workspace::create(&root, "Selection", crate::workspace::Template::Basic)
                .unwrap();
        let startup = crate::workspace::scene_path(&root, &project.manifest).unwrap();
        let mut scene = crate::scene::Scene::load(&startup).unwrap();
        for name in ["A", "B", "Excluded"] {
            scene.name = name.into();
            scene
                .save(&root.join(format!("assets/scenes/{name}.epokmap")))
                .unwrap();
        }
        let mut profile = Profile {
            content: Content::SelectedScenes,
            ..Default::default()
        };
        profile.set_scene_included("assets/scenes/A.epokmap", true);
        profile.set_scene_included("assets/scenes/B.epokmap", true);
        profile.initial_scene = Some("assets/scenes/B.epokmap".into());
        let mut manifest = project.manifest.clone();
        manifest.play = profile.clone();
        crate::workspace::save_manifest(&root, &manifest).unwrap();
        assert_eq!(Profile::load(&root).unwrap(), profile);
        scene.name = "Unsaved A".into();
        let request = input(
            &root,
            root.join("assets/scenes/A.epokmap"),
            scene.clone(),
            profile.clone(),
            false,
        )
        .unwrap();
        let loaded = crate::scene_bank::load_selected(
            &root,
            &request.scene,
            &request.origin,
            Some(&request),
        )
        .unwrap();
        assert_eq!(
            loaded
                .scenes
                .iter()
                .map(|s| s.name.as_str())
                .collect::<Vec<_>>(),
            ["B", "Unsaved A"]
        );
        let initial_edit = input(
            &root,
            root.join("assets/scenes/B.epokmap"),
            scene.clone(),
            profile.clone(),
            false,
        )
        .unwrap();
        assert_eq!(initial_edit.scene.name, "Unsaved A");
        let excluded = input(
            &root,
            root.join("assets/scenes/Excluded.epokmap"),
            scene.clone(),
            profile.clone(),
            false,
        )
        .unwrap();
        assert!(excluded.editor_override.is_none());
        let headless = saved_input(&root, &startup, profile.clone()).unwrap();
        assert_eq!(headless.scene.name, "B");
        assert!(matches!(
            headless.origin,
            crate::scene_dependencies::Origin::Saved(..)
        ));
        let config = crate::project::Config::load(&root).unwrap();
        let original = crate::play_cache::request(&root, &request, &config, false).unwrap();
        profile.initial_scene = Some("assets/scenes/A.epokmap".into());
        let changed = input(
            &root,
            root.join("assets/scenes/A.epokmap"),
            scene.clone(),
            profile.clone(),
            false,
        )
        .unwrap();
        assert_ne!(
            original,
            crate::play_cache::request(&root, &changed, &config, false).unwrap()
        );
        profile.content = Content::WholeGame;
        let whole = input(&root, startup, scene, profile.clone(), false).unwrap();
        let loaded =
            crate::scene_bank::load_selected(&root, &whole.scene, &whole.origin, Some(&whole))
                .unwrap();
        assert_eq!(
            loaded.scenes.len(),
            4,
            "Whole Game must discover unregistered maps"
        );
        profile.content = Content::SelectedScenes;
        profile.set_scene_included("assets/scenes/Missing.epokmap", true);
        assert!(
            profile
                .validate_build(&root)
                .unwrap_err()
                .contains("Missing")
        );
        profile.selected_scenes = vec!["assets/scenes/../../outside.epokmap".into()];
        assert!(profile.validate().is_err());
        drop(project);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn project_profile_persists_and_full_game_preserves_the_open_map() {
        let root = crate::workspace::tests::temp("play-profile");
        let project =
            crate::workspace::create(&root, "Play profile", crate::workspace::Template::Basic)
                .unwrap();
        let manifest = project.manifest.clone();
        let startup = crate::workspace::scene_path(&root, &manifest).unwrap();
        let mut other = crate::scene::Scene::load(&startup).unwrap();
        other.name = "Other".into();
        let path = root.join("assets/scenes/Other.epokmap");
        other.save(&path).unwrap();
        crate::scene_bank::Registry {
            scenes: vec!["assets/scenes/Other.epokmap".into()],
        }
        .save(&root)
        .unwrap();
        let mut editor = crate::editor::Editor::open(project).unwrap();
        let profile = Profile {
            target: Target::Serial,
            content: Content::WholeGame,
            data: DataSource::Host,
            ..Default::default()
        };
        editor.set_play_profile(profile.clone()).unwrap();
        drop(editor);
        let reopened = crate::workspace::Project::open(&root).unwrap();
        assert_eq!(reopened.manifest.play, profile);
        drop(reopened);
        other.actors[0].name = "Unsaved edit".into();
        let request = input(&root, path.clone(), other.clone(), profile, false).unwrap();
        assert_ne!(request.scene.name, "Other");
        let loaded = crate::scene_bank::load_selected(
            &root,
            &request.scene,
            &request.origin,
            Some(&request),
        )
        .unwrap();
        assert_eq!(loaded.scenes.len(), 2);
        assert_eq!(loaded.scenes[1].actors[0].name, "Unsaved edit");
        assert!(matches!(
            loaded.sources[0],
            crate::scene_dependencies::Origin::Editor(..)
        ));
        let only = input(&root, path, other, Profile::default(), false).unwrap();
        std::fs::write(root.join(crate::scene_bank::REGISTRY), "invalid registry").unwrap();
        let loaded =
            crate::scene_bank::load_selected(&root, &only.scene, &only.origin, Some(&only))
                .unwrap();
        assert_eq!(loaded.scenes.len(), 1);
        assert!(
            crate::scene_bank::load_selected(
                &root,
                &request.scene,
                &request.origin,
                Some(&request)
            )
            .is_err()
        );
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn serial_cd_is_rejected_and_ui_normalization_keeps_content() {
        let mut p = Profile {
            target: Target::Serial,
            content: Content::WholeGame,
            data: DataSource::Disc,
            ..Default::default()
        };
        assert!(p.validate().is_err());
        p.normalize();
        assert_eq!(p.data, DataSource::Host);
        assert_eq!(p.content, Content::WholeGame);
        assert!(p.validate().is_ok());
    }
    #[test]
    fn legacy_profile_defaults_to_current_scene_and_transition_text_is_bounded() {
        assert_eq!(
            serde_json::from_str::<Profile>("{}").unwrap(),
            Profile::default()
        );
        let transition = Transition {
            text: "x".repeat(96),
            ..Default::default()
        };
        assert!(transition.validate().is_err());
    }
}
