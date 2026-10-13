//! Navigation editing (World2D initiative, phase 4): the Inspector bands of
//! NavigationRegion2D and NavigationAgent2D, the region drawn in the 2D view (blocked
//! cells, or the navmesh's triangles) and the path a selected agent would take to its
//! target, computed by the same searches the runtime runs.
use crate::{
    editor::Editor,
    navigation2d::{self, Grid, Mesh, NavMode, NavigationAgent2D, NavigationRegion2D},
    world2d,
    world2d_editor::View,
};
use imgui::Ui;
use uuid::Uuid;

pub const ICON_REGION: &str = "\u{f279}";
pub const ICON_AGENT: &str = "\u{f554}";
const AREA: [f32; 4] = [0.35, 0.85, 1.0, 0.10];
const BLOCKED: [f32; 4] = [0.95, 0.35, 0.35, 0.34];
const EDGE: [f32; 4] = [0.35, 0.85, 1.0, 0.75];
const PATH_OK: [f32; 4] = [0.45, 0.95, 0.55, 1.];
const PATH_PARTIAL: [f32; 4] = [1.0, 0.65, 0.25, 1.];

/// A region as the searches see it, in world pixels.
pub enum Built {
    Grid(Grid),
    Mesh(Mesh),
}
impl Built {
    fn covers(&self, p: [f32; 2]) -> bool {
        match self {
            Built::Grid(g) => g.inside(g.cell_of(p)),
            Built::Mesh(m) => m.locate(p).is_some_and(|(_, _, inside)| inside),
        }
    }
    fn path(&self, from: [f32; 2], to: [f32; 2]) -> Option<(Vec<[f32; 2]>, bool)> {
        match self {
            Built::Grid(g) => navigation2d::grid_path(g, from, to),
            Built::Mesh(m) => navigation2d::mesh_path(m, from, to),
        }
    }
}
/// Builds the region on `actor`'s component, in world pixels.
pub fn build(e: &Editor, actor: Uuid, region: &NavigationRegion2D) -> Result<Built, String> {
    let index = e
        .scene
        .actor_index(actor)
        .ok_or("the region left the map")?;
    match region.mode {
        NavMode::Grid => navigation2d::build_grid(
            region,
            world2d::world_point(&e.scene, index, [0., 0.]),
            &e.scene,
            &e.scene.tilesets,
            Some(actor),
        )
        .map(Built::Grid),
        NavMode::Polygon => {
            let outline: Vec<[f32; 2]> = region
                .outline
                .iter()
                .map(|p| world2d::world_point(&e.scene, index, *p))
                .collect();
            navigation2d::build_mesh(&outline).map(Built::Mesh)
        }
    }
}
fn regions(e: &Editor) -> Vec<(Uuid, Built)> {
    e.scene
        .actors
        .iter()
        .filter(|a| a.active)
        .filter_map(|a| {
            let c = world2d::component(a, world2d::NAVIGATION_REGION2D)?;
            let r: NavigationRegion2D = world2d::read(c);
            build(e, a.id, &r).ok().map(|b| (a.id, b))
        })
        .collect()
}
fn position(e: &Editor, actor: Uuid) -> Option<[f32; 2]> {
    let index = e.scene.actor_index(actor)?;
    Some(world2d::world_position(&e.scene, index))
}
/// The path the agent on `actor` would take now: (points, reaches the target), or why not.
pub fn preview(e: &Editor, actor: Uuid) -> Result<(Vec<[f32; 2]>, bool), String> {
    let index = e.scene.actor_index(actor).ok_or("missing")?;
    let agent: NavigationAgent2D = world2d::read(
        world2d::component(&e.scene.actors[index], world2d::NAVIGATION_AGENT2D)
            .ok_or("no agent")?,
    );
    let target = agent
        .target
        .and_then(|t| position(e, t))
        .ok_or("No target: pick one, or set it from gameplay with set_target_position.")?;
    let from = world2d::world_position(&e.scene, index);
    let regions = regions(e);
    let (_, region) = regions
        .iter()
        .find(|(_, r)| r.covers(from))
        .ok_or("No NavigationRegion2D under this node.")?;
    region
        .path(from, target)
        .ok_or_else(|| "The node stands on a blocked spot.".to_string())
}

// ---- Inspector -------------------------------------------------------------------------

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
pub fn band(
    ui: &Ui,
    e: &mut Editor,
    actor: Uuid,
    component: &crate::actor_document::ComponentInstance,
) -> bool {
    let id = component.class.class_id.as_deref().unwrap_or("");
    let (glyph, title) = match id {
        world2d::NAVIGATION_REGION2D => (ICON_REGION, "NavigationRegion2D"),
        world2d::NAVIGATION_AGENT2D => (ICON_AGENT, "NavigationAgent2D"),
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
    if remove {
        e.remove_actor_component(actor, component.id);
        return true;
    }
    if reset {
        if id == world2d::NAVIGATION_REGION2D {
            record::<NavigationRegion2D>(e, actor, component, |_, r| *r = Default::default());
        } else {
            record::<NavigationAgent2D>(e, actor, component, |_, a| *a = Default::default());
        }
        return true;
    }
    if !open {
        return true;
    }
    if id == world2d::NAVIGATION_REGION2D {
        region_band(ui, e, actor, component);
    } else {
        agent_band(ui, e, actor, component);
    }
    true
}
fn region_band(
    ui: &Ui,
    e: &mut Editor,
    actor: Uuid,
    component: &crate::actor_document::ComponentInstance,
) {
    record::<NavigationRegion2D>(e, actor, component, |e, r| {
        let mut mode = usize::from(r.mode == NavMode::Polygon);
        if crate::world2d_editor::segmented(ui, "Kind", &["Grid", "Navmesh"], &mut mode) {
            r.mode = if mode == 0 {
                NavMode::Grid
            } else {
                NavMode::Polygon
            };
        }
        match r.mode {
            NavMode::Grid => {
                let is_layer = |a: &crate::scene::Actor| {
                    world2d::component(a, world2d::TILEMAP_LAYER).is_some()
                };
                crate::world2d_shapes_editor::actor_combo(
                    ui,
                    e,
                    "Tile layer",
                    "None: an area from this node",
                    actor,
                    &is_layer,
                    &mut r.tile_layer,
                );
                if r.tile_layer.is_some() {
                    ui.input_int(crate::gui::field(ui, "Margin (cells)"), &mut r.margin)
                        .build();
                    r.margin = r.margin.clamp(0, 64);
                } else {
                    crate::gui::Drag::new(crate::gui::field(ui, "Cell size (px)"))
                        .speed(0.25)
                        .range(4., 128.)
                        .display_format("%.0f")
                        .build(ui, &mut r.cell_size);
                    crate::gui::Drag::new(crate::gui::field(ui, "Area (px)"))
                        .speed(1.)
                        .range(8., 8192.)
                        .display_format("%.0f")
                        .build_array(ui, &mut r.size);
                }
                crate::gui::toggle(ui, "Static bodies block", &mut r.static_bodies);
                crate::gui::Drag::new(crate::gui::field(ui, "Agent radius (px)"))
                    .speed(0.25)
                    .range(0., 64.)
                    .display_format("%.1f")
                    .build(ui, &mut r.agent_radius);
                crate::gui::toggle(ui, "Diagonal moves", &mut r.diagonal);
                match build(e, actor, r) {
                    Ok(Built::Grid(g)) => {
                        let blocked = g.blocked.iter().filter(|b| **b).count();
                        ui.text_disabled(format!(
                            "{} x {} cells of {:.0} px, {blocked} blocked",
                            g.columns, g.rows, g.cell[0]
                        ));
                    }
                    Ok(Built::Mesh(_)) => {}
                    Err(error) => ui.text_colored([1., 0.45, 0.4, 1.], error),
                }
                crate::gui::muted(
                    ui,
                    "Solid tiles of the layer and the colliders of nodes without a CharacterBody2D block their cells; the agent radius keeps paths that far from them. Best for top-down maps.",
                );
            }
            NavMode::Polygon => {
                match build(e, actor, r) {
                    Ok(Built::Mesh(m)) => ui.text_disabled(format!(
                        "{} points, {} triangles",
                        r.outline.len(),
                        m.triangles.len()
                    )),
                    Ok(Built::Grid(_)) => {}
                    Err(error) => ui.text_colored([1., 0.45, 0.4, 1.], error),
                }
                crate::gui::muted(
                    ui,
                    "Drag the outline's points in the view; click a segment's dot to insert one, right-click to delete. Agents walk anywhere inside it, in straight lines that turn at its corners.",
                );
            }
        }
    });
}
fn agent_band(
    ui: &Ui,
    e: &mut Editor,
    actor: Uuid,
    component: &crate::actor_document::ComponentInstance,
) {
    record::<NavigationAgent2D>(e, actor, component, |e, a| {
        let any = |_: &crate::scene::Actor| true;
        crate::world2d_shapes_editor::actor_combo(
            ui,
            e,
            "Target",
            "None: set from gameplay",
            actor,
            &any,
            &mut a.target,
        );
        crate::gui::Drag::new(crate::gui::field(ui, "Max speed (px/s)"))
            .speed(0.5)
            .range(0., 4096.)
            .display_format("%.0f")
            .build(ui, &mut a.max_speed);
        crate::gui::Drag::new(crate::gui::field(ui, "Arrive distance (px)"))
            .speed(0.1)
            .range(0.5, 256.)
            .display_format("%.1f")
            .build(ui, &mut a.arrive_distance);
        crate::gui::Drag::new(crate::gui::field(ui, "Repath every (s)"))
            .speed(0.01)
            .range(0., 60.)
            .display_format("%.2f")
            .build(ui, &mut a.repath_interval);
        crate::gui::toggle(ui, "Moves the node", &mut a.drives_body);
        if a.drives_body
            && let Some(index) = e.scene.actor_index(actor)
            && let Some(body) =
                world2d::component(&e.scene.actors[index], world2d::CHARACTER_BODY2D)
            && !world2d::read::<world2d::CharacterBody2D>(body).floating
        {
            ui.text_colored(
                [1., 0.75, 0.35, 1.],
                "Its CharacterBody2D is not Floating: set Floating for top-down movement.",
            );
        }
    });
    match preview(e, actor) {
        Ok((points, true)) => ui.text_colored(
            PATH_OK,
            format!("Reaches the target in {} straight legs", points.len()),
        ),
        Ok((_, false)) => ui.text_colored(
            PATH_PARTIAL,
            "The target is unreachable: the agent stops at the closest point",
        ),
        Err(why) => ui.text_disabled(why),
    };
    crate::gui::muted(
        ui,
        "The agent plans in the NavigationRegion2D under it and walks the path: through its CharacterBody2D's move_and_slide, or by moving the node. Gameplay reads is_navigation_finished, take_arrived and the velocity, or sets targets.",
    );
}

// ---- 2D view -----------------------------------------------------------------------------

/// The selected region (cells or triangles) and the selected agent's path.
pub fn overlay(
    e: &Editor,
    draw: &imgui::DrawListMut,
    views: &[View],
    to_screen: &dyn Fn([f32; 2]) -> [f32; 2],
    zoom: f32,
) {
    let Some(selected) = e.selected_actor else {
        return;
    };
    let Some(index) = e.scene.actor_index(selected) else {
        return;
    };
    let actor = &e.scene.actors[index];
    if let Some(c) = world2d::component(actor, world2d::NAVIGATION_REGION2D) {
        let region: NavigationRegion2D = world2d::read(c);
        if let Ok(built) = build(e, selected, &region) {
            draw_region(draw, &built, to_screen, zoom);
        }
    }
    if world2d::component(actor, world2d::NAVIGATION_AGENT2D).is_some()
        && let Ok((points, reaches)) = preview(e, selected)
        && let Some(view) = views.iter().find(|v| v.id == selected)
    {
        let colour = if reaches { PATH_OK } else { PATH_PARTIAL };
        let mut at = to_screen(view.world.t);
        for p in &points {
            let q = to_screen(*p);
            draw.add_line(at, q, colour).thickness(2.).build();
            draw.add_circle(q, 3., colour).filled(true).build();
            at = q;
        }
        if let Some(last) = points.last() {
            draw.add_circle(to_screen(*last), 7., colour)
                .thickness(1.5)
                .build();
        }
    }
}
fn draw_region(
    draw: &imgui::DrawListMut,
    built: &Built,
    to_screen: &dyn Fn([f32; 2]) -> [f32; 2],
    zoom: f32,
) {
    match built {
        Built::Grid(g) => {
            let lo = to_screen(g.origin);
            let hi = to_screen([
                g.origin[0] + g.columns as f32 * g.cell[0],
                g.origin[1] + g.rows as f32 * g.cell[1],
            ]);
            draw.add_rect(lo, hi, AREA).filled(true).build();
            if g.cell[0] * zoom >= 6. {
                for x in 1..g.columns {
                    let sx = lo[0] + (hi[0] - lo[0]) * x as f32 / g.columns as f32;
                    draw.add_line([sx, lo[1]], [sx, hi[1]], [1., 1., 1., 0.08])
                        .build();
                }
                for y in 1..g.rows {
                    let sy = lo[1] + (hi[1] - lo[1]) * y as f32 / g.rows as f32;
                    draw.add_line([lo[0], sy], [hi[0], sy], [1., 1., 1., 0.08])
                        .build();
                }
            }
            for y in 0..g.rows {
                for x in 0..g.columns {
                    if !g.blocked[y * g.columns + x] {
                        continue;
                    }
                    let a = to_screen([
                        g.origin[0] + x as f32 * g.cell[0],
                        g.origin[1] + y as f32 * g.cell[1],
                    ]);
                    let b = to_screen([
                        g.origin[0] + (x + 1) as f32 * g.cell[0],
                        g.origin[1] + (y + 1) as f32 * g.cell[1],
                    ]);
                    draw.add_rect(a, b, BLOCKED).filled(true).build();
                }
            }
            draw.add_rect(lo, hi, EDGE).thickness(1.5).build();
        }
        Built::Mesh(m) => {
            for t in &m.triangles {
                let p = t.map(|v| to_screen(m.vertices[v]));
                draw.add_triangle(p[0], p[1], p[2], [AREA[0], AREA[1], AREA[2], 0.22])
                    .filled(true)
                    .build();
                draw.add_triangle(p[0], p[1], p[2], [EDGE[0], EDGE[1], EDGE[2], 0.35])
                    .build();
            }
        }
    }
}
