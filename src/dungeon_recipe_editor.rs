//! The Dungeon Recipe editor: the recipe's steps in order, the selected step's settings,
//! and a preview of the room graph the recipe grows, with its main path, branches,
//! keys, locks, one-way doors and items.
//!
//! Every finished edit is saved, like the Dungeon Style editor, and has its own undo
//! history; with Live preview on, the map's dungeons that follow the recipe regenerate.
use crate::dungeon_recipe::{self, Graph, MAIN, Place, Recipe, Step};
use crate::editor::Editor;
use imgui::{MouseButton, Ui};
use std::collections::BTreeMap;
use uuid::Uuid;

const ICON: &str = "\u{f70e}";
const ICON_ADD: &str = "\u{f067}";
const ICON_SEED: &str = "\u{f522}";
const AMBER: [f32; 4] = [1., 0.72, 0.3, 1.];
const CANVAS: [f32; 4] = [0.105, 0.11, 0.125, 1.];
const SLOT: [f32; 4] = [1., 1., 1., 0.07];
const LINK: [f32; 4] = [0.72, 0.74, 0.8, 0.95];
const LOCK: [f32; 4] = [0.95, 0.32, 0.26, 1.];
const KEY: [f32; 4] = [1., 0.8, 0.25, 1.];
const ONE_WAY: [f32; 4] = [0.36, 0.62, 1., 1.];
const ITEM: [f32; 4] = [0.4, 0.86, 0.56, 1.];
const SPAWN: [f32; 4] = [0.36, 0.9, 0.48, 1.];
const GOAL: [f32; 4] = [0.95, 0.42, 0.36, 1.];
/// Path colours, the main path first.
const PATHS: [[f32; 4]; 6] = [
    [0.86, 0.68, 0.32, 1.],
    [0.42, 0.62, 0.9, 1.],
    [0.62, 0.48, 0.86, 1.],
    [0.36, 0.74, 0.7, 1.],
    [0.86, 0.5, 0.62, 1.],
    [0.6, 0.72, 0.4, 1.],
];

pub struct Window {
    pub asset: Uuid,
    pub recipe: Recipe,
    undo: Vec<Recipe>,
    redo: Vec<Recipe>,
    dirty: bool,
    status: String,
    selected: Option<usize>,
    /// The seed the graph preview grows.
    pub seed: u32,
    /// Regenerate the map's dungeons that follow this recipe after each edit.
    pub live: bool,
    /// The preview for (recipe, seed), kept until either changes.
    preview: Option<Preview>,
    /// The project's Dungeon Rooms, by id and name, for the Handmade rooms step.
    rooms: Vec<(Uuid, String)>,
    /// The footprints of the rooms the recipe uses, for the preview.
    shapes: BTreeMap<Uuid, crate::dungeon_room::Shape>,
}

/// A recipe and seed, the graph they grow, and each node's floor level (when the recipe
/// has levels) and handmade room (when it has handmade rooms).
type Preview = (
    Recipe,
    u32,
    Result<Graph, String>,
    Vec<u8>,
    Vec<Option<Uuid>>,
);
impl Window {
    fn new(asset: Uuid, recipe: Recipe, seed: u32) -> Self {
        Self {
            asset,
            recipe,
            undo: Vec::new(),
            redo: Vec::new(),
            dirty: false,
            status: String::new(),
            selected: Some(0),
            seed,
            live: true,
            preview: None,
            rooms: Vec::new(),
            shapes: BTreeMap::new(),
        }
    }
    fn checkpoint(&mut self) {
        self.undo.push(self.recipe.clone());
        if self.undo.len() > 64 {
            self.undo.remove(0);
        }
        self.redo.clear();
        self.dirty = true;
    }
    fn graph(&mut self) -> &Result<Graph, String> {
        let fresh = self
            .preview
            .as_ref()
            .is_some_and(|(r, s, _, _, _)| *r == self.recipe && *s == self.seed);
        if !fresh {
            let graph = dungeon_recipe::graph(&self.recipe, self.seed);
            // Levels belong to the laid-out rooms: room i is the i-th node with a room.
            let mut levels = Vec::new();
            let mut made = Vec::new();
            let laid_out = self.recipe.levels().is_some() || !self.recipe.rooms_used().is_empty();
            if let (Ok(graph), true) = (&graph, laid_out)
                && let Ok(layout) =
                    dungeon_recipe::layout_with(&self.recipe, self.seed, &self.shapes, &[])
            {
                levels = vec![0; graph.path.len()];
                made = vec![None; graph.path.len()];
                let nodes = (0..graph.path.len()).filter(|n| graph.active(*n));
                for (i, node) in nodes.take(layout.rooms.len()).enumerate() {
                    levels[node] = layout.level(layout.room_floor_center(i));
                    made[node] = layout.handmade_room(i).map(|h| h.room);
                }
            }
            self.preview = Some((self.recipe.clone(), self.seed, graph, levels, made));
        }
        &self.preview.as_ref().expect("just computed").2
    }
    /// Each node's handmade room in the current preview; empty without handmade rooms.
    fn made(&self) -> Vec<Option<Uuid>> {
        self.preview
            .as_ref()
            .map(|p| p.4.clone())
            .unwrap_or_default()
    }
    /// Each node's floor level in the current preview; empty without levels.
    fn levels(&self) -> Vec<u8> {
        self.preview
            .as_ref()
            .map(|p| p.3.clone())
            .unwrap_or_default()
    }
}

// ---- opening and creating ------------------------------------------------------------------

/// Opens the editor on recipe `id`, previewing the seed of a dungeon that follows it.
pub fn open(e: &mut Editor, id: Uuid) {
    if e.dungeon_recipe.as_ref().is_some_and(|w| w.asset == id) {
        return;
    }
    let recipe = e
        .assets
        .index
        .resolve(id)
        .ok()
        .and_then(|r| dungeon_recipe::document(r).ok())
        .or_else(|| {
            let index = crate::assets::scan(&e.root, &mut Default::default());
            index
                .resolve(id)
                .ok()
                .and_then(|r| dungeon_recipe::document(r).ok())
        });
    let seed = e
        .scene
        .dungeons
        .iter()
        .find(|g| g.recipe == Some(id))
        .map_or(1, |g| g.seed);
    match recipe {
        Some(recipe) => e.dungeon_recipe = Some(Window::new(id, recipe, seed)),
        None => {
            let error = format!("Dungeon Recipe {id} could not be opened");
            e.last_error = Some(error.clone());
            e.log(error);
        }
    }
}

/// Creates a recipe asset starting from `from` and opens it.
pub fn create(e: &mut Editor, name: &str, from: Recipe) -> Result<Uuid, String> {
    let path = dungeon_recipe::fresh_path(&e.root, name);
    let id = dungeon_recipe::create(&e.root, &path, &from)?;
    e.assets.index = crate::assets::scan(&e.root, &mut Default::default());
    e.assets.refresh();
    open(e, id);
    e.log(format!("Created {path}."));
    Ok(id)
}

/// Selects step `index` of the open recipe, for screenshots.
pub fn select_step(e: &mut Editor, index: usize) {
    if let Some(w) = &mut e.dungeon_recipe {
        w.selected = Some(index.min(w.recipe.steps.len().saturating_sub(1)));
    }
}

fn store(e: &mut Editor, w: &Window) -> Result<(), String> {
    let record = match e.assets.index.resolve(w.asset) {
        Ok(record) => record.clone(),
        Err(_) => {
            e.assets.index = crate::assets::scan(&e.root, &mut Default::default());
            e.assets.index.resolve(w.asset)?.clone()
        }
    };
    dungeon_recipe::save(&record, &w.recipe)?;
    e.assets.index = crate::assets::scan(&e.root, &mut Default::default());
    e.assets.refresh();
    Ok(())
}

// ---- window --------------------------------------------------------------------------------

pub fn window(ui: &Ui, e: &mut Editor) {
    let Some(mut w) = e.dungeon_recipe.take() else {
        return;
    };
    let name = e
        .assets
        .index
        .resolve(w.asset)
        .ok()
        .and_then(|r| r.path.file_stem().map(|s| s.to_string_lossy().into_owned()))
        .unwrap_or_else(|| "Dungeon Recipe".into());
    let mut open = true;
    ui.window(format!(
        "{ICON} Dungeon Recipe: {name}###dungeon-recipe-editor"
    ))
    .opened(&mut open)
    .size([1040., 600.], imgui::Condition::FirstUseEver)
    .position([280., 340.], imgui::Condition::FirstUseEver)
    .build(|| body(ui, e, &mut w));
    // A finished edit is saved when valid, then previewed in the map.
    if w.dirty && !ui.is_mouse_down(MouseButton::Left) && !ui.is_any_item_active() {
        w.dirty = false;
        match w.recipe.validate() {
            Ok(()) => match store(e, &w) {
                Ok(()) => {
                    w.status = "Saved".into();
                    if w.live {
                        let count = crate::dungeon_editor::regenerate_with_recipe(e, w.asset);
                        if count > 0 {
                            w.status = format!(
                                "Saved · regenerated {count} dungeon{}",
                                if count == 1 { "" } else { "s" }
                            );
                        }
                    }
                }
                Err(error) => {
                    w.status = error.clone();
                    e.last_error = Some(error);
                }
            },
            // Invalid recipes stay unsaved; the step list shows what to fix.
            Err(error) => w.status = format!("Not saved: {error}"),
        }
    }
    if open {
        e.dungeon_recipe = Some(w);
    }
}

/// The project's Dungeon Rooms, and the footprints of those the recipe uses.
fn refresh_rooms(e: &Editor, w: &mut Window) {
    w.rooms = e
        .assets
        .index
        .usable()
        .filter(|r| r.meta.kind == crate::assets::Kind::DungeonRoom)
        .map(|r| {
            let name = r.path.file_stem().map_or_else(
                || r.meta.id.to_string(),
                |s| s.to_string_lossy().into_owned(),
            );
            (r.meta.id, name)
        })
        .collect();
    w.rooms.sort_by(|a, b| a.1.cmp(&b.1));
    let used = w.recipe.rooms_used();
    if used.iter().ne(w.shapes.keys()) {
        let (rooms, _) = crate::dungeon_room::load(&e.assets.index, &used);
        w.shapes = rooms.iter().map(|(id, r)| (*id, r.shape())).collect();
        w.preview = None;
    }
}

fn body(ui: &Ui, e: &mut Editor, w: &mut Window) {
    let _font = crate::inspector_theme::scope(ui, e.inspector_font);
    refresh_rooms(e, w);
    toolbar(ui, e, w);
    let available = ui.content_region_avail();
    let side = 360_f32.min(available[0] * 0.42);
    ui.child_window("##recipe-steps")
        .size([side, available[1] - 24.])
        .build(|| {
            steps(ui, w);
            ui.separator();
            details(ui, w);
        });
    ui.same_line();
    ui.child_window("##recipe-graph")
        .size([available[0] - side - 8., available[1] - 24.])
        .border(true)
        .build(|| graph_view(ui, w));
    ui.text_disabled(
        "Steps run top to bottom; each one can only use the paths earlier steps grew. The preview shows the room graph for one seed.",
    );
    if ui.is_window_focused_with_flags(imgui::WindowFocusedFlags::ROOT_AND_CHILD_WINDOWS)
        && !crate::gui::text_input_active(ui)
        && (ui.io().key_ctrl || ui.io().key_super)
    {
        if ui.is_key_pressed_no_repeat(imgui::Key::Z) {
            if ui.io().key_shift {
                redo(w);
            } else {
                undo(w);
            }
        } else if ui.is_key_pressed_no_repeat(imgui::Key::Y) {
            redo(w);
        }
    }
}

fn undo(w: &mut Window) {
    if let Some(previous) = w.undo.pop() {
        w.redo.push(std::mem::replace(&mut w.recipe, previous));
        w.dirty = true;
    }
}
fn redo(w: &mut Window) {
    if let Some(next) = w.redo.pop() {
        w.undo.push(std::mem::replace(&mut w.recipe, next));
        w.dirty = true;
    }
}

fn toolbar(ui: &Ui, e: &mut Editor, w: &mut Window) {
    if ui.button(format!("{ICON_ADD} Add step")) {
        ui.open_popup("##add-step");
    }
    #[cfg(test)]
    crate::gui::record_script_control(ui, "Add recipe step");
    if ui.is_item_hovered() {
        ui.tooltip_text("Add a step after the selected one.");
    }
    add_popup(ui, w);
    ui.same_line();
    ui.disabled(w.undo.is_empty(), || {
        if ui.button("\u{f2ea}") {
            undo(w);
        }
    });
    if ui.is_item_hovered_with_flags(imgui::ItemHoveredFlags::ALLOW_WHEN_DISABLED) {
        ui.tooltip_text("Undo (Ctrl+Z)");
    }
    ui.same_line();
    ui.disabled(w.redo.is_empty(), || {
        if ui.button("\u{f2f9}") {
            redo(w);
        }
    });
    if ui.is_item_hovered_with_flags(imgui::ItemHoveredFlags::ALLOW_WHEN_DISABLED) {
        ui.tooltip_text("Redo (Ctrl+Shift+Z)");
    }
    ui.same_line();
    ui.checkbox("Live preview", &mut w.live);
    if ui.is_item_hovered() {
        let users = e
            .scene
            .dungeons
            .iter()
            .filter(|g| g.recipe == Some(w.asset))
            .count();
        ui.tooltip_text(format!(
            "Regenerate the dungeons of this map that follow the recipe after every edit ({users} follow it now)."
        ));
    }
    ui.same_line();
    ui.text_disabled("Seed");
    ui.same_line();
    ui.set_next_item_width(90.);
    let mut seed = w.seed as i32;
    if crate::gui::Drag::new("##recipe-seed")
        .range(0, i32::MAX)
        .build(ui, &mut seed)
    {
        w.seed = seed.max(0) as u32;
    }
    if ui.is_item_hovered() {
        ui.tooltip_text("The seed the preview grows. It does not change any dungeon.");
    }
    ui.same_line();
    if ui.button(ICON_SEED) {
        w.seed = crate::dungeon_editor::next_seed(w.seed);
    }
    if ui.is_item_hovered() {
        ui.tooltip_text("Preview another seed.");
    }
    if !w.status.is_empty() {
        ui.same_line();
        crate::gui::muted(ui, &w.status);
    }
}

// ---- steps ---------------------------------------------------------------------------------

/// The names of the paths the steps before `index` grow, the main path first.
fn paths_before(recipe: &Recipe, index: usize) -> Vec<String> {
    let mut names = Vec::new();
    for step in &recipe.steps[..index.min(recipe.steps.len())] {
        let name = match step {
            Step::MainPath { .. } => MAIN.to_owned(),
            Step::Branch { name, .. } => name.clone(),
            _ => continue,
        };
        if !names.contains(&name) {
            names.push(name);
        }
    }
    names
}

fn step_icon(step: &Step) -> &'static str {
    match step {
        Step::Lattice { .. } => "\u{f84c}",
        Step::MainPath { .. } => "\u{f4d7}",
        Step::Branch { .. } => "\u{f542}",
        Step::Items { .. } => "\u{f3ed}",
        Step::KeyLock { .. } => "\u{f084}",
        Step::OneWay { .. } => "\u{f338}",
        Step::Caves { .. } => "\u{f6fc}",
        Step::Levels { .. } => "\u{f5c3}",
        Step::HandmadeRooms { .. } => "\u{f6d9}",
        Step::Rooms { .. } => "\u{f5cb}",
    }
}

/// One line describing a step's settings.
pub fn step_summary(step: &Step) -> String {
    let range = |r: [u16; 2]| {
        if r[0] == r[1] {
            r[0].to_string()
        } else {
            format!("{}-{}", r[0], r[1])
        }
    };
    let list = |names: &[String]| names.join(", ");
    match step {
        Step::Lattice { size, cell } => {
            format!("{}×{} room slots, {cell} cells each", size[0], size[1])
        }
        Step::MainPath { length } => format!("{} rooms, spawn to goal", range(*length)),
        Step::Branch {
            name,
            from,
            to,
            count,
            length,
        } => {
            let ending = if to.is_empty() {
                "dead ends".to_owned()
            } else {
                format!("loop back to {}", list(to))
            };
            format!(
                "\"{name}\": {count} × {} rooms off {}, {ending}",
                range(*length),
                list(from)
            )
        }
        Step::Items {
            name,
            paths,
            count,
            depth,
            place,
        } => format!(
            "{} × {name} in {}, {}-{}% deep{}",
            range(*count),
            list(paths),
            depth[0],
            depth[1],
            match place {
                Place::Anywhere => String::new(),
                Place::Centre => ", centred".into(),
                Place::Edge => ", against walls".into(),
            }
        ),
        Step::KeyLock {
            key_paths,
            lock_paths,
        } => format!("key in {}, lock on {}", list(key_paths), list(lock_paths)),
        Step::OneWay { chance } => format!("{chance}% of links back towards the spawn"),
        Step::Levels { chance, highest } => {
            format!("{chance}% of links climb or drop a level, up to level {highest}")
        }
        Step::HandmadeRooms { paths, rooms, most } => match rooms.len() {
            0 => format!("{} rooms: add Dungeon Rooms to use", list(paths)),
            n => format!(
                "{} made from {n} Dungeon Room{}",
                match most {
                    0 => format!("{} rooms", list(paths)),
                    1 => format!("1 {} room", list(paths)),
                    m => format!("up to {m} {} rooms", list(paths)),
                },
                if n == 1 { "" } else { "s" }
            ),
        },
        Step::Caves {
            paths,
            density,
            smoothing,
        } => format!(
            "{} rooms become caves, {density}% rock, {smoothing} passes",
            list(paths)
        ),
        Step::Rooms {
            fill,
            corridor_width,
            rounded,
        } => format!(
            "rooms fill {}-{}% of a slot{}, corridors {corridor_width} wide",
            fill[0],
            fill[1],
            if *rounded { ", corners cut" } else { "" }
        ),
    }
}

/// The validation error a step causes, if the recipe's first error is in it.
fn step_error(recipe: &Recipe, index: usize) -> Option<String> {
    let error = recipe.validate().err()?;
    let prefix = format!("Step {} (", index + 1);
    error
        .strip_prefix(&prefix)
        .and_then(|rest| rest.split_once("): "))
        .map(|(_, message)| message.to_owned())
}

fn steps(ui: &Ui, w: &mut Window) {
    crate::gui::muted(ui, "Steps");
    let count = w.recipe.steps.len();
    let mut action: Option<(usize, i32)> = None;
    for index in 0..count {
        let _id = ui.push_id_usize(index);
        let step = &w.recipe.steps[index];
        let selected = w.selected == Some(index);
        let error = step_error(&w.recipe, index);
        let start = ui.cursor_screen_pos();
        let width = ui.content_region_avail()[0];
        let label = format!(
            "{}  {} {}\n      {}",
            index + 1,
            step_icon(step),
            step.label(),
            step_summary(step)
        );
        let _color = error
            .as_ref()
            .map(|_| ui.push_style_color(imgui::StyleColor::Text, AMBER));
        if ui
            .selectable_config(&label)
            .selected(selected)
            .size([width - 70., 0.])
            .build()
        {
            w.selected = Some(index);
        }
        drop(_color);
        if ui.is_item_hovered() {
            ui.tooltip_text(match &error {
                Some(error) => format!("{}\n\n\u{f071} {error}", step.description()),
                None => step.description().to_owned(),
            });
        }
        let end = ui.cursor_screen_pos();
        ui.same_line_with_pos(width - 64.);
        ui.set_cursor_screen_pos([ui.cursor_screen_pos()[0], start[1] + 4.]);
        ui.disabled(index == 0, || {
            if ui.small_button("\u{f062}") {
                action = Some((index, -1));
            }
        });
        if ui.is_item_hovered_with_flags(imgui::ItemHoveredFlags::ALLOW_WHEN_DISABLED) {
            ui.tooltip_text("Move up");
        }
        ui.same_line();
        ui.disabled(index + 1 == count, || {
            if ui.small_button("\u{f063}") {
                action = Some((index, 1));
            }
        });
        if ui.is_item_hovered_with_flags(imgui::ItemHoveredFlags::ALLOW_WHEN_DISABLED) {
            ui.tooltip_text("Move down");
        }
        ui.set_cursor_screen_pos(end);
    }
    if count == 0 {
        crate::gui::muted(ui, "No steps. Add a Main path step to start.");
    }
    if let Some((index, delta)) = action {
        let target = (index as i32 + delta) as usize;
        w.checkpoint();
        w.recipe.steps.swap(index, target);
        w.selected = Some(target);
    }
}

fn add_popup(ui: &Ui, w: &mut Window) {
    ui.popup("##add-step", || {
        for step in Step::defaults() {
            let unique = matches!(
                step,
                Step::Lattice { .. } | Step::MainPath { .. } | Step::Rooms { .. }
            );
            let taken = unique
                && w.recipe
                    .steps
                    .iter()
                    .any(|s| std::mem::discriminant(s) == std::mem::discriminant(&step));
            ui.disabled(taken, || {
                if ui.menu_item(format!("{} {}", step_icon(&step), step.label())) {
                    let at = w
                        .selected
                        .map_or(w.recipe.steps.len(), |s| (s + 1).min(w.recipe.steps.len()));
                    let mut step = step.clone();
                    if let Step::Branch { name, .. } = &mut step {
                        *name = fresh_branch_name(&w.recipe);
                    }
                    w.checkpoint();
                    w.recipe.steps.insert(at, step);
                    w.selected = Some(at);
                }
            });
            if ui.is_item_hovered_with_flags(imgui::ItemHoveredFlags::ALLOW_WHEN_DISABLED) {
                ui.tooltip_text(if taken {
                    "A recipe has one step of this kind."
                } else {
                    step.description()
                });
            }
        }
    });
}

fn fresh_branch_name(recipe: &Recipe) -> String {
    let taken = paths_before(recipe, recipe.steps.len());
    (1..)
        .map(|n| {
            if n == 1 {
                "branch".to_owned()
            } else {
                format!("branch {n}")
            }
        })
        .find(|name| !taken.contains(name))
        .expect("an unused name exists")
}

// ---- details -------------------------------------------------------------------------------

fn range_u16(ui: &Ui, label: &str, value: &mut [u16; 2], limits: (u16, u16), tip: &str) {
    let mut v = value.map(i32::from);
    if crate::gui::Drag::new(crate::gui::field(ui, label))
        .range(i32::from(limits.0), i32::from(limits.1))
        .build_array(ui, &mut v)
    {
        let lo = v[0].clamp(i32::from(limits.0), i32::from(limits.1)) as u16;
        let hi = (v[1].clamp(i32::from(limits.0), i32::from(limits.1)) as u16).max(lo);
        *value = [lo, hi];
    }
    if ui.is_item_hovered() {
        ui.tooltip_text(tip);
    }
}

fn percent_range(ui: &Ui, label: &str, value: &mut [u8; 2], low: u8, tip: &str) {
    let mut v = value.map(i32::from);
    if crate::gui::Drag::new(crate::gui::field(ui, label))
        .range(i32::from(low), 100)
        .display_format("%d %%")
        .build_array(ui, &mut v)
    {
        let lo = v[0].clamp(i32::from(low), 100) as u8;
        let hi = (v[1].clamp(i32::from(low), 100) as u8).max(lo);
        *value = [lo, hi];
    }
    if ui.is_item_hovered() {
        ui.tooltip_text(tip);
    }
}

fn number(ui: &Ui, label: &str, value: &mut u16, limits: (u16, u16), tip: &str) {
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

/// Checkboxes picking paths among `known`.
fn path_picker(ui: &Ui, label: &str, chosen: &mut Vec<String>, known: &[String], tip: &str) {
    let _ = crate::gui::field(ui, label);
    ui.group(|| {
        if known.is_empty() {
            ui.text_disabled("No earlier path");
        }
        for (i, name) in known.iter().enumerate() {
            if i > 0 {
                ui.same_line();
            }
            let mut on = chosen.contains(name);
            let shown = if name == MAIN { "main path" } else { name };
            if ui.checkbox(format!("{shown}##{label}-{name}"), &mut on) {
                if on {
                    chosen.push(name.clone());
                } else {
                    chosen.retain(|n| n != name);
                }
            }
        }
    });
    if ui.is_item_hovered() {
        ui.tooltip_text(tip);
    }
}

fn details(ui: &Ui, w: &mut Window) {
    let Some(index) = w.selected.filter(|i| *i < w.recipe.steps.len()) else {
        crate::gui::muted(ui, "Select a step to edit it.");
        return;
    };
    let known = paths_before(&w.recipe, index);
    let mut step = w.recipe.steps[index].clone();
    let before = step.clone();
    crate::gui::heading(
        ui,
        &format!("{} {}. {}", step_icon(&step), index + 1, step.label()),
    );
    ui.text_wrapped(step.description());
    if let Some(error) = step_error(&w.recipe, index) {
        ui.text_colored(AMBER, format!("\u{f071} {error}"));
    }
    ui.spacing();
    let mut delete = false;
    match &mut step {
        Step::Lattice { size, cell } => {
            let mut v = size.map(i32::from);
            let (lo, hi) = dungeon_recipe::LATTICE_LIMITS;
            if crate::gui::Drag::new(crate::gui::field(ui, "Room slots"))
                .range(i32::from(lo), i32::from(hi))
                .build_array(ui, &mut v)
            {
                *size = v.map(|x| x.clamp(i32::from(lo), i32::from(hi)) as u16);
            }
            if ui.is_item_hovered() {
                ui.tooltip_text("Columns and rows of the lattice. Each slot holds at most one room; paths walk between neighbouring slots.");
            }
            number(
                ui,
                "Slot size",
                cell,
                (dungeon_recipe::min_cell(1), 16),
                "Grid cells per slot side. Rooms grow inside their slot; the rest is wall and corridor.",
            );
            let cells = [size[0] * *cell + 2, size[1] * *cell + 2];
            crate::gui::muted(
                ui,
                format!("The dungeon grid is {}×{} cells.", cells[0], cells[1]),
            );
        }
        Step::MainPath { length } => {
            range_u16(
                ui,
                "Length",
                length,
                (2, 64),
                "Rooms on the path, the spawn and goal rooms included.",
            );
        }
        Step::Branch {
            name,
            from,
            to,
            count,
            length,
        } => {
            let mut text = name.clone();
            if ui
                .input_text(crate::gui::field(ui, "Name"), &mut text)
                .build()
            {
                *name = text.chars().take(32).collect();
            }
            if ui.is_item_hovered() {
                ui.tooltip_text("Later steps refer to this branch's rooms by this name.");
            }
            path_picker(
                ui,
                "Starts off",
                from,
                &known,
                "Each branch starts next to a room of these paths.",
            );
            path_picker(
                ui,
                "Loops back to",
                to,
                &known,
                "Leave empty for dead ends; otherwise each branch ends next to a room of these paths and connects to it.",
            );
            number(ui, "Count", count, (1, 16), "How many branches to grow.");
            range_u16(
                ui,
                "Length",
                length,
                (1, 32),
                "Rooms per branch, smallest and largest.",
            );
        }
        Step::Items {
            name,
            paths,
            count,
            depth,
            place,
        } => {
            let mut text = name.clone();
            if ui
                .input_text(crate::gui::field(ui, "Name"), &mut text)
                .build()
            {
                *name = text.chars().take(32).collect();
            }
            if ui.is_item_hovered() {
                ui.tooltip_text("Style pieces on Item spots can pick items by this name.");
            }
            path_picker(
                ui,
                "In paths",
                paths,
                &known,
                "Rooms these items may go in.",
            );
            range_u16(ui, "Count", count, (1, 32), "How many items to place.");
            percent_range(
                ui,
                "Depth",
                depth,
                0,
                "Where along each path: 0% is its first room, 100% its last. 60-100% keeps rewards at the far end.",
            );
            if let Some(_combo) =
                ui.begin_combo(crate::gui::field(ui, "In the room"), place.label())
            {
                for option in Place::ALL {
                    if ui
                        .selectable_config(option.label())
                        .selected(*place == option)
                        .build()
                    {
                        *place = option;
                    }
                }
            }
            if ui.is_item_hovered() {
                ui.tooltip_text("Where in its room each item goes: any free cell away from the doors, the centre, or against a wall.");
            }
        }
        Step::KeyLock {
            key_paths,
            lock_paths,
        } => {
            path_picker(
                ui,
                "Key in",
                key_paths,
                &known,
                "The key lies in a room of these paths.",
            );
            path_picker(
                ui,
                "Lock on",
                lock_paths,
                &known,
                "The locked door leads into a room of these paths.",
            );
            crate::gui::muted(
                ui,
                "Placements are checked by walking from the spawn and collecting keys, so the key is always reachable without the door it opens.",
            );
        }
        Step::OneWay { chance } => {
            let mut v = i32::from(*chance);
            if ui
                .slider_config(crate::gui::field(ui, "Chance"), 0, 100)
                .display_format("%d %%")
                .build(&mut v)
            {
                *chance = v.clamp(0, 100) as u8;
            }
            if ui.is_item_hovered() {
                ui.tooltip_text("Chance for each link that leads back towards the spawn to open from the far side only, as a shortcut home.");
            }
        }
        Step::HandmadeRooms { paths, rooms, most } => {
            path_picker(
                ui,
                "Paths",
                paths,
                &known,
                "Rooms of these paths are made from the Dungeon Rooms below.",
            );
            let mut v = i32::from(*most);
            if ui
                .slider_config(crate::gui::field(ui, "At most"), 0, 16)
                .display_format(match v {
                    0 => "every room",
                    1 => "%d room",
                    _ => "%d rooms",
                })
                .build(&mut v)
            {
                *most = v.clamp(0, 16) as u8;
            }
            if ui.is_item_hovered() {
                ui.tooltip_text("How many rooms of these paths are made by hand, in the order the recipe grew them; the rest are generated. A handmade room's quads and copied actors count towards the console's budget, so a few special rooms go a long way. 0 makes every room of the paths by hand.");
            }
            crate::gui::muted(ui, "Dungeon Rooms");
            let mut remove = None;
            for (i, id) in rooms.iter().enumerate() {
                let name = w
                    .rooms
                    .iter()
                    .find(|(r, _)| r == id)
                    .map_or("Missing Dungeon Room", |(_, n)| n.as_str());
                ui.bullet_text(name);
                if let Some(shape) = w.shapes.get(id) {
                    ui.same_line();
                    crate::gui::muted(
                        ui,
                        format!(
                            "{}x{} cells, {} door{}",
                            shape.size[0],
                            shape.size[1],
                            shape.doors.len(),
                            if shape.doors.len() == 1 { "" } else { "s" }
                        ),
                    );
                }
                ui.same_line();
                if ui.small_button(format!("Remove##room-{i}")) {
                    remove = Some(i);
                }
            }
            if let Some(i) = remove {
                rooms.remove(i);
            }
            let addable: Vec<&(Uuid, String)> = w
                .rooms
                .iter()
                .filter(|(id, _)| !rooms.contains(id))
                .collect();
            if addable.is_empty() && w.rooms.is_empty() {
                ui.text_wrapped(
                    "No Dungeon Rooms yet. Build a room under one actor, select that actor, then choose GameObject > Save Selection as Dungeon Room.",
                );
            } else if let Some(_combo) =
                ui.begin_combo(crate::gui::field(ui, "Add room"), "Choose a Dungeon Room")
            {
                for (id, name) in addable {
                    if ui.selectable(name) {
                        rooms.push(*id);
                    }
                }
            }
            if ui.is_item_hovered() {
                ui.tooltip_text("Each room of the paths above takes one of these that fits its slot and has doors towards its links, turned as needed; the least used comes first.");
            }
        }
        Step::Levels { chance, highest } => {
            let mut v = i32::from(*chance);
            if ui
                .slider_config(crate::gui::field(ui, "Climb chance"), 0, 100)
                .display_format("%d %%")
                .build(&mut v)
            {
                *chance = v.clamp(0, 100) as u8;
            }
            if ui.is_item_hovered() {
                ui.tooltip_text("Chance for each link to climb or drop one floor level. Its corridor gets a flight of stairs; links whose corridor has no straight stretch for one stay level.");
            }
            let mut v = i32::from(*highest);
            if ui
                .slider_config(crate::gui::field(ui, "Highest level"), 1, 4)
                .build(&mut v)
            {
                *highest = v.clamp(1, 4) as u8;
            }
            if ui.is_item_hovered() {
                ui.tooltip_text(format!(
                    "Rooms stay between the spawn's level and this one. Each level is {:.1} units: four steps of {:.1}.",
                    crate::dungeon::LEVEL_HEIGHT,
                    crate::dungeon::STEP_RISE
                ));
            }
        }
        Step::Caves {
            paths,
            density,
            smoothing,
        } => {
            path_picker(
                ui,
                "Paths",
                paths,
                &known,
                "Rooms of these paths become caves.",
            );
            let mut v = i32::from(*density);
            if ui
                .slider_config(crate::gui::field(ui, "Rock"), 20, 70)
                .display_format("%d %%")
                .build(&mut v)
            {
                *density = v.clamp(20, 70) as u8;
            }
            if ui.is_item_hovered() {
                ui.tooltip_text("How much of each cave starts as rock. More rock gives narrower, twistier caves.");
            }
            let mut passes = u16::from(*smoothing);
            number(
                ui,
                "Smoothing",
                &mut passes,
                (1, 8),
                "Passes that round the rock into chambers. More passes give smoother walls.",
            );
            *smoothing = passes as u8;
            crate::gui::muted(
                ui,
                "Caves always keep their centre, their doors and a way between them.",
            );
        }
        Step::Rooms {
            fill,
            corridor_width,
            rounded,
        } => {
            let _ = crate::gui::toggle(ui, "Cut corners", rounded);
            if ui.is_item_hovered() {
                ui.tooltip_text("Cut the corners of rooms that are not caves: one cell, or three in large rooms.");
            }
            percent_range(
                ui,
                "Fill",
                fill,
                20,
                "How much of its slot each room covers, smallest and largest.",
            );
            number(
                ui,
                "Corridor width",
                corridor_width,
                (1, 3),
                "Corridor width in cells. Width 3 needs slots of at least 9 cells.",
            );
        }
    }
    ui.spacing();
    if ui.button("Remove step") {
        delete = true;
    }
    if delete {
        w.checkpoint();
        w.recipe.steps.remove(index);
        w.selected = if w.recipe.steps.is_empty() {
            None
        } else {
            Some(index.min(w.recipe.steps.len() - 1))
        };
        return;
    }
    if step != before {
        if !w.dirty || w.undo.last().is_none_or(|last| *last != w.recipe) {
            w.checkpoint();
        }
        // A renamed branch keeps the steps that refer to it.
        if let (Step::Branch { name: old, .. }, Step::Branch { name: new, .. }) = (&before, &step)
            && old != new
        {
            for later in &mut w.recipe.steps[index + 1..] {
                rename(later, old, new);
            }
        }
        w.recipe.steps[index] = step;
        w.dirty = true;
    }
}

fn rename(step: &mut Step, old: &str, new: &str) {
    let lists: Vec<&mut Vec<String>> = match step {
        Step::Branch { from, to, .. } => vec![from, to],
        Step::Items { paths, .. } => vec![paths],
        Step::KeyLock {
            key_paths,
            lock_paths,
        } => vec![key_paths, lock_paths],
        _ => Vec::new(),
    };
    for list in lists {
        for name in list.iter_mut() {
            if name == old {
                *name = new.to_owned();
            }
        }
    }
}

// ---- graph preview -------------------------------------------------------------------------

fn graph_view(ui: &Ui, w: &mut Window) {
    let draw = ui.get_window_draw_list();
    let origin = ui.cursor_screen_pos();
    let area = ui.content_region_avail();
    draw.add_rect(origin, [origin[0] + area[0], origin[1] + area[1]], CANVAS)
        .filled(true)
        .build();
    let graph = match w.graph() {
        Ok(graph) => graph.clone(),
        Err(error) => {
            let wrap = ui.push_text_wrap_pos_with_pos(area[0] - 16.);
            ui.set_cursor_screen_pos([origin[0] + 16., origin[1] + 16.]);
            ui.text_colored(AMBER, format!("\u{f071} {error}"));
            wrap.end();
            return;
        }
    };
    let legend = 46.;
    let size = graph.size.map(|v| v as f32);
    let slot = ((area[0] - 40.) / size[0])
        .min((area[1] - 40. - legend) / size[1])
        .max(12.);
    let offset = [
        origin[0] + (area[0] - slot * size[0]) * 0.5,
        origin[1] + (area[1] - legend - slot * size[1]) * 0.5,
    ];
    let centre = |node: usize| {
        let p = graph.point(node);
        [
            offset[0] + (p[0] as f32 + 0.5) * slot,
            offset[1] + (p[1] as f32 + 0.5) * slot,
        ]
    };
    for node in 0..graph.path.len() {
        let c = centre(node);
        let r = slot * 0.42;
        draw.add_rect([c[0] - r, c[1] - r], [c[0] + r, c[1] + r], SLOT)
            .rounding(3.)
            .build();
    }
    let room = slot * 0.27;
    let levels = w.levels();
    let made = w.made();
    // Cave rooms are drawn round.
    let caves = w.recipe.caves();
    for link in &graph.links {
        let (a, b) = (centre(link.a), centre(link.b));
        let color = if link.lock.is_some() {
            LOCK
        } else if link.one_way {
            ONE_WAY
        } else {
            LINK
        };
        draw.add_line(a, b, color).thickness(3.).build();
        let mid = [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5];
        if let Some((key, guarded)) = link.lock {
            // A padlock on the door side of the guarded room.
            let g = centre(guarded);
            let at = [g[0] + (mid[0] - g[0]) * 0.55, g[1] + (mid[1] - g[1]) * 0.55];
            let s = slot * 0.11;
            draw.add_rect([at[0] - s, at[1] - s], [at[0] + s, at[1] + s], LOCK)
                .filled(true)
                .rounding(2.)
                .build();
            draw.add_text([at[0] + s + 2., at[1] - s - 2.], LOCK, key.to_string());
        }
        if link.one_way {
            // An arrow head pointing from a to b.
            let d = [b[0] - a[0], b[1] - a[1]];
            let len = (d[0] * d[0] + d[1] * d[1]).sqrt().max(1.);
            let (u, v) = ([d[0] / len, d[1] / len], [-d[1] / len, d[0] / len]);
            let s = slot * 0.13;
            let tip = [mid[0] + u[0] * s, mid[1] + u[1] * s];
            draw.add_triangle(
                tip,
                [mid[0] - u[0] * s + v[0] * s, mid[1] - u[1] * s + v[1] * s],
                [mid[0] - u[0] * s - v[0] * s, mid[1] - u[1] * s - v[1] * s],
                ONE_WAY,
            )
            .filled(true)
            .build();
        }
    }
    for node in (0..graph.path.len()).filter(|n| graph.active(*n)) {
        let c = centre(node);
        let path = graph.path[node].unwrap_or(0);
        let color = PATHS[path % PATHS.len()];
        let rounding = if caves.contains_key(&graph.paths[path].0) {
            room
        } else {
            4.
        };
        draw.add_rect(
            [c[0] - room, c[1] - room],
            [c[0] + room, c[1] + room],
            color,
        )
        .filled(true)
        .rounding(rounding)
        .build();
        let label = if node == graph.spawn {
            Some(("S", SPAWN))
        } else if node == graph.goal {
            Some(("G", GOAL))
        } else {
            None
        };
        if let Some((text, color)) = label {
            draw.add_rect(
                [c[0] - room, c[1] - room],
                [c[0] + room, c[1] + room],
                color,
            )
            .rounding(rounding)
            .thickness(3.)
            .build();
            let size = ui.calc_text_size(text);
            draw.add_text(
                [c[0] - size[0] * 0.5, c[1] - size[1] * 0.5],
                [0.08, 0.08, 0.1, 1.],
                text,
            );
        }
        // Handmade rooms carry the room icon in their lower corner.
        if made.get(node).copied().flatten().is_some() {
            let icon = crate::dungeon_room_editor::ICON;
            let size = ui.calc_text_size(icon);
            let at = [c[0] - room + 3., c[1] + room - size[1] - 2.];
            draw.add_rect(
                [at[0] - 2., at[1]],
                [at[0] + size[0] + 2., at[1] + size[1]],
                [0.08, 0.08, 0.1, 0.85],
            )
            .filled(true)
            .rounding(3.)
            .build();
            draw.add_text(at, [0.95, 0.96, 0.98, 1.], icon);
        }
        // Rooms above the spawn's floor carry their level in the corner.
        let level = levels.get(node).copied().unwrap_or(0);
        if level > 0 {
            let text = format!("\u{f062}{level}");
            let size = ui.calc_text_size(&text);
            let at = [c[0] + room - size[0] - 3., c[1] - room + 1.];
            draw.add_rect(
                [at[0] - 2., at[1]],
                [at[0] + size[0] + 2., at[1] + size[1]],
                [0.08, 0.08, 0.1, 0.85],
            )
            .filled(true)
            .rounding(3.)
            .build();
            draw.add_text(at, [0.95, 0.96, 0.98, 1.], &text);
        }
        // Items and keys as dots along the room's bottom edge.
        let items: Vec<_> = graph.items.iter().filter(|i| i.node == node).collect();
        for (k, item) in items.iter().enumerate() {
            let at = [c[0] - room + 6. + k as f32 * 9., c[1] + room - 6.];
            let color = if item.lock > 0 { KEY } else { ITEM };
            draw.add_circle(at, 3.6, color).filled(true).build();
            draw.add_circle(at, 3.6, [0.08, 0.08, 0.1, 1.]).build();
        }
        // Hover: the room's path, depth and contents.
        let mouse = ui.io().mouse_pos;
        if ui.is_window_hovered()
            && (mouse[0] - c[0]).abs() <= room
            && (mouse[1] - c[1]).abs() <= room
        {
            let name = &graph.paths[path].0;
            let mut text = format!(
                "{} · room {} of its path",
                if name == MAIN { "main path" } else { name },
                graph.depth[node] + 1
            );
            if node == graph.spawn {
                text.push_str("\nSpawn");
            }
            if node == graph.goal {
                text.push_str("\nGoal");
            }
            if let Some(room) = made.get(node).copied().flatten() {
                let name = w
                    .rooms
                    .iter()
                    .find(|(id, _)| *id == room)
                    .map_or("a Dungeon Room", |(_, n)| n.as_str());
                text.push_str(&format!("\nHandmade: {name}"));
            }
            if let Some(level) = levels.get(node).filter(|l| **l > 0) {
                text.push_str(&format!(
                    "\nLevel {level}: {:.1} units above the spawn",
                    f32::from(*level) * crate::dungeon::LEVEL_HEIGHT
                ));
            }
            for item in &items {
                if item.lock > 0 {
                    text.push_str(&format!("\nKey {}", item.lock));
                } else {
                    text.push_str(&format!("\n{}", item.name));
                }
            }
            ui.tooltip_text(text);
        }
    }
    // Legend: paths by colour, then the markers.
    let mut x = origin[0] + 12.;
    let y = origin[1] + area[1] - legend + 8.;
    for (i, (name, nodes)) in graph.paths.iter().enumerate() {
        let color = PATHS[i % PATHS.len()];
        draw.add_rect([x, y], [x + 12., y + 12.], color)
            .filled(true)
            .rounding(2.)
            .build();
        let label = format!(
            "{} ({})",
            if name == MAIN { "main path" } else { name },
            nodes.len()
        );
        draw.add_text([x + 16., y - 1.], [0.82, 0.84, 0.88, 1.], &label);
        x += 28. + ui.calc_text_size(&label)[0];
    }
    let y = y + 18.;
    let mut x = origin[0] + 12.;
    for (color, label) in [
        (KEY, "key"),
        (LOCK, "locked door"),
        (ONE_WAY, "one-way door"),
        (ITEM, "item"),
    ] {
        draw.add_circle([x + 6., y + 6.], 4.5, color)
            .filled(true)
            .build();
        draw.add_text([x + 16., y - 1.], [0.7, 0.72, 0.78, 1.], label);
        x += 30. + ui.calc_text_size(label)[0];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steps_only_see_paths_grown_before_them() {
        let recipe = dungeon_recipe::builtin();
        assert!(paths_before(&recipe, 1).is_empty());
        assert_eq!(paths_before(&recipe, 2), vec![MAIN.to_owned()]);
        assert_eq!(
            paths_before(&recipe, recipe.steps.len()),
            vec![MAIN.to_owned(), "branch".to_owned(), "loop".to_owned()]
        );
        assert_eq!(fresh_branch_name(&recipe), "branch 2");
    }

    #[test]
    fn renaming_a_branch_follows_into_later_steps() {
        let mut recipe = dungeon_recipe::builtin();
        for step in &mut recipe.steps[3..] {
            rename(step, "branch", "side room");
        }
        assert!(matches!(&recipe.steps[4], Step::Items { paths, .. } if paths == &["side room"]));
        assert!(
            matches!(&recipe.steps[5], Step::KeyLock { key_paths, .. } if key_paths == &["side room"])
        );
    }

    #[test]
    fn errors_point_at_their_step() {
        let mut recipe = dungeon_recipe::builtin();
        if let Step::Items { paths, .. } = &mut recipe.steps[4] {
            *paths = vec!["nowhere".into()];
        }
        assert!(step_error(&recipe, 4).is_some_and(|e| e.contains("nowhere")));
        assert!(step_error(&recipe, 2).is_none());
        for step in &recipe.steps {
            assert!(!step_summary(step).is_empty());
        }
    }
}
