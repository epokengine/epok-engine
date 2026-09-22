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
/// One entry per element a focus link can point at: the actor index the runtime
/// table stores, and the name the author recognises.
pub type FocusTargets<'a> = &'a [(usize, String)];
/// A combo over `targets` plus "None", editing an actor index in place.
fn actor_picker(ui: &imgui::Ui, label: &str, targets: FocusTargets, value: &mut i32) {
    let preview = targets
        .iter()
        .find(|(index, _)| *index as i32 == *value)
        .map_or("None", |(_, name)| name.as_str());
    if let Some(_combo) = ui.begin_combo(crate::gui::field(ui, label), preview) {
        if ui.selectable("None") {
            *value = -1;
        }
        for (index, name) in targets {
            if ui.selectable(format!("{name}##{label}-{index}")) {
                *value = *index as i32;
            }
        }
    }
}
/// Every Font asset in the project, by id and display name, for the Text font
/// combo. The template editor has no asset index and passes an empty slice; a
/// font already chosen there still shows as its UUID.
pub fn font_choices(index: &crate::assets::Index) -> Vec<(uuid::Uuid, String)> {
    let mut out: Vec<_> = index
        .assets
        .iter()
        .filter_map(|(id, records)| match records.as_slice() {
            [record] if record.meta.kind == crate::assets::Kind::Font => {
                Some((*id, record.meta.source.clone()))
            }
            _ => None,
        })
        .collect();
    out.sort_by(|a, b| a.1.cmp(&b.1));
    out
}
pub fn inspector(
    ui: &imgui::Ui,
    e: &mut Actor,
    targets: FocusTargets,
    fonts: &[(uuid::Uuid, String)],
) {
    if let Some(c) = &mut e.canvas
        && heading(ui, "Canvas")
    {
        crate::gui::toggle(ui, "Enabled##canvas", &mut c.enabled);
        ui.text("Screen Space - Overlay");
        actor_picker(ui, "Initial focus", targets, &mut c.focused);
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
        crate::gui::Drag::new(crate::gui::field(ui, "Rotation"))
            .speed(1.)
            .display_format("%.1f")
            .build(ui, &mut r.rotation);
        crate::gui::muted(
            ui,
            "Pixels; +Y up. Size includes anchor stretch. Rotation is degrees about the pivot.",
        );
    }
    let mut remove_image = false;
    let mut remove_text = false;
    let mut remove_progress = false;
    let mut remove_element = false;
    let mut remove_container = false;
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
        if crate::gui::Drag::new(crate::gui::field(ui, "Nine-slice borders (px)"))
            .speed(1.)
            .build_array(ui, &mut borders)
        {
            c.borders = borders.map(|v| v.clamp(0, 256) as u16);
        }
        crate::gui::muted(
            ui,
            "Left, top, right, bottom; zero stretches the whole region.",
        );
        if let Some(_combo) = ui.begin_combo(crate::gui::field(ui, "Tiling"), c.tiling.label()) {
            for tiling in hud::ImageTiling::ALL {
                if ui.selectable(tiling.label()) {
                    c.tiling = tiling;
                }
            }
        }
        crate::gui::muted(
            ui,
            "Tile repeats at texel size and clips; Tile Fit scales to a whole count. With borders only the centre tiles.",
        );
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
        let label = match c.font {
            None => "Built-in".to_string(),
            Some(id) => fonts
                .iter()
                .find(|(v, _)| *v == id)
                .map_or_else(|| id.to_string(), |(_, name)| name.clone()),
        };
        if let Some(_combo) = ui.begin_combo(crate::gui::field(ui, "Font"), label) {
            if ui.selectable("Built-in") {
                c.font = None;
            }
            for (id, name) in fonts {
                if ui.selectable(name) {
                    c.font = Some(*id);
                }
            }
        }
        if let Some(_combo) = ui.begin_combo(crate::gui::field(ui, "Align"), c.align.label()) {
            for align in hud::TextAlign::ALL {
                if ui.selectable(align.label()) {
                    c.align = align;
                }
            }
        }
        crate::gui::muted(
            ui,
            "Built-in is 8 x 16 with Spanish glyphs; an imported font brings its own advances, line height and VRAM cost. Multiline, 511 UTF-8 bytes.",
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
    let element_open = e.layout_element.is_some()
        && crate::gui::section(ui, "Layout Element", || {
            remove_element = ui.menu_item("Remove Layout Element");
        });
    if element_open && let Some(c) = &mut e.layout_element {
        crate::gui::toggle(ui, "Enabled##layout-element", &mut c.enabled);
        for (label, bits) in [
            ("Horizontal", &mut c.horizontal),
            ("Vertical", &mut c.vertical),
        ] {
            ui.text(label);
            for (index, (name, bit)) in SIZE_FLAGS.into_iter().enumerate() {
                if index > 0 {
                    ui.same_line();
                }
                let mut on = *bits & bit != 0;
                if ui.checkbox(format!("{name}##{label}"), &mut on) {
                    *bits = if on { *bits | bit } else { *bits & !bit };
                }
            }
        }
        vector(ui, "Minimum", &mut c.minimum);
        crate::gui::Drag::new(crate::gui::field(ui, "Stretch"))
            .speed(0.1)
            .display_format("%.2f")
            .build(ui, &mut c.stretch);
        crate::gui::muted(
            ui,
            "Read by the parent container: Expand claims leftover space by stretch.",
        );
    }
    let mut remove_focusable = false;
    let container_open = e.layout_container.is_some()
        && crate::gui::section(ui, "Layout Container", || {
            remove_container = ui.menu_item("Remove Layout Container");
        });
    if container_open && let Some(c) = &mut e.layout_container {
        crate::gui::toggle(ui, "Enabled##layout-container", &mut c.enabled);
        if let Some(_combo) = ui.begin_combo(crate::gui::field(ui, "Kind"), c.kind.label()) {
            for kind in hud::LayoutKind::ALL {
                if ui.selectable(kind.label()) {
                    c.kind = kind;
                }
            }
        }
        vector(ui, "Spacing", &mut c.spacing);
        let mut padding = c.padding;
        if crate::gui::Drag::new(crate::gui::field(ui, "Pad left/top/right/bottom"))
            .speed(0.5)
            .display_format("%.1f")
            .build_array(ui, &mut padding)
        {
            c.padding = padding;
        }
        if c.kind == hud::LayoutKind::Grid {
            let mut columns = i32::from(c.columns);
            if crate::gui::Drag::new(crate::gui::field(ui, "Columns"))
                .speed(1.)
                .build(ui, &mut columns)
            {
                c.columns = columns.clamp(1, 255) as u8;
            }
        }
        crate::gui::muted(
            ui,
            "Children ignore their own anchors; this rect measures and places them.",
        );
    }
    let focus_open = e.focusable.is_some()
        && crate::gui::section(ui, "Focusable", || {
            remove_focusable = ui.menu_item("Remove Focusable");
        });
    if focus_open && let Some(c) = &mut e.focusable {
        crate::gui::toggle(ui, "Enabled##focusable", &mut c.enabled);
        for (index, label) in NEIGHBOURS.into_iter().enumerate() {
            actor_picker(ui, label, targets, &mut c.neighbors[index]);
        }
        let mut order = i32::from(c.order);
        if crate::gui::Drag::new(crate::gui::field(ui, "Order"))
            .speed(1.)
            .build(ui, &mut order)
        {
            c.order = order.clamp(0, 255) as u8;
        }
        ui.color_edit3(crate::gui::field(ui, "Highlight##focus"), &mut c.highlight);
        crate::gui::muted(
            ui,
            "The D-pad follows these links; the highlight multiplies this element's colours while it holds focus.",
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
    if remove_element {
        e.layout_element = None;
    }
    if remove_container {
        e.layout_container = None;
    }
    if remove_focusable {
        e.focusable = None;
    }
}
/// The four neighbour links, in the order `Focusable::neighbors` stores them.
const NEIGHBOURS: [&str; 4] = ["Left", "Right", "Up", "Down"];
/// The per-axis size flags a Layout Element writes, in bit order.
const SIZE_FLAGS: [(&str, u8); 4] = [
    ("Fill", 1),
    ("Expand", 2),
    ("Shrink Center", 4),
    ("Shrink End", 8),
];
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
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HudHandle {
    Move,
    Resize([Option<Side>; 2]),
    /// The ring above the top edge. The payload is the angle the cursor held
    /// when the drag began, minus the element's rotation then, so the element
    /// follows the cursor without snapping to it.
    Rotate(f32),
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
/// How far above the top-centre handle the rotation ring sits, in screen pixels.
const ROTATE_REACH: f32 = 22.;
/// A point inside a placement, in HUD units, from its normalized position in the
/// element's own frame: 0,0 is the bottom-left corner and 1,1 the top-right.
/// The corners already carry the rotation, and an affine map is exactly bilinear
/// over them, so this answers for a rotated element as well as a square one.
fn interpolate(p: &hud::Placement, u: f32, v: f32) -> [f32; 2] {
    // corners are top-left, top-right, bottom-left, bottom-right.
    let [a, b, c, d] = p.corners;
    [0usize, 1].map(|i| {
        (1. - u) * (1. - v) * c[i] + u * (1. - v) * d[i] + (1. - u) * v * a[i] + u * v * b[i]
    })
}
/// The handle's point on a placement, in HUD units.
fn handle_point(p: &hud::Placement, sides: [Option<Side>; 2]) -> [f32; 2] {
    let axis = |i: usize| match sides[i] {
        None => 0.5,
        Some(Side::Min) => 0.,
        Some(Side::Max) => 1.,
    };
    interpolate(p, axis(0), axis(1))
}
/// The element's accumulated rotation in degrees, read back off the corners its
/// own transform produced. Zero for everything the runtime did not turn.
fn placement_angle(p: &hud::Placement) -> f32 {
    let [a, b, ..] = p.corners;
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    if dx.abs() < 1e-4 && dy.abs() < 1e-4 {
        0.
    } else {
        dy.atan2(dx).to_degrees()
    }
}
/// A cursor delta in HUD units turned back into the frame `degrees` rotated out of.
fn unrotate(delta: [f32; 2], degrees: f32) -> [f32; 2] {
    if degrees == 0. {
        return delta;
    }
    let (sin, cos) = (-degrees).to_radians().sin_cos();
    [
        delta[0] * cos - delta[1] * sin,
        delta[0] * sin + delta[1] * cos,
    ]
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
/// The rotation ring, in HUD units: above the top-centre handle, along the
/// element's own up direction so it keeps its place once the element turns.
fn rotation_handle(p: &hud::Placement, scale: f32) -> [f32; 2] {
    let top = handle_point(p, [None, Some(Side::Max)]);
    let bottom = handle_point(p, [None, Some(Side::Min)]);
    let (dx, dy) = (top[0] - bottom[0], top[1] - bottom[1]);
    let length = (dx * dx + dy * dy).sqrt();
    let up = if length < 1e-4 {
        [0., 1.]
    } else {
        [dx / length, dy / length]
    };
    let reach = ROTATE_REACH / scale.max(0.01);
    [top[0] + up[0] * reach, top[1] + up[1] * reach]
}
/// The element's pivot in HUD units. Rotation turns about it, so a drag of the
/// ring measures its angle from here.
fn pivot_point(actor: &Actor, p: &hud::Placement) -> [f32; 2] {
    let pivot = actor.rect.as_ref().map_or([0.5; 2], |r| r.pivot);
    interpolate(p, pivot[0], pivot[1])
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
    // A container makes a rect depend on its siblings, so the whole scene is laid
    // out once per frame and every hit test and gizmo indexes that one result.
    let mut boxes = hud::layouts(&e.scene);
    let mut grab = None;
    let mut spin = None;
    if let Some(i) = e.selected
        && let Some(place) = boxes.get(i).copied().flatten()
        && e.scene.actors[i].rect.is_some()
    {
        let reach = HANDLE_HALF / scale;
        grab = HANDLES.into_iter().find(|sides| {
            let h = handle_point(&place, *sides);
            (p[0] - h[0]).abs() <= reach && (p[1] - h[1]).abs() <= reach
        });
        let ring = rotation_handle(&place, scale);
        if grab.is_none() && (p[0] - ring[0]).abs() <= reach && (p[1] - ring[1]).abs() <= reach {
            let pivot = pivot_point(&e.scene.actors[i], &place);
            let held = (p[1] - pivot[1]).atan2(p[0] - pivot[0]).to_degrees();
            spin = Some(held - e.scene.actors[i].rect.as_ref().map_or(0., |r| r.rotation));
        }
    }
    if hovered && ui.is_mouse_clicked(imgui::MouseButton::Left) && !e.playing {
        if grab.is_none() && spin.is_none() {
            e.selected = order
                .iter()
                .rev()
                .find(|i| {
                    boxes.get(**i).copied().flatten().is_some_and(|place| {
                        let r = place.rect;
                        // The hit test stays on the axis-aligned rect: a rotated
                        // element is still picked by the box its layout decided.
                        p[0] >= r[0] && p[0] <= r[0] + r[2] && p[1] >= r[1] && p[1] <= r[1] + r[3]
                    })
                })
                .copied();
        }
        e.hud_drag = e
            .selected
            .filter(|i| e.scene.actors[*i].rect.is_some())
            .map(|i| {
                (
                    i,
                    match (grab, spin) {
                        (Some(sides), _) => HudHandle::Resize(sides),
                        (None, Some(offset)) => HudHandle::Rotate(offset),
                        _ => HudHandle::Move,
                    },
                )
            });
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
        // Position lives in the parent's layout frame and size in this element's
        // own, so each drag is taken back through the rotation that produced the
        // pixels it was measured against.
        let place = boxes.get(i).copied().flatten();
        let angle = place.as_ref().map_or(0., placement_angle);
        let parent_angle = e
            .scene
            .spatial_parent(i)
            .and_then(|parent| boxes.get(parent).copied().flatten())
            .as_ref()
            .map_or(0., placement_angle);
        let pivot = place
            .as_ref()
            .map(|place| pivot_point(&e.scene.actors[i], place));
        let shift = ui.io().key_shift;
        if let Some(r) = &mut e.scene.actors[i].rect {
            match handle {
                HudHandle::Move => {
                    let local = unrotate(delta, parent_angle);
                    r.position[0] += local[0];
                    r.position[1] += local[1];
                }
                HudHandle::Resize(sides) => resize(r, sides, unrotate(delta, angle)),
                HudHandle::Rotate(offset) => {
                    if let Some(pivot) = pivot {
                        let held = (p[1] - pivot[1]).atan2(p[0] - pivot[0]).to_degrees();
                        let value = held - offset;
                        r.rotation = if shift {
                            (value / 15.).round() * 15.
                        } else {
                            value
                        };
                    }
                }
            }
        }
        if e.scene.validate().is_ok() {
            e.changed_coalesced("hud-rect-drag");
        } else {
            e.scene.actors[i].rect = original;
        }
        boxes = hud::layouts(&e.scene);
    }
    let draw = ui.get_window_draw_list();
    if let Some(i) = e.selected
        && let Some(place) = boxes.get(i).copied().flatten()
    {
        let screen = |q: [f32; 2]| {
            [
                origin[0] + q[0] * scale,
                origin[1] + (height - q[1]) * scale,
            ]
        };
        let accent = [1., 0.65, 0.25, 1.];
        let link = [0.45, 0.8, 1., 0.9];
        // Drawn smaller than it grabs, so the hit box stays forgiving without
        // the markers swallowing a small rect.
        let marker = HANDLE_HALF - 2.;
        let resizable = e.scene.actors[i].rect.is_some();
        // Where each neighbour link lands, so a focus table reads as a graph
        // rather than four numbers in the inspector.
        let links: Vec<[f32; 2]> = e.scene.actors[i]
            .focusable
            .as_ref()
            .map(|c| {
                c.neighbors
                    .iter()
                    .filter_map(|n| usize::try_from(*n).ok())
                    .filter_map(|n| boxes.get(n).copied().flatten())
                    .map(|target| interpolate(&target, 0.5, 0.5))
                    .collect()
            })
            .unwrap_or_default();
        let centre = interpolate(&place, 0.5, 0.5);
        let ring = rotation_handle(&place, scale);
        draw.with_clip_rect(
            origin,
            [origin[0] + width * scale, origin[1] + height * scale],
            || {
                // The outline follows the transformed corners, so a rotated
                // element is drawn where it is actually painted.
                let [a, b, c, d] = place.corners.map(screen);
                for (from, to) in [(a, b), (b, d), (d, c), (c, a)] {
                    draw.add_line(from, to, accent).thickness(2.).build();
                }
                for target in &links {
                    draw.add_line(screen(centre), screen(*target), link)
                        .thickness(1.)
                        .build();
                }
                if !resizable {
                    return;
                }
                for sides in HANDLES {
                    let h = screen(handle_point(&place, sides));
                    draw.add_rect(
                        [h[0] - marker, h[1] - marker],
                        [h[0] + marker, h[1] + marker],
                        accent,
                    )
                    .filled(true)
                    .build();
                }
                let h = screen(ring);
                draw.add_line(
                    screen(handle_point(&place, [None, Some(Side::Max)])),
                    h,
                    accent,
                )
                .build();
                draw.add_circle(h, marker, accent).filled(true).build();
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
    /// An unrotated placement: the corners are the rect's own, in the
    /// top-left/top-right/bottom-left/bottom-right order the runtime emits.
    fn placement(r: hud::Rect) -> hud::Placement {
        hud::Placement {
            rect: r,
            corners: [
                [r[0], r[1] + r[3]],
                [r[0] + r[2], r[1] + r[3]],
                [r[0], r[1]],
                [r[0] + r[2], r[1]],
            ],
        }
    }
    #[test]
    fn the_eight_handles_sit_on_the_sides_and_midpoints_they_name() {
        let p = placement([10., 20., 100., 40.]);
        let points: Vec<[f32; 2]> = HANDLES.iter().map(|s| handle_point(&p, *s)).collect();
        assert_eq!(points.len(), 8);
        assert!(points.contains(&[10., 20.]) && points.contains(&[110., 60.]));
        assert!(points.contains(&[60., 20.]) && points.contains(&[10., 40.]));
        let mut sorted = points.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        sorted.dedup();
        assert_eq!(sorted.len(), 8, "handles must not share a point");
        assert_eq!(placement_angle(&p), 0.);
        // The ring sits above the top edge, along the element's own up direction.
        let ring = rotation_handle(&p, 1.);
        assert_eq!(ring[0], 60.);
        assert!((ring[1] - (60. + ROTATE_REACH)).abs() < 0.01);
    }
    #[test]
    fn a_rotated_element_resizes_in_its_own_frame_and_reads_its_angle_back() {
        // A quarter turn maps the element's +X to the screen's +Y, so a handle
        // dragged that way has to come back as a width change, not a height one.
        let (mut e, i) = panel([0.5; 2], [0.5; 2], [0.5; 2]);
        e.scene.actors[i].rect.as_mut().unwrap().rotation = 90.;
        let place = hud::layouts(&e.scene)[i].unwrap();
        let angle = placement_angle(&place);
        assert!((angle - 90.).abs() < 0.2, "angle {angle}");
        let local = unrotate([0., 10.], angle);
        assert!(
            (local[0] - 10.).abs() < 0.05 && local[1].abs() < 0.05,
            "{local:?}"
        );
        let before = hud::layout(&e.scene, i).unwrap();
        resize(
            e.scene.actors[i].rect.as_mut().unwrap(),
            [Some(Side::Max), None],
            local,
        );
        let after = hud::layout(&e.scene, i).unwrap();
        // The layout rect stays axis-aligned: the grabbed side moved by ten and
        // the opposite one did not move at all.
        assert!((after[2] - before[2] - 10.).abs() < 0.05);
        assert!((after[0] - before[0]).abs() < 0.05);
    }
    #[test]
    fn the_corners_of_a_quarter_turn_swap_the_axes_about_the_pivot() {
        let (mut e, i) = panel([0.5; 2], [0.5; 2], [0.5; 2]);
        let square = hud::layouts(&e.scene)[i].unwrap();
        e.scene.actors[i].rect.as_mut().unwrap().rotation = 90.;
        let turned = hud::layouts(&e.scene)[i].unwrap();
        // Layout is untouched by rotation; only the corners move.
        assert_eq!(turned.rect, square.rect);
        let centre = [
            square.rect[0] + square.rect[2] / 2.,
            square.rect[1] + square.rect[3] / 2.,
        ];
        for (before, after) in square.corners.iter().zip(&turned.corners) {
            let expected = [
                centre[0] - (before[1] - centre[1]),
                centre[1] + (before[0] - centre[0]),
            ];
            assert!(
                (after[0] - expected[0]).abs() < 0.05 && (after[1] - expected[1]).abs() < 0.05,
                "{after:?} vs {expected:?}"
            );
        }
    }
}
