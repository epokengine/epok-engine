//! Project-owned output configuration and machine-local editor preferences.
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

/// Compile-time runtime overlays. Disabled overlays allocate no packets.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct DebugHud {
    pub fps: bool,
    pub cpu: bool,
    pub gte: bool,
    pub gpu: bool,
    pub spu_ram: bool,
}
impl DebugHud {
    pub fn header(self) -> String {
        format!("#pragma once\n#define EPOK_DEBUG_FPS {}\n#define EPOK_DEBUG_CPU {}\n#define EPOK_DEBUG_GTE {}\n#define EPOK_DEBUG_GPU {}\n#define EPOK_DEBUG_SPU {}\n", u8::from(self.fps), u8::from(self.cpu), u8::from(self.gte), u8::from(self.gpu), u8::from(self.spu_ram))
    }
}
pub fn debug_hud(root: &Path) -> Result<DebugHud, String> {
    crate::workspace::optional_manifest(root).map(|m| m.map(|m| m.debug).unwrap_or_default())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Rendering {
    pub width: u16,
    pub height: u16,
    /// Static meshes keep their GPU packets across frames (see docs/performance.md).
    pub retained_geometry: bool,
    pub motion_interpolation: bool,
    pub precomputed_visibility: bool,
    pub streaming_geometry: bool,
    pub streaming_pool_pages: u8,
    pub streaming_triangle_budget: u16,
    pub streaming_prefetch: bool,
}
impl Default for Rendering {
    fn default() -> Self {
        Self {
            width: 640,
            height: 480,
            retained_geometry: true,
            motion_interpolation: true,
            precomputed_visibility: false,
            streaming_geometry: false,
            streaming_pool_pages: 4,
            streaming_triangle_budget: 4096,
            streaming_prefetch: true,
        }
    }
}
impl Rendering {
    pub const MODES: [(u16, u16); 10] = [
        (256, 240),
        (320, 240),
        (368, 240),
        (512, 240),
        (640, 240),
        (256, 480),
        (320, 480),
        (368, 480),
        (512, 480),
        (640, 480),
    ];
    pub fn validate(self) -> Result<(), String> {
        if !(2..=8).contains(&self.streaming_pool_pages) {
            return Err("Geometry streaming pool must contain 2..8 pages of 64 KiB.".into());
        }
        if !(512..=8192).contains(&self.streaming_triangle_budget) {
            return Err("Streaming triangle budget must be between 512 and 8192.".into());
        }
        if Self::MODES.contains(&(self.width, self.height)) {
            Ok(())
        } else {
            Err("Choose a supported NTSC resolution (256/320/368/512/640 x 240 or 480).".into())
        }
    }
    pub fn label(self) -> String {
        format!(
            "{} x {}  /  {}",
            self.width,
            self.height,
            if self.height == 480 {
                "Interlaced"
            } else {
                "Progressive"
            }
        )
    }
    pub fn header(self) -> Result<String, String> {
        self.validate()?;
        Ok(format!(
            "// Generated from Project Settings.\n#pragma once\nnamespace epok {{\ninline constexpr int display_width = {};\ninline constexpr int display_height = {};\ninline constexpr bool display_interlaced = {};\ninline constexpr bool retained_geometry = {};\ninline constexpr bool motion_interpolation = {};\ninline constexpr bool precomputed_visibility = {};\ninline constexpr bool streaming_geometry = {};\ninline constexpr unsigned streaming_pool_pages = {};\ninline constexpr bool streaming_prefetch_enabled = {};\n}}\n",
            self.width,
            self.height,
            self.height == 480,
            self.retained_geometry,
            self.motion_interpolation,
            self.precomputed_visibility,
            self.streaming_geometry,
            self.streaming_pool_pages,
            self.streaming_prefetch
        ))
    }
}
pub fn rendering(root: &Path) -> Result<Rendering, String> {
    if let Some(manifest) = crate::workspace::optional_manifest(root)? {
        Ok(manifest.rendering)
    } else {
        Ok(Rendering::default())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Preferences {
    pub serial: crate::play::Serial,
    pub mcp: Mcp,
    pub show_grid: bool,
    pub fly_speed: f32,
    /// Legacy preference accepted for old files; game_scale owns presentation.
    pub integer_scale: bool,
    pub game_scale: GameScale,
    pub emulator_linear_filter: bool,
}
impl Default for Preferences {
    fn default() -> Self {
        Self {
            mcp: Mcp::default(),
            serial: Default::default(),
            show_grid: true,
            fly_speed: 5.,
            integer_scale: true,
            game_scale: GameScale::default(),
            emulator_linear_filter: false,
        }
    }
}
/// Presentation only. PSX video modes have non-square pixels: Fit/Integer
/// preserve the intended 4:3 display aspect, not the raw framebuffer ratio.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameScale {
    #[default]
    Fit,
    Stretch,
    Integer,
}
impl GameScale {
    pub const ALL: [Self; 3] = [Self::Fit, Self::Stretch, Self::Integer];
    pub fn label(self) -> &'static str {
        match self { Self::Fit => "Fit (4:3)", Self::Stretch => "Stretch", Self::Integer => "Integer (4:3)" }
    }
    pub fn image_size(self, frame: [u32; 2], available: [f32; 2]) -> [f32; 2] {
        let available = available.map(|v| if v.is_finite() {v.max(0.)} else {0.});
        if self == Self::Stretch { return available; }
        let width = (frame[0].max(1) as f32).max(frame[1].max(1) as f32 * 4. / 3.);
        let height = width * 3. / 4.;
        let mut scale = (available[0] / width).min(available[1] / height);
        if self == Self::Integer && scale >= 1. { scale = scale.floor(); }
        [width * scale, height * scale]
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Mcp {
    pub enabled: bool,
    pub port: u16,
    pub token: String,
}
impl Default for Mcp {
    fn default() -> Self {
        Self {
            enabled: false,
            port: 8765,
            token: String::new(),
        }
    }
}
impl Mcp {
    pub fn prepare(&mut self) {
        if self.enabled && self.token.is_empty() {
            self.rotate_token();
        }
    }
    pub fn rotate_token(&mut self) {
        self.token = format!(
            "{}{}",
            uuid::Uuid::new_v4().simple(),
            uuid::Uuid::new_v4().simple()
        );
    }
    pub fn endpoint(&self) -> String {
        format!("http://127.0.0.1:{}/mcp", self.port)
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.port < 1024 {
            return Err("MCP port must be between 1024 and 65535.".into());
        }
        if self.enabled
            && (self.token.len() < 32 || !self.token.bytes().all(|c| c.is_ascii_alphanumeric()))
        {
            return Err("Generate an MCP access key before enabling the server.".into());
        }
        Ok(())
    }
}
impl Preferences {
    pub fn load() -> Result<Self, String> {
        Self::read(&crate::workspace::user_data().join("Editor.epokprefs"))
    }
    fn read(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let value: Self = crate::document::from_slice(&fs::read(path).map_err(|e| e.to_string())?)
            .map_err(|e| format!("Editor preferences: {e}"))?;
        value.validate()?;
        Ok(value)
    }
    pub fn validate(&self) -> Result<(), String> {
        self.mcp.validate()?;
        if !self.fly_speed.is_finite() || !(0.01..=1000.).contains(&self.fly_speed) {
            return Err("Camera speed must be between 0.01 and 1000.".into());
        }
        Ok(())
    }
    pub fn save(&self) -> Result<(), String> {
        self.validate()?;
        save_document(
            &crate::workspace::user_data().join("Editor.epokprefs"),
            self,
        )
    }
}
pub fn save_document(path: &Path, value: &impl Serialize) -> Result<(), String> {
    let bytes = if path.extension().is_some_and(|ext| ext == "json") {
        serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?
    } else {
        crate::document::to_vec(value)?
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let temp = path.with_extension(format!("{}.tmp", uuid::Uuid::new_v4()));
    fs::write(&temp, bytes).map_err(|e| e.to_string())?;
    if let Err(error) = fs::rename(&temp, path) {
        let _ = fs::remove_file(temp);
        return Err(error.to_string());
    }
    Ok(())
}
pub fn configure_emulator(portable: &Path, preferences: &Preferences) -> Result<(), String> {
    let path = portable.join("pcsx.json");
    let mut value: serde_json::Value = if path.exists() {
        serde_json::from_slice(&fs::read(&path).map_err(|e| e.to_string())?)
            .map_err(|e| format!("Emulator settings: {e}"))?
    } else {
        serde_json::json!({})
    };
    let root = value
        .as_object_mut()
        .ok_or("Invalid emulator settings object")?;
    let gui = root
        .entry("emulator")
        .or_insert_with(|| serde_json::json!({}));
    gui.as_object_mut()
        .ok_or("Invalid emulator GUI settings")?
        .insert(
            "LinearFiltering".into(),
            preferences.emulator_linear_filter.into(),
        );
    save_document(&path, &value)
}

#[cfg(test)]
mod tests {
    #[test]
    fn game_view_scaling_fills_an_axis_or_stretches_without_cropping() {
        use super::GameScale::*;
        assert_eq!(Fit.image_size([640,480], [1000.,600.]), [800.,600.]);
        assert_eq!(Fit.image_size([640,480], [800.,1000.]), [800.,600.]);
        assert_eq!(Stretch.image_size([640,480], [1000.,600.]), [1000.,600.]);
        assert_eq!(Integer.image_size([640,480], [1000.,600.]), [640.,480.]);
        assert_eq!(Integer.image_size([640,480], [320.,240.]), [320.,240.]);
        for frame in [[256,240],[320,240],[512,480],[640,480]] {
            let size = Fit.image_size(frame, [800.,600.]);
            assert!((size[0]-800.).abs()<0.01 && (size[1]-600.).abs()<0.01);
        }
        assert_eq!(Fit.image_size([640,480], [0.,0.]), [0.,0.]);
        let legacy: super::Preferences = serde_json::from_str(r#"{"integer_scale":true}"#).unwrap();
        assert_eq!(legacy.game_scale, Fit);
        for mode in super::GameScale::ALL {
            let prefs=super::Preferences{game_scale:mode,..Default::default()};
            assert_eq!(serde_json::from_str::<super::Preferences>(&serde_json::to_string(&prefs).unwrap()).unwrap(),prefs);
        }
    }
    #[test]
    fn debug_hud_defaults_and_independent_compile_toggles() {
        let defaults: super::DebugHud = serde_json::from_str("{}").unwrap();
        assert_eq!(defaults, super::DebugHud::default());
        for (field, define) in [("fps", "FPS"), ("cpu", "CPU"), ("gte", "GTE"), ("gpu", "GPU"), ("spu_ram", "SPU")] {
            let value = serde_json::json!({field:true});
            let options: super::DebugHud = serde_json::from_value(value).unwrap();
            let header = options.header();
            assert!(header.contains(&format!("#define EPOK_DEBUG_{define} 1\n")));
            assert_eq!(header.lines().filter(|line| line.ends_with(" 1")).count(), 1);
            assert_eq!(serde_json::from_str::<super::DebugHud>(&serde_json::to_string(&options).unwrap()).unwrap(), options);
        }
    }
    #[test]
    fn debug_hud_settings_invalidate_staged_inputs_and_persist() {
        use crate::artifact_dependencies::{self, Graph};
        let root = crate::workspace::tests::temp("debug-hud-provenance");
        let project = crate::workspace::create(&root, "Debug HUD", crate::workspace::Template::Basic).unwrap();
        let mut manifest = project.manifest.clone();
        drop(project);
        let path = root.join(&manifest.startup_scene);
        let scene = crate::scene::Scene::load(&path).unwrap();
        artifact_dependencies::transaction(&root, |graph| {
            graph.publish("scene-debug-settings", crate::scene_dependencies::hash(manifest.debug), Default::default());
            graph.publish("stage:test", "previous".into(), ["scene-debug-settings".into()].into_iter().collect());
        }).unwrap();
        manifest.debug.cpu = true;
        crate::workspace::save_manifest(&root, &manifest).unwrap();
        assert_eq!(super::debug_hud(&root).unwrap(), manifest.debug);
        crate::scene_dependencies::observe(&root, &path, &scene).unwrap();
        assert!(!Graph::load(&root).unwrap().nodes["stage:test"].stale.is_empty());
    }
    use super::*;
    #[test]
    fn geometry_settings_migrate_and_validate_pool_limits() {
        let mut settings: Rendering =
            serde_json::from_str("{\"width\":320,\"height\":240}").unwrap();
        assert!(!settings.precomputed_visibility);
        assert!(settings.motion_interpolation);
        assert!(!settings.streaming_geometry);
        assert_eq!(settings.streaming_pool_pages, 4);
        assert_eq!(settings.streaming_triangle_budget, 4096);
        assert!(settings.streaming_prefetch);
        for pages in [0, 1, 9, 255] {
            settings.streaming_pool_pages = pages;
            assert!(settings.validate().is_err());
        }
        for pages in 2..=8 {
            settings.streaming_pool_pages = pages;
            settings.validate().unwrap();
        }
        for triangles in [0, 511, 8193, u16::MAX] {
            settings.streaming_triangle_budget = triangles;
            assert!(settings.validate().is_err());
        }
        for triangles in [512, 4096, 8192] {
            settings.streaming_triangle_budget = triangles;
            settings.validate().unwrap();
        }
        settings.streaming_geometry = true;
        settings.motion_interpolation = false;
        settings.precomputed_visibility = true;
        settings.streaming_prefetch = false;
        let decoded: Rendering =
            serde_json::from_str(&serde_json::to_string(&settings).unwrap()).unwrap();
        assert_eq!(decoded, settings);
        let header = settings.header().unwrap();
        assert!(header.contains("precomputed_visibility = true"));
        assert!(header.contains("motion_interpolation = false"));
        assert!(header.contains("streaming_geometry = true"));
        assert!(header.contains("streaming_pool_pages = 8"));
        assert!(header.contains("streaming_prefetch_enabled = false"));
    }
    #[test]
    fn legacy_preferences_keep_mcp_off_and_keys_roundtrip() {
        let legacy: Preferences = serde_json::from_str("{\"fly_speed\":8}").unwrap();
        assert!(!legacy.mcp.enabled);
        assert!(legacy.mcp.token.is_empty());
        let mut preferences = legacy;
        preferences.mcp.enabled = true;
        assert!(preferences.validate().is_err());
        preferences.mcp.prepare();
        preferences.validate().unwrap();
        let decoded: Preferences =
            serde_json::from_str(&serde_json::to_string(&preferences).unwrap()).unwrap();
        assert_eq!(decoded, preferences);
        let old = preferences.mcp.token.clone();
        preferences.mcp.rotate_token();
        assert_ne!(old, preferences.mcp.token);
        assert!(
            !serde_json::to_string(&Rendering::default())
                .unwrap()
                .contains("mcp")
        );
    }
    #[test]
    fn display_provenance_ignores_unconsumed_settings_and_tracks_actual_output() {
        use crate::{artifact_dependencies::Graph, scene_dependencies::Input};
        let root = crate::workspace::tests::temp("display-provenance");
        let project = crate::workspace::create(
            &root,
            "Display provenance",
            crate::workspace::Template::Basic,
        )
        .unwrap();
        let path = root.join(&project.manifest.startup_scene);
        drop(project);
        let mut input = Input::load(&path).unwrap();
        let target = root.join(".epok/build");
        let stage = |scene: &crate::scene::Scene| {
            crate::project::stage_with_origin(
                &root,
                scene,
                &target,
                &Input::editor(path.clone(), scene.clone()).origin,
            )
            .unwrap();
        };
        stage(&input.scene);
        let key = "generated-resource:.epok/build/display.hh";
        let before = Graph::load(&root).unwrap().nodes[key].clone();
        assert_eq!(
            before.dependencies,
            ["display-settings".to_owned()].into_iter().collect()
        );
        input.scene.entities[0].position[0] += 3.;
        crate::scene_dependencies::observe(&root, &path, &input.scene).unwrap();
        assert_eq!(Graph::load(&root).unwrap().nodes[key], before);
        let mut manifest = crate::workspace::read_manifest(&root).unwrap();
        // The triangle budget is consumed by geometry cooking, not display.hh.
        manifest.rendering.streaming_triangle_budget = 1024;
        crate::workspace::save_manifest(&root, &manifest).unwrap();
        crate::scene_dependencies::observe(&root, &path, &input.scene).unwrap();
        assert_eq!(Graph::load(&root).unwrap().nodes[key], before);
        manifest.rendering.width = 320;
        crate::workspace::save_manifest(&root, &manifest).unwrap();
        crate::scene_dependencies::observe(&root, &path, &input.scene).unwrap();
        let stale = Graph::load(&root).unwrap().nodes[key].clone();
        assert!(stale.stale.contains_key("display-settings"));
        assert_eq!(stale.signature, before.signature);
        stage(&input.scene);
        let fresh = Graph::load(&root).unwrap().nodes[key].clone();
        assert!(fresh.stale.is_empty());
        assert_eq!(
            fresh.signature,
            Some(crate::assets::hash(
                &fs::read(target.join("display.hh")).unwrap()
            ))
        );
        let file = crate::workspace::manifest_path(&root).unwrap();
        let bytes = fs::read(&file).unwrap();
        fs::write(&file, b"broken project settings").unwrap();
        crate::scene_dependencies::observe(&root, &path, &input.scene).unwrap();
        assert!(!Graph::load(&root).unwrap().nodes[key].stale.is_empty());
        fs::write(&file, bytes).unwrap();
        crate::scene_dependencies::observe(&root, &path, &input.scene).unwrap();
        assert!(!Graph::load(&root).unwrap().nodes[key].stale.is_empty());
    }
    #[test]
    fn project_modes_migrate_validate_and_export_without_editor_preferences() {
        let root = crate::workspace::editor_home()
            .join(".epok")
            .join(format!("display-settings-{}", uuid::Uuid::new_v4()));
        let project =
            crate::workspace::create(&root, "Display settings", crate::workspace::Template::Basic)
                .unwrap();
        assert_eq!(project.manifest.rendering, Rendering::default());
        drop(project);
        let file = crate::workspace::manifest_path(&root).unwrap();
        let mut legacy: serde_json::Value =
            crate::document::from_slice(&fs::read(&file).unwrap()).unwrap();
        legacy.as_object_mut().unwrap().remove("rendering");
        save_document(&file, &legacy).unwrap();
        assert_eq!(rendering(&root).unwrap(), Rendering::default());
        let mut editor = crate::editor::Editor::new(root.clone());
        let mut manifest = crate::workspace::read_manifest(&root).unwrap();
        let original = fs::read(&file).unwrap();
        manifest.rendering = Rendering {
            width: 1920,
            height: 1080,
            ..Default::default()
        };
        assert!(editor.apply_project_settings(manifest.clone()).is_err());
        assert_eq!(fs::read(&file).unwrap(), original);
        manifest.rendering = Rendering {
            width: 320,
            height: 240,
            ..Default::default()
        };
        manifest.startup_scene = "assets/scenes/missing.epokmap".into();
        assert!(editor.apply_project_settings(manifest).is_err());
        assert_eq!(fs::read(&file).unwrap(), original);
        for (width, height) in Rendering::MODES {
            let mut manifest = crate::workspace::read_manifest(&root).unwrap();
            manifest.rendering = Rendering {
                width,
                height,
                ..Default::default()
            };
            editor.apply_project_settings(manifest).unwrap();
            assert_eq!(editor.scene.display_size, [width, height]);
            assert_eq!(
                rendering(&root).unwrap(),
                Rendering {
                    width,
                    height,
                    ..Default::default()
                }
            );
        }
        // The retained-packet switch round-trips and reaches the generated header.
        let mut manifest = crate::workspace::read_manifest(&root).unwrap();
        manifest.rendering.retained_geometry = false;
        editor.apply_project_settings(manifest).unwrap();
        assert!(!rendering(&root).unwrap().retained_geometry);
        assert!(
            rendering(&root)
                .unwrap()
                .header()
                .unwrap()
                .contains("retained_geometry = false")
        );
        let mut manifest = crate::workspace::read_manifest(&root).unwrap();
        manifest.rendering = Rendering::default();
        editor.apply_project_settings(manifest).unwrap();
        let scene =
            crate::scene::Scene::load(&crate::workspace::startup_scene(&root).unwrap()).unwrap();
        let export = crate::export::export_project(&root, &scene.into()).unwrap();
        assert_eq!(
            fs::read_to_string(export.join("display.hh")).unwrap(),
            Rendering::default().header().unwrap()
        );
        assert!(!export.join("Editor.epokprefs").exists());
    }
    #[test]
    fn preferences_roundtrip_and_emulator_merge_preserve_other_configuration() {
        let root = crate::workspace::editor_home()
            .join(".epok")
            .join(format!("preferences-{}", uuid::Uuid::new_v4()));
        let path = root.join("Editor.epokprefs");
        let preferences = Preferences {
            fly_speed: 12.,
            integer_scale: false,
            ..Default::default()
        };
        save_document(&path, &preferences).unwrap();
        assert_eq!(Preferences::read(&path).unwrap(), preferences);
        let config = root.join("pcsx.json");
        save_document(&config,&serde_json::json!({"emulator":{"LinearFiltering":true,"Bios":"keep.bin"},"gui":{"Fullscreen":true},"SPU":{"Volume":2}})).unwrap();
        configure_emulator(&root, &preferences).unwrap();
        let data: serde_json::Value = serde_json::from_slice(&fs::read(config).unwrap()).unwrap();
        assert_eq!(data["emulator"]["LinearFiltering"], false);
        assert_eq!(data["emulator"]["Bios"], "keep.bin");
        assert_eq!(data["SPU"]["Volume"], 2);
        assert_eq!(data["gui"]["Fullscreen"], true);
    }
    #[test]
    fn hud_anchors_and_pixels_use_project_output_dimensions() {
        let mut scene = crate::scene::Scene::default();
        scene.entities.clear();
        scene.display_size = [640, 480];
        let mut canvas = crate::scene::Entity::cube("Canvas".into());
        canvas.kind = "Empty".into();
        canvas.canvas = Some(Default::default());
        let mut panel = crate::scene::Entity::cube("Panel".into());
        panel.kind = "Empty".into();
        panel.parent = Some(0);
        panel.rect = Some(crate::hud::RectTransform {
            anchor_min: [1., 1.],
            anchor_max: [1., 1.],
            pivot: [1., 1.],
            size: [20., 20.],
            ..Default::default()
        });
        panel.image = Some(crate::hud::Image {
            color: [1., 0., 0.],
            enabled: true,
            ..Default::default()
        });
        scene.entities = vec![canvas, panel];
        assert_eq!(crate::hud::layout(&scene, 1), Some([620., 460., 20., 20.]));
        let pixels = crate::hud::render(&scene);
        assert_eq!(pixels.len(), 640 * 480 * 4);
        assert_eq!(
            &pixels[(10 * 640 + 630) * 4..(10 * 640 + 630) * 4 + 3],
            &[255, 0, 0]
        );
        assert!(
            !serde_json::to_string(&scene)
                .unwrap()
                .contains("display_size")
        );
    }
}
