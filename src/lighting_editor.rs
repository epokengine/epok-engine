use crate::{editor::Editor, lighting::*, scene::Actor};
use std::time::{Duration, Instant};

const AUTO_BAKE_DELAY: Duration = Duration::from_millis(400);
const LIGHT_PREVIEW_INTERVAL: Duration = Duration::from_millis(33);

/// Observe only accepted scene/source edits, never camera frames. A failed input
/// is retried after a new edit or an explicit Build Lighting, not every tick.
pub struct AutoBake {
    inspect: bool,
    pending: Option<(u64, Instant)>,
    attempted: Option<u64>,
    failed: bool,
    geometry: Option<u64>,
    running_geometry: Option<u64>,
    automatic: bool,
    pending_delay: Duration,
}
impl Default for AutoBake {
    fn default() -> Self {
        Self {
            inspect: true,
            pending: None,
            attempted: None,
            failed: false,
            geometry: None,
            running_geometry: None,
            automatic: false,
            pending_delay: AUTO_BAKE_DELAY,
        }
    }
}
impl AutoBake {
    pub fn invalidate(&mut self) {
        self.inspect = true;
    }
    fn observe(&mut self, key: u64, geometry: u64, current: bool, now: Instant) {
        let light_only = self.geometry == Some(geometry);
        self.geometry = Some(geometry);
        self.inspect = false;
        if current {
            self.pending = None;
            self.attempted = None;
            self.failed = false;
        } else if self.attempted == Some(key) {
            self.pending = None;
        } else if self.attempted != Some(key)
            && self.pending.is_none_or(|(pending, _)| pending != key)
        {
            let since = if light_only {
                self.pending.map_or(now, |(_, since)| since)
            } else {
                now
            };
            self.pending = Some((key, since));
            self.pending_delay = if light_only {
                LIGHT_PREVIEW_INTERVAL
            } else {
                AUTO_BAKE_DELAY
            };
            self.failed = false;
        }
    }
    fn ready(&self, now: Instant) -> bool {
        self.pending
            .is_some_and(|(_, since)| now.duration_since(since) >= self.pending_delay)
    }
}

pub fn can_start_bake(e: &Editor) -> bool {
    !e.playing && !e.critical_busy() && !e.assets.busy && e.bake_job.is_none()
}
pub fn needs_rebuild(e: &Editor) -> bool {
    !e.bake_current && (e.scene.bake.is_some() || e.scene.actors.iter().any(baked))
}
pub fn preview_toggle(ui: &imgui::Ui, e: &mut Editor) {
    if ui.checkbox("Lighting", &mut e.view.lighting) {
        e.view_dirty = true;
    }
    if ui.is_item_hovered() {
        ui.tooltip_text("Preview PSX ambient, realtime lights, baked vertex colors and shadows. Off: show material colors without lighting. Baked lighting updates automatically after edits. This setting only affects Scene view.");
    }
}
pub fn preview_status(ui: &imgui::Ui, e: &Editor) {
    if !e.view.lighting {
        ui.text_disabled("Lighting preview off (Unlit)");
    }
    if e.bake_job.is_some() {
        ui.text_disabled("Building lighting in background...");
    } else if e.auto_bake.failed {
        ui.text_colored(
            [1., 0.5, 0.3, 1.],
            "Lighting update failed. Check Console or use Build Lighting to retry.",
        );
    } else if needs_rebuild(e) {
        let _color = ui.push_style_color(imgui::StyleColor::Text, [1., 0.75, 0.3, 1.]);
        ui.text_wrapped("Lighting changed. Updating automatically...");
        #[cfg(test)]
        crate::gui::record_script_control(ui, "Lighting needs rebuilding");
        if ui.is_item_hovered() {
            ui.tooltip_text(
                "Lighting rebuilds in the background after a short pause in editing. The previous result stays visible until the new bake is ready. Play and builds finish their own lighting preparation first.",
            );
        }
    }
}
pub fn start_bake(e: &mut Editor) {
    start_bake_inner(e, false);
}
fn start_bake_inner(e: &mut Editor, automatic: bool) {
    if !can_start_bake(e) {
        return;
    }
    let scene = e.scene.clone();
    e.auto_bake.pending = None;
    e.auto_bake.attempted = Some(fingerprint(&scene));
    e.auto_bake.running_geometry = Some(geometry_fingerprint(&scene));
    e.auto_bake.automatic = automatic;
    e.auto_bake.failed = false;
    let (tx, rx) = std::sync::mpsc::channel();
    e.bake_job = Some(rx);
    if !automatic {
        e.log("Baking vertex lighting and static shadows on PC...");
    }
    std::thread::spawn(move || {
        let _ = tx.send(bake(&scene));
    });
}
pub fn poll(e: &mut Editor) {
    poll_at(e, Instant::now());
}
fn poll_at(e: &mut Editor, now: Instant) {
    let result = e.bake_job.as_ref().and_then(|rx| match rx.try_recv() {
        Ok(r) => Some(r),
        Err(std::sync::mpsc::TryRecvError::Disconnected) => {
            Some(Err("Lighting worker stopped".into()))
        }
        Err(_) => None,
    });
    if let Some(result) = result {
        e.bake_job = None;
        e.auto_bake.invalidate();
        match result {
            Ok(b)
                if b.fingerprint == fingerprint(&e.scene)
                    || e.auto_bake.running_geometry == Some(geometry_fingerprint(&e.scene)) =>
            {
                let current = b.fingerprint == fingerprint(&e.scene);
                let count: usize = b.colors.iter().map(Vec::len).sum();
                e.scene.bake = Some(b);
                // Derived output must not add an Undo step, invalidate authored
                // sources, or cancel an in-flight build that uses these inputs.
                // A just-completed light-only frame is useful as a provisional
                // preview during a drag. Its original signature remains stale;
                // build/export and the final update still require exact inputs.
                e.bake_current = current;
                e.dirty = true;
                e.view_dirty = true;
                if current && !e.auto_bake.automatic {
                    e.log(format!(
                        "Lighting baked: {count} vertex colors ({} bytes). Save Scene to persist.",
                        count * 3
                    ));
                }
            }
            Ok(_) => {
                if !e.auto_bake.automatic {
                    e.log("Scene changed during bake. Result discarded; the latest inputs will update automatically.");
                }
            }
            Err(error) => {
                e.auto_bake.failed = true;
                e.log(format!("Bake failed: {error}"));
            }
        }
    }
    if e.playing || e.critical_busy() || e.assets.busy || e.job.is_some() {
        return;
    }
    if e.auto_bake.inspect {
        let key = fingerprint(&e.scene);
        e.bake_current = crate::lighting::valid_bake(&e.scene);
        e.auto_bake
            .observe(key, geometry_fingerprint(&e.scene), e.bake_current, now);
    }
    if e.bake_current {
        e.auto_bake.pending = None;
    } else if needs_rebuild(e) && e.auto_bake.ready(now) && can_start_bake(e) {
        start_bake_inner(e, true);
    }
}
pub fn create(e: &mut Editor, kind: LightType, child: bool) {
    if e.playing || e.scene.actors.len() >= 512 {
        return;
    }
    let base = if kind == LightType::Directional {
        "Directional Light"
    } else {
        "Point Light"
    };
    let mut entity = Actor::cube(base.into());
    entity.kind = "Empty".into();
    entity.position = [0., 3., 0.];
    entity.light = Some(Light {
        kind,
        ..Light::default()
    });
    if kind == LightType::Directional {
        entity.rotation = [50., -30., 0.];
    }
    if child {
        entity.parent = e.selected;
        entity.position = [0.; 3];
    }
    let mut n = 1;
    while e.scene.actors.iter().any(|v| v.name == entity.name) {
        entity.name = format!("{base}.{n:03}");
        n += 1;
    }
    e.scene.actors.push(entity);
    if let Err(error) = e.scene.validate() {
        e.scene.actors.pop();
        e.log(error);
        return;
    }
    e.selected = Some(e.scene.actors.len() - 1);
    e.reveal_selected = true;
    e.search.clear();
    e.set_scene_2d(false);
    e.changed();
}
pub fn inspector(ui: &imgui::Ui, entity: &mut Actor) {
    if entity.light.is_some() {
        let mut remove = false;
        let open = crate::gui::section(ui, "Light", || {
            remove = ui.menu_item("Remove Light");
        });
        if open && let Some(light) = &mut entity.light {
            crate::gui::toggle(ui, "Enabled##light", &mut light.enabled);
            let mut kind = usize::from(light.kind == LightType::Point);
            if ui.combo_simple_string(
                crate::gui::field(ui, "Type"),
                &mut kind,
                &["Directional", "Point"],
            ) {
                light.kind = if kind == 0 {
                    LightType::Directional
                } else {
                    LightType::Point
                };
            }
            let mut mode = match light.mode {
                LightMode::Baked => 0,
                LightMode::Realtime => 1,
                LightMode::Mixed => 2,
            };
            if ui.combo_simple_string(
                crate::gui::field(ui, "Mode"),
                &mut mode,
                &["Baked", "Realtime", "Mixed"],
            ) {
                light.mode = [LightMode::Baked, LightMode::Realtime, LightMode::Mixed][mode];
            }
            ui.color_edit3(crate::gui::field(ui, "Light Color"), &mut light.color);
            crate::gui::Drag::new(crate::gui::field(ui, "Intensity"))
                .speed(0.01)
                .range(0., 2.)
                .build(ui, &mut light.intensity);
            if light.kind == LightType::Point {
                crate::gui::Drag::new(crate::gui::field(ui, "Range"))
                    .speed(0.1)
                    .range(0.01, 128.)
                    .build(ui, &mut light.range);
            }
            crate::gui::Drag::new(crate::gui::field(ui, "Priority"))
                .range(-100, 100)
                .build(ui, &mut light.priority);
            if light.mode != LightMode::Realtime {
                crate::gui::toggle(ui, "Bake Shadows", &mut light.shadows);
            }
            ui.text_wrapped(
                "Realtime: 1 directional + 1 local light per object. Shadows are baked only.",
            );
            if light.kind == LightType::Directional {
                crate::gui::muted(ui, "Transform +Z is the light's travel direction.");
            }
        }
        if remove {
            entity.light = None;
        }
        ui.separator();
    }
}
pub fn mesh(ui: &imgui::Ui, entity: &mut Actor) {
    let mut mode = if entity.material.unlit {
        0
    } else if entity.lighting.receive == Receive::Baked {
        1
    } else {
        2
    };
    if ui.combo_simple_string(
        crate::gui::field(ui, "Receive Lighting"),
        &mut mode,
        &["Unlit", "Baked Vertex", "Realtime (GTE)"],
    ) {
        entity.material.unlit = mode == 0;
        entity.lighting.receive = if mode == 1 {
            Receive::Baked
        } else {
            Receive::Realtime
        };
        if mode == 1 {
            entity.lighting.static_geometry = true;
        }
    }
    crate::gui::toggle(ui, "Cast Baked Shadows", &mut entity.lighting.cast_shadows);
    crate::gui::toggle(
        ui,
        "Background Pass (PS1)",
        &mut entity.lighting.background_pass,
    );
    if ui.is_item_hovered() {
        ui.tooltip_text("Draw before all ordinary 3D geometry. Only enable for a base surface such as a ground plane that must stay behind every world object. This is not a general depth-sort fix.");
    }
    let mut n = i32::from(entity.lighting.subdivisions);
    if crate::gui::Drag::new(crate::gui::field(ui, "Subdivisions"))
        .range(1, 8)
        .build(ui, &mut n)
    {
        entity.lighting.subdivisions = n.clamp(1, 8) as u8;
    }
    crate::gui::muted(ui, format!("{} triangles", quad_count(entity) * 2));
}
pub fn window(ui: &imgui::Ui, e: &mut Editor) {
    if !e.lighting_window {
        return;
    }
    let mut opened = true;
    ui.window("Lighting").opened(&mut opened).position([80.,90.],imgui::Condition::FirstUseEver).size([480.,530.],imgui::Condition::FirstUseEver).size_constraints([380.,400.],[1400.,1200.]).build(||{
        let old=e.scene.environment.clone();
        let old_fog=e.scene.fog.clone();
        ui.disabled(e.playing,||{
            ui.color_edit3(crate::gui::field(ui, "Ambient"),&mut e.scene.environment.ambient);
            crate::gui::Drag::new(crate::gui::field(ui, "Baked AO Strength")).speed(0.01).range(0.,1.).build(ui,&mut e.scene.environment.ao_strength);
            crate::gui::Drag::new(crate::gui::field(ui, "AO Distance")).speed(0.05).range(0.01,32.).build(ui,&mut e.scene.environment.ao_distance);
            crate::gui::toggle(ui, "Realtime Point Lights",&mut e.scene.environment.point_lights);
            crate::effects::inspector(ui,&mut e.scene.fog);
        });
        if old!=e.scene.environment{e.changed();}
        if old_fog!=e.scene.fog{e.changed();}
        ui.separator();
        let needs=e.scene.actors.iter().any(baked);
        if needs_rebuild(e) || e.bake_job.is_some() {preview_status(ui,e);}else{ui.text_wrapped(if needs{"Bake up to date"}else{"No baked receivers"});}
        ui.disabled(!can_start_bake(e),||{if ui.button("Build Lighting"){start_bake(e);}});
        ui.text_wrapped("Lighting updates automatically in the background after edits. Save Scene to keep the cache. Play / Build also prepare current lighting.");
        ui.separator();
        let count:usize=e.scene.actors.iter().map(quad_count).sum();
        let vertices:usize=e.scene.actors.iter().filter(|e|baked(e)).map(|e|quad_count(e)*4).sum();
        ui.text(format!("Geometry: {} / 7000 triangles",count*2));
        ui.text(format!("Baked colors: {} bytes",vertices*3));
        ui.text_wrapped("Dynamic budget: 1 directional + 1 point per object; 32 active lights. Static objects must stay fixed, including their parents.");
    });
    e.lighting_window = opened;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> (std::path::PathBuf, Editor, Instant) {
        let root = crate::workspace::tests::temp(name);
        let mut editor = Editor::new(root.clone());
        editor.scene = crate::lighting::tests::shadow_scene();
        editor.changed();
        let now = Instant::now();
        poll_at(&mut editor, now);
        assert!(editor.bake_current && editor.bake_job.is_none());
        (root, editor, now)
    }
    // Wait for the actual worker, then deliver its result at a deterministic
    // editor time. Scheduling assertions need no sleeps or race-prone polling.
    fn finish(editor: &mut Editor, now: Instant) {
        let result = editor
            .bake_job
            .take()
            .unwrap()
            .recv_timeout(Duration::from_secs(10))
            .unwrap();
        let (tx, rx) = std::sync::mpsc::channel();
        tx.send(result).unwrap();
        editor.bake_job = Some(rx);
        poll_at(editor, now);
    }
    fn cleanup(root: std::path::PathBuf, editor: Editor) {
        drop(editor);
        if root.exists() {
            std::fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn auto_bake_streams_light_edits_and_does_not_add_an_undo_step() {
        let (root, mut editor, now) = fixture("auto-lighting-debounce");
        let original = editor.scene.clone();
        editor.scene.actors[4].light.as_mut().unwrap().color = [1., 0., 0.];
        editor.changed();
        poll_at(&mut editor, now);
        poll_at(&mut editor, now + Duration::from_millis(15));
        assert!(editor.bake_job.is_none());
        editor.scene.actors[4].light.as_mut().unwrap().intensity = 1.2;
        editor.changed_coalesced("light-intensity");
        poll_at(&mut editor, now + Duration::from_millis(15));
        poll_at(&mut editor, now + Duration::from_millis(32));
        assert!(
            editor.bake_job.is_none(),
            "preview work is bounded to the update interval"
        );
        poll_at(&mut editor, now + Duration::from_millis(33));
        assert!(editor.bake_job.is_some());
        let expected = bake(&editor.scene).unwrap();
        finish(&mut editor, now + Duration::from_secs(1));
        assert_eq!(editor.scene.bake.as_ref(), Some(&expected));
        assert!(editor.bake_current && editor.dirty && editor.view_dirty);
        editor.undo_scene().unwrap();
        assert_eq!(
            editor.scene.actors[4].light.as_ref().unwrap().intensity,
            original.actors[4].light.as_ref().unwrap().intensity,
            "one undo must revert the authored intensity, not the bake"
        );
        editor.undo_scene().unwrap();
        assert_eq!(editor.scene, original);
        cleanup(root, editor);
    }

    #[test]
    fn auto_bake_discards_a_superseded_worker_and_rebuilds_the_latest_light() {
        let (root, mut editor, now) = fixture("auto-lighting-latest");
        let saved = editor.scene.bake.clone();
        editor.scene.actors[4].light.as_mut().unwrap().color = [1., 0., 0.];
        editor.changed();
        poll_at(&mut editor, now);
        poll_at(&mut editor, now + AUTO_BAKE_DELAY);
        editor.scene.actors[4].light.as_mut().unwrap().color = [0., 0., 1.];
        editor.changed();
        // A compatible red result can preview, but cannot become a current
        // bake for the blue edit or bypass its final rebuild.
        finish(&mut editor, now + Duration::from_millis(500));
        assert_ne!(editor.scene.bake, saved);
        assert_ne!(
            editor.scene.bake.as_ref().unwrap().fingerprint,
            fingerprint(&editor.scene)
        );
        assert!(!editor.bake_current && editor.bake_job.is_none());
        poll_at(&mut editor, now + Duration::from_millis(900));
        assert!(editor.bake_job.is_some());
        let expected = bake(&editor.scene).unwrap();
        finish(&mut editor, now + Duration::from_secs(1));
        assert_eq!(editor.scene.bake.as_ref(), Some(&expected));
        poll_at(&mut editor, now + Duration::from_secs(2));
        assert!(editor.bake_job.is_none());
        cleanup(root, editor);
    }

    #[test]
    fn moving_geometry_never_accepts_an_obsolete_light_preview() {
        let (root, mut editor, now) = fixture("auto-lighting-geometry");
        let saved = editor.scene.bake.clone();
        editor.scene.actors[4].light.as_mut().unwrap().color = [1., 0., 0.];
        editor.changed();
        poll_at(&mut editor, now);
        poll_at(&mut editor, now + LIGHT_PREVIEW_INTERVAL);
        editor.scene.actors[1].position[0] += 1.;
        editor.changed();
        finish(&mut editor, now + Duration::from_millis(100));
        assert_eq!(editor.scene.bake, saved);
        assert!(!editor.bake_current);
        cleanup(root, editor);
    }

    #[test]
    fn auto_bake_stays_idle_for_camera_and_unrelated_edits_and_defers_during_play() {
        let (root, mut editor, now) = fixture("auto-lighting-idle");
        for _ in 0..30 {
            editor.view.yaw += 0.01;
            editor.view_dirty = true;
            poll_at(&mut editor, now + Duration::from_secs(1));
            assert!(!editor.auto_bake.inspect && editor.bake_job.is_none());
        }
        editor.scene.name = "Renamed".into();
        editor.changed();
        poll_at(&mut editor, now + Duration::from_secs(2));
        poll_at(&mut editor, now + Duration::from_secs(3));
        assert!(editor.bake_current && editor.bake_job.is_none());
        editor.scene.environment.ambient = [0.2; 3];
        editor.changed();
        editor.playing = true;
        poll_at(&mut editor, now + Duration::from_secs(4));
        assert!(editor.bake_job.is_none());
        editor.playing = false;
        poll_at(&mut editor, now + Duration::from_secs(5));
        poll_at(&mut editor, now + Duration::from_secs(6));
        assert!(editor.bake_job.is_some());
        finish(&mut editor, now + Duration::from_secs(7));
        cleanup(root, editor);
    }

    #[test]
    fn failed_auto_bake_waits_for_new_inputs_instead_of_retrying_every_frame() {
        let (root, mut editor, now) = fixture("auto-lighting-error");
        editor.scene.actors[4].light.as_mut().unwrap().range = 0.;
        editor.changed();
        poll_at(&mut editor, now);
        poll_at(&mut editor, now + AUTO_BAKE_DELAY);
        finish(&mut editor, now + Duration::from_secs(1));
        assert!(editor.auto_bake.failed);
        assert_eq!(
            editor
                .logs
                .iter()
                .filter(|line| line.starts_with("Bake failed:"))
                .count(),
            1
        );
        for second in 2..6 {
            poll_at(&mut editor, now + Duration::from_secs(second));
            assert!(editor.bake_job.is_none());
        }
        assert_eq!(
            editor
                .logs
                .iter()
                .filter(|line| line.starts_with("Bake failed:"))
                .count(),
            1
        );
        editor.scene.actors[4].light.as_mut().unwrap().range = 8.;
        editor.scene.actors[4].light.as_mut().unwrap().color = [0., 1., 0.];
        editor.changed();
        poll_at(&mut editor, now + Duration::from_secs(6));
        poll_at(&mut editor, now + Duration::from_secs(7));
        finish(&mut editor, now + Duration::from_secs(8));
        assert!(editor.bake_current && !editor.auto_bake.failed);
        cleanup(root, editor);
    }

    #[test]
    fn automatic_bakes_are_quiet_and_explicit_builds_report_progress() {
        let (root, mut editor, now) = fixture("auto-lighting-quiet");
        let before = editor.logs.clone();
        for step in 1..=4 {
            editor.scene.actors[4].light.as_mut().unwrap().intensity = 1. + step as f32 * 0.1;
            editor.changed();
            let at = now + Duration::from_secs(step * 2);
            poll_at(&mut editor, at);
            poll_at(&mut editor, at + LIGHT_PREVIEW_INTERVAL);
            assert!(editor.bake_job.is_some());
            finish(&mut editor, at + Duration::from_secs(1));
            assert!(editor.bake_current);
        }
        assert_eq!(editor.logs, before, "live previews must not flood Console");
        start_bake(&mut editor);
        assert!(
            editor
                .logs
                .last()
                .unwrap()
                .starts_with("Baking vertex lighting")
        );
        finish(&mut editor, now + Duration::from_secs(10));
        assert!(editor.logs.last().unwrap().starts_with("Lighting baked:"));
        cleanup(root, editor);
    }
}
