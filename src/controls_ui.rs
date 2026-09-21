//! Desktop controller authoring. Artwork and hit regions share a 1000 x 650 canvas.
use crate::controls::{Axis, Binding, Button, Control, Profile, Settings};
use imgui::{StyleColor as C, StyleVar as V};

const ACCENT: [f32; 4] = [0.38, 0.69, 0.98, 1.];
const MUTED: [f32; 4] = [0.55, 0.60, 0.67, 1.];
const TEXT: [f32; 4] = [0.87, 0.90, 0.94, 1.];
const GROUPS: [&str; 6] = [
    "All controls",
    "Face buttons",
    "D-pad",
    "Shoulders",
    "System",
    "Analog sticks",
];

pub struct State {
    pub texture: Option<imgui::TextureId>,
    pad: usize,
    selected: Control,
    group: usize,
    reveal: bool,
    rename: String,
    #[cfg(test)]
    cross_row: [f32; 2],
    #[cfg(test)]
    triangle_hotspot: [f32; 2],
}
impl Default for State {
    fn default() -> Self {
        Self {
            texture: None,
            pad: 0,
            selected: Control::Button(Button::Cross),
            group: 0,
            reveal: false,
            rename: String::new(),
            #[cfg(test)]
            cross_row: [0.; 2],
            #[cfg(test)]
            triangle_hotspot: [0.; 2],
        }
    }
}

// Button centers and hit extents in the original SVG, independent of DPI/window size.
const HOTSPOTS: [(Button, [f32; 2], [f32; 2]); 16] = [
    (Button::Triangle, [753., 224.], [32., 32.]),
    (Button::Circle, [806., 277.], [32., 32.]),
    (Button::Cross, [753., 330.], [32., 32.]),
    (Button::Square, [700., 277.], [32., 32.]),
    (Button::Up, [247., 235.], [20., 22.]),
    (Button::Right, [289., 277.], [22., 20.]),
    (Button::Down, [247., 319.], [20., 22.]),
    (Button::Left, [205., 277.], [22., 20.]),
    (Button::L1, [240., 163.], [50., 16.]),
    (Button::L2, [240., 115.], [50., 26.]),
    (Button::R1, [760., 163.], [50., 16.]),
    (Button::R2, [760., 115.], [50., 26.]),
    (Button::Select, [445., 296.], [25., 17.]),
    (Button::Start, [554., 296.], [25., 17.]),
    (Button::L3, [380., 405.], [48., 48.]),
    (Button::R3, [620., 405.], [48., 48.]),
];

pub fn page(
    ui: &imgui::Ui,
    settings: &mut Settings,
    state: &mut State,
    capture: &mut Option<(usize, Control)>,
) {
    let _styles = [
        V::FramePadding([9., 5.]),
        V::ItemSpacing([8., 7.]),
        V::FrameRounding(4.),
        V::ChildRounding(6.),
    ]
    .map(|v| ui.push_style_var(v));
    ui.text("Controls");
    ui.text_colored(MUTED, "Desktop input  /  PlayStation virtual pads");
    ui.spacing();
    state.pad = state.pad.min(3);
    let tab_width = ((ui.content_region_avail()[0] - 154.) / 4.).clamp(55., 100.);
    for pad in 0..4 {
        if pad > 0 {
            ui.same_line();
        }
        let active = state.pad == pad;
        let _color = ui.push_style_color(
            C::Button,
            if active {
                [0.16, 0.31, 0.46, 1.]
            } else {
                [0.13, 0.15, 0.18, 1.]
            },
        );
        if ui.button_with_size(format!("Pad {}", pad + 1), [tab_width, 30.]) {
            state.pad = pad;
            *capture = None;
        }
    }
    ui.same_line();
    ui.checkbox("Connected", &mut settings.pads[state.pad].enabled);
    ui.align_text_to_frame_padding();
    ui.text_colored(MUTED, "Profile");
    ui.same_line();
    ui.set_next_item_width((ui.content_region_avail()[0] - 185.).clamp(120., 350.));
    let current = settings
        .profile(state.pad)
        .map(|p| p.name.as_str())
        .unwrap_or("Choose a profile");
    if let Some(_combo) = ui.begin_combo("##input-profile", current) {
        for profile in &settings.profiles {
            if ui
                .selectable_config(format!("{}##{}", profile.name, profile.id))
                .selected(settings.pads[state.pad].profile == Some(profile.id))
                .build()
            {
                settings.pads[state.pad].profile = Some(profile.id);
                *capture = None;
            }
        }
    }
    ui.same_line();
    if ui.button("Manage...") {
        ui.open_popup("input-profile-menu");
    }
    ui.same_line();
    if ui.button("Mouse...") {
        ui.open_popup("input-mouse-options");
    }
    ui.popup("input-mouse-options", || {
        ui.text("Mouse to analog stick");
        ui.set_next_item_width(170.);
        let mut sensitivity = i32::from(settings.mouse_sensitivity);
        if ui.slider("Sensitivity", 1, 100, &mut sensitivity) {
            settings.mouse_sensitivity = sensitivity as u16;
        }
        ui.text_colored(MUTED, "Active while the Game view has input focus.");
    });
    let mut rename = false;
    ui.popup("input-profile-menu", || {
        if ui.menu_item("New profile") {
            let mut p = Profile::keyboard_mouse();
            p.name = format!("Profile {}", settings.profiles.len() + 1);
            settings.pads[state.pad].profile = Some(p.id);
            settings.profiles.push(p);
        }
        if let Some(profile) = settings.profile(state.pad).cloned() {
            if ui.menu_item("Duplicate profile") {
                let mut p = profile.clone();
                p.id = uuid::Uuid::new_v4();
                p.name = format!("{} copy", p.name);
                settings.pads[state.pad].profile = Some(p.id);
                settings.profiles.push(p);
            }
            if ui.menu_item("Rename profile") {
                state.rename = profile.name;
                rename = true;
            }
        }
    });
    if rename {
        ui.open_popup("Rename input profile");
    }
    if let Some(_popup) = ui.begin_modal_popup("Rename input profile") {
        ui.input_text("Name", &mut state.rename).build();
        if ui.button("Save name") && !state.rename.trim().is_empty() {
            if let Some(id) = settings.pads[state.pad].profile
                && let Some(p) = settings.profiles.iter_mut().find(|p| p.id == id)
            {
                p.name = state.rename.trim().into();
            }
            ui.close_current_popup();
        }
        ui.same_line();
        if ui.button("Cancel") {
            ui.close_current_popup();
        }
    }
    ui.spacing();
    let Some(id) = settings.pads[state.pad].profile else {
        ui.text_wrapped("Choose a profile or use Manage to create one for this pad.");
        return;
    };
    let Some(profile) = settings.profiles.iter_mut().find(|p| p.id == id) else {
        return;
    };
    let width = ui.content_region_avail()[0];
    let side_by_side = width >= 650.;
    let height = (ui.content_region_avail()[1] - 2.).max(260.);
    let left = if side_by_side {
        (width * 0.58).max(350.)
    } else {
        width
    };
    let _bg = ui.push_style_color(C::ChildBg, [0.075, 0.087, 0.105, 1.]);
    ui.child_window("controller-art-panel")
        .size([left, if side_by_side { height } else { 355. }])
        .border(true)
        .build(|| {
            let _padding = ui.push_style_var(V::ItemSpacing([8., 10.]));
            ui.text_colored(TEXT, "DUALSHOCK");
            ui.same_line();
            ui.text_colored(MUTED, format!("/  PAD {}", state.pad + 1));
            if left >= 450. {
                ui.text_colored(MUTED, "Select a button to assign an input.");
            }
            let available = ui.content_region_avail();
            let compact = available[0] < 420.;
            let footer = if compact { 94. } else { 117. };
            let image_width = available[0].min((available[1] - footer).max(65.) / (520. / 850.));
            let start = ui.cursor_screen_pos();
            let origin = [start[0] + (available[0] - image_width) * 0.5, start[1]];
            diagram(ui, state, capture, origin, image_width);
            ui.set_cursor_screen_pos([start[0], origin[1] + image_width * (520. / 850.) + 6.]);
            // Establish item bounds exactly once; moving the cursor alone creates phantom scroll space.
            ui.dummy([available[0], 1.]);
            let w = (ui.content_region_avail()[0] - 24.) / 4.;
            for (n, (axis, label)) in [
                (Axis::LeftX, "Left X"),
                (Axis::LeftY, "Left Y"),
                (Axis::RightX, "Right X"),
                (Axis::RightY, "Right Y"),
            ]
            .into_iter()
            .enumerate()
            {
                if n > 0 {
                    ui.same_line();
                }
                let control = Control::Axis(axis);
                let _color = ui.push_style_color(
                    C::Button,
                    if state.selected == control {
                        [0.16, 0.31, 0.46, 1.]
                    } else {
                        [0.13, 0.16, 0.20, 1.]
                    },
                );
                if ui.button_with_size(label, [w, 27.]) {
                    state.selected = control;
                    state.group = 5;
                    state.reveal = true;
                    *capture = Some((state.pad, control));
                }
            }
            ui.separator();
            ui.text_colored(ACCENT, state.selected.label());
            ui.same_line();
            ui.text(
                profile
                    .binding(state.selected)
                    .map_or("Unassigned".into(), Binding::label),
            );
            if !compact {
                ui.text_colored(MUTED, "Click a stick to map L3 / R3. Use X / Y for axes.");
            }
        });
    if side_by_side {
        ui.same_line();
    } else {
        ui.spacing();
    }
    ui.child_window("controller-bindings-panel")
        .size([
            if side_by_side { 0. } else { width },
            if side_by_side { height } else { 380. },
        ])
        .border(true)
        .build(|| {
            ui.text("Input bindings");
            ui.set_next_item_width(-1.);
            ui.combo_simple_string("##input-group", &mut state.group, &GROUPS);
            let list_height = (ui.content_region_avail()[1] - 48.).max(120.);
            ui.child_window("controller-bindings-list")
                .size([0., list_height])
                .build(|| {
                    for (group, controls) in groups() {
                        if state.group != 0 && GROUPS[state.group] != group {
                            continue;
                        }
                        ui.text_colored(MUTED, group);
                        ui.separator();
                        for control in controls {
                            binding_row(ui, profile, state, capture, control);
                        }
                        ui.spacing();
                    }
                });
            ui.separator();
            ui.text_colored(MUTED, "Click to rebind. Right-click to clear.");
        });
    if capture.is_some() {
        ui.open_popup("Assign input");
    }
    if let Some(_modal) = ui.begin_modal_popup("Assign input") {
        if let Some((pad, control)) = *capture {
            ui.text_colored(ACCENT, format!("Pad {}  /  {}", pad + 1, control.label()));
            ui.spacing();
            ui.text(if matches!(control, Control::Axis(_)) {
                "Move the mouse or a gamepad stick."
            } else {
                "Press a key, mouse button or gamepad button."
            });
            ui.spacing();
            ui.separator();
            ui.text_colored(MUTED, "Waiting for input...  Press Esc to cancel.");
        } else {
            ui.close_current_popup();
        }
    }
}

fn groups() -> [(&'static str, Vec<Control>); 5] {
    use Button::*;
    [
        (
            "Face buttons",
            vec![Triangle, Circle, Cross, Square]
                .into_iter()
                .map(Control::Button)
                .collect(),
        ),
        (
            "D-pad",
            vec![Up, Right, Down, Left]
                .into_iter()
                .map(Control::Button)
                .collect(),
        ),
        (
            "Shoulders",
            vec![L1, R1, L2, R2]
                .into_iter()
                .map(Control::Button)
                .collect(),
        ),
        (
            "System",
            vec![Select, Start, L3, R3]
                .into_iter()
                .map(Control::Button)
                .collect(),
        ),
        (
            "Analog sticks",
            Axis::ALL.into_iter().map(Control::Axis).collect(),
        ),
    ]
}

fn binding_row(
    ui: &imgui::Ui,
    profile: &mut Profile,
    state: &mut State,
    capture: &mut Option<(usize, Control)>,
    control: Control,
) {
    let _id = ui.push_id(format!("binding-{control:?}"));
    let start = ui.cursor_screen_pos();
    let width = ui.content_region_avail()[0];
    if ui.invisible_button("assign", [width, 29.]) {
        state.selected = control;
        *capture = Some((state.pad, control));
    }
    #[cfg(test)]
    if control == Control::Button(Button::Cross) {
        state.cross_row = [start[0] + width * 0.5, start[1] + 14.];
    }
    let hovered = ui.is_item_hovered();
    if ui.is_item_clicked_with_button(imgui::MouseButton::Right) {
        profile.mappings.retain(|m| m.control != control);
    }
    if state.reveal && state.selected == control {
        ui.set_scroll_here_y();
        state.reveal = false;
    }
    let selected = state.selected == control;
    let value = profile
        .binding(control)
        .map_or("Unassigned".into(), Binding::label);
    if hovered {
        ui.tooltip_text(format!(
            "{}: {}\nClick to assign | Right-click to clear",
            control.label(),
            value
        ));
    }
    let draw = ui.get_window_draw_list();
    draw.add_rect(
        start,
        [start[0] + width, start[1] + 29.],
        if hovered {
            [0.17, 0.23, 0.30, 1.]
        } else if selected {
            [0.12, 0.21, 0.30, 1.]
        } else {
            [0.10, 0.12, 0.15, 1.]
        },
    )
    .filled(true)
    .rounding(3.)
    .build();
    if selected {
        draw.add_rect(start, [start[0] + 2., start[1] + 29.], ACCENT)
            .filled(true)
            .build();
    }
    let key_left = start[0] + width * 0.49;
    let name_x = start[0] + 29.;
    control_icon(&draw, control, [start[0] + 15., start[1] + 14.]);
    draw.with_clip_rect_intersect([name_x, start[1]], [key_left - 5., start[1] + 29.], || {
        draw.add_text([name_x, start[1] + 6.], TEXT, control.label())
    });
    draw.add_rect(
        [key_left, start[1] + 3.],
        [start[0] + width - 4., start[1] + 26.],
        [0.06, 0.075, 0.095, 1.],
    )
    .rounding(3.)
    .filled(true)
    .build();
    draw.with_clip_rect_intersect(
        [key_left + 6., start[1]],
        [start[0] + width - 8., start[1] + 29.],
        || {
            draw.add_text(
                [key_left + 6., start[1] + 6.],
                if profile.binding(control).is_some() {
                    ACCENT
                } else {
                    MUTED
                },
                value,
            )
        },
    );
}

fn control_icon(draw: &imgui::DrawListMut<'_>, control: Control, p: [f32; 2]) {
    match control {
        Control::Button(Button::Triangle) => {
            draw.add_triangle(
                [p[0], p[1] - 6.],
                [p[0] + 6., p[1] + 5.],
                [p[0] - 6., p[1] + 5.],
                [0.42, 0.77, 0.69, 1.],
            )
            .thickness(1.5)
            .build();
        }
        Control::Button(Button::Circle) => {
            draw.add_circle(p, 5.5, [0.93, 0.52, 0.56, 1.])
                .thickness(1.5)
                .build();
        }
        Control::Button(Button::Cross) => {
            for sign in [-1., 1.] {
                draw.add_line(
                    [p[0] - 5., p[1] - 5. * sign],
                    [p[0] + 5., p[1] + 5. * sign],
                    [0.55, 0.69, 0.91, 1.],
                )
                .thickness(1.5)
                .build();
            }
        }
        Control::Button(Button::Square) => {
            draw.add_rect(
                [p[0] - 5., p[1] - 5.],
                [p[0] + 5., p[1] + 5.],
                [0.84, 0.60, 0.80, 1.],
            )
            .thickness(1.5)
            .build();
        }
        _ => {
            draw.add_circle(p, 2.5, MUTED).filled(true).build();
        }
    }
}

fn diagram(
    ui: &imgui::Ui,
    state: &mut State,
    capture: &mut Option<(usize, Control)>,
    origin: [f32; 2],
    width: f32,
) {
    // Crop transparent margins; hit coordinates still use the original viewBox.
    let scale = width / 850.;
    ui.set_cursor_screen_pos(origin);
    if let Some(texture) = state.texture {
        imgui::Image::new(texture, [width, width * (520. / 850.)])
            .uv0([0.075, 75. / 650.])
            .uv1([0.925, 595. / 650.])
            .build(ui);
    } else {
        ui.dummy([width, width * (520. / 850.)]);
    }
    for (button, center, extent) in HOTSPOTS {
        let point = [
            origin[0] + (center[0] - 75.) * scale,
            origin[1] + (center[1] - 75.) * scale,
        ];
        let size = extent.map(|v| (v * scale).max(7.));
        #[cfg(test)]
        if button == Button::Triangle {
            state.triangle_hotspot = point;
        }
        ui.set_cursor_screen_pos([point[0] - size[0], point[1] - size[1]]);
        let control = Control::Button(button);
        if ui.invisible_button(format!("hotspot-{button:?}"), [size[0] * 2., size[1] * 2.]) {
            state.selected = control;
            state.group = 0;
            state.reveal = true;
            *capture = Some((state.pad, control));
        }
        let hover = ui.is_item_hovered();
        if hover {
            ui.tooltip_text(format!("{} | Click to rebind", control.label()));
        }
        if hover || state.selected == control {
            let draw = ui.get_window_draw_list();
            draw.add_rect(
                [point[0] - size[0] - 2., point[1] - size[1] - 2.],
                [point[0] + size[0] + 2., point[1] + size[1] + 2.],
                ACCENT,
            )
            .rounding(size[0].min(size[1]) * 0.6)
            .thickness(2.)
            .build();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn controls_diagram_and_binding_rows_open_capture_and_clear_assignments() {
        let mut ctx = crate::gui::tests::imgui_context();
        ctx.io_mut().display_size = [1200., 800.];
        ctx.io_mut().delta_time = 1. / 60.;
        ctx.fonts()
            .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
        ctx.fonts().build_rgba32_texture();
        let mut settings = Settings::default();
        let mut state = State::default();
        let mut capture = None;
        fn frame(
            ctx: &mut imgui::Context,
            s: &mut Settings,
            v: &mut State,
            c: &mut Option<(usize, Control)>,
        ) {
            let ui = ctx.frame();
            ui.window("Controls QA")
                .position([0., 0.], imgui::Condition::Always)
                .size([1100., 750.], imgui::Condition::Always)
                .build(|| page(ui, s, v, c));
            ctx.render();
        }
        fn click(
            ctx: &mut imgui::Context,
            s: &mut Settings,
            v: &mut State,
            c: &mut Option<(usize, Control)>,
            pos: [f32; 2],
            button: imgui::MouseButton,
        ) {
            ctx.io_mut().add_mouse_pos_event(pos);
            frame(ctx, s, v, c);
            ctx.io_mut().add_mouse_button_event(button, true);
            frame(ctx, s, v, c);
            ctx.io_mut().add_mouse_button_event(button, false);
            frame(ctx, s, v, c);
        }
        frame(&mut ctx, &mut settings, &mut state, &mut capture);
        frame(&mut ctx, &mut settings, &mut state, &mut capture);
        let cross = state.cross_row;
        click(
            &mut ctx,
            &mut settings,
            &mut state,
            &mut capture,
            cross,
            imgui::MouseButton::Left,
        );
        assert_eq!(capture, Some((0, Control::Button(Button::Cross))));
        // The host resolves capture outside the render loop. The modal must close
        // on the following frame and allow another control to be selected.
        capture = None;
        frame(&mut ctx, &mut settings, &mut state, &mut capture);
        frame(&mut ctx, &mut settings, &mut state, &mut capture);
        let triangle = state.triangle_hotspot;
        click(
            &mut ctx,
            &mut settings,
            &mut state,
            &mut capture,
            triangle,
            imgui::MouseButton::Left,
        );
        assert_eq!(capture, Some((0, Control::Button(Button::Triangle))));
        capture = None;
        frame(&mut ctx, &mut settings, &mut state, &mut capture);
        frame(&mut ctx, &mut settings, &mut state, &mut capture);
        let cross = state.cross_row;
        click(
            &mut ctx,
            &mut settings,
            &mut state,
            &mut capture,
            cross,
            imgui::MouseButton::Right,
        );
        assert!(
            settings
                .profile(0)
                .unwrap()
                .binding(Control::Button(Button::Cross))
                .is_none()
        );
    }
}
