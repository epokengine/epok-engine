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
            Self::Guided => "Balanced (recommended)",
            Self::Exact => "Maximum quality",
            Self::Light => "Fastest drawing",
        }
    }
    pub fn help(self) -> &'static str {
        match self {
            Self::Guided => {
                "Draws faces covering at least one pixel. Measured at the reference renderer's draw cost; tiny specks on about 0.04% of the screen."
            }
            Self::Exact => "Draws every face that shows at all. No specks, more to draw.",
            Self::Light => {
                "Draws faces of at least two pixels. Least to draw; specks on about 0.14% of the screen."
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
    started: Option<std::time::Instant>,
    /// Summary of the last bake, shown in the panel.
    last: Option<String>,
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
            started: None,
            last: None,
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
    e.visibility_bake.started = Some(std::time::Instant::now());
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
            e.visibility_bake.last = Some(format!(
                "Baked {samples} samples in {:.0} s: {:.0} faces drawn per sample on average.",
                r.seconds,
                faces as f64 / samples as f64
            ));
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
    ui.window("Bake Visibility")
        .opened(&mut opened)
        .position([120., 110.], imgui::Condition::FirstUseEver)
        .size([440., 470.], imgui::Condition::FirstUseEver)
        .size_constraints([340., 240.], [f32::MAX, f32::MAX])
        .build(|| contents(ui, e));
    if !opened {
        e.visibility_bake.open = false;
    }
}

fn tip(ui: &imgui::Ui, text: &str) {
    if ui.is_item_hovered_with_flags(imgui::ItemHoveredFlags::ALLOW_WHEN_DISABLED) {
        ui.tooltip_text(text);
    }
}

fn wrapped(ui: &imgui::Ui, color: [f32; 4], text: &str) {
    let _c = ui.push_style_color(imgui::StyleColor::Text, color);
    ui.text_wrapped(text);
}
const MUTED: [f32; 4] = [0.6, 0.62, 0.66, 1.];

fn section(ui: &imgui::Ui, title: &str) {
    ui.spacing();
    ui.text_colored([0.6, 0.62, 0.66, 1.], title);
    ui.separator();
}

fn contents(ui: &imgui::Ui, e: &mut Editor) {
    let Some(track) = &e.scene.visibility_track else {
        ui.text_wrapped(
            "This scene has no camera track yet. Create one with Build > Camera Path, \
             then bake what the camera sees along it.",
        );
        if ui.button("Open Camera Path") {
            e.camera_path.open = true;
        }
        return;
    };
    ui.text_wrapped(
        "Works out, for every point of the camera track, which faces the camera can \
         see, so the game only loads and draws those.",
    );
    let stale = !track.path.is_empty() && !crate::camera_path::samples_current(track);
    let samples = track.samples.len();
    let recordings = recordings(e).len();
    if stale {
        ui.spacing();
        wrapped(
            ui,
            [1., 0.76, 0.28, 1.],
            "\u{f071}  The camera track does not follow the edited path yet.",
        );
        if ui.button("Update samples first") {
            match crate::camera_path::apply(&mut e.scene) {
                Ok(()) => e.changed(),
                Err(error) => e.log(format!("Update samples failed: {error}")),
            }
        }
        tip(
            ui,
            "Sample the Camera Path into the track; the bake then follows the new path.",
        );
    }
    if busy(e) {
        section(ui, "Baking");
        let state = &e.visibility_bake;
        let done = state.progress.load(Ordering::Relaxed);
        let total = state.total.max(1);
        let elapsed = state.started.map_or(0., |t| t.elapsed().as_secs_f32());
        let remaining = if done > 0 {
            format!(
                ", about {} left",
                duration(elapsed / done as f32 * (total - done) as f32)
            )
        } else {
            String::new()
        };
        imgui::ProgressBar::new(done as f32 / total as f32)
            .overlay_text(format!("{done} of {total} samples"))
            .size([-1., 0.])
            .build(ui);
        ui.text_disabled(format!(
            "{} elapsed{remaining}. The editor stays usable.",
            duration(elapsed)
        ));
        return;
    }
    let footer = ui.frame_height_with_spacing() * 2.5
        + if e.visibility_bake.last.is_some() {
            ui.text_line_height_with_spacing() * 3.
        } else {
            0.
        };
    ui.child_window("bake-options")
        .size([0., -footer])
        .build(|| options(ui, e, recordings));
    footer_ui(ui, e, samples);
}

fn options(ui: &imgui::Ui, e: &mut Editor, recordings: usize) {
    let _disabled = ui.begin_disabled(e.playing);
    section(ui, "Quality");
    let state = &mut e.visibility_bake;
    for preset in Preset::ALL {
        if ui.radio_button_bool(preset.label(), state.preset == preset) {
            state.preset = preset;
        }
        ui.indent_by(24.);
        wrapped(ui, MUTED, preset.help());
        ui.unindent_by(24.);
    }

    section(ui, "Camera");
    let state = &mut e.visibility_bake;
    ui.set_next_item_width(140.);
    crate::gui::Drag::new("Draw distance")
        .speed(1.)
        .range(8., 4096.)
        .display_format("%.0f units")
        .build(ui, &mut state.far_world);
    tip(
        ui,
        "Faces farther than this from the camera are never drawn. Shorter distances draw faster and need less memory.",
    );
    ui.checkbox(
        format!("Include camera recordings ({recordings})"),
        &mut state.use_recordings,
    );
    tip(
        ui,
        "Also bake what the camera really saw while playing: enable Play options > Record camera \
         for visibility bake and play the level on Native PC. Recordings are kept in \
         .epok/camera-recordings. Record real play: everything a recording saw is drawn.",
    );
    if state.use_recordings && recordings == 0 {
        ui.indent_by(24.);
        wrapped(
            ui,
            MUTED,
            "No recordings yet: the bake uses the camera track only.",
        );
        ui.unindent_by(24.);
    }
    if state.use_recordings && recordings > 0 {
        ui.indent_by(24.);
        ui.set_next_item_width(116.);
        let mut percent = state.margin * 100.;
        if crate::gui::Drag::new("View margin")
            .speed(0.5)
            .range(0., 50.)
            .display_format("%.0f%%")
            .build(ui, &mut percent)
        {
            state.margin = percent / 100.;
        }
        tip(
            ui,
            "Widens the recorded views, to cover camera poses between recorded sessions.",
        );
        ui.unindent_by(24.);
    }
    ui.checkbox("Generate prepared packet order", &mut state.ordered);
    tip(
        ui,
        "Also store each sample's draw order, so the console skips sorting at run time. \
         Needs static, unlit scenery. Replaces the scene's packet order.",
    );
}

fn footer_ui(ui: &imgui::Ui, e: &mut Editor, samples: usize) {
    ui.separator();
    if let Some(last) = &e.visibility_bake.last {
        wrapped(
            ui,
            [0.42, 0.85, 0.52, 1.],
            &format!("\u{f00c}  {last} Save the scene to keep it."),
        );
    }
    let _disabled = ui.begin_disabled(e.playing);
    let _a = ui.push_style_color(imgui::StyleColor::Button, [0.18, 0.42, 0.72, 1.]);
    let _b = ui.push_style_color(imgui::StyleColor::ButtonHovered, [0.24, 0.52, 0.86, 1.]);
    if ui.button_with_size(format!("Bake {samples} samples"), [-1., 30.]) {
        start(e);
    }
    tip(
        ui,
        "Runs on the GPU in the background. The result replaces the scene's baked visibility.",
    );
}

fn duration(seconds: f32) -> String {
    if seconds < 90. {
        format!("{seconds:.0} s")
    } else {
        format!("{:.0} min", seconds / 60.)
    }
}

/// The `visibility_bake` MCP tool (see its description in mcp_tools.rs).
pub fn mcp(
    e: &mut Editor,
    a: &serde_json::Value,
) -> std::result::Result<serde_json::Value, String> {
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
