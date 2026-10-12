//! Build > Camera Path: author a visibility track's camera path in the
//! editor (see camera_path.rs). Keys are taken from the Scene view; the path
//! is drawn in the Scene view and can be previewed through the view.
use crate::camera_path::{self, Key};
use crate::editor::Editor;

pub struct State {
    pub open: bool,
    /// Draw the path in the Scene view while the panel is closed too.
    pub show: bool,
    pub time: f32,
    pub playing: bool,
    /// Look through the path at `time` in the Scene view.
    pub preview: bool,
    camera: usize,
}
impl Default for State {
    fn default() -> Self {
        Self {
            open: false,
            show: true,
            time: 0.,
            playing: false,
            preview: false,
            camera: 0,
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

pub fn window(ui: &imgui::Ui, e: &mut Editor) {
    if !e.camera_path.open {
        return;
    }
    let mut opened = true;
    ui.window("Camera Path")
        .opened(&mut opened)
        .position([140., 120.], imgui::Condition::FirstUseEver)
        .size([520., 520.], imgui::Condition::FirstUseEver)
        .build(|| contents(ui, e));
    if !opened {
        e.camera_path.open = false;
        e.camera_path.playing = false;
    }
}

fn contents(ui: &imgui::Ui, e: &mut Editor) {
    let _disabled = ui.begin_disabled(e.playing || crate::visibility_bake_ui::busy(e));
    let Some(track) = &e.scene.visibility_track else {
        ui.text_wrapped(
            "A camera path is the route a guided camera follows. Epok samples it into a \
             visibility track, then Build > Bake Visibility lists what the camera sees \
             along it.",
        );
        let cameras = camera_path::cameras(&e.scene);
        if cameras.is_empty() {
            ui.text_wrapped("Add a camera actor at the scene root first.");
            return;
        }
        let state = &mut e.camera_path;
        let current = cameras.get(state.camera).copied().unwrap_or(cameras[0]);
        if let Some(_combo) = ui.begin_combo("Camera", &e.scene.actors[current].name) {
            for (n, &i) in cameras.iter().enumerate() {
                if ui
                    .selectable_config(&e.scene.actors[i].name)
                    .selected(i == current)
                    .build()
                {
                    state.camera = n;
                }
            }
        }
        if ui.button("Create camera path") {
            match camera_path::create(&mut e.scene, current) {
                Ok(()) => {
                    e.changed();
                    e.log("Camera path created with one key at the camera. Add keys from the Scene view.");
                }
                Err(error) => e.log(format!("Create camera path failed: {error}")),
            }
        }
        return;
    };
    if track.navigation.is_some() {
        ui.text_wrapped(
            "This track's samples come from a route graph (player-driven rails). \
             Its path is not editable here.",
        );
        return;
    }
    if track.path.is_empty() {
        ui.text_wrapped(
            "This track's samples were imported. Convert them to editable keys \
             (one per second, one at every cut, and more wherever the camera \
             turns sharply) to edit the path.",
        );
        if ui.button("Convert samples to keys") {
            let keys = camera_path::keys_from_samples(
                track,
                1.,
                camera_path::Tolerance::for_track(&e.scene, track),
            );
            e.scene.visibility_track.as_mut().unwrap().path = keys;
            e.changed();
        }
        return;
    }

    let samples = camera_path::sample_count(&track.path, track.sample_ms);
    let duration = camera_path::duration(&track.path);
    let current = camera_path::samples_current(track);
    let baked = track.samples.iter().any(|s| !s.visible.is_empty());
    let faces = track.faces.len();
    let error = (!current).then(|| camera_path::path_error(track));
    let valid = camera_path::validate(&track.path, track.sample_ms);
    let mut path = track.path.clone();
    let mut sample_ms = track.sample_ms;

    // Preview.
    let state = &mut e.camera_path;
    if state.playing {
        state.time += ui.io().delta_time;
        if state.time > duration {
            state.time = 0.;
        }
    }
    state.time = state.time.clamp(0., duration);
    let mut scrubbed = crate::gui::Drag::new("Time (s)")
        .speed(0.02)
        .range(0., duration)
        .build(ui, &mut state.time);
    ui.same_line();
    if ui.button(if state.playing { "Pause" } else { "Play" }) {
        state.playing = !state.playing;
        state.preview |= state.playing;
    }
    scrubbed |= ui.checkbox("Look through the path", &mut state.preview);
    ui.same_line();
    ui.checkbox("Show path in Scene view", &mut state.show);
    if state.preview && (scrubbed || state.playing) {
        let pose = camera_path::pose_at(&path, state.time);
        look_through(e, pose);
    }

    // Keys.
    ui.separator();
    let (mut edited, mut gesture, mut released) = (false, false, false);
    let mut remove = None;
    let mut go = None;
    if let Some(_table) = ui.begin_table_with_flags(
        "keys",
        5,
        imgui::TableFlags::BORDERS_INNER_H
            | imgui::TableFlags::SCROLL_Y
            | imgui::TableFlags::SIZING_STRETCH_PROP,
    ) {
        for (n, key) in path.iter_mut().enumerate() {
            let _id = ui.push_id_usize(n);
            ui.table_next_row();
            ui.table_next_column();
            ui.text(format!("{n}"));
            ui.table_next_column();
            ui.set_next_item_width(-1.);
            let _first = ui.begin_disabled(n == 0);
            gesture |= crate::gui::Drag::new("##time")
                .speed(0.02)
                .range(0., 3600.)
                .display_format("%.2f s")
                .build(ui, &mut key.time);
            released |= ui.is_item_deactivated_after_edit();
            drop(_first);
            ui.table_next_column();
            ui.set_next_item_width(-1.);
            gesture |= crate::gui::Drag::new("##fov")
                .speed(0.2)
                .range(25., 120.)
                .display_format("FOV %.0f")
                .build(ui, &mut key.fov);
            released |= ui.is_item_deactivated_after_edit();
            ui.table_next_column();
            if n > 0 {
                edited |= ui.checkbox("Cut", &mut key.cut);
                if ui.is_item_hovered() {
                    ui.tooltip_text("The camera jumps to this key instead of travelling to it.");
                }
            }
            ui.table_next_column();
            if ui.small_button("View") {
                go = Some(n);
            }
            if ui.is_item_hovered() {
                ui.tooltip_text("Look through this key in the Scene view.");
            }
            ui.same_line();
            if ui.small_button("Set") {
                *key = Key {
                    cut: key.cut,
                    ..key_from_view(e, key.time)
                };
                edited = true;
            }
            if ui.is_item_hovered() {
                ui.tooltip_text("Set this key's pose to the Scene view's camera.");
            }
            ui.same_line();
            if ui.small_button("x") {
                remove = Some(n);
            }
        }
    }
    if let Some(n) = go {
        look_through(e, (&path[n]).into());
        e.camera_path.preview = false;
        e.camera_path.time = path[n].time;
    }
    if let Some(n) = remove.filter(|_| path.len() > 1) {
        path.remove(n);
        if n == 0 {
            path[0].time = 0.;
            path[0].cut = false;
        }
        edited = true;
    }
    if ui.button("Add key from view") {
        let time = duration + 2.;
        path.push(key_from_view(e, time));
        edited = true;
    }
    if ui.is_item_hovered() {
        ui.tooltip_text("Append a key, 2 s after the last one, at the Scene view's camera.");
    }
    ui.same_line();
    let time = e.camera_path.time;
    let insert_ok = time > 0. && path.iter().all(|k| (k.time - time).abs() > 0.01);
    if ui.button("Insert key at time") && insert_ok {
        let at = path.partition_point(|k| k.time < time);
        path.insert(at, key_from_view(e, time));
        edited = true;
    }
    if ui.is_item_hovered() {
        ui.tooltip_text("Insert a key at the preview time, at the Scene view's camera.");
    }
    ui.set_next_item_width(160.);
    let mut ms = sample_ms as i32;
    if ui.input_int("Sample interval (ms)", &mut ms).build() {
        sample_ms = ms.clamp(16, 1000) as u32;
        edited = true;
    }
    if ui.is_item_hovered() {
        ui.tooltip_text("How often the path is sampled. Shorter intervals follow the path more closely; each sample stores its own visible faces.");
    }

    if edited || gesture {
        let track = e.scene.visibility_track.as_mut().unwrap();
        track.path = path;
        track.sample_ms = sample_ms;
        if gesture && !edited {
            e.changed_coalesced("camera-path");
        } else {
            e.changed();
        }
    }
    if released {
        e.end_coalesced();
    }

    // Samples.
    ui.separator();
    ui.text(format!(
        "{duration:.1} s, {samples} samples, {faces} tracked faces"
    ));
    match &valid {
        Err(error) => ui.text_colored([1., 0.45, 0.35, 1.], error),
        Ok(()) if !current => {
            ui.text_colored(
                [1., 0.8, 0.3, 1.],
                "The track's samples do not follow the path yet.",
            );
            if let Some((position, angle)) = error {
                ui.text_disabled(format!(
                    "Path and samples differ by up to {position:.2} units, {angle:.2} degrees."
                ));
            }
            if ui.button("Update samples") {
                match camera_path::apply(&mut e.scene) {
                    Ok(()) => {
                        e.changed();
                        e.log("Camera path sampled. Bake Visibility next (Build > Bake Visibility...).");
                    }
                    Err(error) => e.log(format!("Update samples failed: {error}")),
                }
            }
            if ui.is_item_hovered() {
                ui.tooltip_text("Replaces the track's samples. Baked lists and the prepared packet order are cleared; bake visibility again.");
            }
        }
        Ok(()) if !baked => {
            ui.text_colored(
                [1., 0.8, 0.3, 1.],
                "Samples follow the path; visibility is not baked.",
            );
            if ui.button("Bake Visibility...") {
                e.visibility_bake.open = true;
            }
        }
        Ok(()) => ui.text("Samples follow the path and visibility is baked."),
    }
    if ui.button("Refresh tracked faces") {
        crate::camera_path::refresh_faces(&mut e.scene);
        e.changed();
    }
    if ui.is_item_hovered() {
        ui.tooltip_text("Track every face of the scene's ordinary meshes, after adding or removing meshes. Baked lists keep the faces that still exist.");
    }
}

/// Draws the path over the Scene view image: the curve, a small frustum per
/// key, and the preview pose. `screen` maps a world point to panel pixels.
pub fn overlay(
    ui: &imgui::Ui,
    e: &Editor,
    clip: [[f32; 2]; 2],
    screen: impl Fn([f32; 3]) -> Option<[f32; 2]>,
) {
    let state = &e.camera_path;
    let Some(track) = &e.scene.visibility_track else {
        return;
    };
    if !(state.open || state.show) || track.path.is_empty() || e.playing {
        return;
    }
    let draw = ui.get_window_draw_list();
    draw.with_clip_rect(clip[0], clip[1], || {
        let line = |a: [f32; 3], b: [f32; 3], color: [f32; 4], width: f32| {
            if let (Some(a), Some(b)) = (screen(a), screen(b)) {
                draw.add_line(a, b, color).thickness(width).build();
            }
        };
        let duration = camera_path::duration(&track.path);
        let steps = ((duration / 0.05) as usize).clamp(1, 2000);
        let mut previous = camera_path::pose_at(&track.path, 0.).position;
        for n in 1..=steps {
            let time = duration * n as f32 / steps as f32;
            let p = camera_path::pose_at(&track.path, time).position;
            let jump = track.path.iter().any(|k| {
                k.cut && k.time > duration * (n - 1) as f32 / steps as f32 && k.time <= time
            });
            if !jump {
                line(previous, p, [1., 0.75, 0.2, 1.], 2.);
            }
            previous = p;
        }
        let frustum = |pose: camera_path::Pose, color: [f32; 4], size: f32| {
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
                line(pose.position, corners[j], color, 1.5);
                line(corners[j], corners[(j + 1) % 4], color, 1.5);
            }
            // Up marker.
            line(corners[0], m.point([0., half * 1.1, size]), color, 1.5);
            line(m.point([0., half * 1.1, size]), corners[1], color, 1.5);
        };
        for key in &track.path {
            frustum(key.into(), [0.35, 0.75, 1., 1.], 0.8);
        }
        if state.open && !state.preview {
            frustum(
                camera_path::pose_at(&track.path, state.time),
                [1., 0.4, 0.3, 1.],
                1.2,
            );
        }
    });
}

fn status(e: &Editor) -> serde_json::Value {
    let Some(track) = &e.scene.visibility_track else {
        return serde_json::json!({"exists": false, "cameras": camera_path::cameras(&e.scene).iter().map(|&i| e.scene.actors[i].id).collect::<Vec<_>>()});
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
