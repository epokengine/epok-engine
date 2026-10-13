//! The Dungeon Generator in the editor: creating one, its Inspector band (settings,
//! warnings with fixes and the PSX budget), the layout preview in the Scene view, and
//! generation with undo.
use crate::dungeon::{self, Generator, Layout};
use crate::dungeon_bake::{self, Report, SCENE_QUADS};
use crate::editor::Editor;
use imgui::Ui;
use std::collections::BTreeMap;
use uuid::Uuid;

pub const ICON: &str = "\u{f6d9}";
const ICON_GENERATE: &str = "\u{f2f1}";
const ICON_SEED: &str = "\u{f522}";
const AMBER: [f32; 4] = [1., 0.72, 0.3, 1.];

/// What the preview knows about a generator's current settings.
#[derive(Clone)]
pub struct Preview {
    pub fingerprint: String,
    pub layout: Result<Layout, String>,
    pub budget: Option<Budget>,
}

/// The PSX budget after generating with the current settings.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Budget {
    pub actors: usize,
    pub quads: usize,
    pub boxes: usize,
    pub navigation: Option<usize>,
    pub tiles: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Request {
    Generate(Uuid),
    NewSeed(Uuid),
    Clear(Uuid),
    Reset(Uuid),
    ResetTransform(Uuid),
}

#[derive(Default)]
pub struct State {
    previews: BTreeMap<Uuid, Preview>,
    pub reports: BTreeMap<Uuid, Report>,
    pending: Option<Request>,
}

/// The generator placed by actor `actor`, if any.
pub fn generator_of(e: &Editor, actor: Uuid) -> Option<&Generator> {
    e.scene.dungeons.iter().find(|g| g.root == actor)
}

// ---- creation ------------------------------------------------------------------------------

/// Creates a 3D Dungeon Generator at the map origin and generates its first dungeon.
pub fn create_3d(e: &mut Editor) {
    if e.playing {
        e.log("Stop the emulator before editing.");
        return;
    }
    let mut root = crate::scene::Actor::cube(e.scene.unique_actor_name("Dungeon"));
    root.kind = "Empty".into();
    root.position = [0.; 3];
    let id = root.id;
    e.scene.actors.push(root);
    e.scene.refresh_actor_hierarchy();
    attach(e, id);
}

/// Makes actor `id` a Dungeon Generator, selects it and generates a first dungeon so
/// the author sees a result straight away.
pub fn attach(e: &mut Editor, id: Uuid) {
    let mut generator = Generator::new(id);
    // A fresh generator starts from a different dungeon each time.
    generator.seed = (generator.id.as_u128() as u32) % 100_000;
    let gid = generator.id;
    e.scene.dungeons.push(generator);
    e.select_actor(Some(id));
    e.changed();
    generate(e, gid);
    frame(e, gid);
}

/// Points the Scene view at a generator's rooms and corridors.
pub fn frame(e: &mut Editor, id: Uuid) {
    let Some(g) = e.scene.dungeons.iter().find(|g| g.id == id).cloned() else {
        return;
    };
    let Some(index) = e.scene.actor_index(g.root) else {
        return;
    };
    // The occupied cells, with a one-cell margin for the walls; the whole grid when
    // the settings do not produce a layout.
    let size = [i32::from(g.rooms.size[0]), i32::from(g.rooms.size[1])];
    let (lo, hi) = match dungeon::layout(&g) {
        Ok(layout) => {
            let cells: Vec<[i32; 2]> = (0..size[1])
                .flat_map(|y| (0..size[0]).map(move |x| [x, y]))
                .filter(|p| layout.at(*p).walkable())
                .collect();
            let lo = [0, 1].map(|k| cells.iter().map(|p| p[k]).min().unwrap_or(0) - 1);
            let hi = [0, 1].map(|k| cells.iter().map(|p| p[k]).max().unwrap_or(size[k]) + 2);
            (lo, hi)
        }
        Err(_) => ([0, 0], size),
    };
    if dungeon_bake::is_2d(&e.scene, g.root) {
        let tile = g
            .tile_set
            .and_then(|t| e.scene.tilesets.get(&t))
            .map_or([16., 16.], |s| s.tile_size.map(f32::from));
        let origin = e.scene.actors[index]
            .root()
            .map(|c| crate::scene_view_mode::read_transform_2d(&c.properties).position)
            .unwrap_or_default();
        let at = |cell: [i32; 2]| -> [f32; 2] {
            std::array::from_fn(|k| origin[k] + (cell[k] - size[k] / 2) as f32 * tile[k])
        };
        e.view_2d.frame(at(lo), at(hi), [960., 600.]);
    } else {
        let frame = dungeon_bake::Frame {
            size,
            cell: g.cell_size,
        };
        let matrix = e.scene.world_matrix(index);
        let low = matrix.point([frame.x(lo[0]), 0., frame.z(lo[1])]);
        let high = matrix.point([frame.x(hi[0]), g.wall_height, frame.z(hi[1])]);
        e.view.frame_bounds(low, high);
    }
    e.view_dirty = true;
}

// ---- generation ----------------------------------------------------------------------------

/// Generates and records one undo step. On failure the map is left untouched.
pub fn generate(e: &mut Editor, id: Uuid) {
    if e.playing {
        e.log("Stop the emulator before generating.");
        return;
    }
    let before = e.scene.clone();
    let selected = e.selected_actor;
    match dungeon_bake::generate(&e.root, &mut e.scene, id) {
        Ok(report) => {
            e.assets.refresh();
            let index = crate::assets::scan(&e.root, &mut Default::default());
            let _ = crate::mesh::resolve(&mut e.scene, &index);
            let _ = crate::tileset::resolve(&mut e.scene, &index);
            let _ = crate::texture::resolve(&mut e.scene, &index);
            e.log(format!("Dungeon generated: {}", report.summary));
            for note in &report.notes {
                e.log(note.clone());
            }
            e.dungeon.reports.insert(id, report);
            e.last_error = None;
            if selected.is_some() {
                e.select_actor(selected);
            }
            e.changed();
        }
        Err(error) => {
            e.scene = before;
            e.select_actor(selected);
            e.last_error = Some(format!("Dungeon not generated: {error}"));
            e.log(format!("Dungeon not generated: {error}"));
        }
    }
}

/// Runs the command the Inspector requested last frame, outside any borrowed panel.
pub fn apply_pending(e: &mut Editor) {
    let Some(request) = e.dungeon.pending.take() else {
        return;
    };
    match request {
        Request::Generate(id) => generate(e, id),
        Request::NewSeed(id) => {
            if let Some(g) = e.scene.dungeons.iter_mut().find(|g| g.id == id) {
                g.seed = next_seed(g.seed);
                generate(e, id);
            }
        }
        Request::Clear(id) => {
            dungeon_bake::clear(&mut e.scene, id);
            e.changed();
            e.log("Generated dungeon content removed; the generator keeps its settings.");
        }
        Request::Reset(id) => {
            if let Some(g) = e.scene.dungeons.iter_mut().find(|g| g.id == id) {
                let keep = (g.id, g.root, g.seed, g.output.clone());
                *g = Generator {
                    id: keep.0,
                    root: keep.1,
                    seed: keep.2,
                    output: keep.3,
                    ..Generator::default()
                };
                e.changed();
            }
        }
        Request::ResetTransform(id) => {
            if let Some(root) = e.scene.dungeons.iter().find(|g| g.id == id).map(|g| g.root)
                && let Some(index) = e.scene.actor_index(root)
            {
                e.scene.actors[index].rotation = [0.; 3];
                e.scene.actors[index].scale = [1.; 3];
                e.changed();
            }
        }
    }
}

/// A different seed that is still easy to read and type.
pub fn next_seed(seed: u32) -> u32 {
    let mut random = crate::dungeon_random::Random::new(seed ^ 0x5eed);
    loop {
        let candidate = random.below(100_000);
        if candidate != seed {
            return candidate;
        }
    }
}

/// Resolves the generator an MCP request names, creating one or applying overrides
/// first. Works on a copy of the scene the caller commits.
pub fn mcp_target(scene: &mut crate::scene::Scene, a: &serde_json::Value) -> Result<Uuid, String> {
    let id = if let Some(domain) = a["create"].as_str() {
        let root = if domain == "2d" {
            crate::world2d::create(
                crate::world2d::Preset::DungeonGenerator,
                &scene.unique_actor_name("Dungeon"),
                [0., 0.],
            )
        } else {
            let mut root = crate::scene::Actor::cube(scene.unique_actor_name("Dungeon"));
            root.kind = "Empty".into();
            root.position = [0.; 3];
            root
        };
        let generator = Generator::new(root.id);
        let id = generator.id;
        scene.actors.push(root);
        scene.refresh_actor_hierarchy();
        scene.dungeons.push(generator);
        id
    } else {
        let wanted = a["actor"]
            .as_str()
            .ok_or("Name the generator's actor, or pass create \"3d\" or \"2d\"")?;
        let actor = scene
            .actors
            .iter()
            .find(|x| x.name == wanted || x.id.to_string() == wanted)
            .ok_or_else(|| format!("No actor named {wanted}"))?;
        scene
            .dungeons
            .iter()
            .find(|g| g.root == actor.id)
            .map(|g| g.id)
            .ok_or_else(|| format!("{wanted} is not a Dungeon Generator"))?
    };
    let index = scene
        .dungeons
        .iter()
        .position(|g| g.id == id)
        .expect("generator just resolved");
    let mut value = serde_json::to_value(&scene.dungeons[index]).map_err(|e| e.to_string())?;
    if let Some(settings) = a.get("settings").and_then(|v| v.as_object()) {
        for (key, field) in settings {
            // The spawn actor may be named instead of identified.
            let field = &match (key.as_str(), field.as_str()) {
                ("spawn_actor", Some(name)) if Uuid::parse_str(name).is_err() => {
                    let actor = scene
                        .actors
                        .iter()
                        .find(|x| x.name == name)
                        .ok_or_else(|| format!("No actor named {name} to move to the spawn"))?;
                    serde_json::json!(actor.id)
                }
                _ => field.clone(),
            };
            if matches!(key.as_str(), "id" | "root" | "output") {
                return Err(format!("{key} is managed by the generator"));
            }
            match (value.get_mut(key), field) {
                (Some(serde_json::Value::Object(current)), serde_json::Value::Object(patch)) => {
                    for (k, v) in patch {
                        current.insert(k.clone(), v.clone());
                    }
                }
                (Some(slot), _) => *slot = field.clone(),
                // Unset optional references are omitted from the document.
                (None, _) if matches!(key.as_str(), "spawn_actor" | "tile_set") => {
                    value[key] = field.clone();
                }
                (None, _) => return Err(format!("Unknown generator setting {key}")),
            }
        }
    }
    if let Some(seed) = a["seed"].as_u64() {
        value["seed"] = serde_json::json!(seed.min(u64::from(u32::MAX)));
    }
    let generator: Generator = serde_json::from_value(value).map_err(|e| e.to_string())?;
    generator.validate()?;
    scene.dungeons[index] = generator;
    Ok(id)
}

// ---- preview -------------------------------------------------------------------------------

/// The layout and budget of a generator's current settings, cached by fingerprint.
pub fn preview(e: &mut Editor, id: Uuid) -> Option<Preview> {
    let generator = e.scene.dungeons.iter().find(|g| g.id == id)?.clone();
    let fingerprint = format!(
        "{}-{}",
        generator.fingerprint(),
        dungeon_bake::is_2d(&e.scene, generator.root)
    );
    if let Some(cached) = e.dungeon.previews.get(&id)
        && cached.fingerprint == fingerprint
    {
        return Some(cached.clone());
    }
    let layout = generator
        .validate()
        .and_then(|_| dungeon::layout(&generator));
    let budget = layout
        .as_ref()
        .ok()
        .and_then(|layout| budget(&e.scene, &generator, layout).ok());
    let preview = Preview {
        fingerprint,
        layout,
        budget,
    };
    e.dungeon.previews.insert(id, preview.clone());
    Some(preview)
}

fn budget(scene: &crate::scene::Scene, g: &Generator, layout: &Layout) -> Result<Budget, String> {
    let output: std::collections::BTreeSet<Uuid> = g
        .output
        .actors
        .iter()
        .flat_map(|a| scene.actor_branch(*a))
        .collect();
    let others = scene.actors.iter().filter(|a| !output.contains(&a.id));
    if dungeon_bake::is_2d(scene, g.root) {
        let tiles = (0..layout.size[1])
            .flat_map(|y| (0..layout.size[0]).map(move |x| [x, y]))
            .filter(|p| layout.at(*p).walkable() || dungeon_bake::wall_cell(layout, *p))
            .count();
        return Ok(Budget {
            actors: others.count() + 3,
            tiles,
            ..Default::default()
        });
    }
    let other_quads: usize = others.clone().map(crate::lighting::quad_count).sum();
    let plan = dungeon_bake::plan_3d(g, layout)?;
    let light = !scene
        .actors
        .iter()
        .any(|a| a.light.is_some() && !output.contains(&a.id));
    let generated = plan.sections.len()
        + 3
        + plan.navigation.as_ref().map_or(0, |n| n.volumes.len())
        + usize::from(light);
    Ok(Budget {
        actors: others.count() + generated,
        quads: other_quads + plan.quads,
        boxes: plan.boxes.len(),
        navigation: plan.navigation.as_ref().map(|n| n.nodes),
        tiles: 0,
    })
}

/// Lines drawing the selected generator's layout in the 3D view: rooms, corridors,
/// doors, spawn and goal, at floor level until generated and above the walls after.
pub fn preview_lines(
    scene: &crate::scene::Scene,
    selected: Option<usize>,
) -> Vec<([f32; 3], [f32; 3], [u8; 3])> {
    let mut lines = Vec::new();
    let Some(actor) = selected.and_then(|i| scene.actors.get(i)) else {
        return lines;
    };
    let Some(g) = scene.dungeons.iter().find(|g| g.root == actor.id) else {
        return lines;
    };
    if dungeon_bake::is_2d(scene, g.root) || g.validate().is_err() {
        return lines;
    }
    let Ok(layout) = dungeon::layout(g) else {
        return lines;
    };
    let Some(root) = scene.actor_index(g.root) else {
        return lines;
    };
    let matrix = scene.world_matrix(root);
    let frame = dungeon_bake::Frame {
        size: layout.size,
        cell: g.cell_size,
    };
    let y = if g.output.is_empty() || g.stale() {
        0.06
    } else {
        g.wall_height + 0.06
    };
    let point = |x: f32, z: f32| matrix.point([x, y, z]);
    let rect = |x0: i32, z0: i32, x1: i32, z1: i32, color: [u8; 3], lines: &mut Vec<_>| {
        let corners = [
            point(frame.x(x0), frame.z(z0)),
            point(frame.x(x1), frame.z(z0)),
            point(frame.x(x1), frame.z(z1)),
            point(frame.x(x0), frame.z(z1)),
        ];
        for i in 0..4 {
            lines.push((corners[i], corners[(i + 1) % 4], color));
        }
    };
    // The grid's outline shows how much room the layout may use.
    rect(
        0,
        0,
        layout.size[0],
        layout.size[1],
        [90, 96, 110],
        &mut lines,
    );
    for (index, room) in layout.rooms.iter().enumerate() {
        let color = if index == layout.spawn {
            [90, 230, 120]
        } else if index == layout.goal {
            [240, 110, 90]
        } else {
            [110, 190, 255]
        };
        rect(
            room.min[0],
            room.min[1],
            room.min[0] + room.size[0],
            room.min[1] + room.size[1],
            color,
            &mut lines,
        );
    }
    // Corridor edges facing a wall.
    for spot in dungeon::spots(&layout) {
        let Some(side) = spot.side else { continue };
        if !spot.corridor && spot.kind != dungeon::SpotKind::Door {
            continue;
        }
        let [x, z] = spot.cell;
        let (a, b) = match side {
            dungeon::Side::North => ((x, z), (x + 1, z)),
            dungeon::Side::South => ((x, z + 1), (x + 1, z + 1)),
            dungeon::Side::West => ((x, z), (x, z + 1)),
            dungeon::Side::East => ((x + 1, z), (x + 1, z + 1)),
        };
        let color = if spot.kind == dungeon::SpotKind::Door {
            [255, 214, 90]
        } else {
            [200, 160, 110]
        };
        lines.push((
            point(frame.x(a.0), frame.z(a.1)),
            point(frame.x(b.0), frame.z(b.1)),
            color,
        ));
    }
    lines
}

// ---- Inspector -----------------------------------------------------------------------------

/// The Dungeon Generator band of a generator's root actor, 2D or 3D. Edits change the
/// map document directly; commands that add or remove actors run next frame.
pub fn inspector(ui: &Ui, e: &mut Editor, actor: Uuid) {
    let Some(id) = generator_of(e, actor).map(|g| g.id) else {
        return;
    };
    let two_d = dungeon_bake::is_2d(&e.scene, actor);
    let mut request = None;
    let open = crate::gui::section(ui, &format!("{ICON} Dungeon Generator##dungeon"), || {
        if crate::gui::script_menu_item(ui, "Generate") {
            request = Some(Request::Generate(id));
        }
        if crate::gui::script_menu_item(ui, "New seed and generate") {
            request = Some(Request::NewSeed(id));
        }
        ui.separator();
        if crate::gui::script_menu_item(ui, "Remove generated content") {
            request = Some(Request::Clear(id));
        }
        if crate::gui::script_menu_item(ui, "Reset settings") {
            request = Some(Request::Reset(id));
        }
    });
    if open {
        let preview = preview(e, id);
        let width = ui.content_region_avail()[0];
        let half = ((width - 6.) * 0.5).max(60.);
        if ui.button_with_size(format!("{ICON_GENERATE} Generate"), [half, 0.]) {
            request = Some(Request::Generate(id));
        }
        #[cfg(test)]
        crate::gui::record_script_control(ui, "Generate dungeon");
        if ui.is_item_hovered() {
            ui.tooltip_text(
                "Build the dungeon from these settings into the map (Ctrl+Z undoes it).",
            );
        }
        ui.same_line();
        if ui.button_with_size(format!("{ICON_SEED} New seed"), [half, 0.]) {
            request = Some(Request::NewSeed(id));
        }
        #[cfg(test)]
        crate::gui::record_script_control(ui, "New dungeon seed");
        if ui.is_item_hovered() {
            ui.tooltip_text("Pick another seed and generate a different dungeon.");
        }
        warnings(ui, e, id, two_d, preview.as_ref(), &mut request);
        settings(ui, e, id, two_d);
        if let Some(preview) = &preview {
            budget_rows(ui, two_d, preview);
        }
    }
    if request.is_some() {
        e.dungeon.pending = request;
    }
    if !ui.is_any_item_active() {
        e.end_coalesced();
    }
}

fn warnings(
    ui: &Ui,
    e: &Editor,
    id: Uuid,
    two_d: bool,
    preview: Option<&Preview>,
    request: &mut Option<Request>,
) {
    let Some(g) = e.scene.dungeons.iter().find(|g| g.id == id) else {
        return;
    };
    let mut list: Vec<(String, Option<(&str, Request)>)> = Vec::new();
    if let Err(error) = g.validate() {
        list.push((error, None));
    } else if let Some(Err(error)) = preview.map(|p| &p.layout) {
        list.push((error.clone(), None));
    }
    if g.output.is_empty() {
        list.push((
            "Nothing generated yet.".into(),
            Some(("Generate", Request::Generate(id))),
        ));
    } else if g.stale() {
        list.push((
            "The settings changed since the dungeon was generated.".into(),
            Some(("Regenerate", Request::Generate(id))),
        ));
    }
    if !two_d {
        if let Some(index) = e.scene.actor_index(g.root)
            && !crate::collision::translation_only(e.scene.world_matrix(index))
        {
            list.push((
                "Generated collision needs a generator that is only moved, not rotated or scaled."
                    .into(),
                Some(("Reset rotation and scale", Request::ResetTransform(id))),
            ));
        }
        if g.half_extent() > 127. {
            list.push((
                format!(
                    "The grid spans {:.0} units; keep it within 254 (smaller cells or grid).",
                    g.half_extent() * 2.
                ),
                None,
            ));
        }
    }
    if let Some(Ok(layout)) = preview.map(|p| &p.layout)
        && layout.rooms.len() < layout.requested
    {
        list.push((
            format!(
                "Only {} of {} rooms fit: enlarge the grid or shrink the rooms.",
                layout.rooms.len(),
                layout.requested
            ),
            None,
        ));
    }
    if let Some(budget) = preview.and_then(|p| p.budget.as_ref()) {
        if budget.actors > 64 {
            list.push((
                format!(
                    "The map would hold {} actors; a map holds 64.",
                    budget.actors
                ),
                None,
            ));
        }
        if budget.quads > SCENE_QUADS {
            list.push((
                format!(
                    "The map would hold {} quads; without streaming a map holds {SCENE_QUADS}.",
                    budget.quads
                ),
                None,
            ));
        }
        if g.navigation && budget.navigation.is_none() && !two_d {
            list.push((
                "The floor is too large for navigation; it will be skipped.".into(),
                None,
            ));
        }
    }
    if list.is_empty() {
        return;
    }
    let draw = ui.get_window_draw_list();
    let start = ui.cursor_screen_pos();
    let width = ui.content_region_avail()[0];
    draw.channels_split(2, |channels| {
        channels.set_current(1);
        ui.dummy([0., 2.]);
        for (i, (text, fix)) in list.iter().enumerate() {
            let _id = ui.push_id_usize(i);
            ui.indent_by(6.);
            ui.text_colored(AMBER, crate::world2d_warnings::ICON_WARNING);
            ui.same_line();
            ui.group(|| {
                let wrap = ui.push_text_wrap_pos_with_pos(ui.cursor_pos()[0] + width - 34.);
                ui.text(text);
                wrap.end();
                if let Some((label, fix)) = fix {
                    let pressed = ui.small_button(label);
                    #[cfg(test)]
                    crate::gui::record_script_control(ui, label);
                    if pressed {
                        *request = Some(*fix);
                    }
                }
            });
            ui.unindent_by(6.);
        }
        ui.dummy([0., 2.]);
        channels.set_current(0);
        let end = [start[0] + width, ui.cursor_screen_pos()[1] - 2.];
        draw.add_rect(start, end, [AMBER[0], AMBER[1], AMBER[2], 0.1])
            .filled(true)
            .rounding(3.)
            .build();
    });
}

fn settings(ui: &Ui, e: &mut Editor, id: Uuid, two_d: bool) {
    let Some(index) = e.scene.dungeons.iter().position(|g| g.id == id) else {
        return;
    };
    let mut g = e.scene.dungeons[index].clone();
    let actors: Vec<(Uuid, String)> = e
        .scene
        .actors
        .iter()
        .filter(|a| a.id != g.root && !g.output.actors.contains(&a.id))
        .filter(|a| !dungeon_bake::is_2d(&e.scene, a.id))
        .map(|a| (a.id, a.name.clone()))
        .collect();
    let tile_sets: Vec<(Uuid, String)> = if two_d {
        e.assets
            .index
            .assets
            .values()
            .flatten()
            .filter(|r| r.meta.kind == crate::assets::Kind::TileSet)
            .map(|r| {
                (
                    r.meta.id,
                    r.path
                        .file_stem()
                        .map(|s| s.to_string_lossy().into_owned())
                        .unwrap_or_default(),
                )
            })
            .collect()
    } else {
        Vec::new()
    };
    let mut seed = g.seed as i32;
    if crate::gui::Drag::new(crate::gui::field(ui, "Seed"))
        .range(0, i32::MAX)
        .build(ui, &mut seed)
    {
        g.seed = seed.max(0) as u32;
    }
    if ui.is_item_hovered() {
        ui.tooltip_text("The same seed and settings always give the same dungeon.");
    }
    let _ = crate::gui::field(ui, "Layout");
    ui.text("Rooms and corridors");
    ui.spacing();
    crate::gui::muted(ui, "Layout (in grid cells)");
    let s = &mut g.rooms;
    let mut size = [i32::from(s.size[0]), i32::from(s.size[1])];
    if crate::gui::Drag::new(crate::gui::field(ui, "Grid size"))
        .range(
            i32::from(dungeon::GRID_LIMITS.0),
            i32::from(dungeon::GRID_LIMITS.1),
        )
        .build_array(ui, &mut size)
    {
        s.size = size.map(|v| {
            v.clamp(
                i32::from(dungeon::GRID_LIMITS.0),
                i32::from(dungeon::GRID_LIMITS.1),
            ) as u16
        });
    }
    drag_u16(
        ui,
        "Rooms",
        &mut s.rooms,
        dungeon::ROOM_LIMITS,
        "How many rooms to place; fewer fit in a crowded grid.",
    );
    let mut room = [i32::from(s.room_min), i32::from(s.room_max)];
    if crate::gui::Drag::new(crate::gui::field(ui, "Room size"))
        .range(
            i32::from(dungeon::ROOM_SIZE_LIMITS.0),
            i32::from(dungeon::ROOM_SIZE_LIMITS.1),
        )
        .build_array(ui, &mut room)
    {
        let (lo, hi) = dungeon::ROOM_SIZE_LIMITS;
        s.room_min = room[0].clamp(i32::from(lo), i32::from(hi)) as u16;
        s.room_max = (room[1].clamp(i32::from(lo), i32::from(hi)) as u16).max(s.room_min);
    }
    if ui.is_item_hovered() {
        ui.tooltip_text("Smallest and largest room side, in cells.");
    }
    drag_u16(
        ui,
        "Spacing",
        &mut s.spacing,
        dungeon::SPACING_LIMITS,
        "Empty cells kept between rooms; corridors need at least three to pass between.",
    );
    drag_u16(
        ui,
        "Corridor width",
        &mut s.corridor_width,
        dungeon::CORRIDOR_LIMITS,
        "Corridor width in cells.",
    );
    drag_u16(
        ui,
        "Loops",
        &mut s.loops,
        (0, 100),
        "Chance, in percent, that each extra connection becomes a loop. 0 gives a tree: one way between any two rooms.",
    );
    ui.spacing();
    if two_d {
        crate::gui::muted(ui, "Tiles");
        let current = g
            .tile_set
            .and_then(|id| {
                tile_sets
                    .iter()
                    .find(|(t, _)| *t == id)
                    .map(|(_, n)| n.clone())
            })
            .unwrap_or_else(|| "Built-in dungeon tiles".into());
        if let Some(_combo) = ui.begin_combo(crate::gui::field(ui, "Tile set"), &current) {
            if ui.selectable("Built-in dungeon tiles") {
                g.tile_set = None;
            }
            for (tile_set, name) in &tile_sets {
                if ui.selectable(name) {
                    g.tile_set = Some(*tile_set);
                }
            }
        }
        if ui.is_item_hovered() {
            ui.tooltip_text(
                "Any TileSet with a terrain set holding terrains named Floor and Wall.",
            );
        }
    } else {
        crate::gui::muted(ui, "3D (in world units)");
        drag_f32(
            ui,
            "Cell size",
            &mut g.cell_size,
            dungeon::CELL_SIZE_LIMITS,
            "World units per grid cell.",
        );
        drag_f32(
            ui,
            "Wall height",
            &mut g.wall_height,
            dungeon::WALL_HEIGHT_LIMITS,
            "Height of walls and their tops.",
        );
        drag_f32(
            ui,
            "Door height",
            &mut g.door_height,
            (0.5, g.wall_height),
            "Doorways are open below this height; a beam fills the rest.",
        );
        let _ = crate::gui::toggle(ui, "Navigation", &mut g.navigation);
        if ui.is_item_hovered() {
            ui.tooltip_text(
                "Add navigation bake volumes over the floor, spaced to stay within 512 nodes.",
            );
        }
        let current = g
            .spawn_actor
            .and_then(|id| {
                actors
                    .iter()
                    .find(|(a, _)| *a == id)
                    .map(|(_, n)| n.clone())
            })
            .unwrap_or_else(|| "None".into());
        if let Some(_combo) = ui.begin_combo(crate::gui::field(ui, "Move to spawn"), &current) {
            if ui.selectable("None") {
                g.spawn_actor = None;
            }
            for (actor, name) in &actors {
                if ui.selectable(format!("{name}##{actor}")) {
                    g.spawn_actor = Some(*actor);
                }
            }
        }
        if ui.is_item_hovered() {
            ui.tooltip_text("An actor placed at the spawn room's centre after each generation, usually the player.");
        }
    }
    if g != e.scene.dungeons[index] {
        e.scene.dungeons[index] = g;
        e.changed_coalesced("dungeon-settings");
    }
}

fn drag_u16(ui: &Ui, label: &str, value: &mut u16, limits: (u16, u16), tip: &str) {
    let mut v = i32::from(*value);
    if crate::gui::Drag::new(crate::gui::field(ui, label))
        .range(i32::from(limits.0), i32::from(limits.1))
        .build(ui, &mut v)
    {
        *value = v.clamp(i32::from(limits.0), i32::from(limits.1)) as u16;
    }
    if ui.is_item_hovered() {
        ui.tooltip_text(tip);
    }
}
fn drag_f32(ui: &Ui, label: &str, value: &mut f32, limits: (f32, f32), tip: &str) {
    if crate::gui::Drag::new(crate::gui::field(ui, label))
        .range(limits.0, limits.1)
        .speed(0.05)
        .display_format("%.2f")
        .build(ui, value)
    {
        *value = value.clamp(limits.0, limits.1);
    }
    if ui.is_item_hovered() {
        ui.tooltip_text(tip);
    }
}

fn meter(ui: &Ui, label: &str, used: usize, limit: usize, unit: &str) {
    let id = crate::gui::field(ui, label);
    let fraction = used as f32 / limit.max(1) as f32;
    let color = if fraction > 1. {
        [0.86, 0.33, 0.30, 1.]
    } else if fraction > 0.8 {
        AMBER
    } else {
        [0.32, 0.66, 0.44, 1.]
    };
    let _bar = ui.push_style_color(imgui::StyleColor::PlotHistogram, color);
    imgui::ProgressBar::new(fraction.min(1.))
        .overlay_text(format!("{used} / {limit} {unit}"))
        .size([-1., 0.])
        .build(ui);
    let _ = id;
}

fn budget_rows(ui: &Ui, two_d: bool, preview: &Preview) {
    ui.spacing();
    crate::gui::muted(ui, "PSX budget after generating");
    if let Some(budget) = &preview.budget {
        meter(ui, "Actors", budget.actors, 64, "actors");
        if two_d {
            meter(
                ui,
                "Tiles",
                budget.tiles,
                crate::tilemap::MAX_CELLS,
                "cells",
            );
        } else {
            meter(ui, "Geometry", budget.quads, SCENE_QUADS, "quads");
            meter(
                ui,
                "Collision",
                budget.boxes,
                crate::collision::MAX_COMPOUND_BOXES,
                "boxes",
            );
            match budget.navigation {
                Some(nodes) => meter(ui, "Navigation", nodes, 512, "nodes"),
                None => {
                    let _ = crate::gui::field(ui, "Navigation");
                    ui.text_disabled("off");
                }
            }
        }
    }
    if let Ok(layout) = &preview.layout {
        crate::gui::muted(ui, dungeon::summary(layout));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_seeds_differ_and_stay_short() {
        let mut seed = 7;
        for _ in 0..50 {
            let next = next_seed(seed);
            assert_ne!(next, seed);
            assert!(next < 100_000);
            seed = next;
        }
    }
}
