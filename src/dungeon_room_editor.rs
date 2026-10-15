//! The Dungeon Room editor: a room's floor plan, seen from above, where its doors, the
//! spots its keys and items go to and the cells it leaves out are clicked on and off,
//! with its cell size and footprint beside it. Rooms are made from the map: build one
//! under an actor, select the actor and choose Save as Dungeon Room; Update from
//! selection captures it again. Each valid edit is saved and regenerates the dungeons
//! whose recipe uses the room.
use crate::dungeon::Side;
use crate::dungeon_room::{self, Door, Room};
use crate::editor::Editor;
use imgui::{MouseButton, Ui};
use uuid::Uuid;

pub const ICON: &str = "\u{f6d9}";
const CANVAS: [f32; 4] = [0.11, 0.12, 0.14, 1.];
const FLOOR: [f32; 4] = [0.22, 0.24, 0.28, 1.];
const CLOSED: [f32; 4] = [0.07, 0.07, 0.08, 1.];
const GRID: [f32; 4] = [0.34, 0.36, 0.42, 1.];
const GEOMETRY: [f32; 4] = [0.45, 0.58, 0.75, 0.3];
const COLLIDER: [f32; 4] = [0.95, 0.6, 0.25, 1.];
const LIGHT: [f32; 4] = [1., 0.86, 0.35, 1.];
const ACTOR: [f32; 4] = [0.45, 0.85, 0.55, 1.];
const DOOR: [f32; 4] = [1., 0.78, 0.2, 1.];
const SPOT: [f32; 4] = [0.4, 0.9, 0.55, 1.];
const HOVER: [f32; 4] = [1., 1., 1., 0.85];
const AMBER: [f32; 4] = [1., 0.75, 0.3, 1.];

pub struct Window {
    pub asset: Uuid,
    pub room: Room,
    dirty: bool,
    status: String,
    undo: Vec<Room>,
}
impl Window {
    fn checkpoint(&mut self) {
        self.undo.push(self.room.clone());
        if self.undo.len() > 64 {
            self.undo.remove(0);
        }
        self.dirty = true;
    }
}

/// Opens the editor on room `id`.
pub fn open(e: &mut Editor, id: Uuid) {
    if e.dungeon_room.as_ref().is_some_and(|w| w.asset == id) {
        return;
    }
    let room = e
        .assets
        .index
        .resolve(id)
        .ok()
        .and_then(|r| dungeon_room::document(r).ok())
        .or_else(|| {
            let index = crate::assets::scan(&e.root, &mut Default::default());
            index
                .resolve(id)
                .ok()
                .and_then(|r| dungeon_room::document(r).ok())
        });
    match room {
        Some(room) => {
            e.dungeon_room = Some(Window {
                asset: id,
                room,
                dirty: false,
                status: String::new(),
                undo: Vec::new(),
            })
        }
        None => {
            let error = format!("Dungeon Room {id} could not be opened");
            e.last_error = Some(error.clone());
            e.log(error);
        }
    }
}

/// The cell size rooms of this map are built for: the first 3D generator's, or 3.
fn cell_size(e: &Editor) -> f32 {
    e.scene
        .dungeons
        .iter()
        .find(|g| !crate::dungeon_bake::is_2d(&e.scene, g.root))
        .map_or(3., |g| g.cell_size)
}

/// Captures the branch under `actor` as a new Dungeon Room and opens it.
pub fn save_selection(e: &mut Editor, actor: Uuid) {
    let result = (|| -> Result<Uuid, String> {
        let mut room = dungeon_room::capture(&e.scene, actor, cell_size(e), &e.assets.index)?;
        if room.doors.is_empty() {
            // A first door in the middle of the near edge, so the room is usable at once;
            // the editor shows where to click for the others.
            room.doors.push(Door {
                cell: [room.size[0] / 2, 0],
                side: Side::North,
            });
        }
        let name = e.scene.actor_index(actor).map_or_else(
            || "DungeonRoom".to_owned(),
            |i| e.scene.actors[i].name.clone(),
        );
        let path = dungeon_room::fresh_path(&e.root, &name);
        let id = dungeon_room::create(&e.root, &path, &room)?;
        e.assets.index = crate::assets::scan(&e.root, &mut Default::default());
        e.assets.refresh();
        e.log(format!(
            "Created {path}: {}x{} cells, {} actors, {} door{}.",
            room.size[0],
            room.size[1],
            room.actors.len(),
            room.doors.len(),
            if room.doors.len() == 1 { "" } else { "s" }
        ));
        Ok(id)
    })();
    match result {
        Ok(id) => open(e, id),
        Err(error) => {
            e.last_error = Some(error.clone());
            e.log(error);
        }
    }
}

fn store(e: &mut Editor, w: &Window) -> Result<(), String> {
    let record = e.assets.index.resolve(w.asset).map_err(|e| e.to_string())?;
    dungeon_room::save(record, &w.room)?;
    e.assets.index = crate::assets::scan(&e.root, &mut Default::default());
    e.assets.refresh();
    Ok(())
}

pub fn window(ui: &Ui, e: &mut Editor) {
    let Some(mut w) = e.dungeon_room.take() else {
        return;
    };
    let name = e
        .assets
        .index
        .resolve(w.asset)
        .ok()
        .and_then(|r| r.path.file_stem().map(|s| s.to_string_lossy().into_owned()))
        .unwrap_or_else(|| "Dungeon Room".into());
    let mut open = true;
    ui.window(format!("{ICON} Dungeon Room: {name}###dungeon-room-editor"))
        .opened(&mut open)
        .size([900., 560.], imgui::Condition::FirstUseEver)
        .position([300., 360.], imgui::Condition::FirstUseEver)
        .build(|| body(ui, e, &mut w));
    if w.dirty && !ui.is_mouse_down(MouseButton::Left) && !ui.is_any_item_active() {
        w.dirty = false;
        match w.room.validate() {
            Ok(()) => match store(e, &w) {
                Ok(()) => {
                    let count = crate::dungeon_editor::regenerate_with_room(e, w.asset);
                    w.status = if count > 0 {
                        format!(
                            "Saved · regenerated {count} dungeon{}",
                            if count == 1 { "" } else { "s" }
                        )
                    } else {
                        "Saved".into()
                    };
                }
                Err(error) => {
                    w.status = error.clone();
                    e.last_error = Some(error);
                }
            },
            Err(error) => w.status = format!("Not saved: {error}"),
        }
    }
    if open {
        e.dungeon_room = Some(w);
    }
}

fn body(ui: &Ui, e: &mut Editor, w: &mut Window) {
    let _font = crate::inspector_theme::scope(ui, e.inspector_font);
    let available = ui.content_region_avail();
    let side = 300_f32.min(available[0] * 0.4);
    ui.child_window("##room-settings")
        .size([side, available[1]])
        .build(|| settings(ui, e, w));
    ui.same_line();
    ui.child_window("##room-plan")
        .size([0., available[1]])
        .build(|| plan(ui, e, w));
}

fn settings(ui: &Ui, e: &mut Editor, w: &mut Window) {
    crate::gui::heading(ui, "Room");
    let mut cell = w.room.cell;
    if ui
        .input_float(crate::gui::field(ui, "Cell size"), &mut cell)
        .step(0.5)
        .display_format("%.2f")
        .build()
    {
        w.checkpoint();
        w.room.cell = cell.clamp(0.5, 8.);
    }
    if ui.is_item_hovered() {
        ui.tooltip_text("The grid cell size the room was built for. Dungeons with another cell size scale it to fit.");
    }
    let (lo, hi) = dungeon_room::SIZE_LIMITS;
    let mut size = w.room.size.map(i32::from);
    if ui
        .slider_config(crate::gui::field(ui, "Size"), i32::from(lo), i32::from(hi))
        .build_array(&mut size)
    {
        w.checkpoint();
        w.room.size = size.map(|v| v.clamp(i32::from(lo), i32::from(hi)) as u16);
        let room = &mut w.room;
        let inside = |c: &[u16; 2], size: [u16; 2]| c[0] < size[0] && c[1] < size[1];
        let s = room.size;
        room.closed.retain(|c| inside(c, s));
        room.spots.retain(|c| inside(c, s));
        room.doors.retain(|d| inside(&d.cell, s));
        let snapshot = room.clone();
        room.doors.retain(|d| snapshot.on_edge(d.cell, d.side));
    }
    if ui.is_item_hovered() {
        ui.tooltip_text("Cells across (x) and deep (z). The content stands from the root's origin towards +X and +Z.");
    }
    ui.spacing();
    let statics = w
        .room
        .actors
        .iter()
        .filter(|a| dungeon_room::is_static(a))
        .count();
    crate::gui::muted(
        ui,
        format!(
            "{} doors · {} spots · {} closed cells",
            w.room.doors.len(),
            w.room.spots.len(),
            w.room.closed.len()
        ),
    );
    let copied = w.room.actors.len() - statics;
    crate::gui::muted(
        ui,
        format!(
            "{statics} static piece{} merge{} into the dungeon · {copied} actor{} copied",
            if statics == 1 { "" } else { "s" },
            if statics == 1 { "s" } else { "" },
            if copied == 1 { " is" } else { "s are" }
        ),
    );
    ui.spacing();
    let selected = e
        .selected_actor
        .filter(|id| e.scene.actors.iter().any(|a| a.logical_parent == Some(*id)));
    ui.disabled(selected.is_none(), || {
        if ui.button("Update from selection")
            && let Some(actor) = selected
        {
            match dungeon_room::capture(&e.scene, actor, w.room.cell, &e.assets.index) {
                Ok(captured) => {
                    w.checkpoint();
                    w.room.actors = captured.actors;
                    w.room.size = [
                        w.room.size[0].max(captured.size[0]),
                        w.room.size[1].max(captured.size[1]),
                    ];
                    for door in captured.doors {
                        if !w.room.doors.contains(&door) {
                            w.room.doors.push(door);
                        }
                    }
                    w.status = "Captured the selection".into();
                }
                Err(error) => w.status = error,
            }
        }
    });
    if ui.is_item_hovered() {
        ui.tooltip_text("Replace the room's content with the children of the selected actor, keeping its doors and spots.");
    }
    ui.same_line();
    ui.disabled(w.undo.is_empty(), || {
        if ui.button("Undo")
            && let Some(room) = w.undo.pop()
        {
            w.room = room;
            w.dirty = true;
        }
    });
    ui.spacing();
    ui.separator();
    ui.text_wrapped("Click outside an edge cell to add or remove a door there. Click a cell to mark where keys and items go; Shift-click to leave the cell out of the floor.");
    ui.spacing();
    ui.text_wrapped("Static pieces (plain meshes, cubes and solid colliders) merge into the dungeon's geometry and collision; lights and actors with behaviour stay actors.");
    if let Err(error) = w.room.validate() {
        ui.spacing();
        ui.text_colored(AMBER, format!("\u{f071} {error}"));
    }
    if !w.status.is_empty() {
        ui.spacing();
        crate::gui::muted(ui, &w.status);
    }
}

/// What the plan's pointer is on.
#[derive(Clone, Copy, PartialEq)]
enum Target {
    Cell([u16; 2]),
    Door(Door),
}

fn plan(ui: &Ui, e: &mut Editor, w: &mut Window) {
    let draw = ui.get_window_draw_list();
    let origin = ui.cursor_screen_pos();
    let area = ui.content_region_avail();
    draw.add_rect(origin, [origin[0] + area[0], origin[1] + area[1]], CANVAS)
        .filled(true)
        .build();
    let [cw, ch] = w.room.size.map(f32::from);
    let margin = 36.;
    let unit = ((area[0] - 2. * margin) / cw)
        .min((area[1] - 2. * margin - 24.) / ch)
        .max(4.);
    let offset = [origin[0] + (area[0] - unit * cw) * 0.5, origin[1] + margin];
    // +Z runs up the screen, as in the Scene view from above.
    let at = |x: f32, z: f32| [offset[0] + x * unit, offset[1] + (ch - z) * unit];
    // Two points as a rectangle's top-left and bottom-right corners.
    let norm = |(a, b): ([f32; 2], [f32; 2])| {
        (
            [a[0].min(b[0]), a[1].min(b[1])],
            [a[0].max(b[0]), a[1].max(b[1])],
        )
    };
    // Floor and closed cells.
    for y in 0..w.room.size[1] {
        for x in 0..w.room.size[0] {
            let color = if w.room.floor([x, y]) { FLOOR } else { CLOSED };
            let (a, b) = norm((
                at(f32::from(x), f32::from(y)),
                at(f32::from(x) + 1., f32::from(y) + 1.),
            ));
            draw.add_rect(a, b, color).filled(true).build();
            draw.add_rect(a, b, GRID).build();
        }
    }
    // The content from above, in cells.
    let cell = w.room.cell;
    let ids: std::collections::BTreeMap<Uuid, usize> = w
        .room
        .actors
        .iter()
        .enumerate()
        .map(|(i, a)| (a.id, i))
        .collect();
    fn matrix(
        room: &Room,
        ids: &std::collections::BTreeMap<Uuid, usize>,
        i: usize,
        depth: usize,
    ) -> crate::transform::Matrix {
        let a = &room.actors[i];
        let local = crate::transform::Matrix::trs(a.position, a.rotation, a.scale);
        match a.attach.as_ref().and_then(|p| ids.get(&p.actor)) {
            Some(&p) if depth < 32 => matrix(room, ids, p, depth + 1).compose(local),
            _ => local,
        }
    }
    for (i, actor) in w.room.actors.iter().enumerate() {
        let m = matrix(&w.room, &ids, i, 0);
        let footprint = |lo: [f32; 3], hi: [f32; 3]| {
            let mut min = [f32::MAX; 2];
            let mut max = [f32::MIN; 2];
            for k in 0..8 {
                let p = m.point(std::array::from_fn(|axis| {
                    if k >> axis & 1 == 0 {
                        lo[axis]
                    } else {
                        hi[axis]
                    }
                }));
                min = [min[0].min(p[0]), min[1].min(p[2])];
                max = [max[0].max(p[0]), max[1].max(p[2])];
            }
            norm((
                at(min[0] / cell, min[1] / cell),
                at(max[0] / cell, max[1] / cell),
            ))
        };
        let centre = m.point([0.; 3]);
        let dot = at(centre[0] / cell, centre[2] / cell);
        if actor.light.is_some() {
            draw.add_circle(dot, 5., LIGHT).filled(true).build();
            continue;
        }
        if !dungeon_room::is_static(actor) {
            draw.add_circle(dot, 5., ACTOR).filled(true).build();
            draw.add_circle(dot, 5., [0.05, 0.05, 0.06, 1.]).build();
            continue;
        }
        if actor.kind == "Mesh" {
            let bounds = actor
                .editable_mesh
                .as_ref()
                .and_then(|mesh| e.assets.index.resolve(mesh.asset).ok())
                .and_then(|r| crate::mesh::document(r).ok())
                .filter(|d| !d.vertices.is_empty())
                .map(|d| {
                    (
                        [0, 1, 2].map(|k| d.vertices.iter().map(|v| v[k]).fold(f32::MAX, f32::min)),
                        [0, 1, 2].map(|k| d.vertices.iter().map(|v| v[k]).fold(f32::MIN, f32::max)),
                    )
                })
                .unwrap_or(([-0.5; 3], [0.5; 3]));
            let (a, b) = footprint(bounds.0, bounds.1);
            draw.add_rect(a, b, GEOMETRY).filled(true).build();
        }
        if let Some(collider) = &actor.collider {
            let parts: Vec<([f32; 3], [f32; 3])> = if collider.boxes.is_empty() {
                let (c, h) = (collider.center, collider.half_extents);
                vec![(
                    std::array::from_fn(|k| c[k] - h[k]),
                    std::array::from_fn(|k| c[k] + h[k]),
                )]
            } else {
                collider
                    .boxes
                    .iter()
                    .map(|b| ([b[0], b[1], b[2]], [b[3], b[4], b[5]]))
                    .collect()
            };
            for (lo, hi) in parts {
                let (a, b) = footprint(lo, hi);
                draw.add_rect(a, b, COLLIDER).thickness(1.5).build();
            }
        }
    }
    // Doors as bars just outside their edge, spots as rings.
    let door_rect = |door: &Door| {
        let [x, y] = door.cell.map(f32::from);
        let t = 0.18;
        norm(match door.side {
            Side::North => (at(x + 0.15, y - t), at(x + 0.85, y)),
            Side::South => (at(x + 0.15, y + 1.), at(x + 0.85, y + 1. + t)),
            Side::West => (at(x - t, y + 0.15), at(x, y + 0.85)),
            Side::East => (at(x + 1., y + 0.15), at(x + 1. + t, y + 0.85)),
        })
    };
    for door in &w.room.doors {
        let (a, b) = door_rect(door);
        draw.add_rect(a, b, DOOR).filled(true).rounding(2.).build();
    }
    for spot in &w.room.spots {
        let c = at(f32::from(spot[0]) + 0.5, f32::from(spot[1]) + 0.5);
        draw.add_circle(c, unit * 0.18, SPOT).thickness(2.5).build();
    }
    // What the pointer is on: an edge just outside the footprint, or a cell.
    let mouse = ui.io().mouse_pos;
    let local = [
        (mouse[0] - offset[0]) / unit,
        ch - (mouse[1] - offset[1]) / unit,
    ];
    let mut target = None;
    if ui.is_window_hovered() {
        let [w_cells, h_cells] = [cw, ch];
        let (x, y) = (local[0].floor(), local[1].floor());
        if (0. ..w_cells).contains(&local[0]) && (0. ..h_cells).contains(&local[1]) {
            target = Some(Target::Cell([x as u16, y as u16]));
        } else {
            let outside = 0.6;
            let door = if (-outside..0.).contains(&local[0]) && (0. ..h_cells).contains(&local[1]) {
                Some(Door {
                    cell: [0, y as u16],
                    side: Side::West,
                })
            } else if (w_cells..w_cells + outside).contains(&local[0])
                && (0. ..h_cells).contains(&local[1])
            {
                Some(Door {
                    cell: [w.room.size[0] - 1, y as u16],
                    side: Side::East,
                })
            } else if (-outside..0.).contains(&local[1]) && (0. ..w_cells).contains(&local[0]) {
                Some(Door {
                    cell: [x as u16, 0],
                    side: Side::North,
                })
            } else if (h_cells..h_cells + outside).contains(&local[1])
                && (0. ..w_cells).contains(&local[0])
            {
                Some(Door {
                    cell: [x as u16, w.room.size[1] - 1],
                    side: Side::South,
                })
            } else {
                None
            };
            target = door.map(Target::Door);
        }
    }
    match target {
        Some(Target::Cell(c)) => {
            let (a, b) = norm((
                at(f32::from(c[0]), f32::from(c[1])),
                at(f32::from(c[0]) + 1., f32::from(c[1]) + 1.),
            ));
            draw.add_rect(a, b, HOVER).thickness(2.).build();
            ui.tooltip_text(if ui.io().key_shift {
                "Shift-click: leave this cell out of the floor, or put it back"
            } else if w.room.spots.contains(&c) {
                "A spot for keys and items: click to remove it"
            } else {
                "Click: keys and items go here first"
            });
            if ui.is_mouse_clicked(MouseButton::Left) {
                w.checkpoint();
                if ui.io().key_shift {
                    if let Some(i) = w.room.closed.iter().position(|p| *p == c) {
                        w.room.closed.remove(i);
                    } else {
                        w.room.closed.push(c);
                        w.room.spots.retain(|s| *s != c);
                        w.room.doors.retain(|d| d.cell != c);
                    }
                } else if let Some(i) = w.room.spots.iter().position(|p| *p == c) {
                    w.room.spots.remove(i);
                } else if w.room.floor(c) {
                    w.room.spots.push(c);
                }
            }
        }
        Some(Target::Door(door)) => {
            let (a, b) = door_rect(&door);
            draw.add_rect(a, b, HOVER)
                .thickness(2.)
                .rounding(2.)
                .build();
            let exists = w.room.doors.contains(&door);
            ui.tooltip_text(if exists {
                "A door: corridors can meet the room here. Click to remove it"
            } else {
                "Click to add a door: corridors can meet the room here"
            });
            if ui.is_mouse_clicked(MouseButton::Left) && w.room.floor(door.cell) {
                w.checkpoint();
                if exists {
                    w.room.doors.retain(|d| *d != door);
                } else {
                    w.room.doors.push(door);
                }
            }
        }
        None => {}
    }
    // Legend.
    let y = origin[1] + area[1] - 22.;
    let mut x = origin[0] + 12.;
    for (color, label) in [
        (GEOMETRY, "static geometry"),
        (COLLIDER, "collider"),
        (LIGHT, "light"),
        (ACTOR, "actor"),
        (DOOR, "door"),
        (SPOT, "spot"),
    ] {
        draw.add_rect([x, y + 3.], [x + 12., y + 15.], color)
            .filled(true)
            .rounding(2.)
            .build();
        draw.add_text([x + 16., y], [0.75, 0.77, 0.82, 1.], label);
        x += 30. + ui.calc_text_size(label)[0];
    }
}
