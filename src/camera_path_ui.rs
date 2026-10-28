//! Build > Camera Path: author a visibility track's camera path in the
//! editor (see camera_path.rs). Keys are taken from the Scene view; the path
//! is drawn in the Scene view, where keys can be picked, and on a timeline
//! where they can be selected, retimed and previewed.
use crate::camera_path::{self, Key};
use crate::editor::Editor;

// Codicon (ea60..) and Font Awesome (f000..) glyphs the editor fonts load
// (see EDITOR_FA_GLYPHS in platform.rs).
const ADD: &str = "\u{ea60}";
const CAMERA: &str = "\u{ead9}";
const PLAY: &str = "\u{eb2c}";
const PAUSE: &str = "\u{ead1}";
const EYE: &str = "\u{f06e}";
const TRASH: &str = "\u{f1f8}";
const AIM: &str = "\u{f05b}";
const START: &str = "\u{f048}";
const PREVIOUS: &str = "\u{f104}";
const NEXT: &str = "\u{f105}";
const CHECK: &str = "\u{f00c}";
const WARNING: &str = "\u{f071}";
const TODO: &str = "\u{f111}";
const SCISSORS: &str = "\u{f0c4}";

const ACCENT: [f32; 4] = [0.33, 0.62, 0.98, 1.];
const KEY: [f32; 4] = [0.45, 0.78, 1., 1.];
const SELECTED: [f32; 4] = [1., 0.68, 0.2, 1.];
const CUT: [f32; 4] = [1., 0.42, 0.38, 1.];
const GOOD: [f32; 4] = [0.42, 0.85, 0.52, 1.];
const PENDING: [f32; 4] = [1., 0.76, 0.28, 1.];
const PLAYHEAD: [f32; 4] = [0.96, 0.96, 0.96, 1.];
const MUTED: [f32; 4] = [0.6, 0.62, 0.66, 1.];

#[derive(Clone, Copy, PartialEq)]
enum Drag {
    Scrub,
    Key(usize),
}

pub struct State {
    pub open: bool,
    /// Draw the path in the Scene view while the panel is closed too.
    pub show: bool,
    pub time: f32,
    pub playing: bool,
    /// Look through the path at `time` in the Scene view.
    pub preview: bool,
    pub selected: Option<usize>,
    camera: usize,
    /// Visible part of the timeline, in seconds; None shows the whole path.
    range: Option<(f32, f32)>,
    drag: Option<Drag>,
    /// The key under the mouse in the Scene view.
    hovered: Option<usize>,
}
impl Default for State {
    fn default() -> Self {
        Self {
            open: false,
            show: true,
            time: 0.,
            playing: false,
            preview: false,
            selected: None,
            camera: 0,
            range: None,
            drag: None,
            hovered: None,
        }
    }
}

fn key_from_view(e: &Editor, time: f32) -> Key {
    let pose = camera_path::pose_from_view(&e.view);
    Key {
        time,
        position: pose.position,
        rotation: pose.rotation,
        fov: pose.fov,
        cut: false,
    }
}

fn look_through(e: &mut Editor, pose: camera_path::Pose) {
    camera_path::view_through(&mut e.view, pose);
    e.view_dirty = true;
}

/// Moves the playhead, looking through the path when that is on.
fn seek(e: &mut Editor, time: f32) {
    let Some(track) = &e.scene.visibility_track else {
        return;
    };
    let time = time.clamp(0., camera_path::duration(&track.path));
    e.camera_path.time = time;
    if e.camera_path.preview {
        let pose = camera_path::pose_at(&track.path, time);
        look_through(e, pose);
    }
}

fn select(e: &mut Editor, n: usize) {
    let Some(key) = e
        .scene
        .visibility_track
        .as_ref()
        .and_then(|t| t.path.get(n))
    else {
        return;
    };
    let time = key.time;
    e.camera_path.selected = Some(n);
    e.camera_path.playing = false;
    seek(e, time);
}

/// Replaces the path; `gesture` folds a drag into one undo step.
fn set_path(e: &mut Editor, path: Vec<Key>, gesture: Option<&'static str>) {
    let Some(track) = e.scene.visibility_track.as_mut() else {
        return;
    };
    track.path = path;
    match gesture {
        Some(key) => e.changed_coalesced(key),
        None => e.changed(),
    }
}

/// Where "Add key" puts the next key: at the playhead inside the path, or
/// two seconds after the last key.
fn add_time(path: &[Key], time: f32) -> (f32, bool) {
    let duration = camera_path::duration(path);
    let inside = path.len() > 1
        && time > 0.05
        && time < duration - 0.05
        && path.iter().all(|k| (k.time - time).abs() > 0.05);
    if inside {
        (time, true)
    } else {
        (duration + if path.is_empty() { 0. } else { 2. }, false)
    }
}

fn add_key(e: &mut Editor) {
    let Some(track) = &e.scene.visibility_track else {
        return;
    };
    let (time, _) = add_time(&track.path, e.camera_path.time);
    let mut path = track.path.clone();
    let at = path.partition_point(|k| k.time < time);
    path.insert(at, key_from_view(e, time));
    set_path(e, path, None);
    e.camera_path.selected = Some(at);
    e.camera_path.time = time;
    e.camera_path.range = None;
}

fn delete_key(e: &mut Editor, n: usize) {
    let Some(track) = &e.scene.visibility_track else {
        return;
    };
    if track.path.len() < 2 || n >= track.path.len() {
        return;
    }
    let mut path = track.path.clone();
    path.remove(n);
    if n == 0 {
        // The next key starts the path.
        let offset = path[0].time;
        for k in &mut path {
            k.time -= offset;
        }
        path[0].cut = false;
    }
    let count = path.len();
    set_path(e, path, None);
    e.camera_path.selected = Some(n.min(count - 1));
}

pub fn window(ui: &imgui::Ui, e: &mut Editor) {
    if !e.camera_path.open {
        return;
    }
    let mut opened = true;
    ui.window("\u{ead9} Camera Path###Camera Path")
        .opened(&mut opened)
        .position([140., 120.], imgui::Condition::FirstUseEver)
        .size([470., 560.], imgui::Condition::FirstUseEver)
        .size_constraints([360., 260.], [f32::MAX, f32::MAX])
        .build(|| contents(ui, e));
    if !opened {
        e.camera_path.open = false;
        e.camera_path.playing = false;
    }
}

fn wrapped_colored(ui: &imgui::Ui, color: [f32; 4], text: &str) {
    let _c = ui.push_style_color(imgui::StyleColor::Text, color);
    ui.text_wrapped(text);
}

fn primary_button(ui: &imgui::Ui, label: &str, width: f32) -> bool {
    let _a = ui.push_style_color(imgui::StyleColor::Button, [0.18, 0.42, 0.72, 1.]);
    let _b = ui.push_style_color(imgui::StyleColor::ButtonHovered, [0.24, 0.52, 0.86, 1.]);
    let _c = ui.push_style_color(imgui::StyleColor::ButtonActive, [0.14, 0.36, 0.64, 1.]);
    ui.button_with_size(label, [width, 0.])
}

fn tip(ui: &imgui::Ui, text: &str) {
    if ui.is_item_hovered_with_flags(imgui::ItemHoveredFlags::ALLOW_WHEN_DISABLED) {
        ui.tooltip_text(text);
    }
}

fn contents(ui: &imgui::Ui, e: &mut Editor) {
    if e.playing {
        wrapped_colored(ui, MUTED, "Stop Play to edit the camera path.");
        return;
    }
    if crate::visibility_bake_ui::busy(e) {
        wrapped_colored(
            ui,
            MUTED,
            "Baking visibility. The path can be edited when the bake finishes.",
        );
        return;
    }
    let Some(track) = &e.scene.visibility_track else {
        return empty(ui, e);
    };
    if track.navigation.is_some() {
        ui.text_wrapped(
            "This track's samples come from a route graph (player-driven rails), \
             so its path is not edited here.",
        );
        return;
    }
    if track.path.is_empty() {
        ui.text_wrapped("This scene's camera track was imported as samples.");
        ui.spacing();
        ui.text_wrapped(
            "Convert it to keys to edit it here: a key every second, one at every \
             cut, and more wherever the camera turns sharply, so the path stays \
             within a fraction of a unit of the original.",
        );
        ui.spacing();
        if primary_button(ui, "Convert samples to keys", -1.) {
            let tolerance = camera_path::Tolerance::for_track(&e.scene, track);
            let keys = camera_path::keys_from_samples(track, 1., tolerance);
            let count = keys.len();
            set_path(e, keys, None);
            e.camera_path.selected = Some(0);
            e.log(format!("Converted the camera track to {count} keys."));
        }
        return;
    }
    steps(ui, e);
    ui.spacing();
    transport(ui, e);
    timeline(ui, e);
    ui.spacing();
    selected_key(ui, e);
    ui.spacing();
    settings(ui, e);
    shortcuts(ui, e);
}

fn empty(ui: &imgui::Ui, e: &mut Editor) {
    ui.text("Guided camera");
    ui.separator();
    ui.text_wrapped(
        "Draw the route the game camera follows. Epok turns it into a camera \
         track and works out what the camera sees along it, so only that is \
         loaded and drawn.",
    );
    ui.spacing();
    for (n, text) in [
        "Create the path for a camera.",
        "Fly the Scene view to each shot and press Add key.",
        "Update samples, then Bake Visibility.",
    ]
    .iter()
    .enumerate()
    {
        ui.text_colored(ACCENT, format!("{}", n + 1));
        ui.same_line();
        ui.text_wrapped(text);
    }
    ui.spacing();
    let cameras = camera_path::cameras(&e.scene);
    if cameras.is_empty() {
        wrapped_colored(
            ui,
            PENDING,
            &format!("{WARNING}  Add a Camera actor at the scene root first."),
        );
        return;
    }
    let state = &mut e.camera_path;
    let current = cameras.get(state.camera).copied().unwrap_or(cameras[0]);
    ui.set_next_item_width(-1.);
    if let Some(_combo) = ui.begin_combo(
        "##camera",
        format!("{CAMERA}  {}", e.scene.actors[current].name),
    ) {
        for (n, &i) in cameras.iter().enumerate() {
            if ui
                .selectable_config(format!("{CAMERA}  {}", e.scene.actors[i].name))
                .selected(i == current)
                .build()
            {
                state.camera = n;
            }
        }
    }
    tip(
        ui,
        "The camera the path drives. The path starts at its current pose.",
    );
    if primary_button(ui, &format!("{ADD}  Create camera path"), -1.) {
        match camera_path::create(&mut e.scene, current) {
            Ok(()) => {
                e.changed();
                e.camera_path.selected = Some(0);
                e.camera_path.time = 0.;
                e.log("Camera path created with one key at the camera.");
            }
            Err(error) => e.log(format!("Create camera path failed: {error}")),
        }
    }
}

/// Where the workflow stands, and the one action that moves it on.
fn steps(ui: &imgui::Ui, e: &mut Editor) {
    let track = e.scene.visibility_track.as_ref().unwrap();
    let valid = camera_path::validate(&track.path, track.sample_ms);
    let current = valid.is_ok() && camera_path::samples_current(track);
    let baked = current && track.samples.iter().any(|s| !s.visible.is_empty());
    let keys = track.path.len();
    let pill = |ui: &imgui::Ui, n: usize, label: &str, done: bool, active: bool| {
        let color = if done {
            GOOD
        } else if active {
            PENDING
        } else {
            MUTED
        };
        ui.text_colored(
            color,
            format!(
                "{} {n} {label}",
                if done {
                    CHECK
                } else if active {
                    WARNING
                } else {
                    TODO
                }
            ),
        );
    };
    pill(
        ui,
        1,
        "Path",
        valid.is_ok() && keys > 1,
        valid.is_err() || keys < 2,
    );
    ui.same_line();
    ui.text_disabled(NEXT);
    ui.same_line();
    pill(
        ui,
        2,
        "Samples",
        current,
        valid.is_ok() && keys > 1 && !current,
    );
    ui.same_line();
    ui.text_disabled(NEXT);
    ui.same_line();
    pill(ui, 3, "Visibility", baked, current && !baked);

    let (color, message, action): (_, String, Option<&str>) = match &valid {
        Err(error) => (CUT, format!("Fix the path: {error}."), None),
        Ok(()) if keys < 2 => (
            ACCENT,
            "Fly the Scene view to the next shot (right-drag to look, WASD to move), then press Add key.".into(),
            None,
        ),
        Ok(()) if !current => (
            PENDING,
            "The camera track does not follow this path yet. Update its samples; baked visibility is then cleared.".into(),
            Some("Update samples"),
        ),
        Ok(()) if !baked => (
            PENDING,
            "Bake what the camera sees along the path.".into(),
            Some("Bake Visibility..."),
        ),
        Ok(()) => (
            GOOD,
            "Ready: the camera follows this path and its visibility is baked.".into(),
            None,
        ),
    };
    let _bg = ui.push_style_color(
        imgui::StyleColor::ChildBg,
        [color[0] * 0.18, color[1] * 0.18, color[2] * 0.18, 1.],
    );
    let height = if action.is_some() { 64. } else { 42. };
    ui.child_window("next-step")
        .size([0., height])
        .border(false)
        .build(|| {
            let start = ui.cursor_screen_pos();
            ui.get_window_draw_list()
                .add_rect(start, [start[0] + 3., start[1] + height], color)
                .filled(true)
                .build();
            ui.set_cursor_pos([12., 6.]);
            wrapped_colored(ui, color, &message);
            if let Some(action) = action {
                ui.set_cursor_pos([12., ui.cursor_pos()[1]]);
                if ui.small_button(action) {
                    if action == "Update samples" {
                        match camera_path::apply(&mut e.scene) {
                            Ok(()) => {
                                e.changed();
                                e.log("Camera path sampled. Bake Visibility next.");
                            }
                            Err(error) => e.log(format!("Update samples failed: {error}")),
                        }
                    } else {
                        e.visibility_bake.open = true;
                    }
                }
            }
        });
}

fn transport(ui: &imgui::Ui, e: &mut Editor) {
    let track = e.scene.visibility_track.as_ref().unwrap();
    let duration = camera_path::duration(&track.path);
    let (add_at, inside) = add_time(&track.path, e.camera_path.time);
    let state = &mut e.camera_path;
    if state.playing {
        state.time += ui.io().delta_time;
        if state.time > duration {
            state.time = 0.;
        }
        let time = state.time;
        seek(e, time);
    }
    let state = &mut e.camera_path;
    if crate::gui::icon(ui, START, "start", "Go to the start", false) {
        state.playing = false;
        seek(e, 0.);
    }
    ui.same_line();
    let state = &mut e.camera_path;
    let playing = state.playing;
    if crate::gui::icon(
        ui,
        if playing { PAUSE } else { PLAY },
        "play",
        if playing {
            "Pause (Space)"
        } else {
            "Play the path through the Scene view (Space)"
        },
        playing,
    ) {
        state.playing = !playing;
        if state.playing {
            state.preview = true;
            if state.time >= duration {
                state.time = 0.;
            }
        }
    }
    ui.same_line();
    let state = &mut e.camera_path;
    let preview = state.preview;
    if crate::gui::icon(
        ui,
        EYE,
        "look",
        if preview {
            "Looking through the path camera. Click to fly the Scene view freely."
        } else {
            "Look through the path camera at the playhead"
        },
        preview,
    ) {
        state.preview = !preview;
        if state.preview {
            let time = state.time;
            seek(e, time);
        } else {
            state.playing = false;
        }
    }
    ui.same_line();
    let time = e.camera_path.time;
    ui.text(format!("{time:6.2} s"));
    ui.same_line();
    ui.text_disabled(format!("/ {duration:.2} s"));
    let label = if inside {
        format!("{ADD}  Insert key")
    } else {
        format!("{ADD}  Add key")
    };
    let width = ui.calc_text_size(&label)[0] + 18.;
    ui.same_line_with_pos((ui.content_region_max()[0] - width).max(ui.cursor_pos()[0]));
    if primary_button(ui, &label, width) {
        add_key(e);
    }
    tip(
        ui,
        &if inside {
            format!("Insert a key at the playhead ({add_at:.2} s) from the Scene view camera (K).")
        } else {
            format!(
                "Add a key {add_at:.2} s into the path, 2 s after the last one, from the Scene view camera (K)."
            )
        },
    );
}

fn nice_step(span: f32, width: f32) -> f32 {
    let wanted = span * 70. / width.max(1.);
    [0.1, 0.25, 0.5, 1., 2., 5., 10., 15., 30., 60., 120., 300.]
        .into_iter()
        .find(|&s| s >= wanted)
        .unwrap_or(600.)
}

fn timeline(ui: &imgui::Ui, e: &mut Editor) {
    let track = e.scene.visibility_track.as_ref().unwrap();
    let path = track.path.clone();
    let duration = camera_path::duration(&path).max(0.5);
    let width = ui.content_region_avail()[0].max(50.);
    let height = 58.;
    let origin = ui.cursor_screen_pos();
    ui.invisible_button("timeline", [width, height]);
    let hovered = ui.is_item_hovered();
    let active = ui.is_item_active();
    let state = &mut e.camera_path;
    let (mut t0, mut t1) = state.range.unwrap_or((0., duration));
    // Zoom around the mouse with the wheel; shift pans.
    let mouse = ui.io().mouse_pos;
    let to_time = |x: f32, t0: f32, t1: f32| t0 + (x - origin[0]) / width * (t1 - t0);
    if hovered && ui.io().mouse_wheel != 0. {
        let wheel = ui.io().mouse_wheel;
        if ui.io().key_shift {
            let shift = -(t1 - t0) * 0.15 * wheel;
            t0 += shift;
            t1 += shift;
        } else {
            let at = to_time(mouse[0], t0, t1);
            let scale = 0.85f32.powf(wheel);
            t0 = at - (at - t0) * scale;
            t1 = at + (t1 - at) * scale;
        }
    }
    if hovered && ui.io().mouse_wheel_h != 0. {
        let shift = (t1 - t0) * 0.15 * ui.io().mouse_wheel_h;
        t0 += shift;
        t1 += shift;
    }
    let span = (t1 - t0).clamp(0.5, duration);
    t0 = t0.clamp(0., duration - span);
    t1 = t0 + span;
    // Keep the playhead in view while playing.
    if state.playing && (state.time < t0 || state.time > t1) {
        t0 = (state.time - span * 0.1).clamp(0., duration - span);
        t1 = t0 + span;
    }
    state.range = ((t1 - t0) < duration - 1e-3).then_some((t0, t1));
    let x_of = |t: f32| origin[0] + (t - t0) / (t1 - t0) * width;
    let ruler = 16.;
    let lane = origin[1] + ruler + (height - ruler) * 0.5;

    // Which key is under the mouse.
    let near = if hovered && state.drag.is_none() {
        path.iter()
            .enumerate()
            .filter(|(_, k)| k.time >= t0 - 1. && k.time <= t1 + 1.)
            .map(|(n, k)| (n, (x_of(k.time) - mouse[0]).abs()))
            .filter(|&(_, d)| d <= 6. && (mouse[1] - lane).abs() <= 14.)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(n, _)| n)
    } else {
        None
    };

    let draw = ui.get_window_draw_list();
    draw.add_rect(
        origin,
        [origin[0] + width, origin[1] + height],
        [0.11, 0.115, 0.125, 1.],
    )
    .filled(true)
    .rounding(4.)
    .build();
    draw.add_rect(
        origin,
        [origin[0] + width, origin[1] + ruler],
        [0.15, 0.155, 0.17, 1.],
    )
    .filled(true)
    .rounding(4.)
    .build();
    draw.with_clip_rect_intersect(origin, [origin[0] + width, origin[1] + height], || {
        let step = nice_step(t1 - t0, width);
        let mut t = (t0 / step).floor() * step;
        while t <= t1 + step {
            let x = x_of(t);
            draw.add_line(
                [x, origin[1] + ruler - 5.],
                [x, origin[1] + height],
                [1., 1., 1., 0.07],
            )
            .build();
            let label = if step < 1. {
                format!("{t:.1}s")
            } else {
                format!("{t:.0}s")
            };
            draw.add_text([x + 3., origin[1] + 1.], MUTED, label);
            t += step;
        }
        // Travelled segments; cuts break the line.
        for pair in path.windows(2) {
            let (a, b) = (x_of(pair[0].time), x_of(pair[1].time));
            if pair[1].cut {
                draw.add_line(
                    [b, origin[1] + ruler + 3.],
                    [b, origin[1] + height - 3.],
                    CUT,
                )
                .thickness(1.5)
                .build();
            } else {
                draw.add_line([a, lane], [b, lane], [0.45, 0.78, 1., 0.55])
                    .thickness(2.)
                    .build();
            }
        }
        for (n, k) in path.iter().enumerate() {
            let x = x_of(k.time);
            if x < origin[0] - 8. || x > origin[0] + width + 8. {
                continue;
            }
            let selected = state.selected == Some(n) || state.hovered == Some(n);
            let size = if selected {
                7.
            } else if near == Some(n) {
                6.5
            } else {
                5.
            };
            let color = if selected {
                SELECTED
            } else if k.cut {
                CUT
            } else if near == Some(n) {
                [0.75, 0.9, 1., 1.]
            } else {
                KEY
            };
            let (top, right, bottom, left) = (
                [x, lane - size],
                [x + size, lane],
                [x, lane + size],
                [x - size, lane],
            );
            draw.add_triangle(top, right, bottom, color)
                .filled(true)
                .build();
            draw.add_triangle(top, bottom, left, color)
                .filled(true)
                .build();
            if selected {
                draw.add_text([x - 3., lane + 8.], SELECTED, format!("{n}"));
            }
        }
        let x = x_of(state.time);
        draw.add_line([x, origin[1]], [x, origin[1] + height], PLAYHEAD)
            .thickness(1.5)
            .build();
        draw.add_triangle(
            [x - 5., origin[1]],
            [x + 5., origin[1]],
            [x, origin[1] + 7.],
            PLAYHEAD,
        )
        .filled(true)
        .build();
    });

    if hovered && state.drag.is_none() {
        if let Some(n) = near {
            let k = &path[n];
            ui.tooltip_text(format!(
                "Key {n} at {:.2} s{}\nClick to select{}. Right-click for options.",
                k.time,
                if k.cut { " (cut)" } else { "" },
                if n > 0 { ", drag to retime" } else { "" }
            ));
        } else if state.range.is_none() {
            ui.tooltip_text("Click or drag to move the playhead. Wheel to zoom.");
        }
    }
    // Interaction.
    if ui.is_item_clicked() {
        let state = &mut e.camera_path;
        state.drag = Some(match near {
            Some(n) => Drag::Key(n),
            None => Drag::Scrub,
        });
        state.playing = false;
        match near {
            Some(n) => select(e, n),
            None => seek(e, to_time(mouse[0], t0, t1)),
        }
    }
    if ui.is_item_clicked_with_button(imgui::MouseButton::Right)
        && let Some(n) = near
    {
        select(e, n);
        ui.open_popup("key-menu");
    }
    match e.camera_path.drag {
        Some(Drag::Scrub) if active => seek(e, to_time(mouse[0], t0, t1)),
        Some(Drag::Key(n)) if active && n > 0 && ui.is_mouse_dragging(imgui::MouseButton::Left) => {
            let low = path[n - 1].time + 0.05;
            let high = path.get(n + 1).map_or(f32::MAX, |k| k.time - 0.05);
            let time = (to_time(mouse[0], t0, t1) * 20.).round() / 20.;
            let time = time.clamp(low, high.max(low));
            if (time - path[n].time).abs() > 1e-4 {
                let mut moved = path.clone();
                moved[n].time = time;
                set_path(e, moved, Some("camera-path-key"));
                e.camera_path.time = time;
            }
        }
        _ => {}
    }
    if !active && e.camera_path.drag.is_some() {
        e.camera_path.drag = None;
        e.end_coalesced();
    }
    key_menu(ui, e);
    wrapped_colored(
        ui,
        MUTED,
        if e.camera_path.range.is_some() {
            "Wheel to zoom, Shift+wheel to scroll."
        } else {
            "Drag keys to retime them. Wheel to zoom."
        },
    );
}

fn key_menu(ui: &imgui::Ui, e: &mut Editor) {
    let Some(n) = e.camera_path.selected else {
        return;
    };
    ui.popup("key-menu", || {
        let Some(key) = e
            .scene
            .visibility_track
            .as_ref()
            .and_then(|t| t.path.get(n))
            .copied()
        else {
            return;
        };
        ui.text_disabled(format!("Key {n} at {:.2} s", key.time));
        ui.separator();
        if ui.menu_item(format!("{EYE}  Look through")) {
            e.camera_path.preview = false;
            look_through(e, key.into());
        }
        if ui.menu_item(format!("{AIM}  Move to Scene view")) {
            move_to_view(e, n);
        }
        if n > 0
            && ui
                .menu_item_config(format!("{SCISSORS}  Cut to this key"))
                .selected(key.cut)
                .build()
        {
            toggle_cut(e, n);
        }
        let count = e
            .scene
            .visibility_track
            .as_ref()
            .map_or(0, |t| t.path.len());
        if ui
            .menu_item_config(format!("{TRASH}  Delete"))
            .enabled(count > 1)
            .build()
        {
            delete_key(e, n);
        }
    });
}

fn move_to_view(e: &mut Editor, n: usize) {
    let Some(track) = &e.scene.visibility_track else {
        return;
    };
    let mut path = track.path.clone();
    let Some(key) = path.get_mut(n) else {
        return;
    };
    *key = Key {
        cut: key.cut,
        ..key_from_view(e, key.time)
    };
    set_path(e, path, None);
}

fn toggle_cut(e: &mut Editor, n: usize) {
    let Some(track) = &e.scene.visibility_track else {
        return;
    };
    let mut path = track.path.clone();
    if n == 0 || n >= path.len() {
        return;
    }
    path[n].cut = !path[n].cut;
    set_path(e, path, None);
}

fn selected_key(ui: &imgui::Ui, e: &mut Editor) {
    let path = e.scene.visibility_track.as_ref().unwrap().path.clone();
    let Some(n) = e.camera_path.selected.filter(|&n| n < path.len()) else {
        wrapped_colored(
            ui,
            MUTED,
            "Select a key on the timeline or click one in the Scene view.",
        );
        return;
    };
    let key = path[n];
    if crate::gui::icon(
        ui,
        PREVIOUS,
        "previous-key",
        "Previous key (Left arrow)",
        false,
    ) && n > 0
    {
        select(e, n - 1);
    }
    ui.same_line();
    ui.text(format!("Key {n}"));
    ui.same_line();
    ui.text_disabled(format!("of {}", path.len()));
    ui.same_line();
    if crate::gui::icon(ui, NEXT, "next-key", "Next key (Right arrow)", false) && n + 1 < path.len()
    {
        select(e, n + 1);
    }
    if key.cut {
        ui.same_line();
        ui.text_colored(CUT, format!("{SCISSORS} Cut"));
    }

    let mut edited = key;
    let mut gesture = false;
    let mut released = false;
    let label_width = ui.calc_text_size("Field of view")[0] + 16.;
    let row = |ui: &imgui::Ui, label: &str| {
        ui.align_text_to_frame_padding();
        ui.text(label);
        ui.same_line_with_pos(label_width);
        ui.set_next_item_width(-1.);
    };
    row(ui, "Time");
    {
        let _first = ui.begin_disabled(n == 0);
        let low = if n == 0 { 0. } else { path[n - 1].time + 0.05 };
        let high = path.get(n + 1).map_or(36000., |k| k.time - 0.05);
        gesture |= crate::gui::Drag::new("##key-time")
            .speed(0.01)
            .range(low, high.max(low))
            .display_format("%.2f s")
            .build(ui, &mut edited.time);
        released |= ui.is_item_deactivated_after_edit();
    }
    tip(
        ui,
        if n == 0 {
            "The first key starts the path at 0 s."
        } else {
            "When the camera reaches this key. Drag, or double-click to type."
        },
    );
    row(ui, "Field of view");
    gesture |= ui
        .slider_config("##key-fov", 25., 120.)
        .display_format("%.0f\u{b0}")
        .build(&mut edited.fov);
    released |= ui.is_item_deactivated_after_edit();
    tip(ui, "Horizontal field of view. Smaller zooms in.");
    row(ui, "Transition");
    {
        let _first = ui.begin_disabled(n == 0);
        let mut mode = usize::from(edited.cut);
        let labels = ["Travel", "Cut"];
        if let Some(_combo) = ui.begin_combo("##key-cut", labels[mode]) {
            for (i, label) in labels.iter().enumerate() {
                if ui.selectable_config(label).selected(i == mode).build() {
                    mode = i;
                }
            }
        }
        if (mode == 1) != edited.cut {
            edited.cut = mode == 1;
            let mut path = path.clone();
            path[n] = edited;
            set_path(e, path, None);
            return;
        }
    }
    tip(
        ui,
        "Travel: the camera moves smoothly from the previous key.\nCut: it jumps to this key, like a film cut.",
    );
    if gesture && edited != key {
        let mut path = path.clone();
        path[n] = edited;
        set_path(e, path, Some("camera-path-key-fields"));
        if edited.time != key.time {
            e.camera_path.time = edited.time;
        }
    }
    if released {
        e.end_coalesced();
    }

    let third =
        ((ui.content_region_avail()[0] - 2. * ui.clone_style().item_spacing[0]) / 3.).max(60.);
    if ui.button_with_size(format!("{EYE}  Look through"), [third, 0.]) {
        e.camera_path.preview = false;
        e.camera_path.playing = false;
        look_through(e, key.into());
    }
    tip(ui, "Show this key's shot in the Scene view.");
    ui.same_line();
    if ui.button_with_size(format!("{AIM}  Move to view"), [third, 0.]) {
        move_to_view(e, n);
    }
    tip(
        ui,
        "Set this key's camera to the Scene view's current shot.",
    );
    ui.same_line();
    {
        let _one = ui.begin_disabled(path.len() < 2);
        if ui.button_with_size(format!("{TRASH}  Delete"), [third, 0.]) {
            delete_key(e, n);
        }
    }
    tip(ui, "Delete this key (Delete).");
}

fn settings(ui: &imgui::Ui, e: &mut Editor) {
    let track = e.scene.visibility_track.as_ref().unwrap();
    let samples = camera_path::sample_count(&track.path, track.sample_ms);
    let faces = track.faces.len();
    let camera = e
        .scene
        .actors
        .iter()
        .find(|a| a.id == track.camera)
        .map_or("?".to_string(), |a| a.name.clone());
    let mut sample_ms = track.sample_ms as i32;
    if !ui.collapsing_header("Settings", imgui::TreeNodeFlags::empty()) {
        return;
    }
    wrapped_colored(
        ui,
        MUTED,
        &format!("{CAMERA}  {camera}  \u{b7}  {samples} samples  \u{b7}  {faces} tracked faces"),
    );
    ui.checkbox("Show path in Scene view", &mut e.camera_path.show);
    tip(ui, "Keep drawing the path while this panel is closed.");
    ui.set_next_item_width(120.);
    if ui
        .input_int("Sample interval (ms)", &mut sample_ms)
        .step(10)
        .build()
    {
        let track = e.scene.visibility_track.as_mut().unwrap();
        track.sample_ms = sample_ms.clamp(16, 1000) as u32;
        e.changed();
    }
    tip(
        ui,
        "How often the path is sampled into the camera track. Shorter intervals follow \
         the path more closely; each sample stores its own visible faces.",
    );
    if ui.button("Refresh tracked faces") {
        camera_path::refresh_faces(&mut e.scene);
        e.changed();
    }
    tip(
        ui,
        "Track every face of the scene's meshes again, after adding or removing meshes. \
         Baked visibility keeps the faces that still exist.",
    );
}

fn shortcuts(ui: &imgui::Ui, e: &mut Editor) {
    if !ui.is_window_focused_with_flags(imgui::WindowFocusedFlags::ROOT_AND_CHILD_WINDOWS)
        || ui.io().want_text_input
        || ui.is_any_item_active()
    {
        return;
    }
    let count = e
        .scene
        .visibility_track
        .as_ref()
        .map_or(0, |t| t.path.len());
    if ui.is_key_pressed(imgui::Key::Space) {
        let state = &mut e.camera_path;
        state.playing = !state.playing;
        state.preview |= state.playing;
    }
    if ui.is_key_pressed(imgui::Key::K) {
        add_key(e);
    }
    if let Some(n) = e.camera_path.selected.filter(|&n| n < count) {
        if ui.is_key_pressed(imgui::Key::LeftArrow) && n > 0 {
            select(e, n - 1);
        }
        if ui.is_key_pressed(imgui::Key::RightArrow) && n + 1 < count {
            select(e, n + 1);
        }
        if ui.is_key_pressed(imgui::Key::Delete) {
            delete_key(e, n);
        }
    }
}

/// Draws the path over the Scene view image: the curve, a small camera per
/// key (numbered, the selected one highlighted), and the playhead camera.
/// `to_panel` maps the viewport's 960x600 pixels to panel pixels. Returns
/// whether the mouse is over a key, which then takes the click.
pub fn overlay(
    ui: &imgui::Ui,
    e: &mut Editor,
    clip: [[f32; 2]; 2],
    to_panel: impl Fn([f32; 2]) -> [f32; 2],
) -> bool {
    e.camera_path.hovered = None;
    let Some(track) = &e.scene.visibility_track else {
        return false;
    };
    let state = &e.camera_path;
    if !(state.open || state.show) || track.path.is_empty() || e.playing {
        return false;
    }
    let view = e.view;
    let screen = |p: [f32; 3]| {
        let p = crate::viewport::project(&view, p);
        (p[2] > 0.05).then(|| (to_panel([p[0], p[1]]), p[2]))
    };
    let mouse = ui.io().mouse_pos;
    let inside = mouse[0] >= clip[0][0]
        && mouse[0] <= clip[1][0]
        && mouse[1] >= clip[0][1]
        && mouse[1] <= clip[1][1];
    // Keys under the mouse: the nearest within a few pixels, only while editing.
    let hovered = (state.open && inside && !state.preview && !e.scene_orientation.owns_pointer)
        .then(|| {
            track
                .path
                .iter()
                .enumerate()
                .filter_map(|(n, k)| {
                    let (p, _) = screen(k.position)?;
                    let d = ((p[0] - mouse[0]).powi(2) + (p[1] - mouse[1]).powi(2)).sqrt();
                    (d <= 9.).then_some((n, d))
                })
                .min_by(|a, b| a.1.total_cmp(&b.1))
                .map(|(n, _)| n)
        })
        .flatten();
    let draw = ui.get_window_draw_list();
    draw.with_clip_rect(clip[0], clip[1], || {
        let line = |a: [f32; 3], b: [f32; 3], color: [f32; 4], width: f32| {
            if let (Some((a, _)), Some((b, _))) = (screen(a), screen(b)) {
                draw.add_line(a, b, color).thickness(width).build();
            }
        };
        let duration = camera_path::duration(&track.path);
        let steps = ((duration / 0.05) as usize).clamp(1, 4000);
        let mut previous = camera_path::pose_at(&track.path, 0.).position;
        for n in 1..=steps {
            let time = duration * n as f32 / steps as f32;
            let p = camera_path::pose_at(&track.path, time).position;
            let jump = track.path.iter().any(|k| {
                k.cut && k.time > duration * (n - 1) as f32 / steps as f32 && k.time <= time
            });
            if !jump {
                line(previous, p, [1., 0.75, 0.2, 0.9], 2.);
            }
            previous = p;
        }
        // Cameras keep a steady size on screen: about 4% of the depth.
        let frustum = |pose: camera_path::Pose, color: [f32; 4], scale: f32, width: f32| {
            let Some((_, depth)) = screen(pose.position) else {
                return;
            };
            let size = depth * 0.04 * scale;
            let m = crate::transform::Matrix::trs(pose.position, pose.rotation, [1.; 3]);
            let half = (pose.fov.to_radians() * 0.5).tan() * size;
            let corners = [
                [-half, half * 0.75, size],
                [half, half * 0.75, size],
                [half, -half * 0.75, size],
                [-half, -half * 0.75, size],
            ]
            .map(|p| m.point(p));
            for j in 0..4 {
                line(pose.position, corners[j], color, width);
                line(corners[j], corners[(j + 1) % 4], color, width);
            }
            // A small roof marks the camera's up.
            let roof = m.point([0., half * 1.25, size]);
            line(corners[0], roof, color, width);
            line(roof, corners[1], color, width);
        };
        let many = track.path.len() > 80;
        for (n, key) in track.path.iter().enumerate() {
            let selected = state.selected == Some(n) && state.open;
            let highlighted = selected || hovered == Some(n);
            let color = if highlighted {
                SELECTED
            } else if key.cut {
                CUT
            } else {
                KEY
            };
            if !many || highlighted {
                frustum(
                    key.into(),
                    color,
                    if highlighted { 1.3 } else { 1. },
                    if highlighted { 2. } else { 1.2 },
                );
            }
            if let Some((p, _)) = screen(key.position) {
                let radius = match (highlighted, many) {
                    (true, _) => 5.,
                    (false, true) => 2.,
                    (false, false) => 3.5,
                };
                draw.add_circle(p, radius, color).filled(true).build();
                if !many || highlighted {
                    draw.add_text([p[0] + 6., p[1] - 16.], color, format!("{n}"));
                }
            }
        }
        if state.open && !state.preview {
            let pose = camera_path::pose_at(&track.path, state.time);
            frustum(pose, PLAYHEAD, 1.4, 2.);
        }
    });
    if let Some(n) = hovered {
        let key = track.path[n];
        ui.tooltip_text(format!(
            "Key {n} at {:.2} s{}. Click to select it.",
            key.time,
            if key.cut { " (cut)" } else { "" }
        ));
        e.camera_path.hovered = Some(n);
        if ui.is_mouse_clicked(imgui::MouseButton::Left) {
            select(e, n);
        }
    }
    hovered.is_some()
}

fn status(e: &Editor) -> serde_json::Value {
    let Some(track) = &e.scene.visibility_track else {
        return serde_json::json!({
            "exists": false,
            "cameras": camera_path::cameras(&e.scene).iter().map(|&i| e.scene.actors[i].id).collect::<Vec<_>>(),
            "revision": crate::mcp_tools::revision(&e.scene),
        });
    };
    serde_json::json!({
        "exists": true,
        "camera": track.camera,
        "editable": track.navigation.is_none(),
        "sample_ms": track.sample_ms,
        "keys": track.path,
        "duration": camera_path::duration(&track.path),
        "samples": track.samples.len(),
        "samples_follow_path": !track.path.is_empty() && camera_path::samples_current(track),
        "baked": track.samples.iter().any(|s| !s.visible.is_empty()),
        "faces": track.faces.len(),
        "path_error": (!track.path.is_empty()).then(|| camera_path::path_error(track)),
        "error": camera_path::validate(&track.path, track.sample_ms).err(),
        "revision": crate::mcp_tools::revision(&e.scene),
    })
}

/// The `camera_path` MCP tool (see its description in mcp_tools.rs).
pub fn mcp(e: &mut Editor, a: &serde_json::Value) -> Result<serde_json::Value, String> {
    let op = a["op"].as_str().ok_or("op is required")?;
    let number = |key: &str| {
        a.get(key)
            .and_then(serde_json::Value::as_f64)
            .map(|v| v as f32)
    };
    match op {
        "read" => return Ok(status(e)),
        "select" => {
            let n = a
                .get("key")
                .and_then(serde_json::Value::as_u64)
                .ok_or("key is required")? as usize;
            let count = e
                .scene
                .visibility_track
                .as_ref()
                .map_or(0, |t| t.path.len());
            if n >= count {
                return Err("No such key".into());
            }
            e.camera_path.open = true;
            select(e, n);
            return Ok(status(e));
        }
        "preview" => {
            let track = e
                .scene
                .visibility_track
                .as_ref()
                .filter(|t| !t.path.is_empty())
                .ok_or("The scene has no camera path")?;
            let time = number("time")
                .unwrap_or(0.)
                .clamp(0., camera_path::duration(&track.path));
            let pose = camera_path::pose_at(&track.path, time);
            e.camera_path.time = time;
            look_through(e, pose);
            return Ok(status(e));
        }
        _ => {}
    }
    crate::mcp_tools::editable(e)?;
    crate::mcp_tools::expect_revision(&e.scene, a)?;
    let editable_track = |e: &mut Editor| -> Result<(), String> {
        match &e.scene.visibility_track {
            None => Err("The scene has no visibility track; create one first".into()),
            Some(t) if t.navigation.is_some() => Err(
                "This track's samples come from a route graph; its path cannot be edited".into(),
            ),
            Some(_) => Ok(()),
        }
    };
    match op {
        "create" => {
            if e.scene.visibility_track.is_some() {
                return Err("The scene already has a visibility track".into());
            }
            let cameras = camera_path::cameras(&e.scene);
            let camera = match a.get("camera").and_then(serde_json::Value::as_str) {
                Some(id) => {
                    let id: uuid::Uuid = id.parse().map_err(|_| "camera must be an actor UUID")?;
                    *cameras
                        .iter()
                        .find(|&&i| e.scene.actors[i].id == id)
                        .ok_or("camera is not a root camera actor")?
                }
                None => *cameras
                    .first()
                    .ok_or("The scene has no root camera actor")?,
            };
            camera_path::create(&mut e.scene, camera)?;
        }
        "convert" => {
            editable_track(e)?;
            let track = e.scene.visibility_track.as_ref().unwrap();
            let tolerance = camera_path::Tolerance::for_track(&e.scene, track);
            let track = e.scene.visibility_track.as_mut().unwrap();
            track.path = camera_path::keys_from_samples(
                track,
                number("every").unwrap_or(1.).max(0.05),
                tolerance,
            );
        }
        "set_keys" => {
            editable_track(e)?;
            let keys: Vec<Key> =
                serde_json::from_value(a.get("keys").cloned().ok_or("keys is required")?)
                    .map_err(|error| format!("keys: {error}"))?;
            let track = e.scene.visibility_track.as_mut().unwrap();
            camera_path::validate(&keys, track.sample_ms)?;
            track.path = keys;
        }
        "add_key_from_view" => {
            editable_track(e)?;
            let track = e.scene.visibility_track.as_ref().unwrap();
            let time = number("time").unwrap_or(
                camera_path::duration(&track.path) + if track.path.is_empty() { 0. } else { 2. },
            );
            let mut keys = track.path.clone();
            if keys.iter().any(|k| (k.time - time).abs() < 0.01) {
                return Err("A key already exists at that time".into());
            }
            let at = keys.partition_point(|k| k.time < time);
            keys.insert(at, key_from_view(e, time));
            let track = e.scene.visibility_track.as_mut().unwrap();
            camera_path::validate(&keys, track.sample_ms)?;
            track.path = keys;
        }
        "sample_ms" => {
            editable_track(e)?;
            let ms = a
                .get("sample_ms")
                .and_then(serde_json::Value::as_u64)
                .ok_or("sample_ms is required")?;
            e.scene.visibility_track.as_mut().unwrap().sample_ms = (ms as u32).clamp(16, 1000);
        }
        "update_samples" => {
            editable_track(e)?;
            camera_path::apply(&mut e.scene)?;
        }
        other => return Err(format!("Unknown camera_path op: {other}")),
    }
    e.changed();
    Ok(status(e))
}

#[cfg(test)]
mod tests {
    #[test]
    fn create_add_and_sample_a_path_in_the_open_scene() {
        let (mut scene, _, _) =
            crate::visibility_bake::tests::scene_with(crate::texture::BlendMode::Cutout);
        scene.visibility_track = None;
        let faces = crate::camera_path::trackable_faces(&scene);
        assert!(!faces.is_empty());
        let camera = crate::camera_path::cameras(&scene)[0];
        crate::camera_path::create(&mut scene, camera).unwrap();
        let track = scene.visibility_track.as_mut().unwrap();
        assert_eq!(track.faces, faces);
        let mut key = track.path[0];
        key.time = 2.;
        key.position[2] += 1.;
        track.path.push(key);
        assert!(!crate::camera_path::samples_current(track));
        crate::camera_path::apply(&mut scene).unwrap();
        let track = scene.visibility_track.as_ref().unwrap();
        assert!(crate::camera_path::samples_current(track));
        assert_eq!(track.samples.len(), 21);
        track.validate(&scene).unwrap();
        // The generated track bakes and cooks like an imported one.
        scene.display_size = [512, 240];
        let lists = crate::visibility_bake::bake(&scene, track, &Default::default()).unwrap();
        assert!(lists.iter().all(|l| !l.is_empty()), "{lists:?}");
        let mut baked = scene.clone();
        for (s, l) in baked
            .visibility_track
            .as_mut()
            .unwrap()
            .samples
            .iter_mut()
            .zip(lists)
        {
            s.visible = l;
        }
        assert!(crate::camera_path::samples_current(
            baked.visibility_track.as_ref().unwrap()
        ));
        crate::visibility_track::cook(&baked).unwrap().unwrap();
    }
}
