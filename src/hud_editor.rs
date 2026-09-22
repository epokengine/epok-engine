use crate::{editor::Editor, hud, scene::Actor};
fn heading(ui: &imgui::Ui, label: &str) -> bool {
    crate::gui::heading(ui, label)
}
fn vector(ui: &imgui::Ui, label: &str, value: &mut [f32; 2]) {
    crate::gui::Drag::new(crate::gui::field(ui, label))
        .speed(0.5)
        .display_format("%.1f")
        .build_array(ui, value);
}
/// The point anchor presets: each sets `anchor_min`, `anchor_max` and `pivot` to
/// the same corner. The combo menu and the preview label read this one table, so
/// a preset can never be offered without being recognised again afterwards.
const ANCHOR_PRESETS: [(&str, [f32; 2]); 7] = [
    ("Top Left", [0., 1.]),
    ("Top Center", [0.5, 1.]),
    ("Top Right", [1., 1.]),
    ("Center", [0.5, 0.5]),
    ("Bottom Left", [0., 0.]),
    ("Bottom Center", [0.5, 0.]),
    ("Bottom Right", [1., 0.]),
];
/// The preset a rect currently sits on, for the combo preview. `position` and
/// `size` are deliberately ignored: an element moved away from the origin is
/// still anchored to its preset.
fn anchor_preset(r: &hud::RectTransform) -> &'static str {
    if r.anchor_min == [0.; 2] && r.anchor_max == [1.; 2] && r.pivot == [0.5; 2] {
        return "Stretch";
    }
    ANCHOR_PRESETS
        .iter()
        .find(|(_, p)| r.anchor_min == *p && r.anchor_max == *p && r.pivot == *p)
        .map_or("Custom", |(name, _)| *name)
}
pub fn inspector(ui: &imgui::Ui, e: &mut Actor) {
    if let Some(c) = &mut e.canvas
        && heading(ui, "Canvas")
    {
        crate::gui::toggle(ui, "Enabled##canvas", &mut c.enabled);
        ui.text("Screen Space - Overlay");
        crate::gui::muted(
            ui,
            "Native pixels; canvas follows Project Settings resolution.",
        );
    }
    if let Some(r) = &mut e.rect
        && heading(ui, "Rect Transform")
    {
        if let Some(_combo) = ui.begin_combo(crate::gui::field(ui, "Anchors"), anchor_preset(r)) {
            for (name, p) in ANCHOR_PRESETS {
                if ui.selectable(name) {
                    r.anchor_min = p;
                    r.anchor_max = p;
                    r.pivot = p;
                    r.position = [0.; 2];
                }
            }
            if ui.selectable("Stretch") {
                r.anchor_min = [0.; 2];
                r.anchor_max = [1.; 2];
                r.pivot = [0.5; 2];
                r.position = [0.; 2];
                r.size = [0.; 2];
            }
        }
        vector(ui, "Position", &mut r.position);
        vector(ui, "Size Delta", &mut r.size);
        vector(ui, "Anchor Min", &mut r.anchor_min);
        vector(ui, "Anchor Max", &mut r.anchor_max);
        vector(ui, "Pivot", &mut r.pivot);
        crate::gui::muted(ui, "Pixels; +Y up. Size includes anchor stretch.");
    }
    let mut remove_image = false;
    let mut remove_text = false;
    let mut remove_progress = false;
    let image_open = e.image.is_some()
        && crate::gui::section(ui, "Image", || {
            remove_image = ui.menu_item("Remove Image");
        });
    if image_open && let Some(c) = &mut e.image {
        crate::gui::toggle(ui, "Enabled##image", &mut c.enabled);
        ui.color_edit3(crate::gui::field(ui, "Color##image"), &mut c.color);
        let mut asset = c.texture.map_or_else(String::new, |id| id.to_string());
        if ui
            .input_text("Texture asset UUID##image", &mut asset)
            .build()
        {
            if asset.trim().is_empty() {
                c.texture = None;
            } else if let Ok(id) = uuid::Uuid::parse_str(asset.trim()) {
                c.texture = Some(id);
            }
        }
        let mut region = c.region.map(i32::from);
        if crate::gui::Drag::new(crate::gui::field(ui, "Atlas x/y/w/h##image"))
            .speed(1.)
            .build_array(ui, &mut region)
        {
            c.region = region.map(|v| v.clamp(0, 256) as u16);
        }
        crate::gui::muted(ui, "Atlas pixels; zero width/height uses the full texture.");
        let mut borders = c.borders.map(i32::from);
        if crate::gui::Drag::new(crate::gui::field(ui, "Slice left/top/right/bottom"))
            .speed(1.)
            .build_array(ui, &mut borders)
        {
            c.borders = borders.map(|v| v.clamp(0, 256) as u16);
        }
    }
    let text_open = e.text.is_some()
        && crate::gui::section(ui, "Text", || {
            remove_text = ui.menu_item("Remove Text");
        });
    if text_open && let Some(c) = &mut e.text {
        crate::gui::toggle(ui, "Enabled##text", &mut c.enabled);
        ui.set_next_item_width(-1.);
        ui.input_text_multiline("##hud-text", &mut c.text, [-1., 80.])
            .build();
        crate::gui::toggle(ui, "Wrap text##hud", &mut c.wrap);
        ui.color_edit3(crate::gui::field(ui, "Color##text"), &mut c.color);
        crate::gui::muted(
            ui,
            "8 x 16 bitmap; Spanish glyphs, multiline, 511 UTF-8 bytes.",
        );
    }
    let progress_open = e.progress.is_some()
        && crate::gui::section(ui, "Progress Bar", || {
            remove_progress = ui.menu_item("Remove Progress Bar");
        });
    if progress_open && let Some(c) = &mut e.progress {
        crate::gui::toggle(ui, "Enabled##progress", &mut c.enabled);
        ui.slider(crate::gui::field(ui, "Value"), 0., 1., &mut c.value);
        ui.color_edit3(crate::gui::field(ui, "Fill##progress"), &mut c.color);
        ui.color_edit3(
            crate::gui::field(ui, "Background##progress"),
            &mut c.background,
        );
    }
    if remove_image {
        e.image = None;
    }
    if remove_text {
        e.text = None;
    }
    if remove_progress {
        e.progress = None;
    }
}
/// Which side of one axis a resize handle grabs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Min,
    Max,
}
/// What a drag started on the selection overlay is doing. `Resize` names the
/// side it grabs per axis; `None` on an axis leaves that axis alone, so the four
/// corners and the four edge midpoints are the eight combinations that grab at
/// least one side.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HudHandle {
    Move,
    Resize([Option<Side>; 2]),
}
/// Corners before edges: the two hit boxes overlap on a small rect, and the
/// corner is the grab an author meant.
const HANDLES: [[Option<Side>; 2]; 8] = [
    [Some(Side::Min), Some(Side::Min)],
    [Some(Side::Max), Some(Side::Min)],
    [Some(Side::Min), Some(Side::Max)],
    [Some(Side::Max), Some(Side::Max)],
    [Some(Side::Min), None],
    [Some(Side::Max), None],
    [None, Some(Side::Min)],
    [None, Some(Side::Max)],
];
/// Half the handle hit box, in screen pixels. The viewport is scaled to fit, so
/// a HUD-unit reach would shrink out of the cursor's way as the author zoomed
/// out; everything below divides by `scale` at the point of comparison instead.
const HANDLE_HALF: f32 = 6.;
/// The handle's point on a resolved rect, in HUD units.
fn handle_point(r: hud::Rect, sides: [Option<Side>; 2]) -> [f32; 2] {
    [0usize, 1].map(|i| match sides[i] {
        None => r[i] + r[i + 2] * 0.5,
        Some(Side::Min) => r[i],
        Some(Side::Max) => r[i] + r[i + 2],
    })
}
/// Move the grabbed sides by `delta` HUD units (+Y up) and pin the opposite
/// sides.
///
/// `resolve` gives `extent = parent_extent * (anchor_max - anchor_min) + size`
/// and `min = anchor_point + position - extent * pivot`, so the parent term is
/// constant in both `size` and `position`: the same compensation is exact for a
/// point anchor and for a stretched one, and no parent rect is needed here.
pub fn resize(r: &mut hud::RectTransform, sides: [Option<Side>; 2], delta: [f32; 2]) {
    for i in 0..2 {
        match sides[i] {
            None => {}
            // d(max) = d(position) + d(size) * (1 - pivot); d(min) picks up
            // -d(size) * pivot, which the position term cancels exactly.
            Some(Side::Max) => {
                r.size[i] += delta[i];
                r.position[i] += delta[i] * r.pivot[i];
            }
            Some(Side::Min) => {
                r.size[i] -= delta[i];
                r.position[i] += delta[i] * (1. - r.pivot[i]);
            }
        }
    }
}
pub fn view(ui: &imgui::Ui, e: &mut Editor, texture: imgui::TextureId) {
    let mut step = false;
    if !e.playing && !e.critical_busy() {
        e.hud_simulation.ensure_edit(
            &e.root,
            &e.scene,
            &e.catalog,
            e.assets.revision,
            e.registry_revision,
        );
    }
    if !e.hud_simulation.interactive {
        let _disabled = ui.begin_disabled(e.playing || e.critical_busy());
        if ui.button("Interact") {
            e.hud_drag = None;
            e.hud_simulation
                .start(e.root.clone(), e.scene.clone(), e.catalog.clone());
        }
        if ui.is_item_hovered() {
            ui.tooltip_text(
                "Test menu input and animations. The scene is already visible without this.",
            );
        }
        ui.same_line();
        crate::gui::muted(
            ui,
            if e.hud_simulation.compiling() {
                "Preparing scene preview..."
            } else {
                "Scene preview"
            },
        );
    } else {
        if ui.button("Back to editing") {
            e.hud_simulation.stop();
            e.view_dirty = true;
        }
        ui.same_line();
        if ui.button("Restart") {
            e.hud_simulation
                .start(e.root.clone(), e.scene.clone(), e.catalog.clone());
        }
        ui.same_line();
        let boundary = e
            .hud_simulation
            .session
            .as_ref()
            .and_then(|s| s.frame.as_ref())
            .is_some_and(|f| !f.requested_scene.is_empty());
        let _disabled = ui.begin_disabled(e.hud_simulation.compiling() || boundary);
        if ui.button(if e.hud_simulation.running {
            "Pause"
        } else {
            "Resume"
        }) {
            e.hud_simulation.running = !e.hud_simulation.running;
        }
        ui.same_line();
        {
            let _disabled = ui.begin_disabled(e.hud_simulation.running);
            step = ui.button("Step");
        }
        if e.hud_simulation.compiling() {
            ui.same_line();
            crate::gui::muted(ui, "Compiling native controllers...");
        }
    }
    if let Some(error) = &e.hud_simulation.error {
        ui.text_colored([1., 0.45, 0.35, 1.], "Native preview unavailable");
        ui.child_window("native-preview-error")
            .size([0., 80.])
            .build(|| ui.text_wrapped(error));
        if ui.small_button("Retry preview") {
            e.hud_simulation.stop();
            e.hud_simulation.error = None;
        }
    }
    let dimensions = e
        .hud_simulation
        .session
        .as_ref()
        .map_or(e.scene.display_size, |s| s.scene.display_size);
    let [width, height] = dimensions.map(f32::from);
    let available = ui.content_region_avail();
    let scale = (available[0] / width)
        .min(
            (available[1]
                - if e.hud_simulation.interactive {
                    65.
                } else {
                    24.
                })
            .max(1.)
                / height,
        )
        .max(0.01);
    let origin0 = ui.cursor_screen_pos();
    let origin = [
        origin0[0] + (available[0] - width * scale) * 0.5,
        origin0[1],
    ];
    ui.set_cursor_screen_pos(origin);
    imgui::Image::new(texture, [width * scale, height * scale]).build(ui);
    let hovered = ui.is_item_hovered();
    if e.hud_simulation.interactive {
        if ui.is_mouse_clicked(imgui::MouseButton::Left) {
            e.hud_simulation.focused = hovered;
        }
        let capture =
            e.hud_simulation.focused && ui.is_window_focused() && !ui.io().want_text_input;
        e.hud_simulation.buttons = 0;
        if capture {
            for (key, bit) in [
                (imgui::Key::UpArrow, 4),
                (imgui::Key::RightArrow, 5),
                (imgui::Key::DownArrow, 6),
                (imgui::Key::LeftArrow, 7),
                (imgui::Key::K, 14),
                (imgui::Key::L, 13),
                (imgui::Key::Enter, 3),
            ] {
                if ui.is_key_down(key) {
                    e.hud_simulation.buttons |= 1 << bit;
                }
            }
        }
        if e.hud_simulation.update(ui.io().delta_time, step) {
            e.view_dirty = true;
        }
        ui.set_cursor_screen_pos([origin0[0], origin[1] + height * scale + 4.]);
        if let Some(frame) = e
            .hud_simulation
            .session
            .as_ref()
            .and_then(|s| s.frame.as_ref())
        {
            crate::gui::muted(
                ui,
                format!(
                    "Frame {} | {} actors | {} draws | {} dropped",
                    frame.number,
                    frame.actors,
                    frame.commands.len(),
                    frame.stats[4]
                ),
            );
            if !frame.requested_scene.is_empty() {
                ui.text_wrapped(format!(
                    "Scene change requested: {}. Simulation stopped at the scene boundary.",
                    frame.requested_scene
                ));
            }
        }
        crate::gui::muted(
            ui,
            "Click preview: arrows, K confirm, L back, Enter Start. Audio is silent; memory cards are temporary.",
        );
        return;
    }
    let mouse = ui.io().mouse_pos;
    let p = [
        (mouse[0] - origin[0]) / scale,
        height - (mouse[1] - origin[1]) / scale,
    ];
    let order = hud::order(&e.scene);
    let mut grab = None;
    if let Some(i) = e.selected
        && let Some(r) = hud::layout(&e.scene, i)
        && e.scene.actors[i].rect.is_some()
    {
        let reach = HANDLE_HALF / scale;
        grab = HANDLES.into_iter().find(|sides| {
            let h = handle_point(r, *sides);
            (p[0] - h[0]).abs() <= reach && (p[1] - h[1]).abs() <= reach
        });
    }
    if hovered && ui.is_mouse_clicked(imgui::MouseButton::Left) && !e.playing {
        if grab.is_none() {
            e.selected = order
                .iter()
                .rev()
                .find(|i| {
                    hud::layout(&e.scene, **i).is_some_and(|r| {
                        p[0] >= r[0] && p[0] <= r[0] + r[2] && p[1] >= r[1] && p[1] <= r[1] + r[3]
                    })
                })
                .copied();
        }
        e.hud_drag = e
            .selected
            .filter(|i| e.scene.actors[*i].rect.is_some())
            .map(|i| (i, grab.map_or(HudHandle::Move, HudHandle::Resize)));
        e.reveal_selected = e.selected.is_some();
        e.search.clear();
        e.view_dirty = true;
    }
    if !ui.is_mouse_down(imgui::MouseButton::Left) {
        e.hud_drag = None;
        // One move or resize of a rect is one undo step, not one per frame.
        e.end_coalesced();
    }
    if !e.playing
        && let Some((i, handle)) = e.hud_drag
        && ui.is_mouse_dragging(imgui::MouseButton::Left)
    {
        // HUD space is +Y up and the cursor's is +Y down, so the vertical delta
        // is negated once here and every handle below works in HUD units.
        let raw = ui.io().mouse_delta;
        let delta = [raw[0] / scale, -raw[1] / scale];
        let original = e.scene.actors[i].rect.clone();
        if let Some(r) = &mut e.scene.actors[i].rect {
            match handle {
                HudHandle::Move => {
                    r.position[0] += delta[0];
                    r.position[1] += delta[1];
                }
                HudHandle::Resize(sides) => resize(r, sides, delta),
            }
        }
        if e.scene.validate().is_ok() {
            e.changed_coalesced("hud-rect-drag");
        } else {
            e.scene.actors[i].rect = original;
        }
    }
    let draw = ui.get_window_draw_list();
    if let Some(i) = e.selected
        && let Some(r) = hud::layout(&e.scene, i)
    {
        let a = [
            origin[0] + r[0] * scale,
            origin[1] + (height - r[1] - r[3]) * scale,
        ];
        let b = [a[0] + r[2] * scale, a[1] + r[3] * scale];
        let accent = [1., 0.65, 0.25, 1.];
        // Drawn smaller than it grabs, so the hit box stays forgiving without
        // the markers swallowing a small rect.
        let marker = HANDLE_HALF - 2.;
        let resizable = e.scene.actors[i].rect.is_some();
        draw.with_clip_rect(
            origin,
            [origin[0] + width * scale, origin[1] + height * scale],
            || {
                draw.add_rect(a, b, accent).thickness(2.).build();
                if !resizable {
                    return;
                }
                for sides in HANDLES {
                    let h = handle_point(r, sides);
                    let c = [
                        origin[0] + h[0] * scale,
                        origin[1] + (height - h[1]) * scale,
                    ];
                    draw.add_rect(
                        [c[0] - marker, c[1] - marker],
                        [c[0] + marker, c[1] + marker],
                        accent,
                    )
                    .filled(true)
                    .build();
                }
            },
        );
    }
    ui.set_cursor_screen_pos([origin0[0], origin[1] + height * scale + 4.]);
    crate::gui::muted(
        ui,
        if e.hud_simulation
            .session
            .as_ref()
            .and_then(|s| s.frame.as_ref())
            .is_some_and(|f| f.actors as usize > e.scene.actors.len())
        {
            format!("HUD {width:.0} x {height:.0} | Procedural UI preview")
        } else {
            format!("HUD {width:.0} x {height:.0} | Drag to move; handles to resize")
        },
    );
}
#[cfg(test)]
mod tests {
    use super::*;
    /// A Canvas with one panel under it, so `hud::layout` walks the real parent
    /// chain the viewport walks.
    fn panel(anchor_min: [f32; 2], anchor_max: [f32; 2], pivot: [f32; 2]) -> (Editor, usize) {
        let mut e = Editor::new(std::env::temp_dir().join("epok-hud-handles"));
        e.create_hud("panel");
        let i = e.selected.unwrap();
        let r = e.scene.actors[i].rect.as_mut().unwrap();
        r.anchor_min = anchor_min;
        r.anchor_max = anchor_max;
        r.pivot = pivot;
        r.position = [0.; 2];
        r.size = [200., 120.];
        (e, i)
    }
    #[test]
    fn every_anchor_preset_is_recognised_again_and_anything_else_is_custom() {
        let mut r = hud::RectTransform::default();
        for (name, p) in ANCHOR_PRESETS {
            r.anchor_min = p;
            r.anchor_max = p;
            r.pivot = p;
            assert_eq!(anchor_preset(&r), name);
        }
        r.anchor_min = [0.; 2];
        r.anchor_max = [1.; 2];
        r.pivot = [0.5; 2];
        assert_eq!(anchor_preset(&r), "Stretch");
        r.pivot = [0.5, 0.4];
        assert_eq!(anchor_preset(&r), "Custom");
        assert_eq!(anchor_preset(&hud::RectTransform::default()), "Center");
    }
    #[test]
    fn dragging_a_side_moves_it_one_to_one_and_pins_the_opposite_side() {
        // Point anchors and stretched anchors, at a centred and an off-centre
        // pivot: the resolved rect must respond identically to all four.
        for anchors in [([0.5; 2], [0.5; 2]), ([0.2, 0.1], [0.8, 0.9])] {
            for pivot in [[0.5; 2], [0.25, 0.75]] {
                let (mut e, i) = panel(anchors.0, anchors.1, pivot);
                let label = format!("{anchors:?} pivot {pivot:?}");
                let before = hud::layout(&e.scene, i).unwrap();
                let rect = |e: &Editor| hud::layout(&e.scene, i).unwrap();
                resize(
                    e.scene.actors[i].rect.as_mut().unwrap(),
                    [Some(Side::Max), None],
                    [10., 0.],
                );
                let right = rect(&e);
                assert!((right[2] - before[2] - 10.).abs() < 0.01, "width {label}");
                assert!((right[0] - before[0]).abs() < 0.01, "left pinned {label}");
                resize(
                    e.scene.actors[i].rect.as_mut().unwrap(),
                    [Some(Side::Min), None],
                    [-10., 0.],
                );
                let left = rect(&e);
                assert!((left[2] - right[2] - 10.).abs() < 0.01, "width {label}");
                assert!(
                    (left[0] - right[0] + 10.).abs() < 0.01,
                    "left moved {label}"
                );
                // +Y is up, so the top side is the maximum of the second axis.
                resize(
                    e.scene.actors[i].rect.as_mut().unwrap(),
                    [None, Some(Side::Max)],
                    [0., 8.],
                );
                let top = rect(&e);
                assert!((top[3] - left[3] - 8.).abs() < 0.01, "height {label}");
                assert!((top[1] - left[1]).abs() < 0.01, "bottom pinned {label}");
                // A corner is the two axes at once and nothing else.
                resize(
                    e.scene.actors[i].rect.as_mut().unwrap(),
                    [Some(Side::Max), Some(Side::Min)],
                    [5., -5.],
                );
                let corner = rect(&e);
                assert!((corner[2] - top[2] - 5.).abs() < 0.01, "corner w {label}");
                assert!((corner[3] - top[3] - 5.).abs() < 0.01, "corner h {label}");
                assert!((corner[0] - top[0]).abs() < 0.01, "corner left {label}");
                assert!(
                    (corner[1] - top[1] + 5.).abs() < 0.01,
                    "corner bottom {label}"
                );
            }
        }
    }
    #[test]
    fn the_eight_handles_sit_on_the_sides_and_midpoints_they_name() {
        let r = [10., 20., 100., 40.];
        let points: Vec<[f32; 2]> = HANDLES.iter().map(|s| handle_point(r, *s)).collect();
        assert_eq!(points.len(), 8);
        assert!(points.contains(&[10., 20.]) && points.contains(&[110., 60.]));
        assert!(points.contains(&[60., 20.]) && points.contains(&[10., 40.]));
        let mut sorted = points.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        sorted.dedup();
        assert_eq!(sorted.len(), 8, "handles must not share a point");
    }
}
