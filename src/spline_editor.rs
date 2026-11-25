//! Author a mesh's spline in the same asset transaction and undo history as Blockout.
use crate::{
    editor::Editor,
    gui::field,
    lighting::sub,
    spline::{MAX_POINTS, MAX_STEPS, Point, Profile, Spline},
    transform::Matrix,
    viewport::{self, View},
};
use imgui::{MouseButton, Ui};
use uuid::Uuid;

#[derive(Default)]
pub struct State {
    selected: usize,
    plane: usize,
    drag: Option<Drag>,
}
#[derive(Clone, Copy)]
struct Drag {
    actor: Uuid,
    point: usize,
    hit: [f32; 3],
    start: [f32; 3],
}
impl State {
    pub fn cancel_drag(&mut self) -> bool {
        self.drag.take().is_some()
    }
    fn normal_axis(&self) -> usize {
        [1, 2, 0][self.plane]
    }
}

fn commit(e: &mut Editor, spline: Spline, selected: usize) -> bool {
    let result = crate::mesh_editor::edit(e, |doc| spline.apply(doc));
    let ok = result.is_ok();
    if ok {
        e.mesh_editor.spline_editor.selected = selected;
    }
    e.mesh_editor.error = result.err();
    ok
}

pub fn panel(ui: &Ui, e: &mut Editor, spline: &Spline, faces: usize, slots: &[crate::mesh::Slot]) {
    let mut curve = spline.clone();
    let mut selected = e
        .mesh_editor
        .spline_editor
        .selected
        .min(curve.points.len() - 1);
    let mut changed = false;
    let mut frame_shape = false;
    ui.separator();
    if let Some(_presets) = ui.begin_combo(field(ui, "Set path shape"), "Choose shape...") {
        for (i, name) in ["Straight", "Circuit", "Vertical loop"].iter().enumerate() {
            if ui.selectable(name) {
                curve.points = match i {
                    1 => Spline::circuit(),
                    2 => Spline::vertical_loop(),
                    _ => Spline::default().points,
                };
                curve.closed = i == 1;
                selected = 0;
                changed = true;
                frame_shape = true;
            }
        }
    }
    let profiles = [Profile::Road, Profile::Railing, Profile::Wall];
    let mut profile = profiles.iter().position(|p| *p == curve.profile).unwrap();
    if ui.combo_simple_string(
        field(ui, "Profile"),
        &mut profile,
        &["Road", "Railing", "Wall"],
    ) {
        curve.profile = profiles[profile];
        (curve.width, curve.height) = curve.profile.dimensions();
        changed = true;
    }
    ui.disabled(curve.points.len() < 3, || {
        changed |= ui.checkbox(field(ui, "Closed circuit"), &mut curve.closed);
    });
    changed |= ui
        .input_float(field(ui, "Width"), &mut curve.width)
        .enter_returns_true(true)
        .build();
    if curve.profile != Profile::Road {
        changed |= ui
            .input_float(field(ui, "Height"), &mut curve.height)
            .enter_returns_true(true)
            .build();
    }
    if curve.profile == Profile::Railing {
        changed |= ui
            .input_float(field(ui, "Post spacing"), &mut curve.post_spacing)
            .enter_returns_true(true)
            .build();
    }
    let mut steps = i32::from(curve.steps);
    if ui
        .input_int(field(ui, "Steps per span"), &mut steps)
        .enter_returns_true(true)
        .build()
    {
        curve.steps = steps.clamp(1, i32::from(MAX_STEPS)) as u8;
        changed = true;
    }
    let mut available = curve.gameplay_channel.is_some();
    if ui.checkbox(field(ui, "Sample from gameplay"), &mut available) {
        curve.gameplay_channel = available.then_some(0);
        changed = true;
    }
    if let Some(channel) = curve.gameplay_channel.as_mut() {
        let mut value = i32::from(*channel);
        if ui
            .input_int(field(ui, "Map path channel"), &mut value)
            .enter_returns_true(true)
            .build()
        {
            *channel = value.clamp(0, 255) as u8;
            changed = true;
        }
    }
    ui.separator();
    ui.combo_simple_string(
        field(ui, "Draw plane"),
        &mut e.mesh_editor.spline_editor.plane,
        &["XZ (ground)", "XY (front)", "YZ (side)"],
    );
    ui.text_wrapped("Shift-click in Scene to extend the path. Drag a point in the draw plane. Press Enter to apply a numeric edit.");
    if let Some(_points) = ui.begin_combo(
        field(ui, "Control point"),
        format!("{} of {}", selected + 1, curve.points.len()),
    ) {
        for i in 0..curve.points.len() {
            if ui
                .selectable_config(format!("Point {}", i + 1))
                .selected(i == selected)
                .build()
            {
                selected = i;
            }
        }
    }
    changed |= ui
        .input_float3(field(ui, "Local XYZ"), &mut curve.points[selected].position)
        .enter_returns_true(true)
        .build();
    changed |= ui
        .input_float(
            field(ui, "Bank (degrees)"),
            &mut curve.points[selected].bank,
        )
        .enter_returns_true(true)
        .build();
    if selected < curve.spans() {
        let point = &mut curve.points[selected];
        let label = point
            .surface
            .and_then(|id| slots.iter().find(|s| s.id == id))
            .map_or("Default (first slot)", |s| s.name.as_str());
        if let Some(_combo) = ui.begin_combo(field(ui, "Outgoing surface"), label) {
            if ui.selectable("Default (first slot)") {
                point.surface = None;
                changed = true;
            }
            for slot in slots {
                if ui.selectable(format!("{}##{}", slot.name, slot.id)) {
                    point.surface = Some(slot.id);
                    changed = true;
                }
            }
        }
        let mut tag = i32::from(point.tag);
        if ui
            .input_int(field(ui, "Outgoing marker"), &mut tag)
            .enter_returns_true(true)
            .build()
        {
            point.tag = tag.clamp(0, 255) as u8;
            changed = true;
        }
    }
    ui.disabled(curve.points.len() >= MAX_POINTS, || {
        if ui.button("Insert after point") {
            let next = if selected < curve.spans() {
                curve.midpoint(selected)
            } else {
                let end = curve.points[selected];
                let delta = sub(end.position, curve.points[selected - 1].position);
                Point {
                    position: std::array::from_fn(|i| end.position[i] + delta[i]),
                    ..end
                }
            };
            selected += 1;
            curve.points.insert(selected, next);
            changed = true;
        }
    });
    ui.same_line();
    ui.disabled(
        curve.points.len() <= if curve.closed { 3 } else { 2 },
        || {
            if ui.button("Remove point") {
                curve.points.remove(selected);
                selected = selected.min(curve.points.len() - 1);
                changed = true;
            }
        },
    );
    if changed {
        if commit(e, curve, selected)
            && frame_shape
            && let Some(target) = target(e)
            && let Some((low, high)) = crate::scene_view_state::bounds(&e.scene, Some(target))
        {
            e.view.frame_bounds_in_panel(low, high, e.scene_panel_size);
            e.view_dirty = true;
        }
    } else {
        e.mesh_editor.spline_editor.selected = selected;
    }
    ui.text_disabled(format!("{faces} generated faces"));
}

fn target(e: &Editor) -> Option<usize> {
    let asset = e.mesh_editor.record.as_ref()?.meta.id;
    let uses_asset = |actor: &crate::scene::Actor| {
        actor
            .editable_mesh
            .as_ref()
            .is_some_and(|mesh| mesh.asset == asset)
    };
    e.mesh_editor
        .target
        .filter(|&i| e.scene.actors.get(i).is_some_and(uses_asset))
        .or_else(|| e.scene.actors.iter().position(uses_asset))
}

/// Intersect the view ray with a local construction plane, including under a
/// rotated or scaled parent. An edge-on or behind-camera plane has no hit.
fn plane_hit(
    view: &View,
    inverse: Matrix,
    pixel: [f32; 2],
    axis: usize,
    coordinate: f32,
) -> Option<[f32; 3]> {
    let origin = inverse.point(view.unproject(pixel, 0.));
    let direction = inverse.vector(sub(view.unproject(pixel, 2.), view.unproject(pixel, 1.)));
    if direction[axis].abs() < 1e-6 {
        return None;
    }
    let depth = (coordinate - origin[axis]) / direction[axis];
    (depth >= 1.).then(|| std::array::from_fn(|i| origin[i] + direction[i] * depth))
}

pub fn overlay(
    ui: &Ui,
    e: &mut Editor,
    pixel: [f32; 2],
    eligible: bool,
    clip: [[f32; 2]; 2],
    to_panel: impl Fn([f32; 2]) -> [f32; 2],
) -> bool {
    if !e.mesh_editor.editing_spline() || e.playing {
        e.mesh_editor.spline_editor.cancel_drag();
        return false;
    }
    let Some(index) = target(e) else {
        e.mesh_editor.spline_editor.cancel_drag();
        return false;
    };
    let world = e.scene.world_matrix(index);
    let Ok(inverse) = world.inverse() else {
        return false;
    };
    let actor = e.scene.actors[index].id;
    let view = e.view;
    let project = |point| {
        let p = viewport::project(&view, world.point(point));
        (p[2] >= 1.).then(|| (to_panel([p[0], p[1]]), p[2]))
    };
    let mut curve = e
        .mesh_editor
        .doc
        .as_ref()
        .unwrap()
        .spline
        .as_ref()
        .unwrap()
        .clone();
    let axis = e.mesh_editor.spline_editor.normal_axis();
    let mouse = ui.io().mouse_pos;
    let hovered = curve
        .points
        .iter()
        .enumerate()
        .filter_map(|(i, p)| {
            let (screen, depth) = project(p.position)?;
            let d = (mouse[0] - screen[0]).hypot(mouse[1] - screen[1]);
            (d <= 9.).then_some((i, d, depth))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1).then(a.2.total_cmp(&b.2)))
        .map(|p| p.0);
    let state = &mut e.mesh_editor.spline_editor;
    let mut owns = state.drag.is_some();
    if !eligible || ui.is_key_pressed(imgui::Key::Escape) {
        state.cancel_drag();
    } else if ui.is_mouse_clicked(MouseButton::Left) {
        if ui.io().key_shift {
            let last = *curve.points.last().unwrap();
            if let Some(position) = plane_hit(&view, inverse, pixel, axis, last.position[axis]) {
                let selected = curve.points.len();
                curve.points.push(Point { position, ..last });
                commit(e, curve, selected);
            } else {
                e.mesh_editor.error =
                    Some("Orbit the view or choose a draw plane facing the camera.".into());
            }
            return true;
        }
        if let Some(i) = hovered {
            state.selected = i;
            let start = curve.points[i].position;
            state.drag = plane_hit(&view, inverse, pixel, axis, start[axis]).map(|hit| Drag {
                actor,
                point: i,
                hit,
                start,
            });
            owns = true;
        }
    }
    if let Some(drag) = e.mesh_editor.spline_editor.drag {
        owns = true;
        let released = ui.is_mouse_released(MouseButton::Left);
        if drag.actor == actor
            && drag.point < curve.points.len()
            && (released || ui.is_mouse_down(MouseButton::Left))
            && let Some(hit) = plane_hit(&view, inverse, pixel, axis, drag.start[axis])
        {
            let delta = sub(hit, drag.hit);
            if delta.iter().any(|c| c.abs() > 0.001) {
                curve.points[drag.point].position =
                    std::array::from_fn(|i| drag.start[i] + delta[i]);
                if released {
                    commit(e, curve.clone(), drag.point);
                }
            }
        }
        if !ui.is_mouse_down(MouseButton::Left) {
            e.mesh_editor.spline_editor.cancel_drag();
        }
    }
    let draw = ui.get_window_draw_list();
    draw.with_clip_rect(clip[0], clip[1], || {
        let line = |a, b| {
            if let (Some((a, _)), Some((b, _))) = (project(a), project(b)) {
                draw.add_line(a, b, [0.15, 0.92, 0.98, 1.])
                    .thickness(2.)
                    .build();
            }
        };
        if let Ok(frames) = curve.frames() {
            for span in frames.windows(2) {
                line(span[0].center, span[1].center);
            }
        }
        for (i, point) in curve.points.iter().enumerate() {
            if let Some((p, _)) = project(point.position) {
                let selected = i == e.mesh_editor.spline_editor.selected;
                let color = if selected {
                    [1., 0.74, 0.18, 1.]
                } else {
                    [0.15, 0.92, 0.98, 1.]
                };
                let radius = if selected || (eligible && hovered == Some(i)) {
                    6.
                } else {
                    4.
                };
                draw.add_circle(p, radius + 1.5, [0.06, 0.09, 0.12, 1.])
                    .filled(true)
                    .build();
                draw.add_circle(p, radius, color).filled(true).build();
            }
        }
    });
    owns
}
