//! Build > Bake Visibility: the visibility bake (see visibility_bake.rs) from
//! the editor, in the background, applied to the open scene like a lighting
//! bake. Presets pick the options per kind of game; recordings come from
//! Play options > Record camera for visibility bake.
use crate::editor::Editor;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
    mpsc,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Preset {
    Guided,
    Exact,
    Light,
}
impl Preset {
    pub const ALL: [Self; 3] = [Self::Guided, Self::Exact, Self::Light];
    pub fn label(self) -> &'static str {
        match self {
            Self::Guided => "Guided camera (recommended)",
            Self::Exact => "Maximum quality (every face)",
            Self::Light => "Lighter lists",
        }
    }
    pub fn help(self) -> &'static str {
        match self {
            Self::Guided => {
                "Drops faces that never cover a whole pixel; recorded cameras widened by 10%. Measured at the reference's draw cost with 0.04% of the screen in sub-pixel specks."
            }
            Self::Exact => "Lists every face that shows at all. No specks; more packets to draw.",
            Self::Light => {
                "Drops faces under 2 pixels. Fewer packets; about 0.14% of the screen in specks."
            }
        }
    }
    fn min_pixels(self) -> f32 {
        match self {
            Self::Guided => 1.,
            Self::Exact => 0.,
            Self::Light => 2.,
        }
    }
}

pub struct Result {
    lists: Vec<Vec<u16>>,
    program: Option<crate::ordered_draw::Program>,
    recorded: usize,
    seconds: f64,
    signature: String,
}

pub struct State {
    pub open: bool,
    pub preset: Preset,
    /// Draw distance in world units (converted to camera units for the bake).
    pub far_world: f32,
    pub use_recordings: bool,
    pub margin: f32,
    pub ordered: bool,
    job: Option<mpsc::Receiver<std::result::Result<Result, String>>>,
    progress: Arc<AtomicUsize>,
    total: usize,
}
impl Default for State {
    fn default() -> Self {
        Self {
            open: false,
            preset: Preset::Guided,
            far_world: 160.,
            use_recordings: true,
            margin: 0.1,
            ordered: true,
            job: None,
            progress: Arc::new(AtomicUsize::new(0)),
            total: 0,
        }
    }
}

fn recordings_dir(e: &Editor) -> std::path::PathBuf {
    e.root.join(".epok/camera-recordings")
}
fn recordings(e: &Editor) -> Vec<std::path::PathBuf> {
    let mut files: Vec<_> = std::fs::read_dir(recordings_dir(e))
        .map(|d| {
            d.filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| p.extension().is_some_and(|x| x == "jsonl"))
                .collect()
        })
        .unwrap_or_default();
    files.sort();
    files
}

pub fn busy(e: &Editor) -> bool {
    e.visibility_bake.job.is_some()
}

pub fn start(e: &mut Editor) {
    if busy(e) || e.playing {
        return;
    }
    let Some(track) = e.scene.visibility_track.clone() else {
        e.log("Bake Visibility needs a scene with a visibility track.");
        return;
    };
    let state = &e.visibility_bake;
    let scene = e.scene.clone();
    let mut options = crate::visibility_bake::Options {
        min_pixels: state.preset.min_pixels(),
        view_margin: state.margin,
        progress: Some(state.progress.clone()),
        ..Default::default()
    };
    options.width = u32::from(scene.display_size[0]);
    options.height = u32::from(scene.display_size[1]);
    options.far = state.far_world / crate::visibility_bake::camera_depth_scale(&scene, &track);
    let files = if state.use_recordings {
        recordings(e)
    } else {
        Vec::new()
    };
    let ordered = state.ordered;
    let signature = crate::baked_preview::signature(&scene);
    state.progress.store(0, Ordering::Relaxed);
    let total = track.samples.len();
    let (tx, rx) = mpsc::channel();
    e.visibility_bake.job = Some(rx);
    e.visibility_bake.total = total;
    e.log(format!(
        "Baking visibility for {total} samples on the GPU{}...",
        if files.is_empty() {
            String::new()
        } else {
            format!(" with {} camera recordings", files.len())
        }
    ));
    std::thread::spawn(move || {
        let run = || -> std::result::Result<Result, String> {
            let start = std::time::Instant::now();
            let mut recorded = Vec::new();
            for file in &files {
                recorded.extend(crate::visibility_bake::CameraCheck::load(file, &options)?);
            }
            let lists =
                crate::visibility_bake::bake_with_cameras(&scene, &track, &options, &recorded)?;
            let program = if ordered {
                Some(crate::visibility_bake::ordered_program(
                    &scene, &track, &lists, &options,
                )?)
            } else {
                None
            };
            Ok(Result {
                lists,
                program,
                recorded: recorded.len(),
                seconds: start.elapsed().as_secs_f64(),
                signature,
            })
        };
        let _ = tx.send(run());
    });
}

pub fn poll(e: &mut Editor) {
    let result = e
        .visibility_bake
        .job
        .as_ref()
        .and_then(|rx| match rx.try_recv() {
            Ok(r) => Some(r),
            Err(mpsc::TryRecvError::Disconnected) => {
                Some(Err("Visibility bake worker stopped".into()))
            }
            Err(_) => None,
        });
    let Some(result) = result else { return };
    e.visibility_bake.job = None;
    match result {
        Ok(r) if r.signature == crate::baked_preview::signature(&e.scene) => {
            let faces: usize = r.lists.iter().map(Vec::len).sum();
            let samples = r.lists.len().max(1);
            if let Some(track) = e.scene.visibility_track.as_mut() {
                for (sample, list) in track.samples.iter_mut().zip(r.lists) {
                    sample.visible = list;
                }
            }
            if let Some(program) = r.program {
                match program.validate(&e.scene) {
                    Ok(()) => e.scene.ordered_draw = Some(program),
                    Err(error) => {
                        e.log(format!("Generated packet order rejected: {error}"));
                    }
                }
            }
            e.changed();
            e.log(format!(
                "Visibility baked in {:.0} s: {:.0} faces per sample{}. Save Scene to persist.",
                r.seconds,
                faces as f64 / samples as f64,
                if r.recorded > 0 {
                    format!(", {} recorded cameras", r.recorded)
                } else {
                    String::new()
                }
            ));
        }
        Ok(_) => e.log("Scene changed during Bake Visibility. Result discarded; bake again."),
        Err(error) => e.log(format!("Bake Visibility failed: {error}")),
    }
}

pub fn window(ui: &imgui::Ui, e: &mut Editor) {
    if !e.visibility_bake.open {
        return;
    }
    let mut opened = true;
    ui.window("Bake Visibility").opened(&mut opened).position([120., 110.], imgui::Condition::FirstUseEver)
        .size([460., 360.], imgui::Condition::FirstUseEver).build(|| {
        if e.scene.visibility_track.is_none() {
            ui.text_wrapped("This scene has no visibility track. Visibility is baked per camera sample of a track.");
            return;
        }
        let busy = busy(e);
        let _disabled = ui.begin_disabled(busy || e.playing);
        let state = &mut e.visibility_bake;
        if let Some(_combo) = ui.begin_combo("Preset", state.preset.label()) {
            for preset in Preset::ALL {
                if ui.selectable_config(preset.label()).selected(preset == state.preset).build() { state.preset = preset; }
            }
        }
        ui.text_wrapped(state.preset.help());
        crate::gui::Drag::new("Draw distance (world units)").speed(1.).range(8., 4096.).build(ui, &mut state.far_world);
        if ui.is_item_hovered() { ui.tooltip_text("Faces farther than this from the camera are not listed. Shorter lists draw faster and fit fewer streaming pages."); }
        let count = recordings(e).len();
        let state = &mut e.visibility_bake;
        ui.checkbox(format!("Use camera recordings ({count})"), &mut state.use_recordings);
        if ui.is_item_hovered() { ui.tooltip_text("Recordings from Play options > Record camera for visibility bake (Native PC), in .epok/camera-recordings. Record real play: everything a recorded camera saw is listed."); }
        crate::gui::Drag::new("Recorded view margin").speed(0.01).range(0., 0.5).build(ui, &mut state.margin);
        ui.checkbox("Generate prepared packet order", &mut state.ordered);
        if ui.is_item_hovered() { ui.tooltip_text("Also build the per-sample draw order (static world, unlit, triangle faces). Replaces the scene's packet order."); }
        drop(_disabled);
        ui.separator();
        if busy {
            let done = e.visibility_bake.progress.load(Ordering::Relaxed);
            let total = e.visibility_bake.total.max(1);
            imgui::ProgressBar::new(done as f32 / total as f32).overlay_text(format!("Sample {done} of {total}")).build(ui);
        } else if ui.button("Bake") {
            start(e);
        }
    });
    if !opened {
        e.visibility_bake.open = false;
    }
}

/// The `visibility_bake` MCP tool (see its description in mcp_tools.rs).
pub fn mcp(e: &mut Editor, a: &serde_json::Value) -> std::result::Result<serde_json::Value, String> {
    let status = |e: &Editor| {
        let state = &e.visibility_bake;
        serde_json::json!({
            "busy": busy(e),
            "done": state.progress.load(Ordering::Relaxed),
            "total": state.total,
            "preset": format!("{:?}", state.preset).to_lowercase(),
            "far": state.far_world,
            "use_recordings": state.use_recordings,
            "recordings": recordings(e).len(),
            "margin": state.margin,
            "ordered": state.ordered,
        })
    };
    match a["op"].as_str() {
        Some("status") => Ok(status(e)),
        Some("start") => {
            crate::mcp_tools::editable(e)?;
            crate::mcp_tools::expect_revision(&e.scene, a)?;
            if busy(e) {
                return Err("A visibility bake is already running".into());
            }
            if e.scene.visibility_track.is_none() {
                return Err("The scene has no visibility track".into());
            }
            let state = &mut e.visibility_bake;
            if let Some(preset) = a["preset"].as_str() {
                state.preset = match preset {
                    "guided" => Preset::Guided,
                    "exact" => Preset::Exact,
                    "light" => Preset::Light,
                    other => return Err(format!("Unknown preset: {other}")),
                };
            }
            if let Some(far) = a["far"].as_f64() {
                state.far_world = (far as f32).clamp(8., 4096.);
            }
            if let Some(margin) = a["margin"].as_f64() {
                state.margin = (margin as f32).clamp(0., 0.5);
            }
            if let Some(v) = a["use_recordings"].as_bool() {
                state.use_recordings = v;
            }
            if let Some(v) = a["ordered"].as_bool() {
                state.ordered = v;
            }
            start(e);
            Ok(status(e))
        }
        _ => Err("op must be status or start".into()),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn bake_runs_in_the_background_and_applies_lists_to_the_open_scene() {
        let root = crate::workspace::tests::temp("visibility-bake-ui");
        let project =
            crate::workspace::create(&root, "Visibility", crate::workspace::Template::Basic)
                .unwrap();
        let mut editor = crate::editor::Editor::open(project).unwrap();
        let (mut scene, track, front_faces) =
            crate::visibility_bake::tests::scene_with(crate::texture::BlendMode::Cutout);
        scene.display_size = [512, 240];
        scene.visibility_track = Some(track);
        editor.scene = scene;
        editor.visibility_bake.ordered = false;
        editor.visibility_bake.use_recordings = false;
        super::start(&mut editor);
        assert!(super::busy(&editor));
        let start = std::time::Instant::now();
        while super::busy(&editor) && start.elapsed() < std::time::Duration::from_secs(60) {
            std::thread::sleep(std::time::Duration::from_millis(20));
            super::poll(&mut editor);
        }
        let visible = &editor.scene.visibility_track.as_ref().unwrap().samples[0].visible;
        assert_eq!(visible.len(), 1, "{visible:?}");
        assert!(usize::from(visible[0]) < front_faces);
        let _ = std::fs::remove_dir_all(&root);
    }
}
