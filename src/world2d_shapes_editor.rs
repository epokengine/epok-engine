//! The World2D content nodes in the 2D view and Inspector (World2D initiative, phase 3):
//! Path2D and PathFollow2D, Line2D, Polygon2D, Particles2D, PointLight2D, AudioPlayer2D,
//! VisibleOnScreenNotifier2D and RemoteTransform2D.
//!
//! Shapes are edited directly on the canvas: drag a point to move it, click the small
//! dot halfway along a segment to insert one, right-click a point to delete it, and
//! Ctrl+click (Cmd+click) empty space to add one at the end. A Path2D's selected point
//! shows curve handles. The Points row under the toolbar switches between editing,
//! adding and deleting, and opens or closes the shape.
use crate::{
    editor::Editor,
    world2d::{
        self, AudioPlayer2D, EmissionShape, Line2D, Notifier2D, Particles2D, Path2D, PathFollow2D,
        PathPoint, Polygon2D, RemoteTransform2D,
    },
    world2d_editor::{Affine, View},
};
use imgui::{MouseButton, Ui};
use uuid::Uuid;

pub const ICON_PATH: &str = "\u{f55b}";
pub const ICON_FOLLOW: &str = "\u{f4d7}";
pub const ICON_LINE: &str = "\u{f715}";
pub const ICON_POLYGON: &str = "\u{f5ee}";
pub const ICON_PARTICLES: &str = "\u{f06d}";
pub const ICON_LIGHT: &str = "\u{f0eb}";
pub const ICON_AUDIO: &str = "\u{f028}";
pub const ICON_NOTIFIER: &str = "\u{f06e}";
pub const ICON_REMOTE: &str = "\u{f0c1}";

const PATH_COLOUR: [f32; 4] = [0.40, 0.85, 1.0, 1.];
const POINT_COLOUR: [f32; 4] = [1.0, 0.62, 0.20, 1.];
const NOTIFIER_COLOUR: [f32; 4] = [0.95, 0.45, 0.85, 1.];
const REMOTE_COLOUR: [f32; 4] = [0.55, 0.85, 0.75, 1.];
const HANDLE_RADIUS: f32 = 5.;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PointMode {
    #[default]
    Edit,
    Add,
    Delete,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Grip {
    Point,
    In,
    Out,
}
#[derive(Clone, Copy, Debug)]
struct PointDrag {
    component: Uuid,
    index: usize,
    grip: Grip,
}

#[derive(Default)]
pub struct State {
    pub mode: PointMode,
    selected: Option<(Uuid, usize)>,
    drag: Option<PointDrag>,
}

/// The editable shape of an actor: the first Path2D, Line2D or Polygon2D.
#[derive(Clone, Debug)]
pub enum Shape {
    Path(Path2D),
    Line(Line2D),
    Polygon(Polygon2D),
    /// A NavigationRegion2D's navmesh outline.
    Navigation(crate::navigation2d::NavigationRegion2D),
}
impl Shape {
    fn points(&self) -> Vec<[f32; 2]> {
        match self {
            Shape::Path(p) => p.points.iter().map(|p| p.position).collect(),
            Shape::Line(l) => l.points.clone(),
            Shape::Polygon(p) => p.points.clone(),
            Shape::Navigation(r) => r.outline.clone(),
        }
    }
    fn closed(&self) -> bool {
        match self {
            Shape::Path(p) => p.closed,
            Shape::Line(l) => l.closed,
            Shape::Polygon(_) | Shape::Navigation(_) => true,
        }
    }
    fn label(&self) -> &'static str {
        match self {
            Shape::Path(_) => "Path2D",
            Shape::Line(_) => "Line2D",
            Shape::Polygon(_) => "Polygon2D",
            Shape::Navigation(_) => "Navmesh outline",
        }
    }
    fn set_point(&mut self, index: usize, value: [f32; 2]) {
        match self {
            Shape::Path(p) => {
                if let Some(point) = p.points.get_mut(index) {
                    point.position = value;
                }
            }
            Shape::Line(l) => {
                if let Some(point) = l.points.get_mut(index) {
                    *point = value;
                }
            }
            Shape::Polygon(p) => {
                if let Some(point) = p.points.get_mut(index) {
                    *point = value;
                }
            }
            Shape::Navigation(r) => {
                if let Some(point) = r.outline.get_mut(index) {
                    *point = value;
                }
            }
        }
    }
    fn insert(&mut self, index: usize, value: [f32; 2]) -> bool {
        match self {
            Shape::Path(p) => p.points.insert(index, PathPoint::at(value)),
            Shape::Line(l) => {
                if l.points.len() >= world2d::LINE_MAX_POINTS {
                    return false;
                }
                l.points.insert(index, value)
            }
            Shape::Polygon(p) => {
                if p.points.len() >= 256 {
                    return false;
                }
                p.points.insert(index, value)
            }
            Shape::Navigation(r) => {
                if r.outline.len() >= crate::navigation2d::MAX_OUTLINE {
                    return false;
                }
                r.outline.insert(index, value)
            }
        }
        true
    }
    fn remove(&mut self, index: usize) {
        let minimum = if matches!(self, Shape::Polygon(_) | Shape::Navigation(_)) {
            3
        } else {
            2
        };
        match self {
            Shape::Path(p) if p.points.len() > minimum => {
                p.points.remove(index);
            }
            Shape::Line(l) if l.points.len() > minimum => {
                l.points.remove(index);
            }
            Shape::Polygon(p) if p.points.len() > minimum => {
                p.points.remove(index);
            }
            Shape::Navigation(r) if r.outline.len() > minimum => {
                r.outline.remove(index);
            }
            _ => {}
        }
    }
}
/// The editable shape of the actor, with its component id.
pub fn shape_of(e: &Editor, actor: Uuid) -> Option<(Uuid, Shape)> {
    let index = e.scene.actor_index(actor)?;
    let actor = &e.scene.actors[index];
    for component in &actor.components {
        match component.class.class_id.as_deref() {
            Some(world2d::PATH2D) => {
                return Some((component.id, Shape::Path(world2d::read(component))));
            }
            // A line that follows nodes takes its points from them, not from the view.
            Some(world2d::LINE2D) => {
                let line: Line2D = world2d::read(component);
                if line.follow.is_empty() {
                    return Some((component.id, Shape::Line(line)));
                }
            }
            Some(world2d::POLYGON2D) => {
                return Some((component.id, Shape::Polygon(world2d::read(component))));
            }
            Some(world2d::NAVIGATION_REGION2D) => {
                let region: crate::navigation2d::NavigationRegion2D = world2d::read(component);
                if region.mode == crate::navigation2d::NavMode::Polygon {
                    return Some((component.id, Shape::Navigation(region)));
                }
            }
            _ => {}
        }
    }
    None
}
fn store(e: &mut Editor, actor: Uuid, component: Uuid, shape: &Shape) {
    let Some(index) = e.scene.actor_index(actor) else {
        return;
    };
    let Some(c) = e.scene.actors[index]
        .components
        .iter_mut()
        .find(|c| c.id == component)
    else {
        return;
    };
    match shape {
        Shape::Path(p) => world2d::write(c, p),
        Shape::Line(l) => world2d::write(c, l),
        Shape::Polygon(p) => world2d::write(c, p),
        Shape::Navigation(r) => world2d::write(c, r),
    }
    e.scene.actors[index].refresh_components();
}
/// Local bounds of the actor's shapes, for picking and framing.
pub fn local_bounds(e: &Editor, actor: Uuid) -> Option<([f32; 2], [f32; 2])> {
    let index = e.scene.actor_index(actor)?;
    let mut lo = [f32::MAX; 2];
    let mut hi = [f32::MIN; 2];
    let mut grow = |p: [f32; 2], r: f32| {
        for k in 0..2 {
            lo[k] = lo[k].min(p[k] - r);
            hi[k] = hi[k].max(p[k] + r);
        }
    };
    for component in &e.scene.actors[index].components {
        match component.class.class_id.as_deref() {
            Some(world2d::PATH2D) => {
                for p in crate::shape2d::sample_path(&world2d::read::<Path2D>(component)) {
                    grow(p, 2.);
                }
            }
            Some(world2d::LINE2D) => {
                let line: Line2D = world2d::read(component);
                for p in &line.points {
                    grow(*p, line.width * 0.5 + 1.);
                }
            }
            Some(world2d::POLYGON2D) => {
                for p in &world2d::read::<Polygon2D>(component).points {
                    grow(*p, 0.);
                }
            }
            Some(world2d::NAVIGATION_REGION2D) => {
                let r: crate::navigation2d::NavigationRegion2D = world2d::read(component);
                match r.mode {
                    crate::navigation2d::NavMode::Polygon => {
                        for p in &r.outline {
                            grow(*p, 0.);
                        }
                    }
                    crate::navigation2d::NavMode::Grid if r.tile_layer.is_none() => {
                        grow([0., 0.], 0.);
                        grow(r.size, 0.);
                    }
                    crate::navigation2d::NavMode::Grid => grow([0., 0.], 8.),
                }
            }
            Some(world2d::NOTIFIER2D) => {
                let r = world2d::read::<Notifier2D>(component).rect;
                grow([r[0], r[1]], 0.);
                grow([r[0] + r[2], r[1] + r[3]], 0.);
            }
            Some(world2d::PARTICLES2D) => {
                let p: Particles2D = world2d::read(component);
                match p.shape {
                    EmissionShape::Point => grow([0., 0.], 8.),
                    EmissionShape::Rectangle => {
                        grow([-p.extents[0].abs(), -p.extents[1].abs()], 2.);
                        grow([p.extents[0].abs(), p.extents[1].abs()], 2.);
                    }
                    EmissionShape::Circle => grow([0., 0.], p.extents[0].abs().max(8.)),
                }
            }
            _ => {}
        }
    }
    (lo[0] <= hi[0]).then_some((lo, hi))
}

// ---- points editing --------------------------------------------------------------------

/// Screen positions of a shape's points, segment midpoints and (for the selected path
/// point) curve handles.
struct Grips {
    points: Vec<[f32; 2]>,
    midpoints: Vec<(usize, [f32; 2], [f32; 2])>,
    handles: Vec<(usize, Grip, [f32; 2])>,
}
fn grips(
    shape: &Shape,
    world: &Affine,
    to_screen: &dyn Fn([f32; 2]) -> [f32; 2],
    selected: Option<usize>,
) -> Grips {
    let local = shape.points();
    let points: Vec<[f32; 2]> = local.iter().map(|p| to_screen(world.point(*p))).collect();
    let n = local.len();
    let segments = if shape.closed() && n > 2 {
        n
    } else {
        n.saturating_sub(1)
    };
    let midpoints = (0..segments)
        .map(|i| {
            let (a, b) = (local[i], local[(i + 1) % n]);
            let m = [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5];
            (i + 1, m, to_screen(world.point(m)))
        })
        .collect();
    let mut handles = Vec::new();
    if let (Shape::Path(path), Some(index)) = (shape, selected)
        && let Some(p) = path.points.get(index)
    {
        for (grip, offset) in [(Grip::In, p.handle_in), (Grip::Out, p.handle_out)] {
            let at = [p.position[0] + offset[0], p.position[1] + offset[1]];
            handles.push((index, grip, to_screen(world.point(at))));
        }
    }
    Grips {
        points,
        midpoints,
        handles,
    }
}
fn near(a: [f32; 2], b: [f32; 2], radius: f32) -> bool {
    (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) <= radius * radius
}
/// Point editing input for the selected actor. Returns true when it used the click, so
/// the node tools ignore it.
#[allow(clippy::too_many_arguments)]
pub fn input(
    ui: &Ui,
    e: &mut Editor,
    view: &View,
    to_screen: &dyn Fn([f32; 2]) -> [f32; 2],
    hovered: bool,
    world: [f32; 2],
    snap: &dyn Fn([f32; 2]) -> [f32; 2],
) -> bool {
    let Some((component, mut shape)) = shape_of(e, view.id) else {
        return false;
    };
    let Some(inverse) = view.world.inverse() else {
        return false;
    };
    let mouse = ui.io().mouse_pos;
    let selected = e
        .world2d
        .points
        .selected
        .filter(|s| s.0 == component)
        .map(|s| s.1);
    let g = grips(&shape, &view.world, to_screen, selected);
    let command = ui.io().key_ctrl || ui.io().key_super;
    // Continue a drag.
    if let Some(drag) = e.world2d.points.drag {
        if !ui.is_mouse_down(MouseButton::Left) {
            e.world2d.points.drag = None;
            e.end_coalesced();
            return true;
        }
        let target = if ui.io().key_alt { world } else { snap(world) };
        let local = inverse.point(target);
        match (drag.grip, &mut shape) {
            (Grip::Point, _) => shape.set_point(drag.index, local),
            (grip, Shape::Path(path)) => {
                if let Some(p) = path.points.get_mut(drag.index) {
                    let offset = [local[0] - p.position[0], local[1] - p.position[1]];
                    let mirrored = [-offset[0], -offset[1]];
                    // Handles stay mirrored unless Shift breaks them apart.
                    let free = ui.io().key_shift;
                    if grip == Grip::Out {
                        p.handle_out = offset;
                        if !free {
                            p.handle_in = mirrored;
                        }
                    } else {
                        p.handle_in = offset;
                        if !free {
                            p.handle_out = mirrored;
                        }
                    }
                }
            }
            _ => {}
        }
        store(e, view.id, drag.component, &shape);
        e.changed_coalesced("world2d-points");
        return true;
    }
    if !hovered || e.playing {
        return false;
    }
    let mode = e.world2d.points.mode;
    // Right-click deletes a point in any mode.
    if ui.is_mouse_clicked(MouseButton::Right)
        && let Some(index) = g
            .points
            .iter()
            .position(|p| near(mouse, *p, HANDLE_RADIUS + 2.))
    {
        shape.remove(index);
        store(e, view.id, component, &shape);
        e.world2d.points.selected = None;
        e.changed();
        return true;
    }
    if !ui.is_mouse_clicked(MouseButton::Left) {
        return false;
    }
    if let Some(&(index, grip, _)) = g.handles.iter().find(|h| near(mouse, h.2, HANDLE_RADIUS)) {
        e.world2d.points.drag = Some(PointDrag {
            component,
            index,
            grip,
        });
        return true;
    }
    if let Some(index) = g
        .points
        .iter()
        .position(|p| near(mouse, *p, HANDLE_RADIUS + 2.))
    {
        if mode == PointMode::Delete {
            shape.remove(index);
            store(e, view.id, component, &shape);
            e.world2d.points.selected = None;
            e.changed();
            return true;
        }
        e.world2d.points.selected = Some((component, index));
        e.world2d.points.drag = Some(PointDrag {
            component,
            index,
            grip: Grip::Point,
        });
        return true;
    }
    if mode != PointMode::Delete
        && let Some(&(index, local, _)) =
            g.midpoints.iter().find(|m| near(mouse, m.2, HANDLE_RADIUS))
    {
        if shape.insert(index, local) {
            store(e, view.id, component, &shape);
            e.changed_coalesced("world2d-points");
            e.world2d.points.selected = Some((component, index));
            e.world2d.points.drag = Some(PointDrag {
                component,
                index,
                grip: Grip::Point,
            });
        }
        return true;
    }
    if mode == PointMode::Add || (mode == PointMode::Edit && command) {
        let local = inverse.point(snap(world));
        let index = shape.points().len();
        if shape.insert(index, local) {
            store(e, view.id, component, &shape);
            e.changed();
            e.world2d.points.selected = Some((component, index));
        }
        return true;
    }
    false
}
/// Draws the selected shape's points, handles and insertion dots.
pub fn overlay(
    e: &Editor,
    draw: &imgui::DrawListMut,
    view: &View,
    to_screen: &dyn Fn([f32; 2]) -> [f32; 2],
) {
    let Some((component, shape)) = shape_of(e, view.id) else {
        return;
    };
    let selected = e
        .world2d
        .points
        .selected
        .filter(|s| s.0 == component)
        .map(|s| s.1);
    let g = grips(&shape, &view.world, to_screen, selected);
    for (index, grip, at) in &g.handles {
        let base = g.points[*index];
        draw.add_line(base, *at, [1., 1., 1., 0.6]).build();
        let half = HANDLE_RADIUS * 0.7;
        draw.add_rect(
            [at[0] - half, at[1] - half],
            [at[0] + half, at[1] + half],
            [0.12, 0.12, 0.14, 1.],
        )
        .filled(true)
        .build();
        draw.add_rect(
            [at[0] - half, at[1] - half],
            [at[0] + half, at[1] + half],
            if *grip == Grip::Out {
                PATH_COLOUR
            } else {
                [0.8, 0.8, 0.85, 1.]
            },
        )
        .build();
    }
    if e.world2d.points.mode != PointMode::Delete {
        for (_, _, at) in &g.midpoints {
            draw.add_circle(*at, 3., [1., 1., 1., 0.45])
                .filled(true)
                .build();
        }
    }
    for (i, p) in g.points.iter().enumerate() {
        let active = selected == Some(i);
        let colour = if e.world2d.points.mode == PointMode::Delete {
            [0.95, 0.35, 0.35, 1.]
        } else {
            POINT_COLOUR
        };
        draw.add_circle(*p, HANDLE_RADIUS, [0.12, 0.12, 0.14, 1.])
            .filled(true)
            .build();
        draw.add_circle(*p, HANDLE_RADIUS, colour)
            .thickness(if active { 2.5 } else { 1.5 })
            .build();
        if active {
            draw.add_circle(*p, 2., colour).filled(true).build();
        }
    }
}
/// The Points row under the 2D toolbar for the selected shape.
pub fn toolbar(ui: &Ui, e: &mut Editor) {
    let Some(actor) = e.selected_actor else {
        return;
    };
    let Some((component, mut shape)) = shape_of(e, actor) else {
        return;
    };
    let _spacing = ui.push_style_var(imgui::StyleVar::ItemSpacing([2., 4.]));
    let start = ui.cursor_screen_pos();
    let width = ui.content_region_avail()[0];
    ui.get_window_draw_list()
        .add_rect(
            [start[0] - 4., start[1] - 2.],
            [start[0] + width + 4., start[1] + 25.],
            [0.40, 0.85, 1.0, 0.06],
        )
        .filled(true)
        .rounding(4.)
        .build();
    ui.align_text_to_frame_padding();
    ui.text_colored(PATH_COLOUR, format!("{ICON_PATH} {}", shape.label()));
    ui.same_line_with_spacing(0., 8.);
    for (mode, glyph, tip, detail) in [
        (
            PointMode::Edit,
            crate::world2d_editor::ICON_SELECT,
            "Edit points",
            "Drag points; click a segment's dot to insert one; right-click deletes; Ctrl+click adds at the end",
        ),
        (
            PointMode::Add,
            crate::world2d_editor::ICON_PLUS,
            "Add points",
            "Click to add a point after the last one",
        ),
        (
            PointMode::Delete,
            crate::world2d_editor::ICON_TRASH,
            "Delete points",
            "Click a point to remove it",
        ),
    ] {
        let pressed = crate::gui::icon(
            ui,
            glyph,
            &format!("points-{tip}"),
            "",
            e.world2d.points.mode == mode,
        );
        if ui.is_item_hovered() {
            ui.tooltip(|| {
                ui.text(tip);
                ui.text_disabled(detail);
            });
        }
        if pressed {
            e.world2d.points.mode = mode;
        }
        ui.same_line();
    }
    ui.same_line_with_spacing(0., 10.);
    let mut changed = false;
    match &mut shape {
        Shape::Path(path) => {
            changed |= ui.checkbox("Closed", &mut path.closed);
            ui.same_line();
            if ui.small_button("Smooth curve") {
                crate::shape2d::smooth(path);
                changed = true;
            }
            if ui.is_item_hovered() {
                ui.tooltip_text("Curve handles through every point");
            }
            ui.same_line();
            if ui.small_button("Straighten") {
                for p in &mut path.points {
                    p.handle_in = [0.; 2];
                    p.handle_out = [0.; 2];
                }
                changed = true;
            }
        }
        Shape::Line(line) => {
            changed |= ui.checkbox("Closed", &mut line.closed);
        }
        Shape::Polygon(_) | Shape::Navigation(_) => {}
    }
    ui.same_line_with_spacing(0., 10.);
    ui.text_disabled(format!("{} points", shape.points().len()));
    if changed {
        store(e, actor, component, &shape);
        e.changed();
    }
    ui.dummy([0., 0.]);
}
/// Delete removes the selected point instead of the node.
pub fn delete_selected_point(e: &mut Editor) -> bool {
    let Some(actor) = e.selected_actor else {
        return false;
    };
    let Some((component, mut shape)) = shape_of(e, actor) else {
        return false;
    };
    let Some((selected, index)) = e.world2d.points.selected else {
        return false;
    };
    if selected != component {
        return false;
    }
    shape.remove(index);
    store(e, actor, component, &shape);
    e.world2d.points.selected = None;
    e.changed();
    true
}

// ---- drawing ---------------------------------------------------------------------------

fn colour_of(value: [u8; 3], alpha: f32) -> [f32; 4] {
    [
        f32::from(value[0]) / 255.,
        f32::from(value[1]) / 255.,
        f32::from(value[2]) / 255.,
        alpha,
    ]
}
fn mix(a: [u8; 3], b: [u8; 3], t: f32) -> [u8; 3] {
    [0, 1, 2].map(|k| (f32::from(a[k]) + (f32::from(b[k]) - f32::from(a[k])) * t).round() as u8)
}
/// Draws the content nodes of one actor (shapes, emitters, gizmos). `selected` adds the
/// path direction and emission outlines.
#[allow(clippy::too_many_arguments)]
pub fn draw_view(
    e: &mut Editor,
    draw: &imgui::DrawListMut,
    view: &View,
    to_screen: &dyn Fn([f32; 2]) -> [f32; 2],
    zoom: f32,
    seconds: f32,
    animate: bool,
    tint: [f32; 3],
    fade: f32,
    selected: bool,
    gizmos: bool,
    offset: [f32; 2],
) -> bool {
    let Some(index) = e.scene.actor_index(view.id) else {
        return false;
    };
    // Parallax copies draw shifted in world pixels.
    let shifted = |p: [f32; 2]| to_screen([p[0] + offset[0], p[1] + offset[1]]);
    let to_screen: &dyn Fn([f32; 2]) -> [f32; 2] = &shifted;
    let components = e.scene.actors[index].components.clone();
    let scale = {
        let m = view.world.m;
        ((m[0][0].powi(2) + m[1][0].powi(2)).sqrt()).max((m[0][1].powi(2) + m[1][1].powi(2)).sqrt())
    };
    let mut drew = false;
    let world = |p: [f32; 2]| to_screen(view.world.point(p));
    for component in &components {
        match component.class.class_id.as_deref() {
            Some(world2d::POLYGON2D) => {
                let poly: Polygon2D = world2d::read(component);
                if !poly.visible {
                    continue;
                }
                let alpha = if poly.blend == crate::texture::BlendMode::Cutout {
                    1.
                } else {
                    0.6
                } * fade;
                let c = colour_of(poly.color, alpha);
                let colour = [c[0] * tint[0], c[1] * tint[1], c[2] * tint[2], c[3]];
                let texture = poly.texture.and_then(|t| e.world2d.textures.get(t));
                match crate::shape2d::triangulate(&poly.points) {
                    Ok(triangles) => {
                        for t in triangles {
                            let p = t.map(|i| world(poly.points[usize::from(i)]));
                            if let Some((id, size)) = texture {
                                let uv = t.map(|i| {
                                    let v = poly.points[usize::from(i)];
                                    [
                                        (v[0] + poly.texture_offset[0]) / f32::from(size[0]),
                                        (v[1] + poly.texture_offset[1]) / f32::from(size[1]),
                                    ]
                                });
                                draw.add_image_quad(id, p[0], p[1], p[2], p[2])
                                    .uv(uv[0], uv[1], uv[2], uv[2])
                                    .col(colour)
                                    .build();
                            } else {
                                draw.add_triangle(p[0], p[1], p[2], colour)
                                    .filled(true)
                                    .build();
                            }
                        }
                    }
                    Err(_) => {
                        // An invalid outline draws as a red frame until it is fixed.
                        let outline: Vec<_> = poly.points.iter().map(|p| world(*p)).collect();
                        draw.add_polyline(outline, [1., 0.35, 0.35, fade])
                            .thickness(2.)
                            .build();
                    }
                }
                drew = true;
            }
            Some(world2d::LINE2D) => {
                let mut line: Line2D = world2d::read(component);
                line.points = line_points(e, view, &line);
                if !line.visible || line.points.len() < 2 {
                    continue;
                }
                let n = line.points.len();
                let segments = if line.closed && n > 2 { n } else { n - 1 };
                let width = (line.width * scale * zoom).max(1.);
                for i in 0..segments {
                    let (a, b) = (world(line.points[i]), world(line.points[(i + 1) % n]));
                    let t = (i as f32 + 0.5) / segments as f32;
                    let c = colour_of(mix(line.begin_color, line.end_color, t), fade);
                    draw.add_line(a, b, [c[0] * tint[0], c[1] * tint[1], c[2] * tint[2], c[3]])
                        .thickness(width)
                        .build();
                    // Joints: a disc closes the corner like the console's square caps.
                    if i + 1 < n || line.closed {
                        draw.add_circle(
                            b,
                            width * 0.5,
                            [c[0] * tint[0], c[1] * tint[1], c[2] * tint[2], c[3]],
                        )
                        .filled(true)
                        .build();
                    }
                }
                drew = true;
            }
            Some(world2d::PATH2D) => {
                let path: Path2D = world2d::read(component);
                let points: Vec<_> = crate::shape2d::sample_path(&path)
                    .into_iter()
                    .map(world)
                    .collect();
                if gizmos || selected {
                    let colour = [
                        PATH_COLOUR[0],
                        PATH_COLOUR[1],
                        PATH_COLOUR[2],
                        if selected { 1. } else { 0.55 } * fade,
                    ];
                    draw.add_polyline(points.clone(), colour)
                        .thickness(if selected { 2. } else { 1.5 })
                        .build();
                    // An arrow halfway along shows which way followers travel.
                    let lengths =
                        crate::shape2d::cumulative_lengths(&crate::shape2d::sample_path(&path));
                    let samples = crate::shape2d::sample_path(&path);
                    if let Some(total) = lengths.last().copied().filter(|t| *t > 1.) {
                        let (p, d) =
                            crate::shape2d::sample_at(&samples, &lengths, total * 0.5, false);
                        let tip = world([p[0] + d[0] * 6., p[1] + d[1] * 6.]);
                        let left =
                            world([p[0] - d[0] * 4. - d[1] * 4., p[1] - d[1] * 4. + d[0] * 4.]);
                        let right =
                            world([p[0] - d[0] * 4. + d[1] * 4., p[1] - d[1] * 4. - d[0] * 4.]);
                        draw.add_triangle(tip, left, right, colour)
                            .filled(true)
                            .build();
                    }
                }
                drew = true;
            }
            Some(world2d::PATH_FOLLOW2D) if gizmos => {
                // Where the follower starts on its path, joined to the node.
                let follow: PathFollow2D = world2d::read(component);
                if let Some((path_world, path)) = follow_path(e, view, &follow) {
                    let samples = crate::shape2d::sample_path(&path);
                    let lengths = crate::shape2d::cumulative_lengths(&samples);
                    let (p, _) = crate::shape2d::sample_at(
                        &samples,
                        &lengths,
                        follow.progress + follow.offset[0],
                        path.closed,
                    );
                    let at = to_screen(path_world.point(p));
                    let here = to_screen(view.world.t);
                    crate::world2d_editor::dashed(draw, here, at, [1., 1., 1., 0.4 * fade], 4.);
                    draw.add_circle(at, 4., PATH_COLOUR).thickness(1.5).build();
                }
            }
            Some(world2d::PARTICLES2D) => {
                let p: Particles2D = world2d::read(component);
                if !p.visible {
                    continue;
                }
                let texture = p.texture.and_then(|t| e.world2d.textures.get(t));
                let time = if animate { seconds } else { p.lifetime * 0.6 };
                for particle in crate::shape2d::preview(&p, time) {
                    let local = if p.local_coords {
                        view.world.point(particle.position)
                    } else {
                        [
                            view.world.t[0] + particle.position[0],
                            view.world.t[1] + particle.position[1],
                        ]
                    };
                    let at = to_screen(local);
                    let c = colour_of(
                        mix(p.start_color, p.end_color, particle.age),
                        fade * if p.blend == crate::texture::BlendMode::Cutout {
                            1.
                        } else {
                            0.75
                        },
                    );
                    let colour = [c[0] * tint[0], c[1] * tint[1], c[2] * tint[2], c[3]];
                    let size = p.scale[0] + (p.scale[1] - p.scale[0]) * particle.age;
                    match texture {
                        Some((id, tex)) => {
                            let mut region = p.region;
                            if region[2] == 0 {
                                region[2] = tex[0].saturating_sub(region[0]);
                            }
                            if region[3] == 0 {
                                region[3] = tex[1].saturating_sub(region[1]);
                            }
                            let frames = p.hframes.max(1);
                            let cell = region[2] / frames;
                            let frame = ((particle.age * f32::from(frames)) as u16).min(frames - 1);
                            let (w, h) = (
                                f32::from(cell) * size * zoom * 0.5,
                                f32::from(region[3]) * size * zoom * 0.5,
                            );
                            let u0 = f32::from(region[0] + frame * cell) / f32::from(tex[0]);
                            let u1 = f32::from(region[0] + (frame + 1) * cell) / f32::from(tex[0]);
                            let v0 = f32::from(region[1]) / f32::from(tex[1]);
                            let v1 = f32::from(region[1] + region[3]) / f32::from(tex[1]);
                            draw.add_image(id, [at[0] - w, at[1] - h], [at[0] + w, at[1] + h])
                                .uv_min([u0, v0])
                                .uv_max([u1, v1])
                                .col(colour)
                                .build();
                        }
                        None => {
                            draw.add_circle(at, (2. * size * zoom).max(1.), colour)
                                .filled(true)
                                .build();
                        }
                    }
                }
                if selected && p.shape != EmissionShape::Point {
                    let colour = [1., 0.8, 0.3, 0.7];
                    if p.shape == EmissionShape::Rectangle {
                        let corners = [[-1., -1.], [1., -1.], [1., 1.], [-1., 1.]]
                            .map(|c| world([c[0] * p.extents[0], c[1] * p.extents[1]]));
                        for i in 0..4 {
                            crate::world2d_editor::dashed(
                                draw,
                                corners[i],
                                corners[(i + 1) % 4],
                                colour,
                                4.,
                            );
                        }
                    } else {
                        draw.add_circle(world([0., 0.]), p.extents[0] * scale * zoom, colour)
                            .num_segments(32)
                            .build();
                    }
                }
                drew = true;
            }
            Some(world2d::NOTIFIER2D) if gizmos => {
                let n: Notifier2D = world2d::read(component);
                let r = n.rect;
                let corners = [
                    [r[0], r[1]],
                    [r[0] + r[2], r[1]],
                    [r[0] + r[2], r[1] + r[3]],
                    [r[0], r[1] + r[3]],
                ]
                .map(world);
                for i in 0..4 {
                    crate::world2d_editor::dashed(
                        draw,
                        corners[i],
                        corners[(i + 1) % 4],
                        [
                            NOTIFIER_COLOUR[0],
                            NOTIFIER_COLOUR[1],
                            NOTIFIER_COLOUR[2],
                            0.8 * fade,
                        ],
                        4.,
                    );
                }
                draw.add_text(
                    [corners[0][0] + 2., corners[0][1] + 1.],
                    NOTIFIER_COLOUR,
                    ICON_NOTIFIER,
                );
                drew = true;
            }
            Some(world2d::REMOTE_TRANSFORM2D) if gizmos => {
                let r: RemoteTransform2D = world2d::read(component);
                if let Some(target) = r.target.and_then(|t| e.scene.actor_index(t)) {
                    let to = to_screen(world2d::world_position(&e.scene, target));
                    let from = to_screen(view.world.t);
                    draw.add_line(
                        from,
                        to,
                        [
                            REMOTE_COLOUR[0],
                            REMOTE_COLOUR[1],
                            REMOTE_COLOUR[2],
                            0.8 * fade,
                        ],
                    )
                    .thickness(1.5)
                    .build();
                    draw.add_circle(to, 4., REMOTE_COLOUR)
                        .thickness(1.5)
                        .build();
                }
                let at = to_screen(view.world.t);
                draw.add_text([at[0] - 7., at[1] - 7.], REMOTE_COLOUR, ICON_REMOTE);
                drew = true;
            }
            Some(world2d::AUDIO_PLAYER2D) if gizmos => {
                let a: AudioPlayer2D = world2d::read(component);
                let at = to_screen(view.world.t);
                draw.add_text([at[0] - 7., at[1] - 7.], [0.9, 0.85, 0.5, fade], ICON_AUDIO);
                if selected && a.max_distance > 0. {
                    draw.add_circle(at, a.max_distance * zoom, [0.9, 0.85, 0.5, 0.35])
                        .num_segments(64)
                        .build();
                }
                drew = true;
            }
            _ => {}
        }
    }
    drew
}
/// The Path2D a follower uses (explicit, or the actor it is attached to) and its world
/// transform.
fn follow_path(e: &Editor, view: &View, follow: &PathFollow2D) -> Option<(Affine, Path2D)> {
    let target = follow.path.or_else(|| {
        let index = e.scene.actor_index(view.id)?;
        e.scene.actors[index].attach.as_ref().map(|a| a.actor)
    })?;
    let index = e.scene.actor_index(target)?;
    let path: Path2D = world2d::read(world2d::component(&e.scene.actors[index], world2d::PATH2D)?);
    let (_, matrix) = crate::world2d_editor::world_of_index(&e.scene, index);
    Some((matrix, path))
}

// ---- inspector -------------------------------------------------------------------------

/// Edits a record through `edit`, committing when it changed.
fn record<T: serde::de::DeserializeOwned + serde::Serialize + Default + PartialEq + Clone>(
    e: &mut Editor,
    actor: Uuid,
    component: &crate::actor_document::ComponentInstance,
    edit: impl FnOnce(&mut Editor, &mut T),
) {
    let mut value: T = world2d::read(component);
    let before = value.clone();
    edit(e, &mut value);
    if value != before
        && let Some(index) = e.scene.actor_index(actor)
        && let Some(c) = e.scene.actors[index]
            .components
            .iter_mut()
            .find(|c| c.id == component.id)
    {
        world2d::write(c, &value);
        e.scene.actors[index].refresh_components();
        e.changed_coalesced("world2d-inspector");
    }
}
fn points_list(ui: &Ui, points: &mut Vec<[f32; 2]>) {
    if let Some(_node) = ui.tree_node(format!("Points ({})", points.len())) {
        let mut remove = None;
        for (i, p) in points.iter_mut().enumerate() {
            let _id = ui.push_id_usize(i);
            ui.set_next_item_width(ui.content_region_avail()[0] - 30.);
            crate::gui::Drag::new(format!("##{i}"))
                .speed(0.5)
                .display_format("%.1f")
                .build_array(ui, p);
            ui.same_line();
            if ui.small_button("x") {
                remove = Some(i);
            }
        }
        if let Some(i) = remove {
            points.remove(i);
        }
    }
}
/// The nodes a Line2D runs through: a rope between moving bodies, a chain, a beam.
fn follow_list(ui: &Ui, e: &Editor, actor: Uuid, follow: &mut Vec<Uuid>) {
    let label = if follow.is_empty() {
        "Follow nodes".to_string()
    } else {
        format!("Follow nodes ({})", follow.len())
    };
    if let Some(_node) = ui.tree_node(label) {
        let mut remove = None;
        for (i, id) in follow.iter_mut().enumerate() {
            let _id = ui.push_id_usize(i);
            let mut value = Some(*id);
            let any = |_: &crate::scene::Actor| true;
            if actor_combo(
                ui,
                e,
                &format!("Node {}", i + 1),
                "(remove)",
                actor,
                &any,
                &mut value,
            ) {
                match value {
                    Some(v) => *id = v,
                    None => remove = Some(i),
                }
            }
        }
        if let Some(i) = remove {
            follow.remove(i);
        }
        if follow.len() < world2d::LINE_FOLLOW_MAX && ui.small_button("Add node") {
            // Start from the selection's last node, or the line's own node.
            follow.push(follow.last().copied().unwrap_or(actor));
        }
        crate::gui::muted(
            ui,
            "The line runs through these nodes in order and follows them while the game runs: ropes on rigid bodies, chains, beams between moving things.",
        );
    }
}
/// The line's points: its own, or its followed nodes in its local pixels.
pub(crate) fn line_points(e: &Editor, view: &View, line: &Line2D) -> Vec<[f32; 2]> {
    if line.follow.is_empty() {
        return line.points.clone();
    }
    let Some(inverse) = view.world.inverse() else {
        return line.points.clone();
    };
    line.follow
        .iter()
        .filter_map(|id| e.scene.actor_index(*id))
        .map(|i| inverse.point(world2d::world_position(&e.scene, i)))
        .collect()
}
/// A combo of the map's 2D actors (other than `exclude`), with a "none" entry.
pub(crate) fn actor_combo(
    ui: &Ui,
    e: &Editor,
    label: &str,
    none: &str,
    exclude: Uuid,
    filter: &dyn Fn(&crate::scene::Actor) -> bool,
    value: &mut Option<Uuid>,
) -> bool {
    let current = value
        .and_then(|id| e.scene.actors.iter().find(|a| a.id == id))
        .map_or_else(|| none.to_string(), |a| a.name.clone());
    let mut changed = false;
    let combo = ui.begin_combo(crate::gui::field(ui, label), current);
    #[cfg(test)]
    crate::gui::record_script_control(ui, label);
    if let Some(_combo) = combo {
        let mut choices: Vec<_> = e
            .scene
            .actors
            .iter()
            .filter(|a| a.id != exclude && filter(a))
            .collect();
        // A map's worth of nodes gets a search; names starting with it come first.
        let (query, enter) = if choices.len() > 8 {
            crate::world2d_editor::combo_filter(ui, "Search nodes")
        } else {
            (String::new(), false)
        };
        if !query.is_empty() {
            choices.retain(|a| a.name.to_lowercase().contains(&query));
            choices.sort_by_key(|a| !a.name.to_lowercase().starts_with(&query));
        } else if ui.selectable_config(none).selected(value.is_none()).build() {
            *value = None;
            changed = true;
        }
        if enter && let Some(first) = choices.first() {
            *value = Some(first.id);
            changed = true;
            ui.close_current_popup();
        }
        for actor in choices {
            let picked = ui
                .selectable_config(format!("{}##{}", actor.name, actor.id))
                .selected(*value == Some(actor.id))
                .build();
            #[cfg(test)]
            crate::gui::record_script_control(ui, &format!("{label}: {}", actor.name));
            if picked {
                *value = Some(actor.id);
                changed = true;
            }
        }
    }
    changed
}
fn texture_combo(ui: &Ui, e: &Editor, label: &str, value: &mut Option<Uuid>) {
    let mut textures: Vec<_> = e
        .assets
        .index
        .usable()
        .filter(|r| r.meta.kind == crate::assets::Kind::Texture)
        .map(|r| (r.meta.id, r.meta.source.clone()))
        .collect();
    textures.sort_by(|a, b| a.1.cmp(&b.1));
    let current = value
        .and_then(|id| textures.iter().find(|t| t.0 == id))
        .map_or_else(|| "None".to_string(), |t| t.1.clone());
    if let Some(_combo) = ui.begin_combo(crate::gui::field(ui, label), current) {
        let (query, enter) = if textures.len() > 8 {
            crate::world2d_editor::combo_filter(ui, "Search textures")
        } else {
            (String::new(), false)
        };
        if query.is_empty()
            && ui
                .selectable_config("None")
                .selected(value.is_none())
                .build()
        {
            *value = None;
        }
        let mut textures = textures;
        textures.retain(|(_, name)| name.to_lowercase().contains(&query));
        if enter && let Some((first, _)) = textures.first() {
            *value = Some(*first);
            ui.close_current_popup();
        }
        for (id, name) in textures {
            if ui
                .selectable_config(&name)
                .selected(*value == Some(id))
                .build()
            {
                *value = Some(id);
            }
        }
    }
}
fn blend_combo(ui: &Ui, value: &mut crate::texture::BlendMode) {
    use crate::texture::BlendMode;
    let modes = [
        BlendMode::Cutout,
        BlendMode::Average,
        BlendMode::Add,
        BlendMode::Subtract,
        BlendMode::AddQuarter,
    ];
    let mut mode = modes.iter().position(|m| m == value).unwrap_or(0);
    if ui.combo_simple_string(
        crate::gui::field(ui, "Blend"),
        &mut mode,
        &[
            "Opaque (cutout)",
            "Average 50%",
            "Add",
            "Subtract",
            "Add 25%",
        ],
    ) {
        *value = modes[mode];
    }
}
fn z_drag(ui: &Ui, value: &mut i16) {
    let mut z = i32::from(*value);
    if crate::gui::Drag::new(crate::gui::field(ui, "Z Index"))
        .range(-4096, 4096)
        .speed(0.2)
        .build(ui, &mut z)
    {
        *value = z.clamp(-4096, 4096) as i16;
    }
}
fn colour_u8(ui: &Ui, label: &str, value: &mut [u8; 3]) {
    let mut c = value.map(|v| f32::from(v) / 255.);
    if ui.color_edit3(crate::gui::field(ui, label), &mut c) {
        *value = c.map(|v| (v * 255.).round().clamp(0., 255.) as u8);
    }
}
fn vec2(ui: &Ui, label: &str, value: &mut [f32; 2], speed: f32) {
    crate::gui::Drag::new(crate::gui::field(ui, label))
        .speed(speed)
        .display_format("%.1f")
        .build_array(ui, value);
}

/// Inspector bands for the content nodes; returns false for components it does not own.
pub fn band(
    ui: &Ui,
    e: &mut Editor,
    actor: Uuid,
    component: &crate::actor_document::ComponentInstance,
) -> bool {
    let id = component.class.class_id.as_deref().unwrap_or("");
    let (glyph, title) = match id {
        world2d::PATH2D => (ICON_PATH, "Path2D"),
        world2d::PATH_FOLLOW2D => (ICON_FOLLOW, "PathFollow2D"),
        world2d::LINE2D => (ICON_LINE, "Line2D"),
        world2d::POLYGON2D => (ICON_POLYGON, "Polygon2D"),
        world2d::PARTICLES2D => (ICON_PARTICLES, "Particles2D"),
        world2d::AUDIO_PLAYER2D => (ICON_AUDIO, "AudioPlayer2D"),
        world2d::NOTIFIER2D => (ICON_NOTIFIER, "VisibleOnScreenNotifier2D"),
        world2d::REMOTE_TRANSFORM2D => (ICON_REMOTE, "RemoteTransform2D"),
        _ => return false,
    };
    let (mut remove, mut reset) = (false, false);
    let open = crate::gui::section(ui, &format!("{glyph} {title}"), || {
        if crate::gui::script_menu_item(ui, "Reset to Defaults") {
            reset = true;
        }
        if crate::gui::script_menu_item(ui, "Remove Component") {
            remove = true;
        }
    });
    if open {
        match id {
            world2d::PATH2D => record::<Path2D>(e, actor, component, |_, p| {
                crate::gui::toggle(ui, "Closed", &mut p.closed);
                let samples = crate::shape2d::sample_path(p);
                let length = crate::shape2d::cumulative_lengths(&samples)
                    .last()
                    .copied()
                    .unwrap_or(0.);
                ui.text_disabled(format!("{} points, {length:.0} px long", p.points.len()));
                if ui.small_button("Smooth curve") {
                    crate::shape2d::smooth(p);
                }
                ui.same_line();
                if ui.small_button("Straighten") {
                    for point in &mut p.points {
                        point.handle_in = [0.; 2];
                        point.handle_out = [0.; 2];
                    }
                }
                let mut positions: Vec<[f32; 2]> = p.points.iter().map(|p| p.position).collect();
                points_list(ui, &mut positions);
                if positions.len() == p.points.len() {
                    for (point, position) in p.points.iter_mut().zip(positions) {
                        point.position = position;
                    }
                } else {
                    p.points = positions.into_iter().map(PathPoint::at).collect();
                }
                crate::gui::muted(
                    ui,
                    "Drag points in the view; click a segment's dot to insert, right-click to delete. Attach nodes with a PathFollow2D to move them along it.",
                );
            }),
            world2d::PATH_FOLLOW2D => record::<PathFollow2D>(e, actor, component, |e, f| {
                let has_path =
                    |a: &crate::scene::Actor| world2d::component(a, world2d::PATH2D).is_some();
                actor_combo(
                    ui,
                    e,
                    "Path",
                    "The node this one is attached to",
                    actor,
                    &has_path,
                    &mut f.path,
                );
                crate::gui::Drag::new(crate::gui::field(ui, "Progress (px)"))
                    .speed(0.5)
                    .display_format("%.1f")
                    .build(ui, &mut f.progress);
                crate::gui::Drag::new(crate::gui::field(ui, "Speed (px/s)"))
                    .speed(0.5)
                    .display_format("%.1f")
                    .build(ui, &mut f.speed);
                vec2(ui, "Offset along / across", &mut f.offset, 0.5);
                let mut mode = if f.ping_pong {
                    1
                } else if f.looping {
                    0
                } else {
                    2
                };
                if crate::world2d_editor::segmented(
                    ui,
                    "At the ends",
                    &["Loop", "Ping-pong", "Stop"],
                    &mut mode,
                ) {
                    f.looping = mode == 0;
                    f.ping_pong = mode == 1;
                }
                crate::gui::toggle(ui, "Rotate along path", &mut f.rotates);
                crate::gui::toggle(ui, "Moving", &mut f.moving);
                crate::gui::muted(
                    ui,
                    "Bodies standing on this node's solid collider ride along: moving platforms.",
                );
            }),
            world2d::LINE2D => record::<Line2D>(e, actor, component, |e, l| {
                crate::gui::Drag::new(crate::gui::field(ui, "Width (px)"))
                    .range(0., 256.)
                    .speed(0.1)
                    .display_format("%.1f")
                    .build(ui, &mut l.width);
                colour_u8(ui, "Start colour", &mut l.begin_color);
                colour_u8(ui, "End colour", &mut l.end_color);
                crate::gui::toggle(ui, "Closed", &mut l.closed);
                blend_combo(ui, &mut l.blend);
                z_drag(ui, &mut l.z_index);
                crate::gui::toggle(ui, "Visible", &mut l.visible);
                follow_list(ui, e, actor, &mut l.follow);
                if l.follow.is_empty() {
                    points_list(ui, &mut l.points);
                    l.points.truncate(world2d::LINE_MAX_POINTS);
                }
            }),
            world2d::POLYGON2D => record::<Polygon2D>(e, actor, component, |e, p| {
                colour_u8(ui, "Colour", &mut p.color);
                texture_combo(ui, e, "Texture", &mut p.texture);
                if p.texture.is_some() {
                    vec2(ui, "Texture offset", &mut p.texture_offset, 0.5);
                }
                blend_combo(ui, &mut p.blend);
                z_drag(ui, &mut p.z_index);
                crate::gui::toggle(ui, "Visible", &mut p.visible);
                match crate::shape2d::triangulate(&p.points) {
                    Ok(t) => ui.text_disabled(format!(
                        "{} points, {} triangles",
                        p.points.len(),
                        t.len()
                    )),
                    Err(error) => ui.text_colored(
                        [1., 0.45, 0.4, 1.],
                        format!("{error}: move a point to fix it"),
                    ),
                }
                points_list(ui, &mut p.points);
            }),
            world2d::PARTICLES2D => {
                record::<Particles2D>(e, actor, component, |e, p| particles_band(ui, e, p))
            }
            world2d::AUDIO_PLAYER2D => record::<AudioPlayer2D>(e, actor, component, |e, a| {
                let mut clips: Vec<_> = e
                    .assets
                    .index
                    .usable()
                    .filter(|r| r.meta.kind == crate::assets::Kind::AudioClip)
                    .map(|r| (r.meta.id, r.meta.source.clone()))
                    .collect();
                clips.sort_by(|x, y| x.1.cmp(&y.1));
                let current = a
                    .audio
                    .clip
                    .and_then(|id| clips.iter().find(|c| c.0 == id))
                    .map_or_else(|| "None".to_string(), |c| c.1.clone());
                if let Some(_combo) = ui.begin_combo(crate::gui::field(ui, "Clip"), current) {
                    if ui.selectable("None") {
                        a.audio.clip = None;
                    }
                    for (id, name) in clips {
                        if ui
                            .selectable_config(&name)
                            .selected(a.audio.clip == Some(id))
                            .build()
                        {
                            a.audio.clip = Some(id);
                        }
                    }
                }
                crate::gui::Drag::new(crate::gui::field(ui, "Volume"))
                    .range(0., 1.)
                    .speed(0.01)
                    .display_format("%.2f")
                    .build(ui, &mut a.audio.volume);
                crate::gui::Drag::new(crate::gui::field(ui, "Pitch"))
                    .range(0.25, 4.)
                    .speed(0.01)
                    .display_format("%.2f")
                    .build(ui, &mut a.audio.pitch);
                crate::gui::toggle(ui, "Autoplay", &mut a.audio.play_on_start);
                crate::gui::Drag::new(crate::gui::field(ui, "Max distance (px)"))
                    .range(0., 8192.)
                    .speed(1.)
                    .display_format("%.0f")
                    .build(ui, &mut a.max_distance);
                crate::gui::Drag::new(crate::gui::field(ui, "Attenuation"))
                    .range(1., 4.)
                    .speed(0.02)
                    .display_format("%.0f")
                    .build(ui, &mut a.attenuation);
                crate::gui::Drag::new(crate::gui::field(ui, "Panning"))
                    .range(0., 1.)
                    .speed(0.01)
                    .display_format("%.2f")
                    .build(ui, &mut a.panning);
                crate::gui::muted(
                    ui,
                    "Loudest at the centre of the view, silent at the max distance, panned towards its side of the screen.",
                );
            }),
            world2d::NOTIFIER2D => record::<Notifier2D>(e, actor, component, |e, n| {
                let mut xy = [n.rect[0], n.rect[1]];
                let mut size = [n.rect[2], n.rect[3]];
                vec2(ui, "Position", &mut xy, 0.5);
                vec2(ui, "Size", &mut size, 0.5);
                n.rect = [xy[0], xy[1], size[0].max(1.), size[1].max(1.)];
                if let Some(bounds) = crate::world2d_editor::sprite_bounds(e, actor)
                    && ui.small_button(format!(
                        "{} Fit to Sprite",
                        crate::world2d_editor::ICON_WAND
                    ))
                {
                    n.rect = [
                        bounds.0[0],
                        bounds.0[1],
                        bounds.1[0] - bounds.0[0],
                        bounds.1[1] - bounds.0[1],
                    ];
                }
                crate::gui::toggle(ui, "Pause node off screen", &mut n.disable_when_hidden);
                crate::gui::muted(
                    ui,
                    "Gameplay reads is_on_screen, take_screen_entered and take_screen_exited. Pausing deactivates the node while it is off screen.",
                );
            }),
            _ => record::<RemoteTransform2D>(e, actor, component, |e, r| {
                let is_2d = |a: &crate::scene::Actor| {
                    a.class.class_id.as_deref() == Some(crate::object_model::ACTOR2D_ID)
                };
                actor_combo(ui, e, "Target", "None", actor, &is_2d, &mut r.target);
                crate::gui::toggle(ui, "Position", &mut r.update_position);
                crate::gui::toggle(ui, "Rotation", &mut r.update_rotation);
                crate::gui::toggle(ui, "Scale", &mut r.update_scale);
                crate::gui::muted(
                    ui,
                    "Copies this node's world transform onto the target every step.",
                );
            }),
        }
    }
    if remove {
        e.remove_actor_component(actor, component.id);
    } else if reset
        && let Some(index) = e.scene.actor_index(actor)
        && let Some(fresh) = world2d::new_component(&e.scene.actors[index], id)
        && let Some(c) = e.scene.actors[index]
            .components
            .iter_mut()
            .find(|c| c.id == component.id)
    {
        c.properties = fresh.properties;
        c.overrides = fresh.overrides;
        e.scene.actors[index].refresh_components();
        e.changed();
    }
    true
}
fn particles_band(ui: &Ui, e: &mut Editor, p: &mut Particles2D) {
    let _ = crate::gui::field(ui, "Presets");
    for (label, preset) in [("Sparks", 0), ("Smoke", 1), ("Rain", 2), ("Dust", 3)] {
        if ui.small_button(label) {
            apply_preset(p, preset);
        }
        ui.same_line();
    }
    ui.new_line();
    crate::gui::toggle(ui, "Emitting", &mut p.emitting);
    crate::gui::toggle(ui, "One shot", &mut p.one_shot);
    let mut amount = i32::from(p.amount);
    if crate::gui::Drag::new(crate::gui::field(ui, "Amount"))
        .range(1, world2d::MAX_PARTICLES as i32)
        .speed(0.2)
        .build(ui, &mut amount)
    {
        p.amount = amount.clamp(1, world2d::MAX_PARTICLES as i32) as u16;
    }
    crate::gui::Drag::new(crate::gui::field(ui, "Lifetime (s)"))
        .range(0.05, 60.)
        .speed(0.01)
        .display_format("%.2f")
        .build(ui, &mut p.lifetime);
    crate::gui::Drag::new(crate::gui::field(ui, "Explosiveness"))
        .range(0., 1.)
        .speed(0.01)
        .display_format("%.2f")
        .build(ui, &mut p.explosiveness);
    if ui.is_item_hovered() {
        ui.tooltip_text("0 is a steady stream; 1 releases the whole amount at once");
    }
    crate::gui::Drag::new(crate::gui::field(ui, "Direction (deg)"))
        .speed(0.5)
        .display_format("%.0f")
        .build(ui, &mut p.direction);
    crate::gui::Drag::new(crate::gui::field(ui, "Spread (deg)"))
        .range(0., 180.)
        .speed(0.5)
        .display_format("%.0f")
        .build(ui, &mut p.spread);
    vec2(ui, "Speed min / max", &mut p.speed, 0.5);
    vec2(ui, "Gravity", &mut p.gravity, 0.5);
    crate::gui::Drag::new(crate::gui::field(ui, "Damping"))
        .range(0., 20.)
        .speed(0.01)
        .display_format("%.2f")
        .build(ui, &mut p.damping);
    crate::gui::Drag::new(crate::gui::field(ui, "Scale start / end"))
        .range(0., 16.)
        .speed(0.01)
        .display_format("%.2f")
        .build_array(ui, &mut p.scale);
    colour_u8(ui, "Start colour", &mut p.start_color);
    colour_u8(ui, "End colour", &mut p.end_color);
    let mut shape = match p.shape {
        EmissionShape::Point => 0,
        EmissionShape::Rectangle => 1,
        EmissionShape::Circle => 2,
    };
    if crate::world2d_editor::segmented(
        ui,
        "Emission shape",
        &["Point", "Rectangle", "Circle"],
        &mut shape,
    ) {
        p.shape = [
            EmissionShape::Point,
            EmissionShape::Rectangle,
            EmissionShape::Circle,
        ][shape];
    }
    match p.shape {
        EmissionShape::Rectangle => vec2(ui, "Half size", &mut p.extents, 0.5),
        EmissionShape::Circle => {
            crate::gui::Drag::new(crate::gui::field(ui, "Radius"))
                .range(0., 4096.)
                .speed(0.5)
                .display_format("%.0f")
                .build(ui, &mut p.extents[0]);
        }
        EmissionShape::Point => {}
    }
    texture_combo(ui, e, "Texture", &mut p.texture);
    if p.texture.is_some() {
        let mut region = p.region.map(i32::from);
        if crate::gui::Drag::new(crate::gui::field(ui, "Region x/y/w/h"))
            .range(0, 256)
            .build_array(ui, &mut region)
        {
            p.region = region.map(|v| v.clamp(0, 256) as u16);
        }
        let mut frames = i32::from(p.hframes);
        if crate::gui::Drag::new(crate::gui::field(ui, "Frames"))
            .range(1, 64)
            .speed(0.05)
            .build(ui, &mut frames)
        {
            p.hframes = frames.clamp(1, 64) as u16;
        }
    } else {
        ui.text_disabled("Without a texture particles are invisible on the console; pick one (a small dot or spark).");
    }
    blend_combo(ui, &mut p.blend);
    z_drag(ui, &mut p.z_index);
    crate::gui::toggle(ui, "Local coordinates", &mut p.local_coords);
    if ui.is_item_hovered() {
        ui.tooltip_text("Particles move with the node instead of staying where they were emitted");
    }
    crate::gui::toggle(ui, "Visible", &mut p.visible);
}
/// Starting points for common effects; textures and colours stay the user's.
fn apply_preset(p: &mut Particles2D, preset: usize) {
    let keep = (p.texture, p.region, p.hframes, p.seed);
    *p = match preset {
        0 => Particles2D {
            amount: 24,
            lifetime: 0.6,
            explosiveness: 1.,
            one_shot: true,
            direction: -90.,
            spread: 180.,
            speed: [60., 140.],
            gravity: [0., 240.],
            scale: [1., 0.4],
            start_color: [255, 230, 140],
            end_color: [255, 90, 40],
            blend: crate::texture::BlendMode::Add,
            ..Default::default()
        },
        1 => Particles2D {
            amount: 20,
            lifetime: 2.,
            direction: -90.,
            spread: 15.,
            speed: [12., 24.],
            gravity: [6., -4.],
            damping: 0.3,
            scale: [0.6, 1.6],
            start_color: [200, 200, 200],
            end_color: [80, 80, 90],
            blend: crate::texture::BlendMode::Average,
            ..Default::default()
        },
        2 => Particles2D {
            amount: 64,
            lifetime: 1.2,
            direction: 100.,
            spread: 4.,
            speed: [180., 220.],
            gravity: [0., 60.],
            shape: EmissionShape::Rectangle,
            extents: [180., 4.],
            start_color: [150, 180, 255],
            end_color: [120, 150, 230],
            blend: crate::texture::BlendMode::Add,
            ..Default::default()
        },
        _ => Particles2D {
            amount: 12,
            lifetime: 0.5,
            explosiveness: 1.,
            one_shot: true,
            direction: -90.,
            spread: 70.,
            speed: [20., 50.],
            gravity: [0., 60.],
            damping: 2.,
            scale: [1., 0.5],
            start_color: [210, 190, 160],
            end_color: [150, 130, 110],
            blend: crate::texture::BlendMode::Average,
            ..Default::default()
        },
    };
    (p.texture, p.region, p.hframes, p.seed) = keep;
}

// ---- generated assets ------------------------------------------------------------------

/// The soft radial texture PointLight2D draws, generated and imported on first use.
pub fn light_texture(e: &mut Editor) -> Result<Uuid, String> {
    const SOURCE: &str = "assets/Sprites/Light2D.png";
    if let Some(record) = e
        .assets
        .index
        .usable()
        .find(|r| r.meta.kind == crate::assets::Kind::Texture && r.meta.source == SOURCE)
    {
        return Ok(record.meta.id);
    }
    let size = 64u32;
    let mut pixels = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let dx = (x as f32 + 0.5) / size as f32 * 2. - 1.;
            let dy = (y as f32 + 0.5) / size as f32 * 2. - 1.;
            let d = (dx * dx + dy * dy).sqrt();
            // Smooth falloff; black adds nothing, so the square's corners vanish.
            let v = ((1. - d).clamp(0., 1.).powf(1.6) * 255.).round() as u8;
            pixels.extend_from_slice(&[v, v, v, 255]);
        }
    }
    let mut bytes = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut bytes, size, size);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
        writer
            .write_image_data(&pixels)
            .map_err(|e| e.to_string())?;
    }
    let path = crate::assets::inside(&e.root, SOURCE)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    crate::project::write_changed(&path, &bytes)?;
    let id = crate::assets::commit(crate::texture::prepare(
        &e.root,
        SOURCE,
        "assets/Sprites/Light2D.epokasset",
        None,
        false,
    )?)?;
    e.assets.index = crate::assets::scan(&e.root, &mut Default::default());
    e.assets.refresh();
    Ok(id)
}
