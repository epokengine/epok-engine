//! Editor-only transform handles, projected with the same camera as the Scene view.
use crate::{editor::Editor, viewport};

pub const ORIENTATION_SIZE: f32 = 72.;
const ORIENTATION_CENTER: [f32; 2] = [36., 36.];
const ORIENTATION_RADIUS: f32 = 27.;
const ORIENTATION_HIT_RADIUS: f32 = 9.;
const AXIS_COLORS: [[f32; 4]; 3] = [
    [0.93, 0.27, 0.22, 1.],
    [0.42, 0.78, 0.24, 1.],
    [0.27, 0.52, 0.96, 1.],
];

#[derive(Clone, Copy, Debug, PartialEq)]
enum OrientationTarget {
    Axis { axis: usize, sign: f32 },
    Isometric,
}

#[derive(Clone, Copy)]
struct OrientationEndpoint {
    axis: usize,
    sign: f32,
    position: [f32; 2],
    depth: f32,
}

fn orientation_endpoint(view: &viewport::View, axis: usize, sign: f32) -> OrientationEndpoint {
    let [right, up, forward] = view.basis();
    OrientationEndpoint {
        axis,
        sign,
        position: [
            ORIENTATION_CENTER[0] + right[axis] * sign * ORIENTATION_RADIUS,
            ORIENTATION_CENTER[1] - up[axis] * sign * ORIENTATION_RADIUS,
        ],
        depth: forward[axis] * sign,
    }
}

fn orientation_target(view: &viewport::View, point: [f32; 2]) -> Option<OrientationTarget> {
    if (point[0] - ORIENTATION_CENTER[0]).abs() <= 7.
        && (point[1] - ORIENTATION_CENTER[1]).abs() <= 7.
    {
        return Some(OrientationTarget::Isometric);
    }
    let mut nearest: Option<(f32, f32, OrientationTarget)> = None;
    for axis in 0..3 {
        for sign in [-1., 1.] {
            let endpoint = orientation_endpoint(view, axis, sign);
            let distance = (point[0] - endpoint.position[0]).hypot(point[1] - endpoint.position[1]);
            if distance <= ORIENTATION_HIT_RADIUS
                && nearest
                    .as_ref()
                    .is_none_or(|(best_distance, best_depth, _)| {
                        distance < *best_distance
                            || ((distance - *best_distance).abs() < 0.01
                                && endpoint.depth < *best_depth)
                    })
            {
                nearest = Some((
                    distance,
                    endpoint.depth,
                    OrientationTarget::Axis { axis, sign },
                ));
            }
        }
    }
    nearest.map(|(_, _, target)| target)
}

fn apply_orientation_target(view: &mut viewport::View, target: OrientationTarget) {
    match target {
        OrientationTarget::Axis { axis, sign } => view.snap_to_axis(axis, -sign),
        OrientationTarget::Isometric => view.snap_to_isometric(),
    }
}

fn add_filled_quad(draw: &imgui::DrawListMut<'_>, points: [[f32; 2]; 4], color: [f32; 4]) {
    draw.add_triangle(points[0], points[1], points[2], color)
        .filled(true)
        .build();
    draw.add_triangle(points[0], points[2], points[3], color)
        .filled(true)
        .build();
}

fn draw_orientation_cube(draw: &imgui::DrawListMut<'_>, center: [f32; 2], hovered: bool) {
    let point = |x: f32, y: f32| [center[0] + x, center[1] + y];
    let top = [
        point(0., -7.),
        point(7., -3.),
        point(0., 1.),
        point(-7., -3.),
    ];
    let right = [point(0., 1.), point(7., -3.), point(7., 4.), point(0., 8.)];
    let left = [
        point(-7., -3.),
        point(0., 1.),
        point(0., 8.),
        point(-7., 4.),
    ];
    add_filled_quad(draw, left, [0.72, 0.77, 0.81, 1.]);
    add_filled_quad(draw, right, [0.55, 0.61, 0.66, 1.]);
    add_filled_quad(draw, top, [0.91, 0.94, 0.96, 1.]);
    if hovered {
        draw.add_circle(center, 10., [1., 0.82, 0.24, 1.])
            .thickness(2.)
            .build();
    }
}

pub fn draw_orientation(ui: &imgui::Ui, view: &mut viewport::View, position: [f32; 2]) -> bool {
    let mouse = [
        ui.io().mouse_pos[0] - position[0],
        ui.io().mouse_pos[1] - position[1],
    ];
    let inside = (0. ..=ORIENTATION_SIZE).contains(&mouse[0])
        && (0. ..=ORIENTATION_SIZE).contains(&mouse[1]);
    let hovered = inside.then(|| orientation_target(view, mouse)).flatten();
    let center = [
        position[0] + ORIENTATION_CENTER[0],
        position[1] + ORIENTATION_CENTER[1],
    ];
    let mut endpoints = Vec::with_capacity(6);
    for axis in 0..3 {
        for sign in [-1., 1.] {
            endpoints.push(orientation_endpoint(view, axis, sign));
        }
    }
    endpoints.sort_by(|a, b| b.depth.total_cmp(&a.depth));

    let draw = ui.get_window_draw_list();
    draw.add_circle(center, 34., [0.05, 0.07, 0.09, 0.34])
        .filled(true)
        .build();
    for endpoint in &endpoints {
        let end = [
            position[0] + endpoint.position[0],
            position[1] + endpoint.position[1],
        ];
        let mut color = AXIS_COLORS[endpoint.axis];
        if endpoint.sign < 0. {
            color[3] = 0.48;
        }
        let is_hovered = hovered
            == Some(OrientationTarget::Axis {
                axis: endpoint.axis,
                sign: endpoint.sign,
            });
        draw.add_line(
            center,
            end,
            if is_hovered {
                [1., 0.82, 0.24, 1.]
            } else {
                color
            },
        )
        .thickness(if is_hovered { 3. } else { 2. })
        .build();
        if endpoint.sign > 0. {
            draw.add_circle(end, if is_hovered { 8. } else { 7. }, color)
                .filled(true)
                .build();
            draw.add_circle(end, if is_hovered { 8. } else { 7. }, [0.95, 0.97, 1., 0.9])
                .thickness(if is_hovered { 2. } else { 1. })
                .build();
            let label = ["X", "Y", "Z"][endpoint.axis];
            draw.add_text([end[0] - 4., end[1] - 7.], [1.; 4], label);
        } else {
            let radius = if is_hovered { 5. } else { 4. };
            draw.add_rect(
                [end[0] - radius, end[1] - radius],
                [end[0] + radius, end[1] + radius],
                if is_hovered {
                    [1., 0.82, 0.24, 1.]
                } else {
                    color
                },
            )
            .filled(true)
            .build();
        }
    }
    draw_orientation_cube(&draw, center, hovered == Some(OrientationTarget::Isometric));

    if let Some(target) = hovered {
        match target {
            OrientationTarget::Axis { axis, sign } => {
                ui.tooltip_text(format!(
                    "View from {}{}",
                    if sign > 0. { "+" } else { "-" },
                    ["X", "Y", "Z"][axis]
                ));
            }
            OrientationTarget::Isometric => ui.tooltip_text("Three-quarter view"),
        }
        if ui.is_window_hovered() && ui.is_mouse_clicked(imgui::MouseButton::Left) {
            apply_orientation_target(view, target);
            return true;
        }
    }
    false
}

pub fn draw(
    ui: &imgui::Ui,
    e: &mut Editor,
    position: [f32; 2],
    size: [f32; 2],
    factor: f32,
    uv: [f32; 2],
) {
    if !ui.is_mouse_down(imgui::MouseButton::Left) {
        e.drag_axis = None;
    }
    let Some(index) = e.selected else {
        return;
    };
    if e.tool == 0 || e.playing {
        return;
    }
    let preview = e.timeline_editor.scene_preview.scene.as_ref().filter(|_| e.timeline_editor.open);
    let scene = preview.unwrap_or(&e.scene);
    let entity = &scene.actors[index];
    let center = entity.position;
    let parent = scene.parent_matrix(index);
    let previewing = preview.is_some();
    let view = e.view;
    let screen = |p| {
        let p = viewport::project(&view, parent.point(p));
        [
            position[0] + (p[0] - (1. - uv[0]) * 480.) * factor,
            position[1] + (p[1] - (1. - uv[1]) * 300.) * factor,
        ]
    };
    let origin = screen(center);
    let radius = 1.3;
    let mut paths = [Vec::new(), Vec::new(), Vec::new()];
    for (axis, path) in paths.iter_mut().enumerate() {
        if e.tool == 2 {
            for i in 0..=64 {
                let angle = i as f32 * std::f32::consts::TAU / 64.;
                let mut p = center;
                p[(axis + 1) % 3] += angle.cos() * radius;
                p[(axis + 2) % 3] += angle.sin() * radius;
                path.push(screen(p));
            }
        } else {
            let mut p = center;
            p[axis] += radius;
            path.extend([origin, screen(p)]);
        }
    }
    let mouse = ui.io().mouse_pos;
    let inside = mouse[0] > position[0] + 50.
        && mouse[1] > position[1]
        && mouse[0] < position[0] + size[0]
        && mouse[1] < position[1] + size[1];
    let hover = if inside {
        paths
            .iter()
            .enumerate()
            .map(|(i, path)| {
                (
                    i,
                    path.windows(2)
                        .map(|p| distance(mouse, p[0], p[1]))
                        .fold(f32::INFINITY, f32::min),
                )
            })
            .filter(|(_, d)| *d < 7.)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(i, _)| i)
    } else {
        None
    };
    if !previewing && ui.is_window_hovered() && !ui.io().key_alt && ui.is_mouse_clicked(imgui::MouseButton::Left) {
        e.drag_axis = hover;
    }
    let draw = ui.get_window_draw_list();
    draw.with_clip_rect(
        position,
        [position[0] + size[0], position[1] + size[1]],
        || {
            for (axis, path) in paths.iter().enumerate() {
                let color = if e.drag_axis == Some(axis) || hover == Some(axis) {
                    [1., 0.85, 0.2, 1.]
                } else {
                    [
                        [0.90, 0.31, 0.26, 1.],
                        [0.55, 0.84, 0.30, 1.],
                        [0.31, 0.58, 0.97, 1.],
                    ][axis]
                };
                for pair in path.windows(2) {
                    draw.add_line(pair[0], pair[1], color).thickness(2.).build();
                }
                if e.tool != 2 {
                    let end = path[1];
                    let d = [end[0] - origin[0], end[1] - origin[1]];
                    let len = d[0].hypot(d[1]).max(1.);
                    let u = [d[0] / len, d[1] / len];
                    if e.tool == 1 {
                        draw.add_triangle(
                            end,
                            [
                                end[0] - u[0] * 13. + u[1] * 4.,
                                end[1] - u[1] * 13. - u[0] * 4.,
                            ],
                            [
                                end[0] - u[0] * 13. - u[1] * 4.,
                                end[1] - u[1] * 13. + u[0] * 4.,
                            ],
                            color,
                        )
                        .filled(true)
                        .build();
                    } else {
                        draw.add_rect(
                            [end[0] - 4., end[1] - 4.],
                            [end[0] + 4., end[1] + 4.],
                            color,
                        )
                        .filled(true)
                        .build();
                    }
                }
            }
        },
    );
    if previewing { e.drag_axis = None; return; }
    if let Some(axis) = e.drag_axis
        && ui.is_mouse_dragging(imgui::MouseButton::Left)
    {
        let end = if e.tool == 2 {
            let mut p = center;
            p[axis] += radius;
            screen(p)
        } else {
            paths[axis][1]
        };
        let delta = ui.io().mouse_delta;
        let object = &mut e.scene.actors[index];
        if e.tool == 2 {
            let a = (mouse[1] - origin[1]).atan2(mouse[0] - origin[0]);
            let b = (mouse[1] - delta[1] - origin[1]).atan2(mouse[0] - delta[0] - origin[0]);
            let degrees = (a - b).sin().atan2((a - b).cos()).to_degrees();
            object.rotation[axis] = (object.rotation[axis] + degrees) % 360.;
        } else {
            let vector = [end[0] - origin[0], end[1] - origin[1]];
            let units = (delta[0] * vector[0] + delta[1] * vector[1])
                / (vector[0] * vector[0] + vector[1] * vector[1]).max(1.)
                * radius;
            if e.tool == 1 {
                object.position[axis] = (object.position[axis] + units).clamp(-128., 128.);
            } else {
                object.scale[axis] = (object.scale[axis] + units).clamp(0.01, 64.);
            }
        }
        // One gizmo drag is one undo step, however many frames it reports.
        e.changed_coalesced("gizmo-drag");
    }
    if e.drag_axis.is_none() || !ui.is_mouse_down(imgui::MouseButton::Left) {
        e.end_coalesced();
    }
}
fn distance(p: [f32; 2], a: [f32; 2], b: [f32; 2]) -> f32 {
    let d = [b[0] - a[0], b[1] - a[1]];
    let t = (((p[0] - a[0]) * d[0] + (p[1] - a[1]) * d[1])
        / (d[0] * d[0] + d[1] * d[1]).max(0.001))
    .clamp(0., 1.);
    (p[0] - a[0] - t * d[0]).hypot(p[1] - a[1] - t * d[1])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orientation_axes_follow_the_camera_basis() {
        let view = viewport::View {
            yaw: 0.,
            pitch: 0.,
            ..Default::default()
        };
        assert_eq!(
            orientation_endpoint(&view, 0, 1.).position,
            [
                ORIENTATION_CENTER[0] + ORIENTATION_RADIUS,
                ORIENTATION_CENTER[1]
            ]
        );
        assert_eq!(
            orientation_endpoint(&view, 1, 1.).position,
            [
                ORIENTATION_CENTER[0],
                ORIENTATION_CENTER[1] - ORIENTATION_RADIUS
            ]
        );
        assert_eq!(
            orientation_endpoint(&view, 2, 1.).position,
            ORIENTATION_CENTER
        );
    }

    #[test]
    fn orientation_hit_testing_selects_signed_ends_and_center() {
        let view = viewport::View::default();
        for axis in 0..3 {
            for sign in [-1., 1.] {
                let endpoint = orientation_endpoint(&view, axis, sign);
                assert_eq!(
                    orientation_target(&view, endpoint.position),
                    Some(OrientationTarget::Axis { axis, sign })
                );
            }
        }
        assert_eq!(
            orientation_target(&view, ORIENTATION_CENTER),
            Some(OrientationTarget::Isometric)
        );
        assert_eq!(orientation_target(&view, [0., 0.]), None);

        let aligned = viewport::View {
            yaw: 0.,
            pitch: 0.,
            ..Default::default()
        };
        assert_eq!(
            orientation_target(&aligned, ORIENTATION_CENTER),
            Some(OrientationTarget::Isometric)
        );
    }

    #[test]
    fn endpoint_actions_view_the_origin_from_the_selected_side() {
        for axis in 0..3 {
            for sign in [-1., 1.] {
                let mut view = viewport::View::default();
                apply_orientation_target(&mut view, OrientationTarget::Axis { axis, sign });
                let forward = view.basis()[2];
                assert!(forward[axis] * -sign > 0.999, "{axis} {sign}: {forward:?}");
            }
        }
    }
}
