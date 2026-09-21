use crate::{
    pipeline::{self, Control, Event},
    project,
    scene::{Actor, Scene},
    scripts, viewport,
};
use std::{
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant},
};

type ScriptCatalog = Result<(Vec<scripts::Script>, crate::blueprint::Registry), String>;

/// A single read-only worker per project. Delayed scene observations are only
/// published if the active document and registry still match their snapshot.
struct SourceObservation {
    fingerprint: Result<u64, String>,
    playback: Result<crate::timeline::PlaybackSources, String>,
    scene: Option<ObservedScene>,
}
struct ObservedScene {
    path: PathBuf,
    scene: Scene,
    registry_revision: u64,
    registry_error: Option<String>,
    dependencies: Result<crate::scene_dependencies::Observation, String>,
    navigation: bool,
}
impl SourceObservation {
    fn read(
        root: &Path,
        path: PathBuf,
        scene: Scene,
        registry: crate::blueprint::Registry,
        registry_revision: u64,
        registry_error: Option<String>,
    ) -> Self {
        let dependencies = crate::scene_dependencies::inspect_with_registry(
            root,
            &path,
            &scene,
            registry_error.as_deref().map_or(Ok(&registry), Err),
        );
        Self {
            fingerprint: project::source_fingerprint(root),
            playback: crate::timeline::inspect_playback(root),
            scene: Some(ObservedScene {
                path,
                scene,
                registry_revision,
                registry_error,
                dependencies,
                navigation: false,
            }),
        }
    }
}

/// CPU/filesystem data only; safe to prepare without an ImGui context on a worker.
pub struct PreparedProject {
    project: crate::workspace::Project,
    path: PathBuf,
    scene: Scene,
    scripts: ScriptCatalog,
    assets: crate::assets::Index,
    asset_cache: crate::assets::ScanCache,
    external_watch: crate::native_metadata::ExternalWatch,
    pub(crate) messages: Vec<crate::scene_loading::Message>,
}
impl PreparedProject {
    pub fn load(
        project: crate::workspace::Project,
        stage: impl Fn(&'static str),
    ) -> Result<Self, String> {
        let mut messages = vec![];
        let mut timing = crate::scene_loading::Progress::new();
        let mut mark = |label| {
            timing.stage(label, &mut |message| messages.push(message));
            stage(label);
        };
        mark("Loading startup scene");
        let path = crate::workspace::scene_path(&project.root, &project.manifest)?;
        let mut scene = Scene::load_unresolved(&path)?;
        mark("Discovering scripts and Blueprint classes");
        let scripts = scripts::catalog(&project.root).and_then(|catalog| {
            crate::blueprint::native_registry(&project.root, &catalog)
                .map(|registry| (catalog, registry))
        });
        mark("Indexing project assets");
        let mut asset_cache = crate::assets::ScanCache::default();
        let assets = crate::assets::scan(&project.root, &mut asset_cache);
        mark("Resolving startup scene resources");
        let _ = crate::mesh::resolve(&mut scene, &assets);
        let _ = crate::terrain::resolve(&mut scene, &assets);
        let _ = crate::skeletal::resolve(&mut scene, &assets);
        let _ = crate::texture::resolve(&mut scene, &assets);
        mark("Establishing source observation baseline");
        let external_watch =
            crate::native_metadata::ExternalWatch::primed(&project.root).unwrap_or_default(); // Normal polling reports unreadable dependency graphs.
        timing.finish("Project data ready", &mut |message| messages.push(message));
        Ok(Self {
            project,
            path,
            scene,
            scripts,
            assets,
            asset_cache,
            external_watch,
            messages,
        })
    }
}

pub struct Editor {
    pub artifact_dependencies: crate::artifact_dependency_ui::State,
    pub blueprint_editor: crate::blueprint_editor::BlueprintEditor,
    pub timeline_editor: crate::timeline_editor::TimelineEditor,
    pub timeline_inspector: crate::timeline_scene::Inspector,
    pub effect_inspector: crate::particle_effect_scene::Inspector,
    pub blueprint_creation: crate::blueprint_workflow::Creation,
    pub actor_creation: crate::actor_workflow::State,
    pub blueprint_debug_enabled: bool,
    pub blueprint_debug_ui: crate::blueprint_debug_ui::State,
    instance_baseline: std::collections::BTreeMap<uuid::Uuid, (Actor, Option<uuid::Uuid>)>,
    pub mcp: crate::mcp::State,
    pub settings: crate::settings_ui::State,
    pub dependencies: crate::dependencies::State,
    pub serial_ui: crate::serial_ui::State,
    pub job_stage: String,
    pub job_progress: Option<f32>,
    pub memory: crate::memory_ui::State,
    pub export_ui: crate::export_ui::State,
    pub preferences: crate::settings::Preferences,
    pub play_profile: crate::play::Profile,
    pub play_warning: Option<String>,
    pub active_play_runtime: crate::play::Runtime,
    pub active_play_target: crate::play::Target,
    pub skeletal_ui: crate::skeletal_ui::State,
    pub mesh_editor: crate::mesh_editor::State,
    pub terrain_editor: crate::terrain_editor::State,
    pub assets: crate::asset_manager::Manager,
    pub scene_loading: Option<crate::scene_loading::Loading>,
    project: Option<crate::workspace::Project>,
    pub return_to_hub: bool,
    pub hub_requested: bool,
    pub about_requested: bool,
    scene_file: PathBuf,
    pub root: PathBuf,
    pub scene: Scene,
    pub lighting_window: bool,
    pub bake_current: bool,
    pub bake_job: Option<std::sync::mpsc::Receiver<Result<crate::lighting::Bake, String>>>,
    pub selected: Option<usize>,
    pub dirty: bool,
    pub view: viewport::View,
    pub view_dirty: bool,
    pub grid: bool,
    pub wire: bool,
    pub logs: Vec<String>,
    pub(crate) log_times: Vec<String>,
    pub console: crate::console::State,
    pub catalog: Vec<scripts::Script>,
    pub class_registry: crate::blueprint::Registry,
    pub(crate) registry_revision: u64,
    pub script_creation: bool,
    pub script_creation_context: crate::actor_scripts::CreationContext,
    pub script_name: String,
    pub script_parent: String,
    pub script_folder: String,
    pub script_search: String,
    pub script_error: Option<String>,
    /// Lua authoring mirrors the C++ dialog: one pending creation at a time.
    pub lua_creation: bool,
    pub lua_creation_context: crate::actor_scripts::CreationContext,
    pub lua_name: String,
    pub lua_parent: String,
    pub lua_folder: String,
    pub lua_search: String,
    pub lua_error: Option<String>,
    pub script_undo: Vec<(Scene, Scene)>,
    pub script_redo: Vec<(Scene, Scene)>,
    pub job: Option<pipeline::Job>,
    pub playing: bool,
    pub paused: bool,
    pub auto_build: bool,
    pub pending_build: bool,
    pub source_error: Option<String>,
    scene_dependency_error: Option<String>,
    pub job_stale: bool,
    job_target: Option<(bool, bool)>,
    restart_target: Option<(bool, bool)>,
    native_play_cache: Option<(String, PathBuf)>,
    native_play_request: Option<String>,
    pub search: String,
    pub close_requested: bool,
    pub should_close: bool,
    pub reset_layout: bool,
    pub focus_console: bool,
    pub focus_scene: bool,
    pub focus_project: bool,
    pub last_error: Option<String>,
    pub game_frame: Option<std::sync::Arc<crate::bridge::Frame>>,
    pub native_frame: Option<std::sync::Arc<crate::native_play::Frame>>,
    pub game_error: Option<String>,
    pub focus_game: bool,
    pub game_capture: bool,
    pub emulator_pid: Option<u32>,
    pub emulator_visible: bool,
    pub tool: usize,
    pub project_search: String,
    pub project_browser: crate::project_browser::State,
    pub selected_asset: Option<PathBuf>,
    pub asset_inspector: crate::asset_inspector::State,
    /// The proportional face the property editor draws with, supplied once the
    /// platform layer has built the font atlas.
    pub inspector_font: Option<imgui::FontId>,
    pub drag_axis: Option<usize>,
    pub scene_click: crate::picking::ClickGesture,
    pub scene_panel_size: [f32; 2],
    pub reveal_selected: bool,
    pub rename: Option<(usize, String)>,
    pub rename_focus: bool,
    pub scene_navigation: bool,
    pub navigation_preview_status: Option<Result<usize,String>>,
    pub scene_look: bool,
    pub raw_look: Option<[f32; 2]>,
    pub scene_view_mode: crate::scene_view_mode::SceneViewMode,
    /// The P4 document actor selected in the Hierarchy. Mutually exclusive with
    /// `selected`: an author never has a legacy entity and an actor selected at once.
    pub selected_actor: Option<uuid::Uuid>,
    /// Inline actor rename in the Hierarchy: the actor and its edited name.
    pub actor_rename: Option<(uuid::Uuid, String)>,
    pub actor_rename_focus: bool,
    /// The Scene window's 2D camera (`SceneViewMode::TwoD`).
    pub view_2d: crate::scene_view_mode::SceneView2D,
    /// The actor being dragged in the 2D view and the world offset between its
    /// origin and the grab point, so the actor does not jump to the cursor.
    pub drag_2d: Option<(uuid::Uuid, [f32; 2])>,
    /// The Map Settings window, opened from the Hierarchy's map root line.
    pub map_settings: bool,
    /// Scene Blueprint parent chosen in Map Settings before one exists.
    pub map_scene_script_parent: String,
    scene_history: crate::scene_view_mode::History<Scene>,
    /// Resolved Object/Actor/Component model, rebuilt when `registry_revision`
    /// moves. The Hierarchy asks for it every frame, so it is not re-derived per call.
    object_model:
        std::cell::RefCell<Option<(u64, Option<std::rc::Rc<crate::object_model::Model>>)>>,
    pub hud_simulation: crate::hud_simulation::State,
    pub hud_drag: Option<(usize, bool)>,
    fingerprint: u64,
    playback_watch: crate::timeline_compile::SourceWatch,
    source_scan: Option<std::sync::mpsc::Receiver<SourceObservation>>,
    source_dirty: bool,
    source_revision: u64,
    source_registry_revision: u64,
    scene_baseline_pending: bool,
    external_watch: crate::native_metadata::ExternalWatch,
    external_observation_error: Option<String>,
    playback_observation_error: Option<String>,
    asset_fingerprint: Option<String>,
    pending_since: Option<Instant>,
    last_poll: Instant,
}
impl Editor {
    pub fn critical_busy(&self) -> bool {
        self.scene_loading.is_some()
            || self.dependencies.busy()
            || (self.job.is_some() && !self.playing)
    }
    pub fn set_buttons(&self, buttons: u16) {
        let buttons = buttons | self.mcp.buttons;
        if let Some(bridge) = self.job.as_ref().and_then(|j| j.bridge.as_ref()) {
            bridge
                .buttons
                .store(buttons, std::sync::atomic::Ordering::Relaxed);
        }
        if let Some(bridge) = self.job.as_ref().and_then(|j| j.native.as_ref()) {
            *bridge.buttons.lock().unwrap() = buttons;
        }
    }
    pub fn open(project: crate::workspace::Project) -> Result<Self, String> {
        Self::open_prepared(PreparedProject::load(project, |_| {})?)
    }
    pub fn open_prepared(prepared: PreparedProject) -> Result<Self, String> {
        let PreparedProject {
            project,
            path,
            scene,
            scripts,
            assets,
            asset_cache,
            external_watch,
            messages,
        } = prepared;
        let handoff = Instant::now();
        let (times, logs): (Vec<_>, Vec<_>) =
            messages.into_iter().map(|m| (m.time, m.text)).unzip();
        let mut editor = Self::from_scene(
            project.root.clone(),
            path,
            scene,
            false,
            logs,
            Some(scripts),
        );
        editor.log_times[..times.len()].clone_from_slice(&times);
        editor.asset_fingerprint = Some(assets.fingerprint());
        editor.assets.adopt(assets, asset_cache);
        editor.external_watch = external_watch;
        editor.scene.display_size = [
            project.manifest.rendering.width,
            project.manifest.rendering.height,
        ];
        if let Ok(bytes) = std::fs::read(project.root.join("UserSettings/SceneView.epokprefs"))
            && let Ok(view) = crate::document::from_slice::<viewport::View>(&bytes)
            && view
                .center
                .iter()
                .chain([
                    &view.yaw,
                    &view.pitch,
                    &view.zoom,
                    &view.distance,
                    &view.fly_speed,
                    &view.phase,
                ])
                .all(|v| v.is_finite())
            && (0.05..=8.).contains(&view.zoom)
            && (1.05..=10000.).contains(&view.distance)
            && (0.01..=1000.).contains(&view.fly_speed)
        {
            editor.view = view;
            editor.selected = None;
        }
        if let Some(pair) = std::env::args()
            .collect::<Vec<_>>()
            .windows(2)
            .find(|v| v[0] == "--preview-model")
        {
            let path = crate::assets::inside(&editor.root, &pair[1])?;
            let package = crate::assets::Package::load(&path)?;
            editor.assets.index = crate::assets::scan(&editor.root, &mut Default::default());
            let record = editor.assets.index.resolve(package.meta.id)?.clone();
            crate::skeletal_ui::open(&mut editor, record);
        }
        editor.project = Some(project);
        if let Some(pair) = std::env::args()
            .collect::<Vec<_>>()
            .windows(2)
            .find(|v| v[0] == "--open-timeline" || v[0] == "--open-effect")
        {
            let path = crate::assets::inside(&editor.root, &pair[1])?;
            editor.refresh_scripts();
            editor.timeline_editor.open(&path)?;
            if std::env::args().any(|arg| arg == "--sequencer-layout") {
                editor.timeline_editor.layout = true;
            }
        }
        if let Some(pair) = std::env::args()
            .collect::<Vec<_>>()
            .windows(2)
            .find(|v| v[0] == "--open-blueprint")
        {
            let path = crate::assets::inside(&editor.root, &pair[1])?;
            editor.refresh_scripts();
            editor.blueprint_editor.open(&path)?;
            editor.blueprint_editor.maximized =
                std::env::args().any(|v| v == "--screenshot-blueprint-canvas");
        }
        if std::env::args().any(|v| v == "--screenshot-project-settings") {
            crate::settings_ui::open_project(&mut editor);
        }
        if std::env::args().any(|v| v == "--screenshot-editor-preferences") {
            crate::settings_ui::open_preferences(&mut editor);
        }
        if std::env::args().any(|v| v == "--screenshot-mcp-settings") {
            crate::settings_ui::open_mcp(&mut editor);
        }
        if std::env::args().any(|v| v == "--screenshot-dependencies") {
            crate::settings_ui::open_dependencies(&mut editor);
            editor.dependencies.warning = false;
        }
        if std::env::args().any(|v| v == "--screenshot-artifact-dependencies") {
            editor.artifact_dependencies.open = true;
        }
        if std::env::args().any(|v| v == "--screenshot-serial-connection") {
            editor.dependencies.warning = false;
            crate::serial_ui::open(&mut editor);
        }
        editor.establish_open_baseline();
        editor.restore_build_report();
        editor.log(format!(
            "Editor ready: {:.1} ms finalization ({} build)",
            handoff.elapsed().as_secs_f64() * 1000.,
            if cfg!(debug_assertions) {
                "Debug"
            } else {
                "Release"
            }
        ));
        if std::env::args().any(|v| v == "--screenshot-memory") {
            let path = editor.root.join(".epok/build/memory-report.json");
            let report: crate::memory::Report =
                serde_json::from_slice(&std::fs::read(&path).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            editor.dependencies.warning = false;
            editor.memory.profile = report.profile.clone();
            editor.memory.debug = report.debug;
            editor.memory.report = Some(report);
            editor.memory.scene_signature = crate::scene_dependencies::signature(&editor.scene);
            editor.memory.open = true;
        }
        if std::env::args().any(|v| v == "--screenshot-build-progress") {
            editor.dependencies.warning = false;
            editor.build(false);
        }
        Ok(editor)
    }

    fn establish_open_baseline(&mut self) {
        // Loading/migration and restoring old build metadata are not live edits.
        // Publish their current provenance so old outputs remain stale, without
        // scheduling a build or stealing focus from Project during startup.
        let _ = crate::mesh::resolve(&mut self.scene, &self.assets.index);
        let _ = crate::terrain::resolve(&mut self.scene, &self.assets.index);
        let _ = crate::skeletal::resolve(&mut self.scene, &self.assets.index);
        let _ = crate::texture::resolve(&mut self.scene, &self.assets.index);
        if let Ok(fingerprint) = project::source_fingerprint(&self.root) {
            self.fingerprint = fingerprint;
        }
        let result = crate::staging_files::observe_native(&self.root).and_then(|_| {
            crate::scene_dependencies::observe_with_registry(
                &self.root,
                &self.scene_file,
                &self.scene,
                self.timeline_editor
                    .registry_error
                    .as_deref()
                    .map_or(Ok(&self.class_registry), Err),
            )
        });
        if let Err(error) = result {
            self.scene_dependency_error = Some(error.clone());
            self.log(format!("Source dependency observation failed: {error}"));
        }
        self.pending_build = false;
        self.pending_since = None;
    }
    pub fn project_name(&self) -> &str {
        self.project
            .as_ref()
            .map_or("Test project", |p| p.manifest.name.as_str())
    }
    pub fn apply_project_settings(
        &mut self,
        manifest: crate::workspace::Manifest,
    ) -> Result<(), String> {
        self.apply_project_configuration(manifest, None)
    }
    pub fn apply_project_configuration(
        &mut self,
        manifest: crate::workspace::Manifest,
        maps: Option<crate::scene_bank::Registry>,
    ) -> Result<(), String> {
        if self.job.is_some() || self.dependencies.busy() {
            return Err("Stop Play or wait for the build before changing settings.".into());
        }
        crate::workspace::validate_name(&manifest.name)?;
        manifest.rendering.validate()?;
        manifest.play.validate()?;
        manifest.transition.validate()?;
        let path = crate::workspace::scene_path(&self.root, &manifest)?;
        // Settings are not an instruction to re-import the startup scene.
        // Validate a newly selected map's document without resolving its assets.
        if self
            .project
            .as_ref()
            .is_none_or(|p| p.manifest.startup_scene != manifest.startup_scene)
        {
            Scene::load_unresolved(&path)?;
        }
        if let Some(maps) = &maps {
            maps.validate(&self.root)?;
        }
        crate::workspace::save_manifest(&self.root, &manifest)?;
        self.project_browser
            .previews
            .configure(&self.root, &self.assets.index);
        if let Some(maps) = &maps {
            maps.save(&self.root)?;
        }
        // Publish exactly the settings just saved, leaving compiled consumers
        // stale for the NEXT build. Do not consume/suppress other file changes
        // or clear an already pending source-edit build.
        crate::scene_dependencies::apply_settings(&self.root, &manifest, maps.as_ref())?;
        self.memory.stale = true;
        self.auto_build = manifest.auto_build;
        self.pending_build |= self.play_profile != manifest.play;
        self.play_profile = manifest.play.clone();
        self.scene.display_size = [manifest.rendering.width, manifest.rendering.height];
        self.view_dirty = true;
        if let Some(project) = self.project.as_mut() {
            let renamed = project.manifest.name != manifest.name;
            project.manifest = manifest;
            if renamed {
                let _ = crate::workspace::remember(project);
            }
        }
        self.log("Project settings saved. Build changes apply on the next Play / Build.");
        Ok(())
    }
    pub fn apply_preferences(
        &mut self,
        mut preferences: crate::settings::Preferences,
    ) -> Result<(), String> {
        if self.critical_busy() {
            return Err("Wait for the current operation before changing preferences.".into());
        }
        preferences.mcp.prepare();
        preferences.save()?;
        self.grid = preferences.show_grid;
        self.view.fly_speed = preferences.fly_speed;
        self.preferences = preferences;
        self.view_dirty = true;
        Ok(())
    }
    #[cfg(test)]
    pub fn new(root: PathBuf) -> Self {
        let loaded = Scene::load(&root.join("assets/scenes/SampleScene.epokmap"));
        let mut logs = vec!["Epok / Dear ImGui / PsyQo / C++ scripts".into()];
        let dirty = loaded.is_err();
        let scene = loaded.unwrap_or_else(|e| {
            logs.push(format!("Using unsaved sample: {e}"));
            Scene::default()
        });
        let path = root.join("assets/scenes/SampleScene.epokmap");
        Self::from_scene(root, path, scene, dirty, logs, None)
    }
    fn from_scene(
        root: PathBuf,
        scene_file: PathBuf,
        scene: Scene,
        dirty: bool,
        mut logs: Vec<String>,
        initial_scripts: Option<ScriptCatalog>,
    ) -> Self {
        let selected = scene
            .actors
            .iter()
            .position(|e| e.kind == "Mesh")
            .or_else(|| (!scene.actors.is_empty()).then_some(0));
        let config = project::Config::load(&root);
        let auto_build = match config {
            Ok(c) => c.auto_build,
            Err(e) => {
                logs.push(e);
                false
            }
        };
        let preferences = crate::settings::Preferences::load().unwrap_or_else(|error| {
            logs.push(error);
            Default::default()
        });
        let opened_at = Self::log_timestamp();
        let log_times = vec![opened_at; logs.len()];
        let mut editor = Self {
            artifact_dependencies: Default::default(),
            blueprint_editor: Default::default(),
            timeline_editor: Default::default(),
            timeline_inspector: Default::default(),
            effect_inspector: Default::default(),
            blueprint_creation: Default::default(),
            actor_creation: Default::default(),
            blueprint_debug_enabled: false,
            blueprint_debug_ui: Default::default(),
            instance_baseline: Default::default(),
            mcp: Default::default(),
            settings: Default::default(),
            dependencies: crate::dependencies::State::new(&root),
            serial_ui: Default::default(),
            job_stage: String::new(),
            job_progress: None,
            memory: Default::default(),
            export_ui: Default::default(),
            preferences: preferences.clone(),
            play_profile: crate::play::Profile::load(&root).unwrap_or_default(),
            play_warning: None,
            active_play_runtime: Default::default(),
            active_play_target: Default::default(),
            mesh_editor: Default::default(),
            terrain_editor: Default::default(),
            skeletal_ui: Default::default(),
            assets: crate::asset_manager::Manager::new(root.clone()),
            scene_loading: None,
            project: None,
            return_to_hub: false,
            hub_requested: false,
            about_requested: false,
            scene_file,
            fingerprint: project::source_fingerprint(&root).unwrap_or(0),
            playback_watch: Default::default(),
            source_scan: None,
            source_dirty: true,
            source_revision: 0,
            source_registry_revision: 0,
            scene_baseline_pending: false,
            external_watch: Default::default(),
            external_observation_error: None,
            playback_observation_error: None,
            asset_fingerprint: None,
            source_error: None,
            scene_dependency_error: None,
            job_stale: false,
            job_target: None,
            restart_target: None,
            root,
            bake_current: crate::lighting::valid_bake(&scene),
            scene,
            lighting_window: false,
            bake_job: None,
            selected,
            dirty,
            view: viewport::View {
                fly_speed: preferences.fly_speed,
                ..Default::default()
            },
            view_dirty: true,
            grid: preferences.show_grid,
            wire: false,
            logs,
            log_times,
            console: Default::default(),
            catalog: vec![],
            class_registry: crate::blueprint::Registry::new(),
            registry_revision: 0,
            script_creation: false,
            script_creation_context: Default::default(),
            script_name: String::new(),
            script_parent: "epok::ActorComponent".into(),
            script_folder: String::new(),
            script_search: String::new(),
            script_error: None,
            lua_creation: false,
            lua_creation_context: Default::default(),
            lua_name: String::new(),
            lua_parent: "epok::ActorComponent".into(),
            lua_folder: String::new(),
            lua_search: String::new(),
            lua_error: None,
            script_undo: vec![],
            script_redo: vec![],
            job: None,
            playing: false,
            paused: false,
            auto_build,
            pending_build: false,
            search: String::new(),
            close_requested: false,
            should_close: false,
            reset_layout: false,
            focus_console: false,
            focus_scene: true,
            focus_project: true,
            last_error: None,
            game_frame: None,
            native_frame: None,
            native_play_cache: None,
            native_play_request: None,
            game_error: None,
            focus_game: false,
            game_capture: false,
            emulator_pid: None,
            emulator_visible: true,
            tool: 1,
            project_search: String::new(),
            project_browser: crate::project_browser::State::default(),
            selected_asset: None,
            asset_inspector: Default::default(),
            inspector_font: None,
            drag_axis: None,
            scene_click: Default::default(),
            scene_panel_size: [960., 600.],
            reveal_selected: false,
            rename: None,
            rename_focus: false,
            scene_navigation: false,
            navigation_preview_status: None,
            scene_look: false,
            raw_look: None,
            scene_view_mode: Default::default(),
            selected_actor: None,
            actor_rename: None,
            actor_rename_focus: false,
            view_2d: Default::default(),
            drag_2d: None,
            map_settings: false,
            map_scene_script_parent: String::new(),
            scene_history: crate::scene_view_mode::History::new(Scene::default(), 32),
            object_model: std::cell::RefCell::new(None),
            hud_simulation: Default::default(),
            hud_drag: None,
            pending_since: None,
            last_poll: Instant::now(),
        };
        if let Some(scripts) = initial_scripts {
            editor.apply_script_catalog(scripts);
        } else {
            editor.refresh_scripts();
        }
        // The default scene Blueprint needs the resolved model, so it is created
        // after the catalog; it is not an edit, so the history baseline and the
        // dirty flag are established from the document it produces.
        editor.ensure_scene_script();
        editor.scene_history.reset(&editor.scene);
        editor.observe_playback_sources();
        editor
    }
    pub fn log(&mut self, s: impl Into<String>) {
        self.log_message(crate::scene_loading::Message::new(s));
    }
    pub(crate) fn log_message(&mut self, message: crate::scene_loading::Message) {
        self.reconcile_log_times();
        self.logs.push(message.text);
        self.log_times.push(message.time);
        if self.logs.len() > 400 {
            let excess = self.logs.len() - 400;
            self.logs.drain(..excess);
            self.log_times.drain(..excess);
        }
    }
    fn log_timestamp() -> String {
        chrono::Local::now()
            .format("%Y-%m-%d %H:%M:%S%.3f")
            .to_string()
    }
    pub(crate) fn reconcile_log_times(&mut self) {
        if self.log_times.len() != self.logs.len() {
            self.log_times = vec![Self::log_timestamp(); self.logs.len()];
        }
    }
    pub(crate) fn clear_logs(&mut self) {
        self.logs.clear();
        self.log_times.clear();
    }
    pub fn scene_path(&self) -> PathBuf {
        self.scene_file.clone()
    }
    pub fn open_scene(&mut self, path: PathBuf) -> Result<(), String> {
        self.check_scene_open(&path)?;
        let mut messages = vec![];
        let result = crate::scene_loading::Prepared::load(
            &self.root,
            path,
            self.scene.display_size,
            &self.class_registry,
            self.assets.scan_cache(),
            |message| messages.push(message),
        );
        for message in messages {
            self.log_message(message);
        }
        self.accept_scene(result?);
        Ok(())
    }
    fn check_scene_open(&self, path: &Path) -> Result<(), String> {
        if self.playing
            || self.job.is_some()
            || self.critical_busy()
            || self.assets.busy
            || self.bake_job.is_some()
        {
            return Err(
                "Stop Play and wait for the current operation before opening a scene.".into(),
            );
        }
        if !path.is_file() {
            return Err(format!("Scene file not found: {}", path.display()));
        }
        Ok(())
    }
    pub fn begin_scene_open(&mut self, path: PathBuf) -> Result<(), String> {
        self.check_scene_open(&path)?;
        self.log(format!("Opening scene: {}", path.display()));
        self.scene_loading = Some(crate::scene_loading::Loading::start(
            self.root.clone(),
            path,
            self.scene.display_size,
            self.class_registry.clone(),
            self.assets.scan_cache(),
        )?);
        self.focus_console = true;
        Ok(())
    }
    fn accept_scene(&mut self, prepared: crate::scene_loading::Prepared) {
        // An open embedded Blueprint belongs to the map that is being replaced.
        if self.blueprint_editor.is_embedded()
            && self.blueprint_editor.embedded_map() != Some(prepared.path.as_path())
        {
            self.blueprint_editor.discard();
        }
        self.scene = prepared.scene;
        self.ensure_scene_script();
        // A different document carries its own history; the previous map's
        // steps must never be replayed onto it.
        self.scene_history.reset(&self.scene);
        self.selected_actor = None;
        self.scene_file = prepared.path;
        // The first observation publishes the new active document without
        // treating navigation/closed editor documents as an authored edit.
        self.scene_baseline_pending = true;
        self.source_dirty = true;
        self.memory.stale = true;
        let fingerprint = prepared.assets.fingerprint();
        let resources_changed = self
            .asset_fingerprint
            .as_ref()
            .is_some_and(|old| old != &fingerprint);
        self.asset_fingerprint = Some(fingerprint);
        self.assets.adopt(prepared.assets, prepared.cache);
        // Navigation reuses resolved resources, but must still invalidate builds
        // if the scan discovered a real imported-resource change during loading.
        if resources_changed {
            self.invalidate_running_build();
            if let Err(error) =
                crate::timeline_compile::observe_resources(&self.root, &self.assets.index)
            {
                self.log(error);
            }
            self.log("Imported assets changed. Build pending.");
        }
        self.reset_scene_tools();
        self.reset_instance_baseline();
        self.dirty = prepared.dirty;
        self.bake_current = crate::lighting::valid_bake(&self.scene);
        self.view_dirty = true;
        self.focus_scene = true;
        self.log(format!("Scene opened: {}", self.scene.name));
    }
    fn poll_scene_open(&mut self) {
        use crate::scene_loading::Event;
        loop {
            let Some(loading) = self.scene_loading.as_mut() else {
                return;
            };
            let result = match loading.events.try_recv() {
                Ok(Event::Message(message)) => {
                    loading.stage = message.text.clone();
                    self.log_message(message);
                    continue;
                }
                Ok(Event::Ready(result)) => *result,
                Err(std::sync::mpsc::TryRecvError::Empty) => return,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    Err("Scene loading stopped unexpectedly.".into())
                }
            };
            let started = loading.started;
            self.scene_loading = None;
            match result {
                Ok(prepared) => {
                    let reloading = prepared.path == self.scene_file;
                    self.accept_scene(prepared);
                    if reloading {
                        self.selected = (!self.scene.actors.is_empty()).then_some(0);
                        self.log("Scene reloaded.");
                    }
                    self.log(format!(
                        "Scene opening completed: {:.1} ms total",
                        started.elapsed().as_secs_f64() * 1000.
                    ));
                }
                Err(error) => {
                    self.log(format!("Scene opening failed: {error}"));
                    self.last_error = Some(error);
                    self.focus_console = true;
                }
            }
            return;
        }
    }
    pub fn reset_scene_tools(&mut self) {
        self.hud_simulation.stop();
        self.mesh_editor = Default::default();
        self.terrain_editor = Default::default();
        self.skeletal_ui = Default::default();
        self.selected_asset = None;
        self.selected = None;
        self.rename = None;
        self.rename_focus = false;
        self.actor_rename = None;
        self.actor_rename_focus = false;
        self.drag_2d = None;
        self.view_2d.reset();
        self.hud_drag = None;
        self.drag_axis = None;
        self.scene_click = Default::default();
    }
    pub fn changed(&mut self) {
        self.apply_change();
        self.scene_history.record(&self.scene);
        self.finish_change();
    }
    /// Reports an edit that belongs to a continuous gesture.
    ///
    /// A drag reports every frame; `key` names the gesture, so all of its frames
    /// collapse into one undo step. The run ends at the first `changed()`, at a
    /// different key, at an undo/redo, or at [`Editor::end_coalesced`], which the
    /// gesture calls when the mouse is released. Discrete edits keep using
    /// `changed()` and are never folded into a neighbour.
    pub fn changed_coalesced(&mut self, key: &'static str) {
        self.apply_change();
        self.scene_history.record_coalesced(&self.scene, key);
        self.finish_change();
    }
    /// Closes the open coalescing run, so the next gesture records its own step.
    pub fn end_coalesced(&mut self) {
        self.scene_history.end_run();
    }
    fn finish_change(&mut self) {
        self.bake_current = crate::lighting::valid_bake(&self.scene);
        self.dirty = true;
        self.view_dirty = true;
    }
    fn apply_change(&mut self) {
        self.scene.sync_actor_components();
        self.source_dirty = true;
        self.invalidate_running_build();
        // This method is called for known editor actions. Metadata propagation
        // and history restoration reset the baseline first, never inferring edits.
        let mut parents = vec![];
        let parent_ids: Vec<_> = self
            .scene
            .actors
            .iter()
            .map(|entity| {
                entity
                    .parent
                    .and_then(|index| self.scene.actors.get(index))
                    .map(|parent| parent.id)
            })
            .collect();
        for (index, entity) in self.scene.actors.iter_mut().enumerate() {
            if let Some((original, original_parent)) = self.instance_baseline.get(&entity.id)
                && original.blueprint_instance == entity.blueprint_instance
            {
                crate::blueprint_templates::record_overrides(original, entity);
                if *original_parent != parent_ids[index] {
                    parents.push(index);
                }
            }
        }
        for index in parents {
            if let Err(error) =
                crate::blueprint_templates::record_parent_override(&mut self.scene, index)
            {
                self.log(error);
            }
        }
        self.reset_instance_baseline();
    }
    /// Compatibility accessor for the render, screenshot and HUD-preview paths:
    /// they only ever asked whether the Canvas editor is showing.
    pub fn scene_2d(&self) -> bool {
        self.scene_view_mode.scene_2d()
    }
    /// Legacy setter: `true` selects the UI (Canvas/HUD) mode, `false` the 3D one.
    pub fn set_scene_2d(&mut self, scene_2d: bool) {
        self.set_scene_view_mode(crate::scene_view_mode::SceneViewMode::from_scene_2d(
            scene_2d,
        ));
    }
    pub fn set_scene_view_mode(&mut self, mode: crate::scene_view_mode::SceneViewMode) {
        if self.scene_view_mode == mode {
            return;
        }
        self.scene_view_mode = mode;
        if !mode.scene_2d() {
            // The HUD preview only runs while its editor is showing.
            self.hud_simulation.running = false;
            self.hud_simulation.buttons = 0;
        }
        self.view_dirty = true;
    }
    /// The resolved class model, or `None` when the project's reflection data
    /// does not resolve. Cached per `registry_revision`.
    pub fn object_model(&self) -> Option<std::rc::Rc<crate::object_model::Model>> {
        let mut cache = self.object_model.borrow_mut();
        if cache
            .as_ref()
            .is_none_or(|(revision, _)| *revision != self.registry_revision)
        {
            let model = self.class_registry.model().ok().map(std::rc::Rc::new);
            *cache = Some((self.registry_revision, model));
        }
        cache.as_ref().and_then(|(_, model)| model.clone())
    }
    /// Selects a P4 document actor, clearing the legacy-entity selection.
    pub fn select_actor(&mut self, actor: Option<uuid::Uuid>) {
        self.selected_actor = actor;
        self.selected = actor.and_then(|id| self.scene.actor_index(id));
        if actor.is_some() {
            self.selected_asset = None;
        }
        self.view_dirty = true;
    }
    pub fn can_undo_scene(&self) -> bool {
        self.scene_history.can_undo() || !self.script_undo.is_empty()
    }
    pub fn can_redo_scene(&self) -> bool {
        self.scene_history.can_redo() || !self.script_redo.is_empty()
    }
    /// General scene undo. The component-attachment stack keeps priority: its
    /// entries carry the Blueprint bookkeeping a raw snapshot swap would lose.
    pub fn undo_scene(&mut self) -> Result<(), String> {
        self.step_history(false)
    }
    pub fn redo_scene(&mut self) -> Result<(), String> {
        self.step_history(true)
    }
    fn step_history(&mut self, redo: bool) -> Result<(), String> {
        if self.playing {
            return Err("Stop Play before undoing scene edits.".into());
        }
        let attachments = if redo {
            &self.script_redo
        } else {
            &self.script_undo
        };
        if !attachments.is_empty() && self.undo_attachment(redo).is_ok() {
            return Ok(());
        }
        let current = self.scene.clone();
        let restored = if redo {
            self.scene_history.redo(&current)
        } else {
            self.scene_history.undo(&current)
        };
        let Some(scene) = restored else {
            return Err(if redo {
                "Nothing to redo.".into()
            } else {
                "Nothing to undo.".into()
            });
        };
        self.restore_scene(scene);
        Ok(())
    }
    /// Adopts a scene from the history without recording it as a new edit.
    fn restore_scene(&mut self, scene: Scene) {
        self.scene = scene;
        self.selected = self
            .selected
            .filter(|index| *index < self.scene.actors.len());
        self.selected_actor = self
            .selected_actor
            .filter(|id| self.scene.actors.iter().any(|actor| actor.id == *id));
        self.rename = None;
        self.actor_rename = None;
        self.drag_2d = None;
        self.hud_drag = None;
        self.drag_axis = None;
        self.reset_instance_baseline();
        self.source_dirty = true;
        self.invalidate_running_build();
        self.bake_current = crate::lighting::valid_bake(&self.scene);
        self.dirty = true;
        self.view_dirty = true;
    }
    /// Appends a P4 document actor of `class` and selects it.
    ///
    /// The components come from the class chain's `EPOK_COMPONENT` defaults; a
    /// class that declares none still gets the root component its domain
    /// requires (design.md section 2), so the actor is valid the moment it
    /// exists. The scene is validated against the model and rolled back on
    /// failure, exactly like entity creation.
    pub fn create_actor(&mut self, class: &str) {
        use crate::{actor_document as doc, object_model as om, reflection_schema::Domain};
        if self.playing {
            self.log("Stop the emulator before editing.");
            return;
        }
        let Some(model) = self.object_model() else {
            self.last_error = Some("The class model is unavailable; reload the project.".into());
            return;
        };
        let Some(resolved) = model.class(class) else {
            self.last_error = Some(format!("`{class}` is not a reflected class."));
            return;
        };
        if self
            .class_registry
            .classes
            .get(&resolved.id)
            .is_some_and(|c| c.provider.id == "blueprint")
        {
            let result = (|| -> Result<Option<usize>, String> {
                let files = crate::blueprint_asset::load_all(&self.root)?;
                let template = crate::blueprint_templates::resolve_assets(
                    &files,
                    &self.class_registry,
                    &resolved.id,
                )?;
                if template.actors.is_empty() {
                    return Ok(None);
                }
                let class = &self.class_registry.classes[&resolved.id];
                let binding = crate::scene::ClassDefaults {
                    name: class.cpp_name.clone(),
                    class_id: Some(class.id.clone()),
                    provider: class.provider.clone(),
                    backend: class.backend.clone(),
                    ..Default::default()
                };
                let placed = crate::blueprint_templates::place(
                    &mut self.scene,
                    &template,
                    binding,
                    &self.class_registry,
                    None,
                )?;
                Ok(Some(placed.root))
            })();
            match result {
                Ok(Some(index)) => {
                    let id = self.scene.actors[index].id;
                    self.last_error = None;
                    self.select_actor(Some(id));
                    self.changed();
                    return;
                }
                Err(error) => {
                    self.last_error = Some(error.clone());
                    self.log(error);
                    return;
                }
                Ok(None) => {}
            }
        }
        let base = resolved
            .cpp_name
            .rsplit("::")
            .next()
            .unwrap_or(&resolved.cpp_name)
            .to_owned();
        let mut name = base.clone();
        let mut suffix = 1;
        while self.scene.actors.iter().any(|a| a.name == name)
            || self.scene.actors.iter().any(|e| e.name == name)
        {
            name = format!("{base}.{suffix:03}");
            suffix += 1;
        }
        let mut actor = doc::ActorInstance::new(
            uuid::Uuid::new_v4(),
            doc::ClassReference::new(&resolved.cpp_name, &resolved.id),
            &name,
        );
        for default in &resolved.default_components {
            let Some(component) = model.class(&default.class) else {
                continue;
            };
            let label = default.name.clone().unwrap_or_else(|| {
                component
                    .cpp_name
                    .rsplit("::")
                    .next()
                    .unwrap_or(&component.cpp_name)
                    .to_owned()
            });
            let mut instance = doc::ComponentInstance::new(
                uuid::Uuid::new_v4(),
                doc::ClassReference::new(&component.cpp_name, &component.id),
                &label,
            );
            instance.root = default.root;
            instance.inherited = true;
            instance.default_id = Some(default.id.clone());
            actor.components.push(instance);
        }
        for (index, default) in resolved.default_components.iter().enumerate() {
            if let Some(parent) = &default.attach_to {
                let parent = resolved
                    .default_components
                    .iter()
                    .position(|c| &c.field == parent)
                    .and_then(|i| actor.components.get(i))
                    .map(|c| c.id);
                if let Some(component) = actor.components.get_mut(index) {
                    component.attach_parent = parent;
                }
            }
        }
        if actor.root().is_none()
            && let Some(root) = match resolved.domain {
                Domain::World3D => Some(om::SCENE_COMPONENT3D_ID),
                Domain::World2D => Some(om::SCENE_COMPONENT2D_ID),
                Domain::UI => Some(om::RECT_TRANSFORM_COMPONENT_ID),
                Domain::None => None,
            }
            .and_then(|id| model.class(id))
        {
            let label = root
                .cpp_name
                .rsplit("::")
                .next()
                .unwrap_or(&root.cpp_name)
                .trim_end_matches("Component")
                .to_owned();
            let mut instance = doc::ComponentInstance::new(
                uuid::Uuid::new_v4(),
                doc::ClassReference::new(&root.cpp_name, &root.id),
                &label,
            );
            instance.root = true;
            actor.components.insert(0, instance);
        }
        actor.refresh_components();
        let id = actor.id;
        self.scene.actors.push(actor);
        if let Err(error) = self.scene.validate_with_model(Some(&model)) {
            self.scene.actors.retain(|a| a.id != id);
            self.last_error = Some(error.clone());
            self.log(error);
            return;
        }
        self.last_error = None;
        self.select_actor(Some(id));
        self.search.clear();
        self.changed();
        self.log(format!("Created {name} ({}).", resolved.cpp_name));
    }
    // -----------------------------------------------------------------------
    // Actor operations (P11). Every one of them ends in `changed()`, so the map
    // is marked dirty and one undo step is recorded, and every one validates the
    // candidate document with `Scene::validate_with_model`, rolling the whole
    // edit back into `last_error` rather than leaving a half-applied change.
    // -----------------------------------------------------------------------

    /// Refuses an actor edit while the emulator runs. Returns whether to proceed.
    fn actor_editable(&mut self) -> bool {
        if self.playing {
            self.log("Stop the emulator before editing.");
            return false;
        }
        true
    }
    /// Publishes `candidate` when the model accepts it; otherwise keeps the
    /// current document untouched and reports why.
    fn commit_actor_edit(&mut self, mut candidate: Scene, message: String) -> bool {
        candidate.refresh_actor_hierarchy();
        for actor in &mut candidate.actors {
            actor.refresh_components();
        }
        let model = self.object_model();
        if let Err(error) = candidate.validate_with_model(model.as_deref()) {
            self.last_error = Some(error.clone());
            self.log(error);
            return false;
        }
        self.scene = candidate;
        self.last_error = None;
        self.changed();
        self.log(message);
        true
    }
    pub fn begin_actor_rename(&mut self, id: uuid::Uuid) {
        if self.playing {
            return;
        }
        if let Some(actor) = self.scene.actors.iter().find(|a| a.id == id) {
            self.actor_rename = Some((id, actor.name.clone()));
            self.actor_rename_focus = true;
        }
    }
    /// Ends the inline rename. `commit` false discards the edit (Escape).
    pub fn finish_actor_rename(&mut self, commit: bool) {
        let Some((id, name)) = self.actor_rename.take() else {
            return;
        };
        self.actor_rename_focus = false;
        let name = name.trim().to_owned();
        if !commit || name.is_empty() {
            return;
        }
        let Some(index) = self.scene.actor_index(id) else {
            return;
        };
        if self.scene.actors[index].name == name {
            return;
        }
        let mut candidate = self.scene.clone();
        candidate.actors[index].name = name.clone();
        self.commit_actor_edit(candidate, format!("Renamed actor to {name}."));
    }
    /// Copies one actor and its logical branch, then selects the copy.
    pub fn duplicate_actor(&mut self, id: uuid::Uuid) {
        if !self.actor_editable() {
            return;
        }
        let mut candidate = self.scene.clone();
        let copy = match candidate.duplicate_actor_branch(id) {
            Ok(copy) => copy,
            Err(error) => {
                self.last_error = Some(error.clone());
                self.log(error);
                return;
            }
        };
        let name = candidate
            .actors
            .iter()
            .find(|a| a.id == copy)
            .map(|a| a.name.clone())
            .unwrap_or_default();
        if self.commit_actor_edit(candidate, format!("Duplicated actor as {name}.")) {
            self.select_actor(Some(copy));
        }
    }
    /// Removes one actor, every actor whose logical parent chain reaches it, and
    /// every attachment that pointed into the removed branch.
    pub fn delete_actor(&mut self, id: uuid::Uuid) {
        if !self.actor_editable() {
            return;
        }
        let mut candidate = self.scene.clone();
        let removed = candidate.delete_actor_branch(id);
        if removed.is_empty() {
            // Deleting nothing is refused rather than recorded as an empty step.
            let error = "That actor is no longer in this map.".to_owned();
            self.last_error = Some(error.clone());
            self.log(error);
            return;
        }
        let count = removed.len();
        if self.commit_actor_edit(
            candidate,
            if count == 1 {
                "Deleted 1 actor.".to_owned()
            } else {
                format!("Deleted {count} actors.")
            },
        ) {
            let selected = self
                .selected_actor
                .filter(|selected| !removed.contains(selected));
            self.select_actor(selected);
            if self
                .actor_rename
                .as_ref()
                .is_some_and(|(actor, _)| removed.contains(actor))
            {
                self.actor_rename = None;
                self.actor_rename_focus = false;
            }
        }
    }
    /// Changes an actor's logical parent. `parent` `None` moves it to the map
    /// root. The spatial `attach` follows only when both actors resolve to the
    /// same domain; across domains, and at the root, it is cleared. A parent that
    /// would close a cycle is refused by `Scene::validate_with_model`.
    pub fn reparent_actor(&mut self, child: uuid::Uuid, parent: Option<uuid::Uuid>) {
        if !self.actor_editable() {
            return;
        }
        if Some(child) == parent {
            let error = "An actor cannot be its own parent.".to_owned();
            self.last_error = Some(error.clone());
            self.log(error);
            return;
        }
        let Some(index) = self.scene.actor_index(child) else {
            return;
        };
        let model = self.object_model();
        let attach =
            crate::mcp_tools::implied_attachment(model.as_deref(), &self.scene, child, parent);
        let mut candidate = self.scene.clone();
        candidate.actors[index].logical_parent = parent;
        candidate.actors[index].attach = attach;
        let name = candidate.actors[index].name.clone();
        let message = match parent.and_then(|p| candidate.actors.iter().find(|a| a.id == p)) {
            Some(target) => format!("Parented {name} to {}.", target.name),
            None => format!("Moved {name} to the map root."),
        };
        self.commit_actor_edit(candidate, message);
    }
    /// Component classes the model accepts on `actor`, as
    /// `(group, cpp_name, label)`. Shared and logic components — the ones no
    /// single domain owns — come first, then the components of the actor's own
    /// domain, each group sorted by label.
    pub fn addable_component_classes(
        &self,
        actor: uuid::Uuid,
    ) -> Vec<(&'static str, String, String)> {
        let Some(model) = self.object_model() else {
            return Vec::new();
        };
        let Some(actor) = self.scene.actors.iter().find(|a| a.id == actor) else {
            return Vec::new();
        };
        let Some(owner) = actor.class.resolve(&model) else {
            return Vec::new();
        };
        let mut classes: Vec<_> = model
            .iter()
            .filter(|class| {
                class.component.is_some() && model.validate_component(&owner.id, &class.id).is_ok()
            })
            .filter(|class| {
                let mut candidate = actor.clone();
                candidate
                    .components
                    .push(crate::actor_document::ComponentInstance::new(
                        uuid::Uuid::new_v4(),
                        crate::actor_document::ClassReference::new(&class.cpp_name, &class.id),
                        "ComponentPreview",
                    ));
                crate::mcp_tools::validate_actor_components(&model, &candidate).is_ok()
            })
            .map(|class| {
                let owners = &class.component.as_ref().expect("a component").owners;
                // "Shared" is a component no single domain claims: it is offered
                // to this actor because of what it does, not where it lives.
                let group = if owners.len() == 1 {
                    "Domain"
                } else {
                    "Shared"
                };
                (
                    group,
                    class.cpp_name.clone(),
                    crate::actor_document::short_class_name(&class.cpp_name).to_owned(),
                )
            })
            .collect();
        // Shared before Domain, then alphabetical inside each group.
        let rank = |group: &str| usize::from(group == "Domain");
        classes.sort_by(|a, b| (rank(a.0), &a.2).cmp(&(rank(b.0), &b.2)));
        classes
    }
    /// Adds one component of `class` to `actor` with a fresh identity, a name
    /// unique inside that actor, `inherited = false` and `root = false`.
    pub fn add_actor_component(&mut self, actor: uuid::Uuid, class: &str) {
        if !self.actor_editable() {
            return;
        }
        let Some(model) = self.object_model() else {
            self.last_error = Some("The class model is unavailable; reload the project.".into());
            return;
        };
        let Some(index) = self.scene.actor_index(actor) else {
            return;
        };
        let Some(resolved) = model.class(class) else {
            self.last_error = Some(format!("`{class}` is not a reflected class."));
            return;
        };
        let owner_class = self.scene.actors[index].class.clone();
        let Some(owner) = owner_class.resolve(&model) else {
            self.last_error = Some(format!(
                "`{}` is not a reflected class; its components cannot be edited.",
                owner_class.name
            ));
            return;
        };
        if let Err(diagnostic) = model.validate_component(&owner.id, &resolved.id) {
            self.last_error = Some(diagnostic.message.clone());
            self.log(diagnostic.message);
            return;
        }
        let mut candidate = self.scene.clone();
        let target = &mut candidate.actors[index];
        let name = crate::actor_document::unique_component_name(
            target,
            crate::actor_document::short_class_name(&resolved.cpp_name),
        );
        let mut component = crate::actor_document::ComponentInstance::new(
            uuid::Uuid::new_v4(),
            crate::actor_document::ClassReference::new(&resolved.cpp_name, &resolved.id),
            &name,
        );
        // These adapters carry an authored document even before an asset is chosen.
        // Seed it so the Inspector projection retains the newly added component.
        match resolved.id.as_str() {
            crate::actor_components::PALETTE => {
                let animator = crate::palette::Animator {
                    texture: target
                        .material
                        .texture
                        .or(target.sprite.as_ref().and_then(|sprite| sprite.texture))
                        .or(target.image.as_ref().and_then(|image| image.texture)),
                    ..Default::default()
                };
                component
                    .properties
                    .insert("palette_animator".into(), serde_json::json!(animator));
            }
            crate::actor_components::TIMELINE => {
                component.properties.insert(
                    "timeline".into(),
                    serde_json::json!(crate::timeline_scene::Component::default()),
                );
            }
            crate::actor_components::EFFECT => {
                component.properties.insert(
                    "particle_effect".into(),
                    serde_json::json!(crate::particle_effect_scene::Component::default()),
                );
            }
            _ => {}
        }
        target.components.push(component);
        // Whole-actor rules (requires/excludes, cardinality, one root) are the
        // model's, not this method's: ask it before the document changes.
        if let Err(error) =
            crate::mcp_tools::validate_actor_components(&model, &candidate.actors[index])
        {
            self.last_error = Some(error.clone());
            self.log(error);
            return;
        }
        self.commit_actor_edit(candidate, format!("Added {name} ({}).", resolved.cpp_name));
    }
    /// Removes one component. The root and class-default (`inherited`) components
    /// are refused: the UI disables both, and this is the second line of defence.
    pub fn remove_actor_component(&mut self, actor: uuid::Uuid, component: uuid::Uuid) {
        if !self.actor_editable() {
            return;
        }
        let Some(index) = self.scene.actor_index(actor) else {
            return;
        };
        let Some(position) = self.scene.actors[index]
            .components
            .iter()
            .position(|c| c.id == component)
        else {
            return;
        };
        let existing = &self.scene.actors[index].components[position];
        if let Some(refusal) = crate::mcp_tools::component_removal_refusal(existing) {
            let refusal = format!("{}: {refusal}", existing.name);
            self.last_error = Some(refusal.clone());
            self.log(refusal);
            return;
        }
        let name = existing.name.clone();
        let mut candidate = self.scene.clone();
        candidate.actors[index].components.remove(position);
        // An attachment that named the removed component loses its target rather
        // than dangling; the actor keeps its logical place.
        for other in &mut candidate.actors {
            if other
                .attach
                .as_ref()
                .is_some_and(|at| at.component == Some(component))
            {
                other.attach = None;
            }
            for sibling in &mut other.components {
                if sibling.attach_parent == Some(component) {
                    sibling.attach_parent = None;
                }
            }
        }
        self.commit_actor_edit(candidate, format!("Removed component {name}."));
    }
    /// The project-wide scene Blueprint parent proposed by Map Settings.
    pub fn project_default_scene_script_parent(&self) -> Option<String> {
        self.project
            .as_ref()
            .and_then(|p| p.manifest.default_scene_script_parent.clone())
    }
    /// The project default scene-Blueprint parent, when it names a class that
    /// really is an `epok::SceneScriptActor` subclass. A manifest that names
    /// something else is a suggestion the editor declines rather than an error:
    /// the native class is used instead.
    fn default_scene_script_parent(&self) -> Option<crate::actor_document::ClassReference> {
        let requested = self.project_default_scene_script_parent()?;
        let model = self.object_model()?;
        let class = model.class(&requested)?;
        model
            .is_a(&class.id, crate::object_model::SCENE_SCRIPT_ACTOR_ID)
            .then(|| crate::actor_document::ClassReference::new(&class.cpp_name, &class.id))
    }
    /// Gives the open map its default scene Blueprint when it has none.
    ///
    /// This is not an edit: it does not mark the document dirty and it does not
    /// touch the file. The map is written with it the next time the author saves.
    ///
    /// A project whose reflection data does not resolve - no `epok::SceneScriptActor`
    /// in the model, because the host cannot extract the runtime headers - gets no
    /// default. Writing a parent the project cannot compile would turn a missing
    /// toolchain into a broken document.
    pub fn ensure_scene_script(&mut self) {
        let Some(model) = self.object_model() else {
            return;
        };
        if model
            .class(crate::object_model::SCENE_SCRIPT_ACTOR_ID)
            .is_none()
        {
            return;
        }
        let parent = self.default_scene_script_parent();
        self.scene.ensure_scene_script(parent.as_ref());
    }
    /// Changes the scene Blueprint's parent as one transaction: the model decides
    /// whether the class may be reparented at all, a speculative compile proves the
    /// graphs still generate, and anything else restores the previous parent. Undo
    /// is the map's, because the scene Blueprint lives in the document.
    pub fn set_scene_script_parent(&mut self, class: &str) {
        if self.playing {
            self.log("Stop the emulator before editing.");
            return;
        }
        let Some(model) = self.object_model() else {
            self.last_error = Some("The class model is unavailable; reload the project.".into());
            return;
        };
        let Some(parent) = model.class(class) else {
            self.last_error = Some(format!("`{class}` is not a reflected class."));
            return;
        };
        if !model.is_a(&parent.id, crate::object_model::SCENE_SCRIPT_ACTOR_ID) {
            self.last_error = Some(format!(
                "`{}` is not an epok::SceneScriptActor subclass; the scene Blueprint's parent was kept.",
                parent.cpp_name
            ));
            return;
        }
        let Some(script) = self.scene.scene_script.as_ref() else {
            self.last_error = Some("This map has no scene Blueprint yet.".into());
            return;
        };
        if script.parent.class_id.as_deref() == Some(parent.id.as_str()) {
            return;
        }
        let restore = self.scene.scene_script.clone();
        let class_id = script.blueprint.id.clone();
        if let Some(diagnostics) = model
            .validate_reparent(&class_id, &parent.id)
            .err()
            .filter(|_| model.class(&class_id).is_some())
        {
            self.last_error = Some(format!(
                "The scene Blueprint's parent was kept.\n{}",
                diagnostics
                    .iter()
                    .map(|d| format!("{}: {}", d.code, d.message))
                    .collect::<Vec<_>>()
                    .join("\n")
            ));
            return;
        }
        let script = self.scene.scene_script.as_mut().expect("checked above");
        script.parent = crate::actor_document::ClassReference::new(&parent.cpp_name, &parent.id);
        script.blueprint.parent = parent.id.clone();
        let rollback = self
            .scene
            .validate_with_model(Some(&model))
            .err()
            .or_else(|| self.speculative_scene_script_compile().err());
        if let Some(error) = rollback {
            self.scene.scene_script = restore;
            self.last_error = Some(format!(
                "The scene Blueprint's parent was kept; no graph data was touched.\n{error}"
            ));
            return;
        }
        self.last_error = None;
        self.changed();
        self.sync_embedded_blueprint();
        self.log(format!(
            "Scene Blueprint now derives from {}.",
            parent.cpp_name
        ));
    }
    /// Compiles the project with the map's current, unsaved scene Blueprint. The
    /// generated code is discarded: only whether it generates is interesting.
    fn speculative_scene_script_compile(&self) -> Result<(), String> {
        let Some(draft) = crate::blueprint_asset::embedded(&self.scene_file, &self.scene) else {
            return Ok(());
        };
        let mut files = crate::blueprint_asset::load_all(&self.root)?;
        files.retain(|file| file.path != draft.path);
        files.push(draft);
        files.sort_by(|a, b| a.path.cmp(&b.path));
        let mut native = self.class_registry.clone();
        native.classes.retain(|_, c| c.provider.id != "blueprint");
        crate::blueprint_compile::compile(&self.root, &native, &files)
            .map(|_| ())
            .map_err(|errors| {
                errors
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("\n")
            })
    }
    /// Opens this map's own Blueprint in the Blueprint editor. The document is the
    /// map's: it is never written to a `.epokbp`, and saving it saves the map.
    pub fn open_scene_blueprint(&mut self) -> Result<(), String> {
        let file = crate::blueprint_asset::embedded(&self.scene_file, &self.scene)
            .ok_or("This map has no scene Blueprint yet.")?;
        self.blueprint_editor.open_embedded(&file)?;
        self.blueprint_editor.maximized = true;
        Ok(())
    }
    /// Two-way synchronisation between the open embedded document and the map.
    ///
    /// An edit in the Blueprint editor becomes a map edit through `changed()`, so
    /// the map's dirty flag, Undo history and Save are the only ones. A change that
    /// came from the other direction - a scene Undo, a reopened map - is adopted by
    /// the Blueprint editor instead.
    pub fn sync_embedded_blueprint(&mut self) {
        if self.blueprint_editor.embedded_map() != Some(self.scene_file.as_path()) {
            return;
        }
        if let Some(edited) = self.blueprint_editor.take_embedded_edit() {
            if let Some(script) = self.scene.scene_script.as_mut() {
                script.blueprint = edited;
                // The Blueprint editor reports continuously while a node is
                // dragged; one gesture there is one step in the map's history.
                self.changed_coalesced("scene-blueprint-sync");
            }
            return;
        }
        if let Some(file) = crate::blueprint_asset::embedded(&self.scene_file, &self.scene) {
            self.blueprint_editor.adopt_embedded(&file);
        }
    }
    /// Creates this map's embedded scene Blueprint from the parent chosen in
    /// Map Settings, or the project default when the author has not chosen one.
    pub fn create_scene_script(&mut self) {
        use crate::actor_document as doc;
        if self.playing {
            self.log("Stop the emulator before editing.");
            return;
        }
        if self.scene.scene_script.is_some() {
            self.last_error = Some("This map already has a scene Blueprint.".into());
            return;
        }
        let Some(model) = self.object_model() else {
            self.last_error = Some("The class model is unavailable; reload the project.".into());
            return;
        };
        let requested = if self.map_scene_script_parent.is_empty() {
            self.project_default_scene_script_parent()
                .unwrap_or_else(|| crate::object_model::SCENE_SCRIPT_ACTOR_ID.to_owned())
        } else {
            self.map_scene_script_parent.clone()
        };
        let Some(parent) = model.class(&requested) else {
            self.last_error = Some(format!("`{requested}` is not a reflected class."));
            return;
        };
        let name = format!("{}_SceneScript", self.scene.name);
        self.scene.scene_script = Some(doc::SceneScript {
            parent: doc::ClassReference::new(&parent.cpp_name, &parent.id),
            blueprint: crate::blueprint_asset::BlueprintAsset::new(name.clone(), parent.id.clone()),
        });
        if let Err(error) = self.scene.validate_with_model(Some(&model)) {
            self.scene.scene_script = None;
            self.last_error = Some(error.clone());
            self.log(error);
            return;
        }
        self.last_error = None;
        self.changed();
        self.log(format!("Created scene Blueprint {name}."));
    }
    pub fn reset_instance_baseline(&mut self) {
        self.instance_baseline = self
            .scene
            .actors
            .iter()
            .filter(|e| e.blueprint_instance.is_some())
            .map(|e| {
                (
                    e.id,
                    (
                        e.clone(),
                        e.parent
                            .and_then(|index| self.scene.actors.get(index))
                            .map(|parent| parent.id),
                    ),
                )
            })
            .collect();
    }
    pub fn save(&mut self) -> bool {
        match self.scene.save(&self.scene_path()) {
            Ok(()) => {
                self.dirty = false;
                self.log("Scene saved.");
                true
            }
            Err(e) => {
                self.log(format!("Save failed: {e}"));
                false
            }
        }
    }
    pub fn has_unsaved_changes(&self) -> bool {
        // An embedded scene Blueprint is never separately unsaved: its edits are
        // the map's, and `dirty` already reports them.
        self.dirty
            || (!self.blueprint_editor.is_embedded() && self.blueprint_editor.dirty())
            || self.timeline_editor.dirty()
    }
    pub fn save_all(&mut self) -> bool {
        if self.critical_busy() {
            self.log("Wait for the current operation before saving.");
            return false;
        }
        if self.timeline_editor.dirty()
            && let Err(error) = self.timeline_editor.save()
        {
            self.log(format!("Timeline save failed: {error}"));
            return false;
        }
        // A map's own Blueprint has no file of its own: the map save below writes
        // it, so the outstanding edit is folded into the document first.
        self.sync_embedded_blueprint();
        self.blueprint_editor.save_to_map = false;
        if !self.blueprint_editor.is_embedded() && self.blueprint_editor.dirty() {
            if let Err(error) = self.blueprint_editor.save() {
                self.log(format!("Blueprint save failed: {error}"));
                return false;
            }
            self.refresh_scripts();
        }
        !self.dirty || self.save()
    }
    pub fn refresh_scripts(&mut self) {
        if self.blueprint_editor.open {
            self.blueprint_editor.compile_requested = true;
        }
        let catalog = scripts::catalog(&self.root).and_then(|catalog| {
            crate::blueprint::native_registry(&self.root, &catalog)
                .map(|registry| (catalog, registry))
        });
        self.apply_script_catalog(catalog);
    }
    fn apply_script_catalog(&mut self, catalog: ScriptCatalog) {
        self.registry_revision = self.registry_revision.wrapping_add(1);
        self.source_dirty = true;
        self.external_watch.request_refresh();
        match catalog {
            Ok((c, registry)) => {
                self.timeline_editor.registry_error = None;
                if let Err(error) =
                    crate::timeline_compile::observe_reflection(&self.root, &registry)
                {
                    self.timeline_editor.registry_error = Some(error.clone());
                    self.log(error);
                }
                self.class_registry = registry;
                self.catalog = c;
                let previous = self.scene.clone();
                let result = crate::blueprint_asset::load_all(&self.root).and_then(|files| {
                    crate::blueprint_templates::refresh_instances(
                        &mut self.scene,
                        &files,
                        &self.class_registry,
                    )
                });
                if let Err(error) = result {
                    self.log(format!("Blueprint instances preserved: {error}"));
                }
                self.reset_instance_baseline();
                if self.scene != previous {
                    self.changed();
                }
            }
            Err(e) => {
                self.timeline_editor.registry_error = Some(e.clone());
                if let Err(error) = crate::blueprint_dependencies::invalidate_all(&self.root, &e) {
                    self.log(error);
                }
                if let Err(error) = crate::timeline_compile::invalidate_all(&self.root, &e) {
                    self.log(error);
                }
                self.log(format!("Script metadata: {e}"));
                if let Ok(native) = scripts::native_catalog(&self.root) {
                    // Reflection itself is a common cause of this failure, so fall
                    // back to the legacy sidecar declarations rather than dropping
                    // the whole catalog and leaving nothing attachable.
                    let registry = crate::blueprint::native_registry(&self.root, &native)
                        .unwrap_or_else(|_| {
                            crate::blueprint::registry_from_catalog(&self.root, &native)
                        });
                    for (id, class) in registry.classes {
                        self.class_registry.classes.insert(id, class);
                    }
                    if self.catalog.is_empty() {
                        self.catalog = native;
                    }
                }
                self.reset_instance_baseline();
            }
        }
        // Editor tooling only. A definition file that cannot be written costs a
        // Lua author completion, never a catalog: the refresh continues.
        if let Err(error) = crate::lua_api_stub::write(&self.root, &self.class_registry) {
            self.log(format!("Lua API definitions: {error}"));
        }
    }
    pub fn attach(&mut self, name: &str) {
        if self.playing {
            return;
        }
        let result = (|| {
            let class = self
                .class_registry
                .named(name)
                .cloned()
                .ok_or("The ActorComponent class is unresolved.")?;
            if class.provider.id == "blueprint" {
                return crate::blueprint_workflow::attach_asset(self, &class.source.file);
            }
            let index = self.selected.ok_or("Select an Actor first.")?;
            let before = self.scene.clone();
            let candidate =
                crate::actor_scripts::assign(&before, index, &class, &self.class_registry)?;
            crate::blueprint_workflow::commit_scene(self, before, candidate, Some(index));
            Ok(())
        })();
        match result {
            Ok(()) => self.last_error = None,
            Err(error) => {
                self.log(&error);
                self.last_error = Some(error);
            }
        }
    }
    pub fn undo_attachment(&mut self, redo: bool) -> Result<(), String> {
        if self.playing {
            return Err("Stop Play before changing components.".into());
        }
        let stack = if redo {
            &mut self.script_redo
        } else {
            &mut self.script_undo
        };
        let (before, after) = stack
            .last()
            .ok_or("No component attachment to undo/redo.")?;
        if self.scene != *if redo { before } else { after } {
            return Err(
                "Intervening scene edits prevent attachment undo/redo; no changes were made."
                    .into(),
            );
        }
        self.scene = if redo { after.clone() } else { before.clone() };
        let entry = stack.pop().unwrap();
        if redo {
            self.script_undo.push(entry);
        } else {
            self.script_redo.push(entry);
        }
        self.reset_instance_baseline();
        self.changed();
        Ok(())
    }
    pub fn open_code(&mut self, path: &Path, line: Option<usize>) {
        let result = (|| -> Result<(), String> {
            let config = project::Config::load(&self.root)?;
            let exe = if !config.code.is_empty() {
                project::Config::executable(&self.root, &config.code)
            } else if cfg!(windows) {
                std::env::var_os("LOCALAPPDATA")
                    .map(PathBuf::from)
                    .unwrap_or_default()
                    .join("Programs/Microsoft VS Code/Code.exe")
            } else {
                PathBuf::from("code")
            };
            let mut command = Command::new(exe);
            command.arg("--reuse-window").arg(&self.root);
            if let Some(line) = line {
                command
                    .arg("--goto")
                    .arg(format!("{}:{line}", path.display()));
            } else {
                command.arg(path);
            }
            pipeline::quiet(&mut command);
            command
                .spawn()
                .map_err(|e| format!("VS Code: {e}. Set code in Editor.epokconfig"))?;
            Ok(())
        })();
        if let Err(e) = result {
            self.log(e);
        }
    }
    pub fn build(&mut self, run: bool) {
        self.hud_simulation.stop();
        self.build_target(run, false);
    }
    pub fn analyze_memory(&mut self) {
        self.build_request(false, false, true);
    }
    fn restore_build_report(&mut self) {
        if self.memory.summary.is_some() {
            return;
        }
        if let Some(summary) =
            crate::build_report::Summary::load(&self.root, self.blueprint_debug_enabled)
        {
            let current = crate::play::input(
                &self.root,
                self.scene_path(),
                self.scene.clone(),
                self.play_profile.clone(),
                false,
            )
            .and_then(|input| {
                crate::project::Config::load(&self.root).and_then(|config| {
                    crate::play_cache::request(
                        &self.root,
                        &input,
                        &config,
                        self.blueprint_debug_enabled,
                    )
                })
            })
            .ok();
            self.memory.stale = summary.request.is_none()
                || summary.request != current
                || summary.verify_inputs(&self.root).is_err();
            let target = if summary.debug {
                "stage:.epok/build-blueprint-debug"
            } else {
                "stage:.epok/build"
            };
            self.memory.stale |= crate::artifact_dependencies::Graph::load(&self.root)
                .ok()
                .and_then(|graph| graph.nodes.get(target).map(|node| !node.stale.is_empty()))
                .unwrap_or(true);
            self.memory.status_current = !self.memory.stale;
            self.memory.scene_signature = crate::scene_dependencies::signature(&self.scene);
            self.memory.profile = summary.profile.clone();
            self.memory.debug = summary.debug;
            self.memory.report = summary.report(&self.root);
            self.memory.summary = Some(summary);
        }
    }
    pub fn show_memory_report(&mut self) {
        self.restore_build_report();
        self.memory.stale |= self.pending_build
            || self.memory.profile != self.play_profile
            || self.memory.debug != self.blueprint_debug_enabled;
        if self.memory.report.is_some() {
            self.memory.error = None;
            self.memory.open = true;
            return;
        }
        self.memory.prompt = Some(match &self.memory.summary {
            Some(summary) if summary.verify_inputs(&self.root).is_ok() => {
                crate::memory_ui::Prompt::GenerateMissing {
                    automatic: summary.automatic_report,
                }
            }
            _ => crate::memory_ui::Prompt::BuildFirst,
        });
    }
    pub fn generate_memory_report(&mut self) {
        if self.critical_busy() || self.job.is_some() {
            return;
        }
        let Some(summary) = self.memory.summary.clone() else {
            self.show_memory_report();
            return;
        };
        self.memory.pending = true;
        self.memory.report_only = true;
        self.memory.error = None;
        self.memory.open = false;
        self.job_stale = false;
        self.job_target = None;
        self.focus_console = true;
        self.console.force_follow();
        self.job_stage = "Generating asset and memory report".into();
        self.job = Some(pipeline::Job::report(self.root.clone(), summary));
    }
    pub fn set_play_profile(&mut self, mut profile: crate::play::Profile) -> Result<(), String> {
        if self.job.is_some() || self.dependencies.busy() {
            return Err("Stop Play before changing its profile.".into());
        }
        profile.normalize();
        profile.validate()?;
        let mut manifest = crate::workspace::read_manifest(&self.root)?;
        manifest.play = profile.clone();
        crate::workspace::save_manifest(&self.root, &manifest)?;
        self.play_profile = profile.clone();
        self.memory.stale = true;
        self.pending_build = true;
        if let Some(project) = self.project.as_mut() {
            project.manifest.play = profile.clone();
        }
        if let Some(draft) = self.settings.project.as_mut() {
            draft.play = profile;
        }
        Ok(())
    }
    pub fn build_disc(&mut self) {
        self.build_target(false, true);
    }
    fn build_target(&mut self, run: bool, physical_disc: bool) {
        self.build_request(run, physical_disc, false);
    }
    fn build_request(&mut self, run: bool, physical_disc: bool, analyze: bool) {
        self.memory.status_current = false;
        if !physical_disc && let Err(error) = self.play_profile.validate_build(&self.root) {
            self.play_warning = Some(error.clone());
            self.last_error = Some(error.clone());
            self.log(error);
            return;
        }
        if self.scene_loading.is_some() {
            self.log("Wait for scene loading before Build/Play.");
            return;
        }
        if self.assets.busy || self.bake_job.is_some() {
            self.pending_build = false;
            self.log("Wait for asset import or lighting bake before Build/Play.");
            return;
        }
        if self.dependencies.busy() {
            self.pending_build = false;
            self.log("Wait for dependency installation to finish before Build/Play.");
            return;
        }
        if self.blueprint_editor.dirty() {
            self.pending_build = false;
            self.log("Save or discard the open Blueprint before Build/Play. Unsaved graphs are never replaced by a stale cooked asset.");
            return;
        }
        if self.timeline_editor.dirty() {
            self.pending_build = false;
            self.log("Save or discard the open Timeline/ParticleEffect before Build/Play. Unsaved effects are never replaced by a stale cooked asset.");
            return;
        }
        if self.job.is_some() {
            self.log("Build/emulator already active.");
            return;
        }
        if run
            && self.play_profile.runtime == crate::play::Runtime::PlayStation
            && self.play_profile.target == crate::play::Target::Serial
            && !crate::serial_support::tools_installed()
        {
            self.pending_build = false;
            self.serial_ui.open = true;
            self.serial_ui.draft = Some(self.preferences.serial.clone());
            self.serial_ui.tools_ready = false;
            self.serial_ui.error = None;
            self.serial_ui.status = "Serial components are missing or need repair. Download them into this Epok installation to continue.".into();
            return;
        }
        // Establish source observation history for this request before its worker
        // publishes outputs, so a just-built new asset does not restart itself.
        self.observe_playback_sources();
        self.pending_build = false;
        self.restart_target = None;
        self.job_stale = false;
        self.job_target = Some((run, physical_disc));
        if run {
            self.game_frame = None;
            self.native_frame = None;
            self.game_error = None;
        }
        self.last_error = None;
        self.focus_console = true;
        self.pending_since = None;
        let mut input = match crate::play::input(
            &self.root,
            self.scene_path(),
            self.scene.clone(),
            self.play_profile.clone(),
            physical_disc,
        ) {
            Ok(input) => input,
            Err(error) => {
                self.job_target = None;
                self.last_error = Some(error.clone());
                self.log(error);
                return;
            }
        };
        self.native_play_request = None;
        if run && self.play_profile.runtime == crate::play::Runtime::NativePc {
            let key = crate::scene_dependencies::hash((
                crate::scene_dependencies::signature(&input.scene),
                self.source_revision,
                self.assets.revision,
                &input.play_settings_signature,
            ));
            input.native_play_cache = self
                .native_play_cache
                .as_ref()
                .filter(|(cached, path)| cached == &key && path.is_file())
                .map(|(_, path)| path.clone());
            input.native_play_scene_ready = true;
            input.native_play_catalog = Some(self.catalog.clone());
            input.native_play_assets = Some(self.assets.index.clone());
            self.native_play_request = Some(key);
        }
        self.active_play_runtime = self.play_profile.runtime;
        self.active_play_target = self.play_profile.target;
        self.console.force_follow();
        self.memory.building_scene_signature = crate::scene_dependencies::signature(&self.scene);
        self.memory.stale = true;
        self.memory.report_only = false;
        self.job_progress = None;
        self.job_stage = if run && self.active_play_runtime == crate::play::Runtime::NativePc {
            "Building Native PC runtime"
        } else if run && self.active_play_target == crate::play::Target::Serial {
            "Preparing PSX connection"
        } else {
            "Building PSX program"
        }
        .into();
        self.job = Some(if analyze {
            self.memory.pending = true;
            self.memory.error = None;
            self.memory.open = false;
            pipeline::Job::analyze(self.root.clone(), input, self.blueprint_debug_enabled)
        } else {
            pipeline::Job::start_with_target(
                self.root.clone(),
                input,
                run,
                self.blueprint_debug_enabled,
                physical_disc,
            )
        });
        self.log(if physical_disc {
            "Packaging physical PSX disc..."
        } else if run && self.active_play_runtime == crate::play::Runtime::NativePc {
            "Building for Native PC Play..."
        } else if run {
            "Building for PSX Play..."
        } else {
            "Building C++ / PsyQo..."
        });
    }
    pub fn reparent(&mut self, index: usize, parent: Option<usize>, keep_world: bool) {
        if self.playing {
            return;
        }
        match self.scene.reparent(index, parent, keep_world) {
            Ok(()) => {
                self.selected_asset = None;
                self.selected = Some(index);
                self.changed();
            }
            Err(error) => {
                self.last_error = Some(error.clone());
                self.log(error);
            }
        }
    }
    pub fn begin_rename(&mut self, index: usize) {
        if self.playing {
            return;
        }
        self.selected_asset = None;
        self.selected = Some(index);
        self.rename = Some((index, self.scene.actors[index].name.clone()));
        self.rename_focus = true;
        self.reveal_selected = true;
        self.search.clear();
        self.scene_navigation = false;
    }
    pub fn finish_rename(&mut self, commit: bool) {
        if let Some((index, name)) = self.rename.take()
            && commit
            && index < self.scene.actors.len()
        {
            let name = name.trim();
            if name.is_empty() || name.len() > 128 || name.chars().any(char::is_control) {
                self.log("Name must contain 1-128 bytes without control characters");
                return;
            }
            if self.scene.actors[index].name != name {
                self.scene.actors[index].name = name.into();
                self.changed();
            }
        }
    }
    pub fn create_hud(&mut self, kind: &str) {
        if self.playing {
            return;
        }
        let original = self.scene.clone();
        let mut parent = self.selected.filter(|i| {
            self.scene.actors[*i].canvas.is_some() || self.scene.actors[*i].rect.is_some()
        });
        if kind != "canvas" && parent.is_none() {
            parent = self.scene.actors.iter().position(|e| e.canvas.is_some());
        }
        if kind != "canvas" && parent.is_none() {
            let mut canvas = Actor::cube("Canvas".into());
            canvas.kind = "Empty".into();
            canvas.position = [0.; 3];
            canvas.canvas = Some(Default::default());
            parent = Some(self.scene.actors.len());
            self.scene.actors.push(canvas);
        }
        let mut entity = Actor::cube(
            match kind {
                "canvas" => "Canvas",
                "text" => "Text",
                "progress" => "Progress Bar",
                "panel" => "Panel",
                _ => "Image",
            }
            .into(),
        );
        entity.kind = "Empty".into();
        entity.position = [0.; 3];
        if kind == "canvas" {
            entity.canvas = Some(Default::default());
        } else {
            entity.parent = parent;
            entity.rect = Some(Default::default());
            match kind {
                "text" => {
                    entity.text = Some(Default::default());
                    entity.rect.as_mut().unwrap().size = [160., 16.];
                }
                "progress" => {
                    entity.progress = Some(Default::default());
                    entity.rect.as_mut().unwrap().size = [120., 16.];
                }
                "panel" => {
                    entity.image = Some(Default::default());
                    entity.rect.as_mut().unwrap().size = [200., 120.];
                }
                _ => entity.image = Some(Default::default()),
            }
        }
        let base = entity.name.clone();
        let mut suffix = 1;
        while self.scene.actors.iter().any(|e| e.name == entity.name) {
            entity.name = format!("{base}.{suffix:03}");
            suffix += 1;
        }
        self.scene.actors.push(entity);
        if let Err(error) = self.scene.validate() {
            self.scene = original;
            self.log(error);
            return;
        }
        self.selected_asset = None;
        self.selected = Some(self.scene.actors.len() - 1);
        self.reveal_selected = true;
        self.search.clear();
        self.set_scene_2d(true);
        self.changed();
    }
    /// A node pause owns a native breakpoint. HTTP resume would execute that
    /// same hook again; the debug bridge rearms it only after its return.
    fn resume_blueprint_node(
        &mut self,
        debug: &mut crate::blueprint_debug::State,
    ) -> Result<bool, String> {
        if !debug.at_node {
            return Ok(false);
        }
        debug.request(crate::blueprint_debug::Command::Resume)?;
        self.paused = false;
        self.blueprint_editor.debug_node = None;
        Ok(true)
    }

    pub fn action(&mut self, action: &str) {
        if action == "instantiate-actor" || action == "instantiate-child-actor" {
            let parent = (action == "instantiate-child-actor")
                .then_some(self.selected)
                .flatten()
                .and_then(|index| self.scene.actors.get(index))
                .map(|actor| actor.id);
            crate::actor_workflow::begin(self, parent);
            return;
        }
        if self.critical_busy() && !(action == "play" && self.job.is_some()) {
            self.log("Editing is locked until the current operation finishes.");
            return;
        }
        if action.starts_with("light-") {
            crate::lighting_editor::create(
                self,
                if action.contains("point") {
                    crate::lighting::LightType::Point
                } else {
                    crate::lighting::LightType::Directional
                },
                action.ends_with("child"),
            );
            return;
        }
        if let Some(kind) = action.strip_prefix("ui-") {
            self.create_hud(kind);
            return;
        }

        if self.playing
            && [
                "add",
                "add-child",
                "blockout-mesh",
                "terrain-create",
                "empty",
                "child",
                "duplicate",
                "delete",
                "reload",
                "new-script",
                "new-lua",
            ]
            .contains(&action)
        {
            self.log("Stop the emulator before editing.");
            return;
        }
        match action {
            "save" => {
                self.save_all();
            }
            "reload" => {
                if self.dirty {
                    self.log("Save pending changes before Reload.");
                } else if let Err(error) = self.begin_scene_open(self.scene_path()) {
                    self.last_error = Some(error.clone());
                    self.focus_console = true;
                    self.log(error);
                }
            }
            "add" | "add-child" | "empty" | "child" => {
                if self.scene.actors.len() >= 512 {
                    self.log("Editor limit: 512 objects.");
                    return;
                }
                let mut entity = Actor::cube(
                    if action == "add" || action == "add-child" {
                        "Cube"
                    } else {
                        "GameObject"
                    }
                    .into(),
                );
                if action == "empty" || action == "child" {
                    entity.kind = "Empty".into();
                    entity.position = [0.; 3];
                }
                if action == "child" || action == "add-child" {
                    entity.parent = self.selected;
                    entity.position = [0.; 3];
                }
                let base = entity.name.clone();
                let mut suffix = 1;
                while self.scene.actors.iter().any(|e| e.name == entity.name) {
                    entity.name = format!("{base}.{suffix:03}");
                    suffix += 1;
                }
                self.scene.actors.push(entity);
                if let Err(error) = self.scene.validate() {
                    self.scene.actors.pop();
                    self.log(error);
                    return;
                }
                self.selected_asset = None;
                self.selected = Some(self.scene.actors.len() - 1);
                self.reveal_selected = true;
                self.search.clear();
                self.changed();
            }
            "blockout-mesh" => crate::mesh_editor::allocate_actor_data(self),
            "terrain-create" => {
                if let Err(error) = crate::terrain_editor::create(self) {
                    self.log(error);
                }
            }
            "duplicate" => {
                if let Some(i) = self.selected {
                    let original_ids = (0..self.scene.actors.len())
                        .filter(|index| self.scene.is_descendant(*index, i))
                        .map(|index| self.scene.actors[index].id)
                        .collect::<Vec<_>>();
                    let first = self.scene.actors.len();
                    match self.scene.duplicate_branch(i) {
                        Ok(index) => {
                            let identities = original_ids
                                .into_iter()
                                .zip(self.scene.actors[first..].iter().map(|e| e.id))
                                .collect();
                            crate::blueprint_refs::remap_duplicate(
                                &mut self.scene,
                                first,
                                &identities,
                                &self.class_registry,
                            );
                            self.selected_asset = None;
                            self.selected = Some(index);
                            self.changed();
                        }
                        Err(error) => self.log(error),
                    }
                }
            }
            "delete" => {
                if let Some(i) = self.selected {
                    self.scene.delete_branch(i);
                    self.selected_asset = None;
                    self.selected =
                        (!self.scene.actors.is_empty()).then(|| i.min(self.scene.actors.len() - 1));
                    self.changed();
                }
            }
            "play" => {
                self.memory.status_current = false;
                if let Some(job) = &self.job {
                    job.control(Control::Stop);
                    self.restart_target = None;
                    self.job_target = None;
                    self.game_capture = false;
                    self.set_buttons(0);
                    self.focus_game = false;
                    self.focus_console = false;
                    self.focus_scene = true;
                    self.log("Stopping...");
                    self.job_stage = "Cancelling operation...".into();
                } else {
                    self.build(true);
                }
            }
            "build" => self.build(false),
            "export-disc" => crate::export_ui::open(self),
            "build-disc" => self.build_disc(),
            "pause" => {
                if self.playing && !self.serial_ui.command_pending {
                    let debug = self
                        .job
                        .as_ref()
                        .and_then(|job| job.bridge.as_ref())
                        .map(|bridge| bridge.debug.clone());
                    if let Some(debug) = debug {
                        let result = self.resume_blueprint_node(&mut debug.lock().unwrap());
                        match result {
                            Ok(true) => return,
                            Err(error) => {
                                self.log(error);
                                return;
                            }
                            Ok(false) => {}
                        }
                    }
                    if let Some(job) = &self.job {
                        job.control(if self.paused {
                            Control::Resume
                        } else {
                            Control::Pause
                        });
                        if self.active_play_runtime == crate::play::Runtime::PlayStation
                            && self.active_play_target == crate::play::Target::Serial
                        {
                            self.serial_ui.command_pending = true;
                        }
                    }
                }
            }
            "serial-reset" | "serial-pause" | "serial-resume" => {
                if self.playing
                    && self.active_play_runtime == crate::play::Runtime::PlayStation
                    && self.active_play_target == crate::play::Target::Serial
                    && !self.serial_ui.command_pending
                    && let Some(job) = &self.job
                {
                    job.control(match action {
                        "serial-reset" => Control::Reset,
                        "serial-pause" => Control::Pause,
                        _ => Control::Resume,
                    });
                    self.serial_ui.command_pending = true;
                }
            }
            "new-blueprint" => crate::blueprint_workflow::begin(self, None),
            "new-script" => {
                self.script_creation = true;
                self.script_creation_context = Default::default();
                self.script_name.clear();
                self.script_parent = "epok::ActorComponent".into();
                self.script_folder.clear();
                self.script_search.clear();
                self.script_error = None;
            }
            "new-lua" => {
                self.lua_creation = true;
                self.lua_creation_context = Default::default();
                self.lua_name.clear();
                self.lua_parent = "epok::ActorComponent".into();
                self.lua_folder.clear();
                self.lua_search.clear();
                self.lua_error = None;
            }
            "edit-script" => {
                self.open_code(&self.root.join("assets/scripts"), None);
            }
            "settings" => crate::settings_ui::open_project(self),
            "rename" => {
                if let Some(i) = self.selected {
                    self.begin_actor_rename(self.scene.actors[i].id);
                }
            }
            "frame-selected" => {
                if let Some(i) = self.selected {
                    let scene = self
                        .timeline_editor
                        .scene_preview
                        .scene
                        .as_ref()
                        .filter(|_| self.timeline_editor.open && !self.playing)
                        .unwrap_or(&self.scene);
                    let world = scene.world_matrix(i);
                    let points: Vec<_> = scene
                        .actors
                        .iter()
                        .enumerate()
                        .filter(|(index, actor)| {
                            actor.kind == "Mesh"
                                && scene.is_active(*index)
                                && scene.is_descendant(*index, i)
                        })
                        .flat_map(|(index, actor)| {
                            let world = scene.world_matrix(index);
                            let points = if let Some(c) = &actor.skeletal_mesh {
                                c.model
                                    .as_ref()
                                    .map(|m| m.points(c.clip, c.time, c.looping))
                                    .unwrap_or_default()
                            } else {
                                crate::lighting::quads(actor)
                                    .into_iter()
                                    .flat_map(|q| q.points)
                                    .collect()
                            };
                            points.into_iter().map(move |p| world.point(p))
                        })
                        .collect();
                    {
                        if !points.is_empty() {
                            let low: [f32; 3] = std::array::from_fn(|c| {
                                points.iter().map(|p| p[c]).fold(f32::INFINITY, f32::min)
                            });
                            let high: [f32; 3] = std::array::from_fn(|c| {
                                points
                                    .iter()
                                    .map(|p| p[c])
                                    .fold(f32::NEG_INFINITY, f32::max)
                            });
                            self.view
                                .frame_bounds_in_panel(low, high, self.scene_panel_size);
                            self.view_dirty = true;
                            return;
                        }
                    }
                    let center = world.point([0.; 3]);
                    let extents: [f32; 3] = std::array::from_fn(|i| {
                        world.0[i][..3].iter().map(|v| v.abs()).sum::<f32>() * 0.5
                    });
                    self.view.frame_bounds_in_panel(
                        std::array::from_fn(|i| center[i] - extents[i]),
                        std::array::from_fn(|i| center[i] + extents[i]),
                        self.scene_panel_size,
                    );
                    self.view_dirty = true;
                }
            }
            "reset-view" => {
                self.view = viewport::View::default();
                self.view_dirty = true;
            }
            "export" => match crate::export::export_project(
                &self.root,
                &crate::scene_dependencies::Input::editor(self.scene_path(), self.scene.clone()),
            ) {
                Ok(p) => self.log(format!("Exported {}", p.display())),
                Err(e) => self.log(e),
            },
            _ => {}
        }
    }
    pub fn tick(&mut self) {
        // Continue collecting child results/timeouts even when Scene is hidden.
        // Rendering cadence and input still belong to the visible HUD viewport.
        if self.hud_simulation.update(0., false) {
            self.view_dirty = true;
        }
        if self.scene_loading.is_some() {
            self.poll_scene_open();
            return; // Present completion before resuming observers/imports.
        }
        self.dependencies.poll();
        if self.dependencies.busy() {
            return;
        }
        let mut profile_last = Instant::now();
        let profile_enabled = std::env::var_os("EPOK_PROFILE_STARTUP").is_some();
        let mut profile_mark = |stage: &str| {
            if profile_enabled {
                let now = Instant::now();
                let ms = now.duration_since(profile_last).as_secs_f64() * 1000.;
                if ms > 5. {
                    eprintln!("[editor-tick] {stage}: {ms:.1} ms");
                }
                profile_last = now;
            }
        };
        self.assets.tick();
        profile_mark("asset worker");
        if crate::texture::ids(&self.scene)
            != self.scene.textures.keys().copied().collect::<Vec<_>>()
        {
            let _ = crate::texture::resolve(&mut self.scene, &self.assets.index);
            self.view_dirty = true;
        }
        crate::skeletal_ui::tick(self);
        if std::mem::take(&mut self.assets.refresh_editor) {
            self.project_browser
                .previews
                .configure(&self.root, &self.assets.index);
            let fingerprint = self.assets.index.fingerprint();
            if self.asset_fingerprint.as_ref() != Some(&fingerprint) {
                let _ = crate::mesh::resolve(&mut self.scene, &self.assets.index);
                let _ = crate::terrain::resolve(&mut self.scene, &self.assets.index);
                let _ = crate::skeletal::resolve(&mut self.scene, &self.assets.index);
                let _ = crate::texture::resolve(&mut self.scene, &self.assets.index);
                crate::skeletal_ui::synchronize(self);
                crate::mesh_editor::synchronize(self);
                crate::terrain_editor::synchronize(self);
                self.view_dirty = true;
            }
            if self
                .asset_fingerprint
                .as_ref()
                .is_some_and(|old| old != &fingerprint)
            {
                self.invalidate_running_build();
                if let Err(error) =
                    crate::timeline_compile::observe_resources(&self.root, &self.assets.index)
                {
                    self.log(error);
                }
                self.log("Imported assets changed. Build pending.");
            }
            self.asset_fingerprint = Some(fingerprint);
        }
        for message in std::mem::take(&mut self.assets.messages) {
            self.log(message);
        }
        crate::lighting_editor::poll(self);
        profile_mark("asset resolution");
        let events = self
            .job
            .as_ref()
            .map(|j| j.events.try_iter().collect::<Vec<_>>())
            .unwrap_or_default();
        for event in events {
            match event {
                Event::Log(s) => self.log(s),
                Event::Stage(s) => {
                    self.job_stage = s;
                    self.job_progress = None;
                }
                Event::Progress(value) => {
                    self.job_progress = value.is_finite().then_some(value.clamp(0., 1.))
                }
                Event::MemoryReport(report) => {
                    if !self.job_stale {
                        self.memory.profile = report.profile.clone();
                        self.memory.debug = report.debug;
                        self.memory.report = Some(*report);
                        self.memory.path.clear();
                        self.memory.selected = None;
                    }
                }
                Event::BuildSummary(summary) => {
                    if !self.job_stale {
                        if summary.report_hash.is_none() {
                            self.memory.report = None;
                        }
                        self.memory.profile = summary.profile.clone();
                        self.memory.debug = summary.debug;
                        if !self.memory.report_only {
                            self.memory.scene_signature =
                                self.memory.building_scene_signature.clone();
                            self.memory.stale = false;
                            self.memory.status_current = true;
                        }
                        self.memory.summary = Some(*summary);
                    }
                }
                Event::SerialPorts(ports) => self.serial_ui.ports = ports,
                Event::SerialConfigured(serial) => {
                    self.preferences.serial = serial.clone();
                    if let Some(prefs) = self.settings.preferences.as_mut() {
                        prefs.serial = serial.clone();
                    }
                    self.serial_ui.draft = Some(serial);
                    if let Err(error) = self.preferences.save() {
                        self.log(format!("Could not remember the serial adapter: {error}"));
                    }
                }
                Event::SerialStatus(s) => {
                    self.serial_ui.status = s;
                    self.serial_ui.error = None;
                    self.serial_ui.tools_ready = crate::serial_support::tools_installed();
                }
                Event::SerialIssue(s) => {
                    self.serial_ui.open = true;
                    self.serial_ui.error = Some(s);
                    self.serial_ui.tools_ready = crate::serial_support::tools_installed();
                }
                Event::LightingBaked(bake) => {
                    if bake.fingerprint == crate::lighting::fingerprint(&self.scene) {
                        self.scene.bake = Some(bake);
                        // This is derived output from the current build, not a
                        // new authored edit. Do not cancel the job producing it.
                        self.bake_current = true;
                        self.dirty = true;
                        self.view_dirty = true;
                    }
                }
                Event::Built(p) => {
                    if self.active_play_runtime == crate::play::Runtime::NativePc
                        && let Some(key) = self.native_play_request.clone()
                    {
                        self.native_play_cache = Some((key, p.clone()));
                    }
                    self.external_watch.request_refresh();
                    self.source_dirty = true;
                    self.log(format!("Built {}", p.display()));
                }
                Event::Running(pid) => {
                    self.emulator_pid = Some(pid);
                    self.emulator_visible =
                        self.active_play_runtime == crate::play::Runtime::PlayStation;
                    self.focus_game = self.active_play_runtime == crate::play::Runtime::NativePc
                        || self.active_play_target == crate::play::Target::Embedded;
                    self.playing = true;
                    self.paused = false;
                    if self.job_stale {
                        if let Some(job) = &self.job {
                            job.control(Control::Stop);
                        }
                    } else {
                        self.log(
                            if self.active_play_runtime == crate::play::Runtime::NativePc {
                                "Native PC runtime running. Connecting Game view..."
                            } else {
                                "PCSX-Redux running. Connecting Game view..."
                            },
                        );
                    }
                }
                Event::SerialConnected => {
                    self.playing = true;
                    self.paused = false;
                    self.focus_console = true;
                    self.log("PSX runtime and resident handler connected. Pause / Continue and Reset PSX are available. Stop only disconnects the PC session.");
                }
                Event::SerialCommandPending(pending) => {
                    self.serial_ui.command_pending = pending;
                }
                Event::Paused(p) => {
                    self.paused = p;
                    let target = if self.active_play_runtime == crate::play::Runtime::NativePc {
                        "Native PC runtime"
                    } else if self.active_play_target == crate::play::Target::Serial {
                        "PSX"
                    } else {
                        "Emulator"
                    };
                    self.log(format!(
                        "{target} {}.",
                        if p { "paused" } else { "resumed" }
                    ));
                }
                Event::Finished(result) => {
                    self.serial_ui.command_pending = false;
                    self.external_watch.request_refresh();
                    if std::mem::take(&mut self.memory.pending) {
                        self.memory.open = true;
                        self.memory.error = result.as_ref().err().cloned();
                        self.memory.stale |= self.job_stale;
                    }
                    self.job_progress = None;
                    let resume_serial =
                        std::mem::take(&mut self.serial_ui.play_after_prepare) && result.is_ok();
                    let stale = self.job_stale;
                    self.job = None;
                    let target = self.job_target.take();
                    self.job_stale = false;
                    // A queued Running event may arrive after Stop, even in
                    // this same batch. Never leave a finished session focused
                    // on Game; build-only jobs must not change the Scene tab.
                    if self.playing {
                        self.focus_scene = true;
                    }
                    self.focus_game = false;
                    self.playing = false;
                    self.paused = false;
                    self.game_capture = false;
                    self.emulator_pid = None;
                    self.game_frame = None;
                    self.native_frame = None;
                    if stale {
                        self.log(
                            "Previous build/Play stopped. Changed sources require a fresh build.",
                        );
                    } else {
                        match result {
                            Ok(()) => self.log("Ready."),
                            Err(e) => {
                                // Retry only after a subsequent source edit. A
                                // compiler failure must not turn requested Play
                                // into a silent build-only recovery.
                                self.restart_target = target.filter(|(run, _)| *run);
                                self.last_error = Some(e.clone());
                                self.focus_console = true;
                                self.log(e);
                            }
                        }
                    }
                    if resume_serial {
                        self.build(true);
                    }
                }
            }
        }
        if !self.job_stale
            && let Some(bridge) = self.job.as_ref().and_then(|j| j.bridge.as_ref())
        {
            let state = bridge.state.lock().unwrap();
            if let Some(frame) = &state.frame {
                if self.game_frame.is_none()
                    && self.active_play_target == crate::play::Target::Embedded
                    && let Some(pid) = self.emulator_pid
                {
                    crate::native::emulator_window(pid, false);
                    self.emulator_visible = false;
                }
                self.game_frame = Some(frame.clone());
            }
            if let Some(error) = &state.error {
                self.game_error = Some(error.clone());
                self.game_capture = false;
                if let Some(pid) = self.emulator_pid {
                    crate::native::emulator_window(pid, true);
                    self.emulator_visible = true;
                }
            }
        }
        if !self.job_stale
            && let Some(native) = self.job.as_ref().and_then(|job| job.native.as_ref())
        {
            let state = native.state.lock().unwrap();
            if let Some(frame) = &state.frame {
                self.native_frame = Some(frame.clone());
            }
            if let Some(error) = &state.error {
                self.game_error = Some(error.clone());
                self.game_capture = false;
            }
        }
        // Camera navigation is read-only. Keep processing build/emulator events,
        // but defer idle dependency publication until the drag ends so graph
        // maintenance cannot consume a camera frame. Build/Play still validates
        // its own inputs and active jobs continue observing live edits normally.
        if self.scene_navigation && self.job.is_none() {
            return;
        }
        self.observe_external_sources();
        self.observe_project_sources();
        // Observation refreshes the editor and marks outputs stale. Only an
        // explicit Build, Play or analysis request may launch the compiler.
    }
    fn observe_project_sources(&mut self) {
        if self.source_revision != self.assets.revision
            || self.source_registry_revision != self.registry_revision
        {
            self.source_dirty = true;
        }
        let observation = if let Some(receiver) = &self.source_scan {
            match receiver.try_recv() {
                Ok(observation) => observation,
                Err(std::sync::mpsc::TryRecvError::Empty) => return,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => SourceObservation {
                    fingerprint: Err("Source observation worker stopped".into()),
                    playback: Err("Source observation worker stopped".into()),
                    scene: None,
                },
            }
        } else {
            if !self.source_dirty || self.last_poll.elapsed() < Duration::from_millis(300) {
                return;
            }
            self.source_dirty = false;
            self.source_revision = self.assets.revision;
            self.source_registry_revision = self.registry_revision;
            self.last_poll = Instant::now();
            let root = self.root.clone();
            let path = self.scene_file.clone();
            let scene = self.scene.clone();
            let registry = self.class_registry.clone();
            let registry_revision = self.registry_revision;
            let registry_error = self.scene_registry_error().map(str::to_owned);
            let navigation = self.scene_baseline_pending;
            let (sender, receiver) = std::sync::mpsc::channel();
            match std::thread::Builder::new()
                .name("epok-source-observation".into())
                .spawn(move || {
                    let mut observation = SourceObservation::read(
                        &root,
                        path,
                        scene,
                        registry,
                        registry_revision,
                        registry_error,
                    );
                    if let Some(scene) = &mut observation.scene {
                        scene.navigation = navigation;
                    }
                    let _ = sender.send(observation);
                }) {
                Ok(_) => {
                    self.source_scan = Some(receiver);
                    return;
                }
                Err(error) => SourceObservation {
                    fingerprint: Err(format!("Cannot observe sources: {error}")),
                    playback: Err(format!("Cannot observe sources: {error}")),
                    scene: None,
                },
            }
        };
        self.source_scan = None;
        self.accept_playback_sources(observation.playback);
        match observation.fingerprint {
            Ok(hash) => {
                let recovered = self.source_error.take().is_some();
                if hash != self.fingerprint || recovered {
                    self.fingerprint = hash;
                    if recovered {
                        self.last_error = None;
                    }
                    self.invalidate_running_build();
                    self.refresh_scripts();
                    // Reflection may be slower than the debounce interval.
                    self.pending_since = Some(Instant::now());
                    self.log("Source changes detected. Fresh build required.");
                }
            }
            Err(error) => {
                if self.source_error.as_ref() != Some(&error) {
                    self.invalidate_running_build();
                    self.source_error = Some(error.clone());
                    self.last_error = Some(format!("Source validation failed: {error}"));
                    self.timeline_editor.registry_error = Some(error.clone());
                    if let Err(failure) =
                        crate::timeline_compile::invalidate_all(&self.root, &error)
                    {
                        self.log(failure);
                    }
                    self.log(format!(
                        "Source validation failed; affected artifacts are stale: {error}"
                    ));
                }
            }
        }
        // Source refresh can replace types/bindings while the worker reads.
        // Discard that result; the next poll observes the new document/types.
        let observed_scene = observation.scene.filter(|observed| {
            observed.path == self.scene_file
                && observed.scene == self.scene
                && observed.registry_revision == self.registry_revision
                && observed.registry_error.as_deref() == self.scene_registry_error()
        });
        if observed_scene.is_none() {
            self.source_dirty = true;
        }
        match crate::staging_files::observe_native(&self.root).and_then(|_| {
            observed_scene.map_or(Ok(None), |observed| {
                let dependencies = observed.dependencies?;
                let changed = if observed.navigation {
                    dependencies.publish_navigation(&self.root)
                } else {
                    dependencies.publish(&self.root)
                }?;
                Ok(Some((changed, observed.navigation)))
            })
        }) {
            Ok(Some((changed, navigation))) => {
                self.scene_baseline_pending = false;
                let recovered = self.scene_dependency_error.take().is_some();
                if changed || (recovered && !navigation) {
                    self.invalidate_running_build();
                    self.log("Scene dependencies changed. Fresh build required.");
                }
            }
            Ok(None) => {} // An obsolete/pending read cannot report recovery.
            Err(error) => {
                if self.scene_dependency_error.as_ref() != Some(&error) {
                    self.scene_dependency_error = Some(error.clone());
                    self.invalidate_running_build();
                    self.log(format!("Source dependency observation failed: {error}"));
                }
            }
        }
    }
    fn scene_registry_error(&self) -> Option<&str> {
        self.source_error
            .as_deref()
            .or(self.timeline_editor.registry_error.as_deref())
            .or(self.external_observation_error.as_deref())
    }
    fn observe_external_sources(&mut self) {
        let target = if self.blueprint_debug_enabled {
            ".epok/build-blueprint-debug"
        } else {
            ".epok/build"
        };
        match self.external_watch.poll(&self.root, target) {
            Ok(Some(affected)) => {
                self.source_dirty = true; // Includes host configuration changes.
                let recovered = self.external_observation_error.take().is_some();
                if affected || recovered {
                    self.invalidate_running_build();
                    self.refresh_scripts();
                    self.pending_since = Some(Instant::now());
                    self.log("External native inputs changed. Fresh build required.");
                }
            }
            Ok(None) => {}
            Err(error) => {
                if self.external_observation_error.as_ref() != Some(&error) {
                    self.external_observation_error = Some(error.clone());
                    self.invalidate_running_build();
                    self.log(format!("External native input observation failed: {error}"));
                }
            }
        }
        if let Some(warning) = self.external_watch.take_warning() {
            self.log(warning);
        }
    }

    fn observe_playback_sources(&mut self) {
        self.accept_playback_sources(crate::timeline::inspect_playback(&self.root));
    }
    fn accept_playback_sources(
        &mut self,
        catalog: Result<crate::timeline::PlaybackSources, String>,
    ) {
        let target = if self.blueprint_debug_enabled {
            ".epok/build-blueprint-debug"
        } else {
            ".epok/build"
        };
        let observation = catalog
            .as_ref()
            .map_err(Clone::clone)
            .and_then(|catalog| self.playback_watch.accept(&self.root, target, catalog));
        match observation {
            Ok((valid, affected)) => {
                let recovered = self.playback_observation_error.take().is_some();
                self.timeline_editor.catalog_error = self.timeline_editor.asset.as_ref()
                    .filter(|asset| !valid.contains(&asset.id))
                    .map(|asset| format!("TimelineAsset {} cannot be identified uniquely in the current source catalog", asset.id));
                if affected || recovered {
                    self.invalidate_running_build();
                    self.log("Playback inputs changed. Fresh build required.");
                }
                self.blueprint_editor.accept_playback_sources(catalog);
            }
            Err(error) => {
                self.timeline_editor.catalog_error = Some(error.clone());
                if self.playback_observation_error.as_ref() != Some(&error) {
                    self.invalidate_running_build();
                    self.playback_observation_error = Some(error.clone());
                    self.log(format!("Playback source observation failed: {error}"));
                }
            }
        }
    }
    // Running games keep their launched snapshot. Cancel only an in-flight
    // build whose inputs changed; never schedule an automatic replacement.
    fn invalidate_running_build(&mut self) {
        // Any accepted source/asset/settings invalidation also invalidates the
        // zero-scan Native PC fast path. The immutable on-disk cache remains
        // available to normal preparation if its full content hash still fits.
        self.native_play_cache = None;
        self.memory.stale = true;
        self.pending_build = true;
        self.pending_since = Some(Instant::now());
        if let Some(job) = &self.job
            && !self.job_stale
            && !self.playing
        {
            self.restart_target = self.job_target;
            self.job_stale = true;
            job.control(Control::Stop);
            self.set_buttons(0);
            self.game_capture = false;
            self.game_frame = None;
            self.native_frame = None;
            self.game_error =
                Some("Build inputs changed. Press Build or Play to try again.".into());
            self.log(
                "Stopping the build because its inputs changed. Press Build or Play when ready.",
            );
        }
    }
}

impl Drop for Editor {
    fn drop(&mut self) {
        self.set_buttons(0);
        self.job = None;
        self.scene_loading = None;
        // The project lock is released only after the worker/emulator has stopped.
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn serial_controls_wait_for_confirmation_and_stop_still_disconnects() {
        let root =
            std::env::temp_dir().join(format!("epok-serial-controls-{}", uuid::Uuid::new_v4()));
        let mut editor = super::Editor::new(root.clone());
        editor.auto_build = false;
        editor.active_play_target = crate::play::Target::Serial;
        let (job, events, controls) = crate::pipeline::Job::test_channels();
        editor.job = Some(job);
        editor.playing = true;
        editor.action("pause");
        assert!(matches!(
            controls.try_recv(),
            Ok(crate::pipeline::Control::Pause)
        ));
        assert!(editor.serial_ui.command_pending && !editor.paused);
        editor.action("serial-reset");
        assert!(controls.try_recv().is_err(), "Commands must not overlap");
        events
            .send(crate::pipeline::Event::SerialCommandPending(false))
            .unwrap();
        events.send(crate::pipeline::Event::Paused(true)).unwrap();
        editor.tick();
        assert!(editor.paused && !editor.serial_ui.command_pending);
        editor.action("pause");
        assert!(matches!(
            controls.try_recv(),
            Ok(crate::pipeline::Control::Resume)
        ));
        assert!(
            editor.paused,
            "Sending Continue alone must not mark the PSX resumed"
        );
        editor.serial_ui.command_pending = false;
        editor.action("serial-reset");
        assert!(matches!(
            controls.try_recv(),
            Ok(crate::pipeline::Control::Reset)
        ));
        editor.action("play");
        assert!(matches!(
            controls.try_recv(),
            Ok(crate::pipeline::Control::Stop)
        ));
        events
            .send(crate::pipeline::Event::Finished(Ok(())))
            .unwrap();
        editor.tick();
        assert!(!editor.playing && !editor.paused && !editor.serial_ui.command_pending);
        drop(editor);
        if root.exists() {
            std::fs::remove_dir_all(root).unwrap();
        }
    }
    fn poll_sources(editor: &mut super::Editor) {
        // Explicit diagnostic request. Production observations are event-driven.
        editor.source_dirty = true;
        editor.last_poll = std::time::Instant::now() - std::time::Duration::from_secs(1);
        editor.tick();
        let started = std::time::Instant::now();
        while editor.source_scan.is_some() {
            assert!(
                started.elapsed().as_secs() < 10,
                "Source observation timed out"
            );
            std::thread::sleep(std::time::Duration::from_millis(2));
            editor.tick();
        }
    }
    use super::*;
    #[test]
    #[ignore = "Requires installed native SDK; builds a disposable project without launching"]
    fn editor_build_report_lifecycle_with_native_compiler() {
        let root = crate::workspace::tests::temp("native-report-lifecycle");
        let project =
            crate::workspace::create(&root, "Report lifecycle", crate::workspace::Template::Basic)
                .unwrap();
        let mut editor = Editor::open(project).unwrap();
        let finish = |editor: &mut Editor| {
            let started = Instant::now();
            while editor.job.is_some() {
                editor.tick();
                assert!(
                    started.elapsed() < Duration::from_secs(180),
                    "{:?}",
                    editor.logs
                );
                std::thread::sleep(Duration::from_millis(10));
            }
            assert!(
                editor.last_error.is_none(),
                "{:?}\n{:?}",
                editor.last_error,
                editor.logs
            );
            poll_sources(editor);
        };
        editor.show_memory_report();
        assert_eq!(
            editor.memory.prompt,
            Some(crate::memory_ui::Prompt::BuildFirst)
        );
        editor.memory.prompt = None;
        editor.analyze_memory();
        finish(&mut editor);
        assert!(
            editor.memory.status_current && !editor.memory.stale,
            "{:?}",
            editor.logs
        );
        assert!(editor.memory.report.is_some() && editor.memory.open);
        editor.memory.open = false;
        editor.show_memory_report();
        assert!(editor.memory.open && editor.job.is_none() && editor.memory.prompt.is_none());
        editor.memory.open = false;
        let mut settings = editor.project.as_ref().unwrap().manifest.clone();
        settings.build.generate_asset_report = false;
        editor.apply_project_configuration(settings, None).unwrap();
        editor.build(false);
        finish(&mut editor);
        assert!(
            editor.memory.report.is_none() && editor.memory.status_current && !editor.memory.stale
        );
        editor.show_memory_report();
        assert_eq!(
            editor.memory.prompt,
            Some(crate::memory_ui::Prompt::GenerateMissing { automatic: false })
        );
        editor.memory.prompt = None;
        let exe = std::fs::read(root.join(".epok/build/epok.ps-exe")).unwrap();
        editor.generate_memory_report();
        finish(&mut editor);
        assert!(editor.memory.open && editor.memory.report.is_some());
        assert_eq!(
            exe,
            std::fs::read(root.join(".epok/build/epok.ps-exe")).unwrap()
        );
        editor.memory.open = false;
        editor.scene.actors[0].active = !editor.scene.actors[0].active;
        editor.changed();
        editor.show_memory_report();
        assert!(editor.memory.open && editor.memory.stale && editor.job.is_none());
        drop(editor);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn asset_report_button_covers_missing_fresh_disabled_and_stale_builds() {
        let root = crate::workspace::tests::temp("report-button-states");
        let mut editor = Editor::new(root.clone());
        editor.show_memory_report();
        assert_eq!(
            editor.memory.prompt,
            Some(crate::memory_ui::Prompt::BuildFirst)
        );
        assert!(editor.job.is_none());
        editor.memory.prompt = None; // Cancel is inert.
        let summary = crate::build_report::tests::fixture(&root, false, false);
        editor.memory.summary = Some(summary.clone());
        editor.show_memory_report();
        assert_eq!(
            editor.memory.prompt,
            Some(crate::memory_ui::Prompt::GenerateMissing { automatic: false })
        );
        assert!(editor.job.is_none());
        let summary = crate::build_report::tests::fixture(&root, true, true);
        editor.memory.prompt = None;
        editor.memory.summary = Some(summary.clone());
        editor.memory.report = summary.report(&root);
        editor.memory.scene_signature = crate::scene_dependencies::signature(&editor.scene);
        editor.memory.stale = false;
        editor.memory.status_current = true;
        editor.show_memory_report();
        assert!(editor.memory.open && !editor.memory.stale && editor.job.is_none());
        editor.memory.open = false;
        editor.scene.actors[0].active = !editor.scene.actors[0].active;
        editor.changed();
        editor.show_memory_report();
        assert!(editor.memory.open && editor.memory.stale && editor.job.is_none());
        assert!(editor.memory.report.is_some());
        // Even an unsuccessful Play attempt hides previous status totals.
        editor.play_profile.content = crate::play::Content::SelectedScenes;
        editor.build(true);
        assert!(!editor.memory.status_current && editor.memory.report.is_some());
        drop(editor);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn successful_build_publishes_sizes_without_opening_report_automatically() {
        let root = crate::workspace::tests::temp("report-build-events");
        let mut editor = Editor::new(root.clone());
        let summary = crate::build_report::tests::fixture(&root, true, true);
        let (job, events, _controls) = pipeline::Job::test_channels();
        editor.job = Some(job);
        editor.memory.building_scene_signature =
            crate::scene_dependencies::signature(&editor.scene);
        editor.memory.stale = true;
        events
            .send(Event::MemoryReport(Box::new(
                summary.report(&root).unwrap(),
            )))
            .unwrap();
        events
            .send(Event::BuildSummary(Box::new(summary.clone())))
            .unwrap();
        events.send(Event::Finished(Ok(()))).unwrap();
        editor.tick();
        assert!(
            editor.memory.status_current && !editor.memory.stale && editor.memory.report.is_some()
        );
        assert!(!editor.memory.open);
        let (job, events, _controls) = pipeline::Job::test_channels();
        editor.job = Some(job);
        editor.memory.pending = true;
        events
            .send(Event::MemoryReport(Box::new(
                summary.report(&root).unwrap(),
            )))
            .unwrap();
        events.send(Event::BuildSummary(Box::new(summary))).unwrap();
        events.send(Event::Finished(Ok(()))).unwrap();
        editor.tick();
        assert!(editor.memory.open && !editor.memory.pending);
        drop(editor);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn scene_activation_refreshes_view_without_compiling_or_stopping_play() {
        let root = crate::workspace::tests::temp("manual-scene-edit");
        let mut editor = Editor::new(root.clone());
        editor.auto_build = true; // Legacy preference is no longer a scheduler.
        for active in [false, true] {
            editor.view_dirty = false;
            editor.scene.actors[0].active = active;
            editor.changed();
            editor.pending_since = Some(Instant::now() - Duration::from_secs(2));
            poll_sources(&mut editor);
            assert!(editor.view_dirty && editor.pending_build && editor.job.is_none());
        }
        let (job, _events, controls) = pipeline::Job::test_channels();
        editor.job = Some(job);
        editor.playing = true;
        editor.changed();
        assert!(controls.try_recv().is_err());
        assert!(editor.playing && !editor.job_stale && editor.pending_build);
        drop(editor);
        if root.exists() {
            std::fs::remove_dir_all(root).unwrap();
        }
    }
    #[test]
    fn empty_selected_scenes_warns_before_build_or_serial_setup() {
        let root = crate::workspace::tests::temp("empty-selected-play");
        let mut editor = Editor::new(root.clone());
        editor.play_profile.content = crate::play::Content::SelectedScenes;
        editor.play_profile.target = crate::play::Target::Serial;
        for run in [false, true] {
            editor.build(run);
            assert!(editor.job.is_none());
            assert!(editor.play_warning.as_ref().unwrap().contains("no scenes"));
            assert!(!editor.serial_ui.open);
        }
        drop(editor);
        if root.exists() {
            std::fs::remove_dir_all(root).unwrap();
        }
    }
    /// The three native classes Map Settings reasons about. Hand built for the same
    /// reason the P5/P6 compiler tests build theirs: extraction needs the MIPS
    /// include paths, which this host may not have.
    fn scene_script_registry() -> crate::blueprint::Registry {
        use crate::reflection_schema as schema;
        let class = |id: &str, cpp_name: &str, parent: Option<&str>| schema::Class {
            family: None,
            domain: None,
            placement: Default::default(),
            component: None,
            default_components: vec![],
            explicit_abstract: false,
            id: id.into(),
            provider: schema::native_provider(),
            backend: schema::native_backend(),
            cpp_name: cpp_name.into(),
            parent: parent.map(str::to_owned),
            abstract_class: false,
            final_class: false,
            timeline_component: None,
            blueprintable: true,
            properties: vec![],
            functions: vec![],
            source: schema::Location {
                file: "runtime/object_model.hpp".into(),
                line: 1,
                column: 1,
            },
        };
        use crate::object_model as om;
        let mut actor = class(om::ACTOR_ID, "epok::Actor", None);
        actor.family = Some(schema::ClassFamily::Actor);
        actor.abstract_class = true;
        actor.explicit_abstract = true;
        let mut script = class(
            om::SCENE_SCRIPT_ACTOR_ID,
            "epok::SceneScriptActor",
            Some(om::ACTOR_ID),
        );
        script.placement.scene_managed = true;
        let mut actor3d = class(om::ACTOR3D_ID, "epok::Actor3D", Some(om::ACTOR_ID));
        actor3d.domain = Some(schema::Domain::World3D);
        actor3d.placement = schema::Placement {
            placeable: true,
            spawnable: true,
            scene_managed: false,
        };
        let mut registry = crate::blueprint::Registry::new();
        for class in [actor, script, actor3d] {
            registry.classes.insert(class.id.clone(), class);
        }
        registry
    }

    /// Map Settings offers only `SceneScriptActor` subclasses, and a parent change is
    /// one transaction: the model decides first, and a refusal leaves the document -
    /// including every graph in the scene Blueprint - exactly as it was.
    #[test]
    fn changing_the_scene_blueprint_parent_is_a_transaction_that_rolls_back() {
        let root = crate::workspace::tests::temp("scene-script-parent");
        let mut editor = Editor::new(root.clone());
        editor.class_registry = scene_script_registry();
        editor.registry_revision = editor.registry_revision.wrapping_add(1);
        let model = editor.object_model().expect("the fixture model resolves");
        assert_eq!(
            model
                .scene_script_parents()
                .map(|c| c.cpp_name.as_str())
                .collect::<Vec<_>>(),
            vec!["epok::SceneScriptActor"],
            "the combo lists SceneScriptActor subclasses and nothing else"
        );
        drop(model);
        editor.ensure_scene_script();
        let before = editor.scene.clone();
        assert!(before.scene_script.is_some());

        editor.set_scene_script_parent("epok::Actor3D");
        assert!(
            editor
                .last_error
                .as_deref()
                .is_some_and(|e| e.contains("not an epok::SceneScriptActor subclass")),
            "{:?}",
            editor.last_error
        );
        assert_eq!(editor.scene, before, "a refused parent changes nothing");

        editor.set_scene_script_parent("Nonexistent");
        assert!(editor.last_error.is_some());
        assert_eq!(editor.scene, before);

        // The same class is accepted, and Undo steps back through the map's history.
        editor.set_scene_script_parent("epok::SceneScriptActor");
        assert_eq!(
            editor.scene, before,
            "the parent it already has is not an edit"
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn scene_undo_restores_the_previous_document_and_stops_at_the_oldest_step() {
        let root = crate::workspace::tests::temp("scene-undo");
        let mut editor = Editor::new(root.clone());
        let original = editor.scene.clone();
        assert!(!editor.can_undo_scene());
        assert!(editor.undo_scene().is_err());
        editor.scene.name = "First".into();
        editor.changed();
        editor.scene.name = "Second".into();
        editor.changed();
        assert!(editor.can_undo_scene());
        editor.undo_scene().unwrap();
        assert_eq!(editor.scene.name, "First");
        editor.undo_scene().unwrap();
        assert_eq!(editor.scene, original);
        assert!(editor.undo_scene().is_err());
        editor.redo_scene().unwrap();
        assert_eq!(editor.scene.name, "First");
        editor.redo_scene().unwrap();
        assert_eq!(editor.scene.name, "Second");
        assert!(editor.redo_scene().is_err());
        // Play owns the document; undo is refused rather than applied blindly.
        editor.playing = true;
        assert!(editor.undo_scene().is_err());
        editor.playing = false;
        // The view mode is the old boolean plus a 2D world, and never edits the scene.
        let before = editor.scene.clone();
        editor.set_scene_view_mode(crate::scene_view_mode::SceneViewMode::TwoD);
        assert!(!editor.scene_2d());
        editor.set_scene_2d(true);
        assert!(editor.scene_2d());
        editor.set_scene_2d(false);
        assert_eq!(editor.scene, before);
        drop(editor);
        if root.exists() {
            std::fs::remove_dir_all(root).unwrap();
        }
    }
    /// The Hierarchy's and Inspector's actor commands, exercised through the
    /// same `Editor` methods the UI calls.
    #[test]
    fn add_component_choices_respect_the_existing_component_set() {
        use crate::{actor_document::tests as fixture, object_model as om};
        let root = crate::workspace::tests::temp("component-choices");
        let mut editor = Editor::new(root.clone());
        editor.auto_build = false;
        editor.scene.actors.clear();
        let mut registry = fixture::registry();
        let mut needs_audio =
            fixture::class("needs-audio", "NeedsAudio", Some(om::ACTOR_COMPONENT_ID));
        needs_audio.component = Some(crate::reflection_schema::ComponentContract {
            requires: vec![om::AUDIO_COMPONENT_ID.into()],
            ..Default::default()
        });
        let mut excludes_audio = fixture::class(
            "excludes-audio",
            "ExcludesAudio",
            Some(om::ACTOR_COMPONENT_ID),
        );
        excludes_audio.component = Some(crate::reflection_schema::ComponentContract {
            excludes: vec![om::AUDIO_COMPONENT_ID.into()],
            ..Default::default()
        });
        for class in [
            needs_audio,
            excludes_audio,
            fixture::class(
                crate::actor_components::TIMELINE,
                "epok::TimelineComponent",
                Some(om::ACTOR_COMPONENT_ID),
            ),
            fixture::class(
                crate::actor_components::EFFECT,
                "epok::ParticleEffectComponent",
                Some(om::ACTOR_COMPONENT_ID),
            ),
        ] {
            registry.classes.insert(class.id.clone(), class);
        }
        editor.class_registry = registry;
        editor.registry_revision += 1;
        editor.create_actor("epok::Actor3D");
        let actor = editor.selected_actor.unwrap();
        let names = |editor: &Editor| {
            editor
                .addable_component_classes(actor)
                .into_iter()
                .map(|(_, name, _)| name)
                .collect::<Vec<_>>()
        };
        let offered = names(&editor);
        assert!(offered.iter().any(|name| name == "ExcludesAudio"));
        for hidden in [
            "NeedsAudio",
            "epok::Actor3D",
            "epok::ActorComponent",
            "epok::SceneComponent3D",
            "epok::SceneComponent2D",
            "epok::UIComponent",
        ] {
            assert!(
                !offered.iter().any(|name| name == hidden),
                "must not offer {hidden}"
            );
        }
        editor.add_actor_component(actor, "epok::AudioComponent");
        let offered = names(&editor);
        assert!(offered.iter().any(|name| name == "NeedsAudio"));
        assert!(offered.iter().any(|name| name == "epok::AudioComponent"));
        assert!(!offered.iter().any(|name| name == "ExcludesAudio"));
        editor.add_actor_component(actor, "NeedsAudio");
        assert!(!names(&editor).iter().any(|name| name == "NeedsAudio"));
        editor.add_actor_component(actor, "epok::TimelineComponent");
        editor.add_actor_component(actor, "epok::ParticleEffectComponent");
        assert!(editor.last_error.is_none(), "{:?}", editor.last_error);
        assert!(editor.scene.actors[0].timeline.is_some());
        assert!(editor.scene.actors[0].particle_effect.is_some());
        editor.scene.sync_actor_components();
        assert!(
            editor.scene.actors[0]
                .components
                .iter()
                .any(|c| c.class.name == "epok::TimelineComponent")
        );
        assert!(
            !names(&editor)
                .iter()
                .any(|name| name == "epok::TimelineComponent")
        );
        drop(editor);
        if root.exists() {
            std::fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn actor_operations_rename_duplicate_delete_reparent_and_edit_components() {
        let root = crate::workspace::tests::temp("actor-operations");
        let mut editor = Editor::new(root.clone());
        editor.auto_build = false;
        editor.scene.actors.clear();
        editor.catalog = crate::mcp_tests::actor_catalog();
        editor.class_registry =
            crate::blueprint::registry_from_catalog(&editor.root, &editor.catalog);
        editor.registry_revision += 1;
        assert!(editor.object_model().is_some());
        editor.create_actor("epok::Actor3D");
        let hero = editor.selected_actor.unwrap();
        editor.create_actor("epok::Actor3D");
        let child = editor.selected_actor.unwrap();
        editor.create_actor("epok::Actor2D");
        let sprite = editor.selected_actor.unwrap();
        assert_eq!(editor.scene.actors.len(), 3);

        // Rename is inline and commits only what the author accepted.
        editor.begin_actor_rename(hero);
        editor.actor_rename.as_mut().unwrap().1 = "Champion".into();
        editor.finish_actor_rename(false);
        assert_eq!(editor.scene.actors[0].name, "Actor3D");
        editor.begin_actor_rename(hero);
        editor.actor_rename.as_mut().unwrap().1 = "Champion".into();
        editor.finish_actor_rename(true);
        assert_eq!(editor.scene.actors[0].name, "Champion");

        // Same domain: the parent brings a spatial attachment. Across domains it
        // does not, and the map root clears both.
        editor.reparent_actor(child, Some(hero));
        assert_eq!(editor.scene.actors[1].logical_parent, Some(hero));
        assert_eq!(
            editor.scene.actors[1].attach.as_ref().map(|a| a.actor),
            Some(hero)
        );
        editor.reparent_actor(sprite, Some(hero));
        assert_eq!(editor.scene.actors[2].logical_parent, Some(hero));
        assert!(editor.scene.actors[2].attach.is_none());
        // A cycle is refused and the document is untouched.
        let before = editor.scene.clone();
        editor.reparent_actor(hero, Some(child));
        assert_eq!(editor.scene, before);
        assert!(editor.last_error.is_some());

        // Components: fresh identity, unique name, never root and never inherited.
        editor.add_actor_component(hero, "epok::AudioComponent");
        editor.add_actor_component(hero, "epok::AudioComponent");
        let components = &editor.scene.actors[0].components;
        assert_eq!(components.len(), 3);
        assert_eq!(components[1].name, "AudioComponent");
        assert_eq!(components[2].name, "AudioComponent.001");
        assert!(!components[1].root && !components[1].inherited);
        assert_ne!(components[1].id, components[2].id);
        // The Add Component popup offers exactly what the model accepts, with
        // the shared components first.
        let offered = editor.addable_component_classes(hero);
        assert_eq!(offered[0].0, "Shared");
        assert!(
            offered
                .iter()
                .any(|(_, class, _)| class == "epok::AudioComponent")
        );
        assert!(
            !offered
                .iter()
                .any(|(_, class, _)| class == "epok::SceneComponent2D")
        );
        // The root and class defaults are refused; the set is otherwise the
        // model's to judge.
        let audio = editor.scene.actors[0].components[1].id;
        let root_component = editor.scene.actors[0].components[0].id;
        editor.remove_actor_component(hero, root_component);
        assert_eq!(editor.scene.actors[0].components.len(), 3);
        editor.scene.actors[0].components[1].inherited = true;
        editor.remove_actor_component(hero, audio);
        assert_eq!(editor.scene.actors[0].components.len(), 3);
        editor.scene.actors[0].components[1].inherited = false;
        editor.remove_actor_component(hero, audio);
        assert_eq!(editor.scene.actors[0].components.len(), 2);
        editor.add_actor_component(hero, "epok::SceneComponent2D");
        assert_eq!(editor.scene.actors[0].components.len(), 2);

        // Duplicate copies the whole logical branch with fresh identities.
        let before = editor.scene.actors.len();
        editor.duplicate_actor(hero);
        assert_eq!(editor.scene.actors.len(), before * 2);
        let copy = editor.selected_actor.unwrap();
        assert_ne!(copy, hero);
        assert_eq!(editor.scene.actors[before].name, "Champion.001");
        let ids: std::collections::BTreeSet<_> = editor
            .scene
            .actors
            .iter()
            .flat_map(|a| a.components.iter().map(|c| c.id).chain([a.id]))
            .collect();
        assert_eq!(
            ids.len(),
            editor
                .scene
                .actors
                .iter()
                .map(|a| a.components.len() + 1)
                .sum::<usize>()
        );

        // Delete takes the branch and clears what pointed into it.
        editor.delete_actor(copy);
        assert_eq!(editor.scene.actors.len(), before);
        editor.select_actor(Some(hero));
        editor.delete_actor(hero);
        assert!(editor.scene.actors.is_empty());
        assert!(editor.selected_actor.is_none());
        assert!(editor.selected.is_none());
        // Deleting nothing is refused rather than recorded as an empty step.
        let before = editor.scene.clone();
        editor.delete_actor(hero);
        assert_eq!(editor.scene, before);
        // Undo walks back through the whole sequence.
        editor.undo_scene().unwrap();
        assert!(!editor.scene.actors.is_empty());
        drop(editor);
        if root.exists() {
            std::fs::remove_dir_all(root).unwrap();
        }
    }
    /// A drag reports every frame; the history keeps one step for the gesture.
    #[test]
    fn a_coalesced_gesture_records_one_scene_history_step() {
        let root = crate::workspace::tests::temp("coalesced-gesture");
        let mut editor = Editor::new(root.clone());
        let original = editor.scene.clone();
        for step in 0..50 {
            editor.scene.actors[0].position[0] = step as f32;
            editor.changed_coalesced("gizmo-drag");
        }
        editor.end_coalesced();
        assert!(editor.dirty && editor.view_dirty);
        editor.undo_scene().unwrap();
        assert_eq!(editor.scene, original);
        assert!(editor.undo_scene().is_err());
        // A different key, and a discrete edit, each start their own step.
        editor.redo_scene().unwrap();
        editor.scene.name = "Other".into();
        editor.changed_coalesced("hud-rect-drag");
        editor.scene.name = "Discrete".into();
        editor.changed();
        editor.undo_scene().unwrap();
        assert_eq!(editor.scene.name, "Other");
        editor.undo_scene().unwrap();
        assert_eq!(editor.scene.name, original.name);
        drop(editor);
        if root.exists() {
            std::fs::remove_dir_all(root).unwrap();
        }
    }
    #[test]
    fn lighting_output_does_not_cancel_its_own_manual_build() {
        let root = crate::workspace::tests::temp("manual-build-lighting");
        let mut editor = Editor::new(root.clone());
        let (job, events, controls) = pipeline::Job::test_channels();
        editor.job = Some(job);
        editor.job_target = Some((false, false));
        let bake = crate::lighting::bake(&editor.scene).unwrap();
        events.send(Event::LightingBaked(bake)).unwrap();
        editor.tick();
        assert!(editor.bake_current && editor.view_dirty);
        assert!(!editor.job_stale && !editor.pending_build);
        assert!(controls.try_recv().is_err());
        drop(editor);
        if root.exists() {
            std::fs::remove_dir_all(root).unwrap();
        }
    }
    #[test]
    #[ignore = "requires a disposable EPOK_IDLE_PROJECT and the installed SDK"]
    fn profile_apply_debug_settings_without_reload() {
        let root = PathBuf::from(std::env::var_os("EPOK_IDLE_PROJECT").unwrap());
        let mut editor = Editor::open(crate::workspace::Project::open(&root).unwrap()).unwrap();
        editor
            .open_scene(root.join("assets/scenes/ForestClearing.epokmap"))
            .unwrap();
        let started = Instant::now();
        while started.elapsed() < Duration::from_secs(2) {
            editor.tick();
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(
            !editor.pending_build && editor.job.is_none(),
            "{:?}",
            editor.logs
        );
        let original = editor.project.as_ref().unwrap().manifest.clone();
        let revision = editor.registry_revision;
        let scene = editor.scene.clone();
        for enabled in [!original.debug.cpu, original.debug.cpu] {
            let mut draft = original.clone();
            draft.debug.cpu = enabled;
            let started = Instant::now();
            editor
                .apply_project_configuration(draft, Some(crate::scene_bank::read(&root).unwrap()))
                .unwrap();
            eprintln!(
                "Ironwood Apply Debug: {:.2} ms",
                started.elapsed().as_secs_f64() * 1000.
            );
            assert!(
                started.elapsed() < Duration::from_secs(1),
                "Apply should only save settings"
            );
            let started = Instant::now();
            while started.elapsed() < Duration::from_secs(3) {
                editor.tick();
                assert!(
                    !editor.pending_build && editor.job.is_none(),
                    "{:?}",
                    editor.logs
                );
                assert_eq!(editor.registry_revision, revision);
                assert_eq!(editor.scene, scene);
                std::thread::sleep(Duration::from_millis(10));
            }
        }
    }
    #[test]
    fn a_catalog_refresh_publishes_the_generated_lua_definitions() {
        let root = crate::workspace::tests::temp("lua-definitions");
        let project =
            crate::workspace::create(&root, "Definitions", crate::workspace::Template::Basic)
                .unwrap();
        let mut editor = Editor::open(project).unwrap();
        editor.refresh_scripts();
        let stub = std::fs::read_to_string(root.join(crate::lua_api_stub::STUB_PATH))
            .expect("the refresh writes the Lua definitions");
        assert!(stub.starts_with("---@meta\n"), "{stub}");
        assert!(
            stub.contains("function epok.input.held(button, port) end"),
            "{stub}"
        );
        // Every class the refresh published is in the file, named as Lua sees it.
        let class = editor
            .class_registry
            .classes
            .values()
            .map(|class| class.cpp_name.replace("::", "."))
            .min()
            .expect("the project publishes at least one class");
        assert!(
            stub.contains(&format!("---@class {class}")),
            "{class}\n{stub}"
        );
        let _ = std::fs::remove_dir_all(&root);
    }
    #[test]
    fn applying_debug_settings_preserves_scene_and_does_not_auto_build_or_refresh_scripts() {
        let root = crate::workspace::tests::temp("apply-debug-with-auto-build");
        let project =
            crate::workspace::create(&root, "Debug settings", crate::workspace::Template::Basic)
                .unwrap();
        let mut editor = Editor::open(project).unwrap();
        editor.auto_build = true;
        let mut manifest = editor.project.as_ref().unwrap().manifest.clone();
        let generated = "generated-scene:.epok/build/scene.hh";
        crate::artifact_dependencies::transaction(&root, |graph| {
            graph.publish(
                "scene-debug-settings",
                crate::scene_dependencies::hash(manifest.debug),
                Default::default(),
            );
            graph.publish(
                generated,
                "old-build".into(),
                ["scene-debug-settings".into()].into(),
            );
        })
        .unwrap();
        // A queued observation must not revive old settings after Apply.
        let delayed = crate::scene_dependencies::inspect_with_registry(
            &root,
            &editor.scene_path(),
            &editor.scene,
            Ok(&editor.class_registry),
        )
        .unwrap();
        let revision = editor.registry_revision;
        editor.scene.actors[0].position[0] += 3.;
        let scene = editor.scene.clone();
        manifest.debug.cpu = true;
        let started = Instant::now();
        editor
            .apply_project_configuration(
                manifest.clone(),
                Some(crate::scene_bank::read(&root).unwrap()),
            )
            .unwrap();
        eprintln!("Apply Debug: {:?}", started.elapsed());
        assert_eq!(
            editor.scene, scene,
            "Apply must not reload the open document"
        );
        assert_eq!(editor.registry_revision, revision);
        assert!(!delayed.publish(&root).unwrap());
        poll_sources(&mut editor);
        let started = Instant::now();
        while started.elapsed() < Duration::from_millis(1400) {
            editor.tick();
            assert!(
                editor.job.is_none() && !editor.pending_build,
                "{:?}",
                editor.logs
            );
            std::thread::sleep(Duration::from_millis(10));
        }
        assert_eq!(
            editor.registry_revision, revision,
            "Debug settings must not reload scripts"
        );
        assert_eq!(crate::settings::debug_hud(&root).unwrap(), manifest.debug);
        assert!(
            !crate::artifact_dependencies::Graph::load(&root)
                .unwrap()
                .nodes[generated]
                .stale
                .is_empty()
        );
        // An independent real source edit must still trigger automatic work.
        std::fs::create_dir_all(root.join("assets/scripts")).unwrap();
        std::fs::write(
            root.join("assets/scripts/Changed.cpp"),
            b"// changed concurrently with Apply\n",
        )
        .unwrap();
        editor.auto_build = false;
        editor.apply_project_settings(manifest).unwrap();
        poll_sources(&mut editor);
        assert!(editor.pending_build);
    }
    #[test]
    fn switching_scenes_keeps_old_outputs_stale_without_auto_build_and_idle_reads() {
        let root = crate::workspace::tests::temp("navigation-with-auto-build");
        std::fs::create_dir_all(root.join("assets/scenes")).unwrap();
        let mut editor = Editor::new(root.clone());
        editor.save();
        let first = editor.scene_path();
        let other = root.join("assets/scenes/Other.epokmap");
        let mut scene = editor.scene.clone();
        scene.name = "Other".into();
        scene.save(&other).unwrap();
        let source = "scene-editor:assets/scenes/SampleScene.epokmap";
        let generated = "generated-scene:.epok/build/scene.hh";
        crate::artifact_dependencies::transaction(&root, |graph| {
            graph.publish(
                source,
                crate::scene_dependencies::signature(&editor.scene),
                Default::default(),
            );
            graph.publish(generated, "old-build".into(), [source.into()].into());
        })
        .unwrap();
        editor.establish_open_baseline();
        editor.auto_build = true;
        for path in [&other, &first, &other] {
            editor.open_scene(path.clone()).unwrap();
            let started = Instant::now();
            while started.elapsed() < Duration::from_millis(1400) {
                editor.tick();
                assert!(
                    !editor.pending_build && editor.job.is_none(),
                    "{:?}",
                    editor.logs
                );
                std::thread::sleep(Duration::from_millis(10));
            }
        }
        assert!(
            !crate::artifact_dependencies::Graph::load(&root)
                .unwrap()
                .nodes[generated]
                .stale
                .is_empty()
        );
        let last_poll = editor.last_poll;
        let external_read = editor.external_watch.last_started_for_test();
        for _ in 0..100 {
            editor.tick();
            assert!(editor.source_scan.is_none());
            assert_eq!(
                editor.last_poll, last_poll,
                "Idle editor must not rescan sources"
            );
            assert_eq!(editor.external_watch.last_started_for_test(), external_read);
            std::thread::sleep(Duration::from_millis(10));
        }
        // Unsaved edits are internal events, even when the last build consumed
        // a different scene. No file notification is needed to schedule work.
        editor.scene.actors[0].position[0] += 1.;
        editor.changed();
        assert!(editor.pending_build && editor.source_dirty);
        editor.auto_build = false;
        poll_sources(&mut editor);
        drop(editor);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn native_project_event_wakes_source_validation_without_forced_polling() {
        let root = crate::workspace::tests::temp("event-driven-source-validation");
        std::fs::create_dir_all(root.join("assets/scripts")).unwrap();
        let mut editor = Editor::new(root.clone());
        editor.auto_build = false;
        editor.establish_open_baseline();
        std::fs::write(root.join("assets/scripts/Event.cpp"), b"// OS event\n").unwrap();
        let started = Instant::now();
        while !editor.pending_build {
            editor.tick();
            assert!(
                started.elapsed() < Duration::from_secs(8),
                "{:?}",
                editor.logs
            );
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(
            editor
                .logs
                .iter()
                .any(|line| line.contains("Source changes detected"))
        );
        drop(editor);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn camera_drag_defers_idle_observation_without_losing_pending_work() {
        let root = crate::workspace::tests::temp("camera-observation");
        let mut editor = Editor::new(root);
        editor.auto_build = false;
        editor.last_poll = Instant::now() - Duration::from_secs(1);
        editor.scene_navigation = true;
        editor.pending_build = true;
        editor.tick();
        assert!(editor.source_scan.is_none());
        assert!(editor.pending_build);
        editor.scene_navigation = false;
        editor.tick();
        assert!(editor.source_scan.is_some());
        poll_sources(&mut editor);
    }
    #[test]
    fn delayed_scene_read_cannot_clear_errors_or_publish_an_old_document() {
        let root = crate::workspace::tests::temp("delayed-scene-observation");
        std::fs::create_dir_all(root.join("assets/scenes")).unwrap();
        Scene::default()
            .save(&root.join("assets/scenes/SampleScene.epokmap"))
            .unwrap();
        let mut editor = Editor::new(root.clone());
        editor.auto_build = false;
        let old = SourceObservation::read(
            &root,
            editor.scene_path(),
            editor.scene.clone(),
            editor.class_registry.clone(),
            editor.registry_revision,
            editor.scene_registry_error().map(str::to_owned),
        );
        editor.scene.name = "Newer unsaved scene".into();
        editor.changed();
        editor.scene_dependency_error = Some("Waiting for a fresh observation".into());
        let (sender, receiver) = std::sync::mpsc::channel();
        editor.source_scan = Some(receiver);
        sender
            .send(old)
            .unwrap_or_else(|_| panic!("Observation receiver dropped"));
        editor.tick();
        assert!(editor.scene_dependency_error.is_some());
        assert_eq!(editor.scene.name, "Newer unsaved scene");
        poll_sources(&mut editor);
        assert!(editor.scene_dependency_error.is_none());
        let (key, expected) = crate::scene_dependencies::Origin::Editor(
            editor.scene_path(),
            crate::scene_dependencies::signature(&editor.scene),
        )
        .dependency(&root, "")
        .unwrap();
        assert_eq!(
            crate::artifact_dependencies::Graph::load(&root)
                .unwrap()
                .nodes[&key]
                .signature
                .as_ref(),
            Some(&expected)
        );
    }
    #[test]
    fn scene_open_is_background_atomic_and_reports_timed_progress() {
        let root = crate::workspace::tests::temp("background-scene-open");
        std::fs::create_dir_all(root.join("assets/scenes")).unwrap();
        let path = root.join("assets/scenes/Other.epokmap");
        let scene = Scene {
            name: "Background scene".into(),
            ..Default::default()
        };
        scene.save(&path).unwrap();
        let mut editor = Editor::new(root.clone());
        let previous = editor.scene.clone();
        editor.begin_scene_open(path.clone()).unwrap();
        assert!(editor.critical_busy());
        assert_eq!(
            editor.scene, previous,
            "Only polling may publish a loaded scene"
        );
        assert!(editor.begin_scene_open(path.clone()).is_err());
        editor.action("delete");
        assert_eq!(editor.scene, previous, "Edits are locked while loading");
        let started = Instant::now();
        while editor.scene_loading.is_some() {
            assert!(started.elapsed() < Duration::from_secs(10));
            editor.tick();
            std::thread::sleep(Duration::from_millis(2));
        }
        assert_eq!(editor.scene.name, scene.name);
        assert_eq!(editor.scene_path(), path);
        assert!(!editor.critical_busy() && !editor.dirty);
        assert!(
            editor
                .logs
                .iter()
                .any(|s| s.starts_with("Indexing scene assets:"))
        );
        assert!(
            editor
                .logs
                .iter()
                .any(|s| s.starts_with("Scene opening completed:"))
        );
        assert_eq!(editor.logs.len(), editor.log_times.len());
        for time in &editor.log_times {
            chrono::NaiveDateTime::parse_from_str(time, "%Y-%m-%d %H:%M:%S%.3f").unwrap();
        }
        let broken = root.join("assets/scenes/Broken.epokmap");
        std::fs::write(&broken, b"invalid scene").unwrap();
        editor.scene.name = "Preserve unsaved changes".into();
        editor.changed();
        let preserved = editor.scene.clone();
        editor.begin_scene_open(broken).unwrap();
        let started = Instant::now();
        while editor.scene_loading.is_some() {
            assert!(started.elapsed() < Duration::from_secs(10));
            editor.tick();
            std::thread::sleep(Duration::from_millis(2));
        }
        assert_eq!(editor.scene, preserved);
        assert_eq!(editor.scene_path(), path);
        assert!(editor.dirty && editor.last_error.is_some());
        assert!(
            editor
                .logs
                .last()
                .unwrap()
                .starts_with("Scene opening failed:")
        );
        drop(editor);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    #[ignore = "requires a disposable project copy in EPOK_IDLE_PROJECT"]
    fn profile_scene_open() {
        let root = PathBuf::from(std::env::var_os("EPOK_IDLE_PROJECT").unwrap());
        let started = Instant::now();
        let project = crate::workspace::Project::open(&root).unwrap();
        eprintln!("Project open: {:?}", started.elapsed());
        let prepared = PreparedProject::load(project, |stage| {
            eprintln!("{:?}: {stage}", started.elapsed());
        })
        .unwrap();
        let handoff = Instant::now();
        let mut editor = Editor::open_prepared(prepared).unwrap();
        eprintln!("Editor handoff: {:?}", handoff.elapsed());
        let switch = Instant::now();
        let path = std::env::var_os("EPOK_PROFILE_SCENE")
            .map(|path| root.join(path))
            .unwrap_or_else(|| editor.scene_path());
        editor.begin_scene_open(path).unwrap();
        eprintln!("Scene open request: {:?}", switch.elapsed());
        let mut maximum_load_tick = Duration::ZERO;
        while editor.scene_loading.is_some() {
            assert!(switch.elapsed() < Duration::from_secs(60));
            let tick = Instant::now();
            editor.tick();
            maximum_load_tick = maximum_load_tick.max(tick.elapsed());
            std::thread::sleep(Duration::from_millis(16));
        }
        eprintln!(
            "Scene reopen: {:?}; maximum loading tick: {maximum_load_tick:?}",
            switch.elapsed()
        );
        for (message, time) in editor.logs.iter().zip(&editor.log_times) {
            eprintln!("[{time}] {message}");
        }
        editor.auto_build = true;
        let ticks = Instant::now();
        let mut maximum = Duration::ZERO;
        while ticks.elapsed() < Duration::from_secs(3) {
            let tick = Instant::now();
            editor.tick();
            maximum = maximum.max(tick.elapsed());
            assert!(
                !editor.pending_build && editor.job.is_none(),
                "Navigation must not auto-build: {:?}",
                editor.logs
            );
            std::thread::sleep(Duration::from_millis(16));
        }
        eprintln!("Maximum idle tick: {maximum:?}");
        editor.scene_navigation = true;
        let navigation = Instant::now();
        let mut maximum_navigation = Duration::ZERO;
        while navigation.elapsed() < Duration::from_secs(3) {
            let tick = Instant::now();
            editor.view.look([1., 0.], false);
            editor.tick();
            maximum_navigation = maximum_navigation.max(tick.elapsed());
            std::thread::sleep(Duration::from_millis(16));
        }
        eprintln!("Maximum camera-navigation tick (excluding rendering): {maximum_navigation:?}");
        editor.scene_navigation = false;
        poll_sources(&mut editor);
    }
    #[test]
    #[ignore = "requires an isolated project copy in EPOK_IDLE_PROJECT and the configured host extractor"]
    fn opened_project_remains_idle_with_auto_build_enabled() {
        let root = std::env::var_os("EPOK_IDLE_PROJECT")
            .expect("Set EPOK_IDLE_PROJECT to an isolated project copy");
        let project = crate::workspace::Project::open(Path::new(&root)).unwrap();
        let mut editor = Editor::open(project).unwrap();
        editor.auto_build = true;
        let started = Instant::now();
        while started.elapsed() < Duration::from_secs(5) {
            editor.tick();
            assert!(
                editor.job.is_none() && !editor.pending_build,
                "Opening must stay idle: {:?}",
                editor.logs
            );
            std::thread::sleep(Duration::from_millis(20));
        }
        println!("Project stayed idle: {}", editor.project_name());
    }
    #[test]
    #[ignore = "Requires disposable Ironwood copy in EPOK_IDLE_PROJECT and pinned native build tools"]
    fn blueprint_assignment_manual_build_completes_once() {
        let root = PathBuf::from(std::env::var("EPOK_IDLE_PROJECT").unwrap());
        let mut editor = Editor::open(crate::workspace::Project::open(&root).unwrap()).unwrap();
        editor.auto_build = true;
        let index = editor
            .scene
            .actors
            .iter()
            .position(|e| e.name.contains("Cube"))
            .unwrap();
        editor.selected = Some(index);
        editor.attach("None");
        crate::blueprint_workflow::attach_asset(
            &mut editor,
            &root.join("assets/Blueprints/BP_Cube.epokbp"),
        )
        .unwrap();
        let started = Instant::now();
        let mut builds = 0;
        let mut finished = None;
        editor.build(false);
        while started.elapsed() < Duration::from_secs(240) {
            editor.tick();
            for line in std::mem::take(&mut editor.logs) {
                assert!(
                    !line.starts_with("mipsel-none-elf-g++ "),
                    "Compiler command flooded Console: {line}"
                );
                eprintln!("[{:.1}] {line}", started.elapsed().as_secs_f32());
                if line == "Building C++ / PsyQo..." {
                    builds += 1;
                }
            }
            assert!(builds <= 1, "Assignment restarted its own build");
            if builds == 1 && editor.job.is_none() && !editor.pending_build {
                assert!(editor.last_error.is_none(), "{:?}", editor.last_error);
                let since = finished.get_or_insert_with(Instant::now);
                if since.elapsed() >= Duration::from_secs(3) {
                    let log = std::fs::read_to_string(root.join(".epok/Build.log")).unwrap();
                    assert!(
                        log.contains("mipsel-none-elf-g++ "),
                        "Full compiler commands must remain available in the build log"
                    );
                    assert_eq!(editor.scene.actors[index].class.name, "BP_Cube");
                    return;
                }
            } else {
                finished = None;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        panic!(
            "Build did not finish: starts={builds}, stale={}, pending={}",
            editor.job_stale, editor.pending_build
        );
    }
    #[test]
    fn opening_stale_scene_provenance_does_not_queue_a_build_but_live_edits_do() {
        let root = crate::workspace::tests::temp("startup-build-baseline");
        std::fs::create_dir_all(root.join("assets/scenes")).unwrap();
        let mut editor = Editor::new(root.clone());
        editor.save();
        editor.auto_build = true;
        let source = "scene-editor:assets/scenes/SampleScene.epokmap";
        let generated = "generated-scene:.epok/build/scene.hh";
        crate::artifact_dependencies::transaction(&root, |graph| {
            graph.publish(source, "previous-session".into(), Default::default());
            graph.publish(generated, "old-build".into(), [source.into()].into());
        })
        .unwrap();
        editor.establish_open_baseline();
        assert!(
            !crate::artifact_dependencies::Graph::load(&root)
                .unwrap()
                .nodes[generated]
                .stale
                .is_empty(),
            "Startup must still invalidate obsolete output"
        );
        for _ in 0..4 {
            poll_sources(&mut editor);
            assert!(
                !editor.pending_build && editor.job.is_none(),
                "Opening a cached project must not compile it: {:?}",
                editor.logs
            );
        }
        editor.scene.actors[0].position[0] += 1.;
        editor.changed();
        poll_sources(&mut editor);
        assert!(
            editor.pending_build,
            "Authored edits still request the normal debounced build"
        );
        drop(editor);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn toolbar_resume_routes_node_pause_through_bridge_and_preserves_failed_state() {
        use crate::blueprint_debug::{Command, State};
        let mut editor = Editor::new(std::env::temp_dir().join("epok-toolbar-node-resume"));
        let mut debug = State::default();
        editor.paused = true;
        editor.blueprint_editor.debug_node = Some("node".into());
        // Ordinary frame/HTTP pauses remain handled by the existing control path.
        assert!(!editor.resume_blueprint_node(&mut debug).unwrap());
        assert!(editor.paused);
        assert!(debug.next_command().is_none());
        debug.at_node = true;
        debug.paused = true;
        assert!(editor.resume_blueprint_node(&mut debug).unwrap());
        assert!(matches!(debug.next_command(), Some(Command::Resume)));
        assert!(!editor.paused);
        assert!(editor.blueprint_editor.debug_node.is_none());
        // A full command queue must not fall back to unsafe HTTP resume or
        // optimistically clear the editor's stopped-node presentation.
        editor.paused = true;
        editor.blueprint_editor.debug_node = Some("node".into());
        for _ in 0..32 {
            debug.request(Command::Pause).unwrap();
        }
        assert!(editor.resume_blueprint_node(&mut debug).is_err());
        assert!(editor.paused);
        assert_eq!(editor.blueprint_editor.debug_node.as_deref(), Some("node"));
    }
    #[test]
    fn rename_cancel_validation_and_hud_component_creation() {
        let mut e = Editor::new(std::env::temp_dir().join("epok-rename-hud"));
        let original = e.scene.actors[1].name.clone();
        e.begin_rename(1);
        e.rename.as_mut().unwrap().1 = "Cancel".into();
        e.finish_rename(false);
        assert_eq!(e.scene.actors[1].name, original);
        e.begin_rename(1);
        e.rename.as_mut().unwrap().1 = "  ".into();
        e.finish_rename(true);
        assert_eq!(e.scene.actors[1].name, original);
        e.begin_rename(1);
        e.rename.as_mut().unwrap().1 = "Player".into();
        e.finish_rename(true);
        assert_eq!(e.scene.actors[1].name, "Player");
        e.create_hud("progress");
        let i = e.selected.unwrap();
        assert!(e.scene.actors[i].progress.is_some());
        assert!(e.scene.actors[i].rect.is_some());
        assert!(
            e.scene.actors[e.scene.actors[i].parent.unwrap()]
                .canvas
                .is_some()
        );
        e.scene.validate().unwrap();
        let before = e.scene.clone();
        e.playing = true;
        e.create_hud("text");
        assert_eq!(before, e.scene);
    }
    #[test]
    fn primitive_creation_uses_the_requested_hierarchy_context() {
        let mut e = Editor::new(std::env::temp_dir().join("epok-creation-context"));
        e.selected = Some(1);
        e.action("empty");
        let parent = e.selected.unwrap();
        assert_eq!(e.scene.actors[parent].kind, "Empty");
        assert_eq!(e.scene.actors[parent].parent, None);
        e.scene.actors[parent].position = [3., 2., 1.];
        e.scene.actors[parent].rotation = [0., 45., 0.];
        e.action("add-child");
        let child = e.selected.unwrap();
        assert_eq!(e.scene.actors[child].parent, Some(parent));
        assert_eq!(e.scene.actors[child].kind, "Mesh");
        assert_eq!(e.scene.actors[child].position, [0.; 3]);
        assert_eq!(e.scene.world_matrix(child).point([0.; 3]), [3., 2., 1.]);
        e.action("add");
        assert_eq!(e.scene.actors[e.selected.unwrap()].parent, None);
        let before = e.scene.clone();
        e.playing = true;
        e.action("add-child");
        assert_eq!(e.scene, before);
    }
    #[test]
    fn watcher_marks_native_changes_pending_without_starting_build() {
        let path = std::env::temp_dir().join(format!("epok-watch-{}", std::process::id()));
        std::fs::create_dir_all(path.join("assets/scripts")).unwrap();
        let mut e = Editor::new(path.clone());
        e.auto_build = true;
        std::fs::write(path.join("assets/scripts/Test.cpp"), "// changed").unwrap();
        poll_sources(&mut e);
        assert!(e.pending_build);
        assert!(e.job.is_none());
        e.pending_since = Some(Instant::now() - Duration::from_secs(1));
        // Legacy projects with auto_build=true must remain manual too, even
        // after the old debounce deadline and all source observers settle.
        let started = Instant::now();
        while started.elapsed() < Duration::from_secs(1) {
            e.tick();
            assert!(e.job.is_none(), "{:?}", e.logs);
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(e.pending_build);
        std::fs::remove_file(path.join("assets/scripts/Test.cpp")).unwrap();
    }
    #[test]
    fn invalid_source_marks_outputs_stale_and_keeps_running_snapshot() {
        let root = crate::workspace::tests::temp("invalid-timeline-watch");
        std::fs::create_dir_all(root.join("assets/Timelines")).unwrap();
        let source = crate::timeline::TimelineAsset::new("Watched".into());
        let path = root.join("assets/Timelines/Watched.timeline.json");
        let valid = serde_json::to_vec(&source).unwrap();
        std::fs::write(&path, &valid).unwrap();
        let mut independent = crate::timeline::TimelineAsset::new("Independent".into());
        let independent_path = root.join("assets/Timelines/Independent.timeline.json");
        std::fs::write(&independent_path, serde_json::to_vec(&independent).unwrap()).unwrap();
        let untouched = crate::timeline::TimelineAsset::new("Untouched".into());
        std::fs::write(
            root.join("assets/Timelines/Untouched.timeline.json"),
            serde_json::to_vec(&untouched).unwrap(),
        )
        .unwrap();
        crate::timeline_compile::refresh(&root, &source, &crate::blueprint::Registry::new())
            .unwrap();
        let mut editor = Editor::new(root.clone());
        crate::timeline_compile::refresh(&root, &independent, &crate::blueprint::Registry::new())
            .unwrap();
        let independent_cache = root.join(format!(".epok/timelines/{}.json", independent.id));
        let independent_bytes = std::fs::read(&independent_cache).unwrap();
        editor.timeline_editor.open(&independent_path).unwrap();
        crate::timeline_compile::refresh(&root, &untouched, &crate::blueprint::Registry::new())
            .unwrap();
        let untouched_cache = root.join(format!(".epok/timelines/{}.json", untouched.id));
        let untouched_bytes = std::fs::read(&untouched_cache).unwrap();
        crate::artifact_dependencies::transaction(&root, |graph| {
            graph.publish(
                "stage:.epok/build",
                "fixture".into(),
                [format!("cooked-timeline:{}", source.id)]
                    .into_iter()
                    .collect(),
            );
        })
        .unwrap();
        editor.auto_build = false;
        let (job, events, controls) = pipeline::Job::test_channels();
        editor.job = Some(job);
        editor.job_target = Some((true, false));
        editor.playing = true;
        std::fs::write(
            root.join("assets/Timelines/UnusedBroken.timeline.json"),
            b"{",
        )
        .unwrap();
        poll_sources(&mut editor);
        assert!(
            controls.try_recv().is_err(),
            "An unrelated catalog error must not stop Play"
        );
        assert!(!editor.job_stale && !editor.pending_build && editor.source_error.is_none());
        // Reflection needs the MIPS SDK, which is absent on some hosts; the
        // point here is that a broken timeline never adds a registry error.
        let registry_error = editor.timeline_editor.registry_error.clone();
        std::fs::write(&path, b"{ invalid timeline").unwrap();
        poll_sources(&mut editor);
        assert!(controls.try_recv().is_err());
        assert!(!editor.job_stale && editor.pending_build && editor.source_error.is_none());
        assert!(editor.playing);
        let cache: crate::timeline_compile::PreviewCache = serde_json::from_slice(
            &std::fs::read(root.join(format!(".epok/timelines/{}.json", source.id))).unwrap(),
        )
        .unwrap();
        assert!(cache.stale && cache.compiled.is_some());
        let logs = editor.logs.len();
        poll_sources(&mut editor);
        assert!(controls.try_recv().is_err());
        assert_eq!(editor.logs.len(), logs);
        assert!(editor.timeline_editor.catalog_error.is_none());
        assert_eq!(editor.timeline_editor.registry_error, registry_error);
        assert_eq!(
            std::fs::read(&independent_cache).unwrap(),
            independent_bytes
        );
        independent.duration_ticks += 68;
        std::fs::write(&independent_path, serde_json::to_vec(&independent).unwrap()).unwrap();
        poll_sources(&mut editor);
        let cache: crate::timeline_compile::PreviewCache =
            serde_json::from_slice(&std::fs::read(&independent_cache).unwrap()).unwrap();
        assert!(
            cache.stale,
            "Other sources must still be observed while the first error persists"
        );
        assert!(controls.try_recv().is_err());
        let duplicate = root.join("assets/Timelines/Duplicate.timeline.json");
        std::fs::write(&duplicate, serde_json::to_vec(&independent).unwrap()).unwrap();
        poll_sources(&mut editor);
        assert!(editor.timeline_editor.catalog_error.is_some());
        editor
            .timeline_editor
            .validate(&root, &crate::blueprint::Registry::new());
        // The duplicate name must never recompile an unrelated timeline. A host
        // without the MIPS SDK has no reflection registry, and that error alone
        // marks every cache stale, so compare the compiled payload and tie the
        // stale flag to the registry state instead of the raw bytes.
        let compiled = |bytes: &[u8]| {
            let cache: crate::timeline_compile::PreviewCache =
                serde_json::from_slice(bytes).unwrap();
            (serde_json::to_value(&cache.compiled).unwrap(), cache.stale)
        };
        let (payload, stale) = compiled(&std::fs::read(&untouched_cache).unwrap());
        assert_eq!(payload, compiled(&untouched_bytes).0);
        assert_eq!(stale, editor.timeline_editor.registry_error.is_some());
        std::fs::remove_file(duplicate).unwrap();
        poll_sources(&mut editor);
        assert!(editor.timeline_editor.catalog_error.is_none());
        editor.action("play");
        assert!(matches!(controls.try_recv(), Ok(Control::Stop)));
        events.send(Event::Finished(Ok(()))).unwrap();
        editor.tick();
        assert!(editor.job.is_none() && !editor.playing);
        assert!(
            editor.last_error.is_none(),
            "The cancelled old worker cannot replace current diagnostics"
        );
        std::fs::write(path, &valid).unwrap();
        poll_sources(&mut editor);
        assert!(editor.source_error.is_none() && editor.pending_build && editor.job.is_none());
        assert_eq!(editor.restart_target, None);
    }
    #[test]
    fn explicit_stop_cancels_pending_automatic_play_restart() {
        let mut editor = Editor::new(crate::workspace::tests::temp("stop-restart"));
        let (job, _events, controls) = pipeline::Job::test_channels();
        editor.job = Some(job);
        editor.job_target = Some((true, false));
        editor.invalidate_running_build();
        assert!(matches!(controls.try_recv(), Ok(Control::Stop)));
        assert_eq!(editor.restart_target, Some((true, false)));
        editor.action("play");
        assert!(matches!(controls.try_recv(), Ok(Control::Stop)));
        assert_eq!(editor.restart_target, None);
    }
    #[test]
    fn stop_returns_to_scene_even_when_running_arrives_late() {
        for late_running in [false, true] {
            let mut editor = Editor::new(crate::workspace::tests::temp("stop-scene"));
            let (job, events, controls) = pipeline::Job::test_channels();
            editor.auto_build = false;
            editor.job = Some(job);
            editor.job_target = Some((true, false));
            editor.playing = !late_running;
            editor.game_capture = true;
            editor.focus_scene = false;
            editor.focus_game = true;
            editor.focus_console = true;
            editor.action("play");
            assert!(matches!(controls.try_recv(), Ok(Control::Stop)));
            assert!(editor.focus_scene);
            assert!(!editor.focus_game && !editor.focus_console && !editor.game_capture);
            // Simulate the UI consuming the first focus request before the
            // asynchronous pipeline acknowledges Stop.
            editor.focus_scene = false;
            if late_running {
                events.send(Event::Running(0)).unwrap();
            }
            events.send(Event::Finished(Ok(()))).unwrap();
            editor.tick();
            assert!(editor.focus_scene);
            assert!(!editor.focus_game && !editor.playing && !editor.game_capture);
            assert!(editor.job.is_none());
        }
    }
    #[test]
    fn build_completion_does_not_select_scene() {
        let mut editor = Editor::new(crate::workspace::tests::temp("build-tab"));
        let (job, events, _controls) = pipeline::Job::test_channels();
        editor.auto_build = false;
        editor.job = Some(job);
        editor.job_target = Some((false, false));
        editor.focus_scene = false;
        events.send(Event::Finished(Ok(()))).unwrap();
        editor.tick();
        assert!(!editor.focus_scene && !editor.focus_game);
    }
    #[test]
    fn unsaved_effect_defers_build_and_preserves_automatic_restart_target() {
        let root = crate::workspace::tests::temp("unsaved-effect-restart");
        std::fs::create_dir_all(root.join("assets/Effects")).unwrap();
        let path = root.join("assets/Effects/Fire.particle-effect.json");
        let effect = crate::particle_effect::ParticleEffect::new("Fire".into());
        std::fs::write(&path, serde_json::to_vec(&effect).unwrap()).unwrap();
        let mut editor = Editor::new(root);
        editor.timeline_editor.open(&path).unwrap();
        editor
            .timeline_editor
            .edit(|asset| asset.duration_ticks += 68);
        editor.restart_target = Some((true, false));
        editor.pending_build = true;
        editor.build_target(true, false);
        assert!(editor.job.is_none());
        assert_eq!(editor.restart_target, Some((true, false)));
        assert!(
            editor
                .logs
                .last()
                .unwrap()
                .contains("Save or discard the open Timeline/ParticleEffect")
        );
    }
    #[test]
    #[ignore = "requires EPOK_NATIVE_PROJECT and a native C++ compiler"]
    fn native_pc_cached_replay_meets_budget_and_reset_relaunches() {
        fn until(editor: &mut Editor, label: &str, ready: impl Fn(&Editor) -> bool) {
            let started = Instant::now();
            while !ready(editor) {
                editor.tick();
                assert!(
                    started.elapsed() < Duration::from_secs(120),
                    "{label}: {:?}",
                    editor.logs
                );
                std::thread::sleep(Duration::from_millis(5));
            }
        }

        let root = PathBuf::from(std::env::var_os("EPOK_NATIVE_PROJECT").unwrap());
        let project = crate::workspace::Project::open(&root).unwrap();
        let mut editor = Editor::open(project).unwrap();
        editor.play_profile.runtime = crate::play::Runtime::NativePc;
        editor.play_profile.content = crate::play::Content::CurrentScene;
        editor.build(true);
        until(&mut editor, "initial Native PC Play", |editor| {
            editor.playing
                && editor.native_frame.is_some()
                && editor.source_scan.is_none()
                && !editor.source_dirty
        });
        editor.action("play");
        until(&mut editor, "stop initial Native PC Play", |editor| {
            editor.job.is_none()
        });

        let started = Instant::now();
        editor.build(true);
        until(&mut editor, "cached Native PC Play", |editor| {
            editor.playing && editor.native_frame.is_some()
        });
        let cached_start = started.elapsed();
        println!("Cached Native PC Play: {cached_start:?}");
        assert!(
            cached_start < Duration::from_millis(750),
            "cached Native PC Play took {cached_start:?}: {:?}",
            editor.logs
        );
        assert!(
            editor
                .logs
                .iter()
                .any(|line| line.contains("Reusing current Native PC runtime"))
        );

        let cadence_start = Instant::now();
        let first_frame = editor.native_frame.as_ref().unwrap().number;
        while cadence_start.elapsed() < Duration::from_secs(1) {
            editor.tick();
            std::thread::sleep(Duration::from_millis(2));
        }
        editor.tick();
        let frames = editor.native_frame.as_ref().unwrap().number - first_frame;
        let rate = frames as f64 / cadence_start.elapsed().as_secs_f64();
        println!("Native PC snapshot cadence: {rate:.1} fps");
        assert!(
            rate >= 50.,
            "Native PC snapshot cadence fell to {rate:.1} fps"
        );

        if std::env::var_os("EPOK_NATIVE_EXPECT_AUDIO").is_some() {
            let audio_started = Instant::now();
            until(&mut editor, "Native PC audio output", |editor| {
                editor
                    .native_frame
                    .as_ref()
                    .is_some_and(|frame| frame.audio_voices > 0)
            });
            println!(
                "Native PC audio output ready: {:?}",
                audio_started.elapsed()
            );
        }

        if let (Some(buttons), Ok(expected_clip)) = (
            std::env::var("EPOK_NATIVE_ANIMATION_INPUT")
                .ok()
                .and_then(|value| value.parse::<u16>().ok()),
            std::env::var("EPOK_NATIVE_ANIMATION_CLIP"),
        ) {
            editor.set_buttons(buttons);
            until(
                &mut editor,
                "Native PC animation clip transition",
                |editor| {
                    editor.native_frame.as_ref().is_some_and(|frame| {
                        frame.scene.actors.iter().any(|actor| {
                            actor.skeletal_mesh.as_ref().is_some_and(|component| {
                                component.model.as_ref().is_some_and(|model| {
                                    component.clip.is_some_and(|active| {
                                        model.clips.iter().any(|(id, clip)| {
                                            *id == active && clip.name.contains(&expected_clip)
                                        })
                                    })
                                })
                            })
                        })
                    })
                },
            );
            editor.set_buttons(0);
            println!("Native PC animation reached clip containing `{expected_clip}`");
        }

        let pid = editor.emulator_pid.unwrap();
        editor
            .job
            .as_ref()
            .unwrap()
            .control(crate::pipeline::Control::Reset);
        until(&mut editor, "reset Native PC Play", |editor| {
            editor.playing && editor.emulator_pid.is_some_and(|reset| reset != pid)
        });
        editor.action("play");
        until(&mut editor, "stop reset Native PC Play", |editor| {
            editor.job.is_none()
        });

        if let Some(source) = std::env::var_os("EPOK_NATIVE_SOURCE").map(PathBuf::from) {
            struct Restore {
                path: PathBuf,
                bytes: Vec<u8>,
                modified: std::time::SystemTime,
            }
            impl Drop for Restore {
                fn drop(&mut self) {
                    let _ = std::fs::write(&self.path, &self.bytes);
                    let _ = std::fs::File::options()
                        .write(true)
                        .open(&self.path)
                        .and_then(|file| {
                            file.set_times(std::fs::FileTimes::new().set_modified(self.modified))
                        });
                }
            }
            let restore = Restore {
                bytes: std::fs::read(&source).unwrap(),
                modified: std::fs::metadata(&source).unwrap().modified().unwrap(),
                path: source.clone(),
            };
            let baseline = editor.fingerprint;
            let mut changed = restore.bytes.clone();
            changed.extend_from_slice(
                format!("\n// Native PC iteration test {}\n", uuid::Uuid::new_v4()).as_bytes(),
            );
            std::fs::write(&source, changed).unwrap();
            until(&mut editor, "observe Native PC C++ edit", |editor| {
                editor.fingerprint != baseline
                    && editor.source_scan.is_none()
                    && editor.assets.revision == editor.source_revision
            });
            let started = Instant::now();
            editor.build(true);
            until(&mut editor, "incremental Native PC C++ Play", |editor| {
                editor.playing && editor.native_frame.is_some()
            });
            let incremental = started.elapsed();
            println!("Incremental Native PC C++ Play: {incremental:?}");
            assert!(
                incremental < Duration::from_secs(3),
                "incremental Native PC C++ Play took {incremental:?}: {:?}",
                editor.logs
            );
            editor.action("play");
            until(&mut editor, "stop incremental Native PC Play", |editor| {
                editor.job.is_none()
            });
            drop(restore);
        }
    }
    #[test]
    #[ignore = "requires pinned MIPS tools and PCSX-Redux; owns port 8077 and rebuilds/restarts real Play"]
    fn native_source_edit_restarts_real_play_and_recovers_after_compiler_failure() {
        fn until(editor: &mut Editor, label: &str, ready: impl Fn(&Editor) -> bool) {
            let started = Instant::now();
            loop {
                editor.tick();
                if ready(editor) {
                    break;
                }
                assert!(
                    started.elapsed() < Duration::from_secs(150),
                    "{label}: {:?}",
                    editor.logs
                );
                std::thread::sleep(Duration::from_millis(20));
            }
        }
        let root = crate::workspace::tests::temp("native-restart");
        let project =
            crate::workspace::create(&root, "NativeRestart", crate::workspace::Template::Sample)
                .unwrap();
        let mut editor = Editor::open(project).unwrap();
        let timeline_path = root.join("assets/Timelines/Used.timeline.json");
        std::fs::create_dir_all(timeline_path.parent().unwrap()).unwrap();
        let timeline = crate::timeline::TimelineAsset::new("Used".into());
        let timeline_bytes = serde_json::to_vec(&timeline).unwrap();
        std::fs::write(&timeline_path, &timeline_bytes).unwrap();
        editor.scene.actors[0].timeline = Some(crate::timeline_scene::Component {
            asset: Some(timeline.id),
            ..Default::default()
        });
        editor.changed();
        editor.auto_build = true;
        editor.build(true);
        until(&mut editor, "Initial Play", |e| {
            e.playing && e.game_frame.is_some()
        });
        let initial_pid = editor.emulator_pid.unwrap();
        let executable = root.join(".epok/build/epok.ps-exe");
        let original_executable = std::fs::read(&executable).unwrap();
        std::fs::write(
            root.join("assets/Timelines/UnusedBroken.timeline.json"),
            b"{",
        )
        .unwrap();
        std::fs::create_dir_all(root.join("assets/Effects")).unwrap();
        std::fs::write(
            root.join("assets/Effects/UnusedBroken.particle-effect.json"),
            b"{",
        )
        .unwrap();
        let stable_since = Instant::now();
        while stable_since.elapsed() < Duration::from_secs(2) {
            editor.tick();
            assert_eq!(editor.emulator_pid, Some(initial_pid));
            assert!(!editor.job_stale && !editor.pending_build);
            std::thread::sleep(Duration::from_millis(20));
        }
        std::fs::write(&timeline_path, b"{ broken required timeline").unwrap();
        until(&mut editor, "Rejected invalid used timeline", |e| {
            e.job.is_none()
                && e.last_error
                    .as_ref()
                    .is_some_and(|error| error.contains("Missing TimelineAsset"))
        });
        assert!(!editor.playing && editor.game_frame.is_none());
        std::fs::write(&timeline_path, &timeline_bytes).unwrap();
        until(
            &mut editor,
            "Recovered used timeline with unrelated catalog errors",
            |e| {
                e.playing
                    && !e.job_stale
                    && e.emulator_pid.is_some_and(|pid| pid != initial_pid)
                    && e.game_frame.is_some()
            },
        );
        assert_eq!(std::fs::read(&executable).unwrap(), original_executable);
        let first_pid = editor.emulator_pid.unwrap();
        let source = root.join("assets/scripts/Spinner.cpp");
        let original = std::fs::read_to_string(&source).unwrap();
        let changed = original.replace("360.0", "180.0");
        assert_ne!(changed, original);
        std::fs::write(&source, &changed).unwrap();
        until(&mut editor, "Restarted Play", |e| {
            e.playing
                && !e.job_stale
                && e.emulator_pid.is_some_and(|pid| pid != first_pid)
                && e.game_frame.is_some()
        });
        let second_pid = editor.emulator_pid.unwrap();
        let updated_executable = std::fs::read(&executable).unwrap();
        assert_ne!(updated_executable, original_executable);
        assert_eq!(
            std::fs::read(root.join(".epok/build/scripts/Spinner.cpp")).unwrap(),
            changed.as_bytes()
        );
        std::fs::write(&source, format!("{changed}\ninvalid C++ source;\n")).unwrap();
        until(&mut editor, "Rejected invalid native source", |e| {
            e.job.is_none()
                && e.last_error
                    .as_ref()
                    .is_some_and(|error| error.contains("compilation failed"))
        });
        assert!(!editor.playing && editor.game_frame.is_none());
        // A failed automatic Play rebuild must remember the user's target so
        // repairing the file resumes Play, rather than silently doing Build only.
        std::fs::write(&source, &original).unwrap();
        until(&mut editor, "Recovered Play", |e| {
            e.playing
                && !e.job_stale
                && e.emulator_pid.is_some_and(|pid| pid != second_pid)
                && e.game_frame.is_some()
        });
        assert_eq!(std::fs::read(&executable).unwrap(), original_executable);
        let recovered_pid = editor.emulator_pid.unwrap();
        let saved_scene = std::fs::read(editor.scene_path()).unwrap();
        editor.scene.actors[1].position[0] += 3.;
        editor.changed();
        until(&mut editor, "Restarted edited scene", |e| {
            e.playing
                && !e.job_stale
                && e.emulator_pid.is_some_and(|pid| pid != recovered_pid)
                && e.game_frame.is_some()
        });
        assert_ne!(std::fs::read(&executable).unwrap(), original_executable);
        assert_eq!(std::fs::read(editor.scene_path()).unwrap(), saved_scene);
        let scene_pid = editor.emulator_pid.unwrap();
        let stable_since = Instant::now();
        while stable_since.elapsed() < Duration::from_secs(2) {
            editor.tick();
            assert_eq!(
                editor.emulator_pid,
                Some(scene_pid),
                "Lighting cache must not trigger another restart"
            );
            assert!(!editor.job_stale);
            std::thread::sleep(Duration::from_millis(20));
        }
        // The C++ include is outside assets and the project. Only the actual
        // compiler dependency capture can discover it for the editor watcher.
        let external = root.with_extension("shared.hpp");
        std::fs::write(&external, "#define EPOK_SPINNER_PERIOD 360.0\n").unwrap();
        let external_source = format!(
            "#include \"{}\"\n{}",
            external.to_string_lossy().replace('\\', "/"),
            original.replace("360.0", "EPOK_SPINNER_PERIOD")
        );
        std::fs::write(&source, external_source).unwrap();
        until(&mut editor, "Play with an external include", |e| {
            e.playing
                && !e.job_stale
                && e.emulator_pid.is_some_and(|pid| pid != scene_pid)
                && e.game_frame.is_some()
        });
        let external_pid = editor.emulator_pid.unwrap();
        let external_executable = std::fs::read(&executable).unwrap();
        let modified = std::fs::metadata(&external).unwrap().modified().unwrap();
        std::fs::write(&external, "#define EPOK_SPINNER_PERIOD 180.0\n").unwrap();
        std::fs::File::options()
            .write(true)
            .open(&external)
            .unwrap()
            .set_times(std::fs::FileTimes::new().set_modified(modified))
            .unwrap();
        until(&mut editor, "Restart after external include edit", |e| {
            e.playing
                && !e.job_stale
                && e.emulator_pid.is_some_and(|pid| pid != external_pid)
                && e.game_frame.is_some()
        });
        assert_ne!(std::fs::read(&executable).unwrap(), external_executable);
        let changed_external_pid = editor.emulator_pid.unwrap();
        std::fs::remove_file(&external).unwrap();
        until(&mut editor, "Reject missing external include", |e| {
            e.job.is_none() && e.last_error.is_some()
        });
        assert!(!editor.playing && editor.game_frame.is_none());
        std::fs::write(&external, "#define EPOK_SPINNER_PERIOD 360.0\n").unwrap();
        until(&mut editor, "Recover repaired external include", |e| {
            e.playing
                && !e.job_stale
                && e.emulator_pid
                    .is_some_and(|pid| pid != changed_external_pid)
                && e.game_frame.is_some()
        });
        assert_eq!(std::fs::read(&executable).unwrap(), external_executable);
        editor.action("play");
        until(&mut editor, "Explicit Stop", |e| e.job.is_none());
        assert!(editor.restart_target.is_none());
        println!("Native restart fixture: {}", root.display());
    }
    #[test]
    fn editor_actions_preserve_scene_and_bindings() {
        let path = std::env::temp_dir().join(format!("epok-actions-{}", uuid::Uuid::new_v4()));
        let mut e = Editor::new(path.clone());
        e.action("add");
        assert_eq!(e.scene.actors.len(), 5);
        e.action("duplicate");
        assert_eq!(e.scene.actors.len(), 6);
        e.playing = true;
        e.action("delete");
        assert_eq!(e.scene.actors.len(), 6);
        e.playing = false;
        e.action("delete");
        assert!(e.save());
        e.action("reload");
        let started = Instant::now();
        while e.scene_loading.is_some() {
            assert!(started.elapsed() < Duration::from_secs(10));
            e.tick();
            std::thread::sleep(Duration::from_millis(2));
        }
        assert_eq!(e.scene.actors.len(), 5);
        while !e.scene.actors.is_empty() {
            e.action("delete");
        }
        assert_eq!(e.selected, None);
        drop(e);
        std::fs::remove_dir_all(path).unwrap();
    }
    #[test]
    fn attachment_undo_restores_overrides_and_rejects_intervening_edits() {
        let root = crate::workspace::tests::temp("attach-undo");
        let project =
            crate::workspace::create(&root, "Undo", crate::workspace::Template::Sample).unwrap();
        let mut editor = Editor::open(project).unwrap();
        editor.selected = Some(0);
        let before = editor.scene.clone();
        editor.attach("Spinner");
        assert!(editor.last_error.is_none(), "{:?}", editor.last_error);
        let attached = editor.scene.clone();
        assert_eq!(
            attached.actors[0].components.len(),
            before.actors[0].components.len() + 1
        );
        editor.undo_attachment(false).unwrap();
        assert_eq!(editor.scene, before);
        editor.undo_attachment(true).unwrap();
        assert_eq!(editor.scene, attached);
        editor.scene.name = "Intervening edit".into();
        assert!(editor.undo_attachment(false).is_err());
        drop(editor);
        std::fs::remove_dir_all(root).unwrap();
    }
}
