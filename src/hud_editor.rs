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
        if let Some(_combo) = ui.begin_combo(crate::gui::field(ui, "Anchors"), "Presets...") {
            for (name, p) in [
                ("Top Left", [0., 1.]),
                ("Top Center", [0.5, 1.]),
                ("Top Right", [1., 1.]),
                ("Center", [0.5, 0.5]),
                ("Bottom Left", [0., 0.]),
                ("Bottom Center", [0.5, 0.]),
                ("Bottom Right", [1., 0.]),
            ] {
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
    let mut handle = false;
    if let Some(i) = e.selected
        && let Some(r) = hud::layout(&e.scene, i)
        && e.scene.actors[i].rect.is_some()
    {
        handle = (p[0] - r[0] - r[2]).abs() < 5. / scale && (p[1] - r[1]).abs() < 5. / scale;
    }
    if hovered && ui.is_mouse_clicked(imgui::MouseButton::Left) && !e.playing {
        if !handle {
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
            .map(|i| (i, handle));
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
        && let Some((i, resize)) = e.hud_drag
        && ui.is_mouse_dragging(imgui::MouseButton::Left)
    {
        let delta = ui.io().mouse_delta.map(|v| v / scale);
        let original = e.scene.actors[i].rect.clone();
        if let Some(r) = &mut e.scene.actors[i].rect {
            if resize {
                r.size[0] += delta[0];
                r.size[1] += delta[1];
                r.position[0] += delta[0] * r.pivot[0];
                r.position[1] -= delta[1] * (1. - r.pivot[1]);
            } else {
                r.position[0] += delta[0];
                r.position[1] -= delta[1];
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
        draw.with_clip_rect(
            origin,
            [origin[0] + width * scale, origin[1] + height * scale],
            || {
                draw.add_rect(a, b, [1., 0.65, 0.25, 1.])
                    .thickness(2.)
                    .build();
                draw.add_rect(
                    [b[0] - 4., b[1] - 4.],
                    [b[0] + 4., b[1] + 4.],
                    [1., 0.65, 0.25, 1.],
                )
                .filled(true)
                .build();
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
            format!("HUD {width:.0} x {height:.0} | Drag to move; corner to resize")
        },
    );
}
