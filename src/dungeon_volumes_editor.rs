//! The Volumes window: a generator's whole grid from above with the layout its settings
//! give, where generation volumes are drawn by dragging, moved, resized by their corner
//! and removed with a right-click. The layout follows every change.
use crate::dungeon::{self, Cell, Layout};
use crate::dungeon_volumes::{Kind, Volume};
use crate::editor::Editor;
use imgui::{MouseButton, Ui};
use uuid::Uuid;

pub const ICON: &str = "\u{f1b2}";
const CANVAS: [f32; 4] = [0.11, 0.12, 0.14, 1.];
const ROOM: [f32; 4] = [0.36, 0.42, 0.52, 1.];
const CORRIDOR: [f32; 4] = [0.27, 0.31, 0.38, 1.];
const SLOT: [f32; 4] = [0.3, 0.32, 0.38, 1.];
const BLOCKED: [f32; 4] = [0.8, 0.3, 0.25, 0.18];
const RED: [f32; 4] = [1., 0.45, 0.4, 1.];

#[derive(Clone, Copy)]
enum Drag {
    New([i32; 2]),
    Move {
        index: usize,
        grab: [i32; 2],
        from: Volume,
    },
    Resize {
        index: usize,
        from: Volume,
    },
}

pub struct Window {
    pub generator: Uuid,
    kind: Kind,
    drag: Option<Drag>,
    /// The layout for the generator's fingerprint it was made for.
    cached: Option<(String, [i32; 2], Result<Layout, String>)>,
}

pub fn open(e: &mut Editor, id: Uuid) {
    if e.dungeon_volumes
        .as_ref()
        .is_some_and(|w| w.generator == id)
    {
        return;
    }
    e.dungeon_volumes = Some(Window {
        generator: id,
        kind: Kind::KeepOut,
        drag: None,
        cached: None,
    });
}

pub fn window(ui: &Ui, e: &mut Editor) {
    let Some(mut w) = e.dungeon_volumes.take() else {
        return;
    };
    let Some(index) = e.scene.dungeons.iter().position(|g| g.id == w.generator) else {
        return;
    };
    let name = e
        .scene
        .actor_index(e.scene.dungeons[index].root)
        .map_or_else(|| "Dungeon".into(), |i| e.scene.actors[i].name.clone());
    let mut open = true;
    ui.window(format!("{ICON} Volumes: {name}###dungeon-volumes"))
        .opened(&mut open)
        .size([820., 560.], imgui::Condition::FirstUseEver)
        .position([360., 200.], imgui::Condition::FirstUseEver)
        .build(|| {
            let _font = crate::inspector_theme::scope(ui, e.inspector_font);
            body(ui, e, &mut w, index);
        });
    if open {
        e.dungeon_volumes = Some(w);
    }
}

fn body(ui: &Ui, e: &mut Editor, w: &mut Window, index: usize) {
    let g = e.scene.dungeons[index].clone();
    let fingerprint = g.fingerprint();
    if w.cached.as_ref().is_none_or(|(f, _, _)| *f != fingerprint) {
        match dungeon::layout_on_grid(&g, &e.assets.index) {
            Ok((size, layout)) => w.cached = Some((fingerprint, size, layout)),
            Err(error) => {
                ui.text_colored(RED, error);
                return;
            }
        }
    }
    let Some((_, size, layout)) = &w.cached else {
        return;
    };
    let (size, layout) = (*size, layout.as_ref());
    // The tools: one button per kind, the selected one drawn by dragging.
    for kind in Kind::ALL {
        let color = kind.color();
        let selected = w.kind == kind;
        let _tint = selected.then(|| {
            ui.push_style_color(
                imgui::StyleColor::Button,
                [color[0] * 0.45, color[1] * 0.45, color[2] * 0.45, 1.],
            )
        });
        if ui.button(kind.label()) {
            w.kind = kind;
        }
        if ui.is_item_hovered() {
            ui.tooltip_text(kind.tip());
        }
        ui.same_line();
    }
    ui.new_line();
    let recipe = g.source == dungeon::Source::Recipe;
    crate::gui::muted(
        ui,
        if recipe {
            "Drag on the grid to draw the selected volume; drag one to move it, its corner to resize it; right-click removes it. Recipes follow Keep out and Only here: a lattice slot that touches a closed cell holds no room and no corridor."
        } else {
            "Drag on the grid to draw the selected volume; drag one to move it, its corner to resize it; right-click removes it."
        },
    );
    match layout {
        Ok(layout) => crate::gui::muted(ui, dungeon::summary(layout)),
        Err(error) => ui.text_colored(RED, error),
    }
    if ui.button(format!("{} Generate", crate::dungeon_editor::ICON_GENERATE)) {
        crate::dungeon_editor::request_generate(e, g.id);
    }
    if ui.is_item_hovered() {
        ui.tooltip_text("Build the dungeon with these volumes into the map (Ctrl+Z undoes it).");
    }
    ui.same_line();
    crate::gui::muted(
        ui,
        format!(
            "{} volume{} · grid {}x{} cells",
            g.volumes.len(),
            if g.volumes.len() == 1 { "" } else { "s" },
            size[0],
            size[1]
        ),
    );
    // The canvas.
    let draw = ui.get_window_draw_list();
    let origin = ui.cursor_screen_pos();
    let area = ui.content_region_avail();
    let area = [area[0].max(100.), area[1].max(100.)];
    draw.add_rect(origin, [origin[0] + area[0], origin[1] + area[1]], CANVAS)
        .filled(true)
        .build();
    let unit = ((area[0] - 16.) / size[0] as f32)
        .min((area[1] - 16.) / size[1] as f32)
        .max(2.);
    let offset = [
        origin[0] + (area[0] - unit * size[0] as f32) * 0.5,
        origin[1] + (area[1] - unit * size[1] as f32) * 0.5,
    ];
    // In 3D the grid's rows run towards +Z, up the screen as in the Scene view; in 2D
    // they run down, as on the screen.
    let up = !crate::dungeon_bake::is_2d(&e.scene, g.root);
    let at = |x: f32, y: f32| {
        let y = if up { size[1] as f32 - y } else { y };
        [offset[0] + x * unit, offset[1] + y * unit]
    };
    // A rectangle of cells on the screen, its corners in screen order.
    let span_of = |x0: f32, y0: f32, x1: f32, y1: f32| {
        let (a, b) = (at(x0, y0), at(x1, y1));
        (
            [a[0].min(b[0]), a[1].min(b[1])],
            [a[0].max(b[0]), a[1].max(b[1])],
        )
    };
    draw.add_rect(
        at(0., 0.),
        at(size[0] as f32, size[1] as f32),
        [0.15, 0.16, 0.19, 1.],
    )
    .filled(true)
    .build();
    if let Ok(layout) = layout {
        for y in 0..size[1].min(layout.size[1]) {
            for x in 0..size[0].min(layout.size[0]) {
                let color = match layout.at([x, y]) {
                    Cell::Room(_) => ROOM,
                    Cell::Corridor => CORRIDOR,
                    Cell::Empty => continue,
                };
                let (a, b) = span_of(x as f32, y as f32, x as f32 + 1., y as f32 + 1.);
                draw.add_rect(a, b, color).filled(true).build();
            }
        }
    }
    let blocked = crate::dungeon_volumes::blocked(&g.volumes, size);
    if recipe && let Ok((recipe, _)) = crate::dungeon_recipe::of(&e.assets.index, &g) {
        // Lattice slots, the closed ones shaded.
        let (lattice, span) = recipe.lattice();
        for sy in 0..lattice[1] {
            for sx in 0..lattice[0] {
                let (x0, y0) = ((1 + sx * span) as f32, (1 + sy * span) as f32);
                let (a, b) = span_of(x0, y0, x0 + span as f32, y0 + span as f32);
                let closed = (1 + sy * span..1 + (sy + 1) * span).any(|y| {
                    (1 + sx * span..1 + (sx + 1) * span)
                        .any(|x| blocked[(y * size[0] + x) as usize])
                });
                if closed && g.volumes.iter().any(|v| v.kind.shapes_recipes()) {
                    draw.add_rect(a, b, BLOCKED).filled(true).build();
                }
                draw.add_rect(a, b, SLOT).build();
            }
        }
    }
    // The pointer, in cells.
    ui.set_cursor_screen_pos(origin);
    ui.invisible_button("##volumes-canvas", area);
    let hovered = ui.is_item_hovered();
    let mouse = ui.io().mouse_pos;
    // The pointer may be far off (or nowhere) when the window is not hovered.
    let cell_at = |p: [f32; 2]| {
        let column = ((p[0] - offset[0]) / unit)
            .floor()
            .clamp(-1., size[0] as f32) as i32;
        let row = ((p[1] - offset[1]) / unit)
            .floor()
            .clamp(-1., size[1] as f32) as i32;
        [
            column.clamp(0, size[0] - 1),
            if up { size[1] - 1 - row } else { row }.clamp(0, size[1] - 1),
        ]
    };
    let cell = cell_at(mouse);
    let mut volumes = g.volumes.clone();
    let topmost = |volumes: &[Volume], p: [i32; 2]| volumes.iter().rposition(|v| v.contains(p));
    let corner_of = |v: &Volume| {
        let m = v.max();
        let c = at(m[0] as f32, m[1] as f32);
        (mouse[0] - c[0]).abs() < unit.max(8.) * 0.7 && (mouse[1] - c[1]).abs() < unit.max(8.) * 0.7
    };
    if hovered && ui.is_mouse_clicked(MouseButton::Left) {
        w.drag = Some(match topmost(&volumes, cell) {
            Some(i) if corner_of(&volumes[i]) => Drag::Resize {
                index: i,
                from: volumes[i],
            },
            Some(i) => Drag::Move {
                index: i,
                grab: cell,
                from: volumes[i],
            },
            None => {
                // Grabbing a corner just outside the rectangle resizes too.
                match volumes.iter().rposition(corner_of) {
                    Some(i) => Drag::Resize {
                        index: i,
                        from: volumes[i],
                    },
                    None => Drag::New(cell),
                }
            }
        });
    }
    let mut changed = false;
    let mut pending = None;
    match w.drag {
        Some(Drag::New(start)) => {
            pending = Some(Volume::normalized(start, cell, w.kind));
        }
        Some(Drag::Move { index, grab, from }) if index < volumes.len() => {
            let mut v = from;
            v.min = [
                (from.min[0] + cell[0] - grab[0]).clamp(0, size[0] - from.size[0]),
                (from.min[1] + cell[1] - grab[1]).clamp(0, size[1] - from.size[1]),
            ];
            changed |= volumes[index] != v;
            volumes[index] = v;
        }
        Some(Drag::Resize { index, from }) if index < volumes.len() => {
            let mut v = Volume::normalized(from.min, cell, from.kind);
            v.min = from.min;
            v.size = [
                (cell[0] - from.min[0] + 1).max(1),
                (cell[1] - from.min[1] + 1).max(1),
            ];
            changed |= volumes[index] != v;
            volumes[index] = v;
        }
        _ => {}
    }
    if w.drag.is_some() && !ui.is_mouse_down(MouseButton::Left) {
        if let Some(v) = pending.take() {
            volumes.push(v);
            changed = true;
        }
        w.drag = None;
    }
    if hovered
        && ui.is_mouse_clicked(MouseButton::Right)
        && let Some(i) = topmost(&volumes, cell)
    {
        volumes.remove(i);
        changed = true;
    }
    // The volumes and the one being drawn.
    for (i, v) in volumes.iter().chain(pending.iter()).enumerate() {
        let c = v.kind.color();
        let m = v.max();
        let (a, b) = span_of(v.min[0] as f32, v.min[1] as f32, m[0] as f32, m[1] as f32);
        // Large volumes (mirrors, areas to stay in) tint lightly so the layout shows.
        let fill = match v.kind {
            Kind::KeepOut | Kind::Room => 0.22,
            _ => 0.07,
        };
        draw.add_rect(a, b, [c[0], c[1], c[2], fill])
            .filled(true)
            .build();
        draw.add_rect(a, b, c).thickness(2.).build();
        if v.kind == Kind::KeepOut {
            draw.add_line(a, b, [c[0], c[1], c[2], 0.6]).build();
            draw.add_line([b[0], a[1]], [a[0], b[1]], [c[0], c[1], c[2], 0.6])
                .build();
        }
        if matches!(v.kind, Kind::MirrorX | Kind::MirrorY) {
            let mid = if v.kind == Kind::MirrorX {
                let x = (v.min[0] + m[0]) as f32 * 0.5;
                (at(x, v.min[1] as f32), at(x, m[1] as f32))
            } else {
                let y = (v.min[1] + m[1]) as f32 * 0.5;
                (at(v.min[0] as f32, y), at(m[0] as f32, y))
            };
            draw.add_line(mid.0, mid.1, c).thickness(2.).build();
        }
        // The resize handle, on the far corner.
        let handle = at(m[0] as f32, m[1] as f32);
        draw.add_rect(
            [handle[0] - 5., handle[1] - 5.],
            [handle[0] + 5., handle[1] + 5.],
            c,
        )
        .filled(true)
        .build();
        draw.add_text(
            [a[0] + 4., a[1] + 2.],
            c,
            format!("{} {}", i + 1, v.kind.label()),
        );
    }
    if hovered {
        let tip = match topmost(&volumes, cell) {
            Some(i) => format!(
                "{} {} · {}x{} cells from {},{} · right-click removes it",
                i + 1,
                volumes[i].kind.label(),
                volumes[i].size[0],
                volumes[i].size[1],
                volumes[i].min[0],
                volumes[i].min[1]
            ),
            None => format!(
                "Cell {},{} · drag to draw {}",
                cell[0],
                cell[1],
                w.kind.label()
            ),
        };
        ui.tooltip_text(tip);
    }
    // Moving and resizing show live; one drag is one undo step.
    if changed {
        e.scene.dungeons[index].volumes = volumes;
        e.changed_coalesced("dungeon-volumes");
    }
}
