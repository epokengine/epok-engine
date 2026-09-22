//! Terrain sculpt and paint tool.
//!
//! Undo stores whole-grid snapshots. That is affordable precisely because the
//! authored payload is a compact grid: a 32x32 terrain is about six kilobytes,
//! so thirty-two snapshots cost less than one expanded mesh document would.
use crate::{
    assets,
    brush::{self, Brush, Falloff, Mode, Stroke},
    editor::Editor,
    terrain::{self, Document},
};
use imgui::{Condition, Ui};
use std::sync::Arc;
use uuid::Uuid;

const UNDO_DEPTH: usize = 32;

/// Brush modes as a strip of icons. Glyphs come from the Font Awesome face the
/// editor already ships; each one is rasterized into the UI atlas by
/// `platform::EDITOR_FA_GLYPHS`, so a codepoint added here must be added there.
const MODE_ICONS: [(Mode, &str, &str); 7] = [
    (
        Mode::Raise,
        "\u{f0aa}",
        "Raise (1): add height under the brush",
    ),
    (
        Mode::Lower,
        "\u{f0ab}",
        "Lower (2): remove height under the brush",
    ),
    (
        Mode::Smooth,
        "\u{f773}",
        "Smooth (3): average each corner with its neighbours",
    ),
    (
        Mode::Flatten,
        "\u{f547}",
        "Flatten (4): level to the height under the first click",
    ),
    (
        Mode::Set,
        "\u{f140}",
        "Set (5): drive towards the target height",
    ),
    (
        Mode::Noise,
        "\u{f522}",
        "Noise (6): add deterministic value noise",
    ),
    (
        Mode::Paint,
        "\u{f1fc}",
        "Paint (7): assign the selected atlas tile",
    ),
];
const FALLOFF_ICONS: [(Falloff, &str, &str); 4] = [
    (
        Falloff::Smooth,
        "\u{25d0}",
        "Smooth falloff: flat centre, soft rim",
    ),
    (Falloff::Linear, "\u{f1fe}", "Linear falloff"),
    (
        Falloff::Sharp,
        "\u{f6fc}",
        "Sharp falloff: concentrates the stroke near the centre",
    ),
    (
        Falloff::Constant,
        "\u{25a0}",
        "Constant: no rim, the whole disc moves together",
    ),
];

/// A labelled row of toggle icons, one highlighted.
fn icon_strip<T: Copy + PartialEq>(
    ui: &Ui,
    label: &str,
    items: &[(T, &str, &str)],
    current: &mut T,
) {
    ui.text(label);
    ui.same_line();
    for (k, (value, glyph, tip)) in items.iter().enumerate() {
        if k > 0 {
            ui.same_line();
        }
        if crate::gui::icon(
            ui,
            glyph,
            &format!("terrain-{label}-{k}"),
            tip,
            *current == *value,
        ) {
            *current = *value;
        }
    }
}
/// Stroke samples are spaced by a fraction of the brush radius so a fast drag
/// stays continuous without applying hundreds of redundant samples.
const STROKE_SPACING: f32 = 0.35;

fn button(ui: &Ui, label: &str) -> bool {
    let pressed = ui.button(label);
    #[cfg(test)]
    BUTTONS.with(|buttons| {
        let a = ui.item_rect_min();
        let b = ui.item_rect_max();
        buttons
            .borrow_mut()
            .insert(label.to_owned(), [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5]);
    });
    pressed
}
#[cfg(test)]
thread_local! {static BUTTONS: std::cell::RefCell<std::collections::BTreeMap<String,[f32;2]>> = const {std::cell::RefCell::new(std::collections::BTreeMap::new())};}

pub struct State {
    pub open: bool,
    pub record: Option<assets::Record>,
    pub doc: Option<Document>,
    pub target: Option<usize>,
    pub brush: Brush,
    /// Bumped on every grid change so the GPU preview knows to rebuild.
    pub revision: u64,
    pub error: Option<String>,
    /// World-space centre of the brush ring drawn in the viewport.
    pub cursor: Option<[f32; 3]>,
    /// The same point in the terrain's local space, which is what the
    /// decal samples heights in.
    pub cursor_local: Option<[f32; 3]>,
    pub path: String,
    stroke: Stroke,
    /// Grid snapshot taken when the current stroke began.
    pending: Option<Document>,
    undo: Vec<Document>,
    redo: Vec<Document>,
    create_cells: [i32; 2],
    create_size: f32,
}
impl Default for State {
    fn default() -> Self {
        Self {
            open: false,
            record: None,
            doc: None,
            target: None,
            brush: Brush::default(),
            revision: 0,
            error: None,
            cursor: None,
            cursor_local: None,
            path: String::new(),
            stroke: Stroke::default(),
            pending: None,
            undo: vec![],
            redo: vec![],
            create_cells: [32, 32],
            create_size: 4.,
        }
    }
}
impl State {
    /// Whether the viewport should route mouse drags to the brush instead of
    /// to selection and the transform gizmo.
    pub fn sculpting(&self) -> bool {
        self.open && self.doc.is_some() && self.target.is_some()
    }
}

pub fn open(e: &mut Editor, record: assets::Record, target: Option<usize>) {
    e.terrain_editor.open = false;
    preview(e);
    let target = target.or_else(|| {
        e.scene.actors.iter().position(|v| {
            v.terrain
                .as_ref()
                .is_some_and(|t| t.asset == record.meta.id)
        })
    });
    match terrain::document(&record) {
        Ok(doc) => {
            e.terrain_editor = State {
                open: true,
                path: assets::path_string(&e.root, &record.path),
                record: Some(record),
                doc: Some(doc),
                target,
                ..Default::default()
            };
            preview(e);
        }
        Err(error) => e.log(error),
    }
}

/// Push the in-editor grid into every scene actor that references it, so the
/// viewport shows the stroke before it is saved.
pub fn preview(e: &mut Editor) {
    if let (Some(record), Some(doc)) = (&e.terrain_editor.record, &e.terrain_editor.doc) {
        let id = record.meta.id;
        let shown = Arc::new(doc.clone());
        for actor in &mut e.scene.actors {
            if let Some(t) = &mut actor.terrain
                && t.asset == id
            {
                t.document = Some(shown.clone());
                t.error = None;
            }
        }
    }
    e.terrain_editor.revision = e.terrain_editor.revision.wrapping_add(1);
    e.view_dirty = true;
}

pub fn synchronize(e: &mut Editor) {
    let Some(old) = e.terrain_editor.record.clone() else {
        return;
    };
    match e.assets.index.resolve(old.meta.id).cloned() {
        Ok(record) => {
            if record.revision != old.revision {
                // A background scan can finish after our own save. Never let
                // its stale result overwrite the grid being sculpted.
                if std::fs::read(&record.path)
                    .is_ok_and(|bytes| assets::hash(&bytes) == record.revision)
                {
                    match terrain::document(&record) {
                        Ok(doc) => {
                            e.terrain_editor.doc = Some(doc);
                            e.terrain_editor.undo.clear();
                            e.terrain_editor.redo.clear();
                            e.terrain_editor.pending = None;
                            e.terrain_editor.record = Some(record);
                            e.terrain_editor.error = Some(
                                "External terrain changes loaded. Sculpt history was reset.".into(),
                            );
                        }
                        Err(error) => e.terrain_editor.error = Some(error),
                    }
                }
            } else {
                e.terrain_editor.record.as_mut().unwrap().path = record.path;
            }
        }
        Err(error) => e.terrain_editor.error = Some(error),
    }
    preview(e);
}

fn publish(e: &mut Editor, doc: Document) -> Result<(), String> {
    let record = e
        .terrain_editor
        .record
        .as_ref()
        .ok_or("No terrain open")?
        .clone();
    let revision = terrain::save(&record, &doc)?;
    let mut record = record;
    record.meta.source_hash = assets::hash(&doc.encode());
    record.revision = revision;
    e.terrain_editor.record = Some(record);
    e.terrain_editor.doc = Some(doc);
    e.assets.refresh();
    preview(e);
    e.changed();
    Ok(())
}

/// Apply a whole edit at once: used by the panel controls, never by a stroke.
pub fn edit(
    e: &mut Editor,
    change: impl FnOnce(&mut Document) -> Result<(), String>,
) -> Result<(), String> {
    if e.playing {
        return Err("Stop Play before editing terrain".into());
    }
    let before = e.terrain_editor.doc.clone().ok_or("No terrain open")?;
    let mut after = before.clone();
    change(&mut after)?;
    after.validate()?;
    after.cook_limits()?;
    if before == after {
        return Ok(());
    }
    publish(e, after)?;
    push_undo(e, before);
    Ok(())
}

fn push_undo(e: &mut Editor, before: Document) {
    e.terrain_editor.undo.push(before);
    if e.terrain_editor.undo.len() > UNDO_DEPTH {
        e.terrain_editor.undo.remove(0);
    }
    e.terrain_editor.redo.clear();
}

pub fn undo(e: &mut Editor, redo: bool) -> Result<(), String> {
    if e.playing {
        return Err("Stop Play before editing terrain".into());
    }
    let entry = if redo {
        e.terrain_editor.redo.last()
    } else {
        e.terrain_editor.undo.last()
    }
    .cloned()
    .ok_or("No terrain operation to undo/redo")?;
    let current = e.terrain_editor.doc.clone().ok_or("No terrain open")?;
    publish(e, entry)?;
    if redo {
        e.terrain_editor.redo.pop();
        e.terrain_editor.undo.push(current);
    } else {
        e.terrain_editor.undo.pop();
        e.terrain_editor.redo.push(current);
    }
    Ok(())
}

/// World ray under a viewport pixel, as (origin, direction).
fn ray(e: &Editor, pixel: [f32; 2]) -> ([f32; 3], [f32; 3]) {
    let start = e.view.unproject(pixel, 1.);
    let next = e.view.unproject(pixel, 2.);
    (start, std::array::from_fn(|i| next[i] - start[i]))
}

/// Terrain hit under a viewport pixel, in local grid space and in world space.
pub fn hover(e: &Editor, pixel: [f32; 2]) -> Option<([f32; 3], [f32; 3])> {
    let index = e.terrain_editor.target?;
    let doc = e.terrain_editor.doc.as_ref()?;
    let actor = e.scene.actors.get(index)?;
    actor
        .terrain
        .as_ref()
        .filter(|t| Some(t.asset) == e.terrain_editor.record.as_ref().map(|r| r.meta.id))?;
    let world = e.scene.world_matrix(index);
    let inverse = world.inverse().ok()?;
    let (origin, direction) = ray(e, pixel);
    let local = doc.raycast(inverse.point(origin), inverse.vector(direction))?;
    Some((local, world.point(local)))
}

/// Viewport press/drag/release for the brush. Returns whether the brush
/// consumed the interaction, in which case selection and the gizmo stay out of
/// the way.
pub fn drag(ui: &Ui, e: &mut Editor, pixel: [f32; 2], eligible: bool) -> bool {
    if !e.terrain_editor.sculpting() {
        return false;
    }
    let down = ui.is_mouse_down(imgui::MouseButton::Left);
    let pressed = ui.is_mouse_clicked(imgui::MouseButton::Left);
    let released = ui.is_mouse_released(imgui::MouseButton::Left);
    let hit = if eligible || e.terrain_editor.stroke.active() {
        hover(e, pixel)
    } else {
        None
    };
    let cursor = hit.map(|(_, world)| world);
    // The decal is rebuilt from this on every render, so a moving cursor
    // must request a frame even when no stroke changes the grid.
    if cursor != e.terrain_editor.cursor {
        e.view_dirty = true;
    }
    e.terrain_editor.cursor = cursor;
    e.terrain_editor.cursor_local = hit.map(|(local, _)| local);
    if e.playing {
        return false;
    }
    if pressed && eligible {
        let Some((local, _)) = hit else {
            return false;
        };
        let Some(doc) = e.terrain_editor.doc.clone() else {
            return false;
        };
        // Flatten levels to the surface actually clicked on, which is what
        // makes it usable without typing a height first.
        if e.terrain_editor.brush.mode.samples_reference() {
            e.terrain_editor.brush.reference = doc.sample(local[0], local[2]);
        }
        e.terrain_editor.pending = Some(doc);
        let samples = e.terrain_editor.stroke.begin(local);
        apply(e, &samples);
        return true;
    }
    if e.terrain_editor.stroke.active() {
        if down {
            if let Some((local, _)) = hit {
                let spacing = (e.terrain_editor.brush.radius * STROKE_SPACING).max(0.05);
                let samples = e.terrain_editor.stroke.extend(local, spacing);
                apply(e, &samples);
            }
            return true;
        }
        if released || !down {
            finish(e);
            return true;
        }
    }
    false
}

fn apply(e: &mut Editor, samples: &[[f32; 3]]) {
    let brush = e.terrain_editor.brush;
    let Some(doc) = &mut e.terrain_editor.doc else {
        return;
    };
    let mut changed = false;
    for sample in samples {
        changed |= doc.sculpt(&brush, *sample);
    }
    if changed {
        preview(e);
    }
}

/// End the stroke: validate once, save once, and record one undo entry for the
/// whole stroke rather than one per sample.
pub fn finish(e: &mut Editor) {
    if !e.terrain_editor.stroke.end() {
        e.terrain_editor.pending = None;
        return;
    }
    let Some(before) = e.terrain_editor.pending.take() else {
        return;
    };
    let Some(after) = e.terrain_editor.doc.clone() else {
        return;
    };
    if before == after {
        return;
    }
    if let Err(error) = after.validate().and_then(|()| after.cook_limits()) {
        e.terrain_editor.doc = Some(before);
        e.terrain_editor.error = Some(error);
        preview(e);
        return;
    }
    match publish(e, after) {
        Ok(()) => {
            push_undo(e, before);
            e.terrain_editor.error = None;
        }
        Err(error) => {
            e.terrain_editor.doc = Some(before);
            e.terrain_editor.error = Some(error);
            preview(e);
        }
    }
}

pub fn shortcuts(ui: &Ui, e: &mut Editor) {
    // Both tools can be open at once. The blockout tool claims Ctrl+Z first,
    // so a single press must never undo in two histories.
    if !e.terrain_editor.open || e.mesh_editor.open || ui.io().want_text_input {
        return;
    }
    if ui.io().key_ctrl && ui.is_key_pressed(imgui::Key::Z) {
        let result = undo(e, false);
        report(e, result);
    }
    if ui.io().key_ctrl && ui.is_key_pressed(imgui::Key::Y) {
        let result = undo(e, true);
        report(e, result);
    }
}

/// Brush keys, live only while the tool owns the viewport.
pub fn brush_shortcuts(ui: &Ui, e: &mut Editor) {
    if !e.terrain_editor.sculpting() || ui.io().want_text_input || ui.io().key_ctrl {
        return;
    }
    if ui.is_key_pressed(imgui::Key::LeftBracket) {
        e.terrain_editor.brush.radius =
            (e.terrain_editor.brush.radius / 1.25).max(brush::MIN_RADIUS);
    }
    if ui.is_key_pressed(imgui::Key::RightBracket) {
        e.terrain_editor.brush.radius =
            (e.terrain_editor.brush.radius * 1.25).min(brush::MAX_RADIUS);
    }
    for (key, mode) in [
        (imgui::Key::Alpha1, Mode::Raise),
        (imgui::Key::Alpha2, Mode::Lower),
        (imgui::Key::Alpha3, Mode::Smooth),
        (imgui::Key::Alpha4, Mode::Flatten),
        (imgui::Key::Alpha5, Mode::Set),
        (imgui::Key::Alpha6, Mode::Noise),
        (imgui::Key::Alpha7, Mode::Paint),
    ] {
        if ui.is_key_pressed(key) {
            e.terrain_editor.brush.mode = mode;
        }
    }
}

fn report(e: &mut Editor, result: Result<(), String>) {
    match result {
        Ok(()) => e.terrain_editor.error = None,
        Err(error) => e.terrain_editor.error = Some(error),
    }
}

fn fresh_path() -> String {
    format!("assets/Terrain/Terrain-{}.epokasset", Uuid::new_v4())
}

/// Create a new terrain asset and an actor that renders it.
pub fn create(e: &mut Editor) -> Result<(), String> {
    if e.playing {
        return Err("Stop Play before creating terrain".into());
    }
    if e.scene.actors.len() >= 512 {
        return Err("Scene entity limit: 512".into());
    }
    let cells = [
        e.terrain_editor.create_cells[0].clamp(1, i32::from(terrain::MAX_CELLS)) as u16,
        e.terrain_editor.create_cells[1].clamp(1, i32::from(terrain::MAX_CELLS)) as u16,
    ];
    let doc = Document::new(cells, e.terrain_editor.create_size);
    doc.validate()?;
    let path = fresh_path();
    let id = terrain::create(&e.root, &path, &doc)?;
    // A new terrain draws the engine's ground atlas out of the box; without it
    // the surface is flat grey and there is nothing to paint a path with.
    let atlas = terrain::ensure_builtin_atlas(&e.root)?;
    e.assets.refresh();
    let index = assets::scan(&e.root, &mut Default::default());
    let record = index.resolve(id).cloned()?;
    let mut actor = crate::scene::Actor::cube("Terrain".into());
    actor.position = [0.; 3];
    actor.material = Default::default();
    let mut component = terrain::Component::new(id);
    component.document = Some(Arc::new(doc));
    component.atlas = terrain::BUILTIN_ATLAS_GRID;
    component.autotile = true;
    component.material.texture = Some(atlas);
    actor.terrain = Some(component);
    actor.lighting.static_geometry = true;
    // Terrain is the canonical baked surface: one bake gives it per-corner
    // light and ambient occlusion that cost nothing per frame.
    actor.lighting.receive = crate::lighting::Receive::Baked;
    e.scene.actors.push(actor);
    let target = e.scene.actors.len() - 1;
    // A baked surface in a scene with no light renders at ambient only, which
    // reads as black ground. Give the first terrain in an unlit scene a sun,
    // the same one the arena template uses, so the default is a lit terrain
    // rather than a silhouette. An existing light is left alone.
    if !e.scene.actors.iter().any(|a| a.light.is_some()) {
        let mut sun = crate::scene::Actor::cube("Sun".into());
        sun.kind = "Empty".into();
        sun.rotation = [58., -32., 0.];
        sun.light = Some(crate::lighting::Light {
            mode: crate::lighting::LightMode::Mixed,
            intensity: 0.65,
            ..Default::default()
        });
        e.scene.actors.push(sun);
    }
    e.selected = Some(target);
    e.changed();
    open(e, record, Some(target));
    Ok(())
}

/// Add an actor for the terrain already open, when the scene has none.
pub fn add_instance(e: &mut Editor) -> Result<(), String> {
    if e.scene.actors.len() >= 512 {
        return Err("Scene entity limit: 512".into());
    }
    let record = e
        .terrain_editor
        .record
        .as_ref()
        .ok_or("No terrain open")?
        .clone();
    let doc = terrain::document(&record)?;
    let mut actor = crate::scene::Actor::cube("Terrain".into());
    actor.position = [0.; 3];
    actor.material = Default::default();
    let mut component = terrain::Component::new(record.meta.id);
    component.document = Some(Arc::new(doc));
    actor.terrain = Some(component);
    actor.lighting.static_geometry = true;
    actor.lighting.receive = crate::lighting::Receive::Baked;
    e.scene.actors.push(actor);
    e.terrain_editor.target = Some(e.scene.actors.len() - 1);
    e.selected = e.terrain_editor.target;
    preview(e);
    e.changed();
    Ok(())
}

pub fn window(ui: &Ui, e: &mut Editor) {
    if !e.terrain_editor.open {
        return;
    }
    let mut open = true;
    ui.window("Terrain")
        .opened(&mut open)
        .position([20., 80.], Condition::FirstUseEver)
        .size(
            [460., (ui.io().display_size[1] - 110.).min(700.)],
            Condition::FirstUseEver,
        )
        .size_constraints([360., 400.], [1200., 1200.])
        .build(|| {
            if let Some(error) = &e.terrain_editor.error {
                let _color = ui.push_style_color(imgui::StyleColor::Text, [1., 0.5, 0.3, 1.]);
                ui.text_wrapped(error);
            }
            let Some(doc) = e.terrain_editor.doc.clone() else {
                ui.text_wrapped("No terrain open.");
                return;
            };
            let Some(record) = e.terrain_editor.record.clone() else {
                return;
            };
            ui.text_wrapped(assets::path_string(&e.root, &record.path));
            let users = e
                .scene
                .actors
                .iter()
                .filter(|v| v.terrain.as_ref().is_some_and(|t| t.asset == record.meta.id))
                .count();
            ui.text_wrapped(format!(
                "Shared asset: {users} instance(s). Strokes save when the mouse is released; Ctrl+Z / Ctrl+Y undo and redo."
            ));
            ui.disabled(e.playing, || {
                if crate::gui::icon(ui, "\u{f2ea}", "terrain-undo", "Undo stroke (Ctrl+Z)", false) {
                    let result = undo(e, false);
                    report(e, result);
                }
                ui.same_line();
                if crate::gui::icon(ui, "\u{f2f9}", "terrain-redo", "Redo stroke (Ctrl+Y)", false) {
                    let result = undo(e, true);
                    report(e, result);
                }
                if users == 0 {
                    crate::gui::inline(ui, "Add instance to scene");
                    if button(ui, "Add instance to scene") {
                        let result = add_instance(e);
                        report(e, result);
                    }
                }
                if let Some(_tabs) = ui.tab_bar("Terrain tabs") {
                    if let Some(_tab) = ui.tab_item("Sculpt") {
                        sculpt_panel(ui, e);
                    }
                    if let Some(_tab) = ui.tab_item("Surface") {
                        surface_panel(ui, e);
                    }
                    if let Some(_tab) = ui.tab_item("Grid") {
                        grid_panel(ui, e, &doc);
                    }
                    if let Some(_tab) = ui.tab_item("Budget") {
                        budget_panel(ui, e, &doc);
                    }
                }
            });
        });
    if !open {
        e.terrain_editor.open = false;
        e.terrain_editor.cursor = None;
        e.terrain_editor.cursor_local = None;
        preview(e);
    }
}

fn sculpt_panel(ui: &Ui, e: &mut Editor) {
    icon_strip(ui, "Brush", &MODE_ICONS, &mut e.terrain_editor.brush.mode);
    icon_strip(
        ui,
        "Falloff",
        &FALLOFF_ICONS,
        &mut e.terrain_editor.brush.falloff,
    );
    crate::gui::muted(
        ui,
        format!(
            "{} · {} falloff",
            e.terrain_editor.brush.mode.label(),
            e.terrain_editor.brush.falloff.label()
        ),
    );
    crate::gui::Drag::new(crate::gui::field(ui, "Radius"))
        .speed(0.1)
        .range(brush::MIN_RADIUS, brush::MAX_RADIUS)
        .build(ui, &mut e.terrain_editor.brush.radius);
    crate::gui::Drag::new(crate::gui::field(ui, "Strength"))
        .speed(0.01)
        .range(0., brush::MAX_STRENGTH)
        .build(ui, &mut e.terrain_editor.brush.strength);
    if e.terrain_editor.brush.mode == Mode::Set {
        crate::gui::Drag::new(crate::gui::field(ui, "Target height"))
            .speed(0.05)
            .range(-128., 127.99)
            .build(ui, &mut e.terrain_editor.brush.target);
    }
    if e.terrain_editor.brush.mode == Mode::Noise {
        let mut seed = e.terrain_editor.brush.seed as i32;
        if crate::gui::Drag::new(crate::gui::field(ui, "Noise seed"))
            .speed(1.)
            .range(0, i32::MAX)
            .build(ui, &mut seed)
        {
            e.terrain_editor.brush.seed = seed.max(0) as u32;
        }
    }
    ui.text_wrapped(
        "Drag in Scene to sculpt. [ and ] resize the brush; 1-7 pick a brush. Flatten levels to the height under the first click.",
    );
}

fn surface_panel(ui: &Ui, e: &mut Editor) {
    let Some(index) = e.terrain_editor.target else {
        ui.text_wrapped("Add an instance to the scene to edit its surface.");
        return;
    };
    let Some(component) = e.scene.actors.get(index).and_then(|a| a.terrain.clone()) else {
        return;
    };
    let mut next = component.clone();
    let mut atlas = [i32::from(next.atlas[0]), i32::from(next.atlas[1])];
    if crate::gui::Drag::new(crate::gui::field(ui, "Atlas columns / rows"))
        .speed(0.1)
        .range(1, 8)
        .build_array(ui, &mut atlas)
    {
        next.atlas = [atlas[0].clamp(1, 8) as u8, atlas[1].clamp(1, 8) as u8];
    }
    tile_picker(ui, e, &next);
    let mut rotation = usize::from(e.terrain_editor.brush.tile_rotation.min(4));
    if ui.combo_simple_string(
        crate::gui::field(ui, "Tile rotation"),
        &mut rotation,
        &["0", "90", "180", "270", "Scatter"],
    ) {
        e.terrain_editor.brush.tile_rotation = rotation.min(4) as u8;
    }
    let mut autotile = next.autotile;
    if ui.checkbox(crate::gui::field(ui, "Autotile borders"), &mut autotile) {
        next.autotile = autotile;
    }
    let mut merge = i32::from(next.merge);
    if crate::gui::Drag::new(crate::gui::field(ui, "Merge flat cells"))
        .speed(0.05)
        .range(0, 3)
        .build(ui, &mut merge)
    {
        next.merge = merge.clamp(0, 3) as u8;
    }
    ui.text_wrapped(
        "Merging collapses coplanar cells that share a tile into one quad. It is the only decimation the cooked format allows, and it stretches the tile across the merged cells.",
    );
    let mut collision = next.collision;
    if ui.checkbox(
        crate::gui::field(ui, "Heightfield collision"),
        &mut collision,
    ) {
        next.collision = collision;
    }
    ui.text_wrapped("Collision samples the grid directly. The actor must not be rotated, and must be scaled uniformly in X and Z.");
    crate::gui::muted(
        ui,
        "One material covers the whole terrain; painting picks a tile inside its atlas, so the surface stays a single texture page.",
    );
    ui.color_edit3(crate::gui::field(ui, "Color"), &mut next.material.color);
    ui.checkbox("Unlit", &mut next.material.unlit);
    crate::texture::picker(ui, &e.assets.index, &mut next.material);
    if next != component
        && let Some(actor) = e.scene.actors.get_mut(index)
    {
        actor.terrain = Some(next);
        e.changed();
        e.view_dirty = true;
    }
}

/// The atlas as a clickable grid, laid out exactly as the texture is. Picking
/// a tile by its position is what makes an atlas legible; a tile number is
/// not. Tiles of the engine's own atlas are named in their tooltip.
fn tile_picker(ui: &Ui, e: &mut Editor, component: &terrain::Component) {
    let builtin = component.material.texture == Some(terrain::BUILTIN_ATLAS_ID)
        && component.atlas == terrain::BUILTIN_ATLAS_GRID;
    if component.autotile {
        // One button per material row: the baker picks the tile within it.
        ui.text("Paint material");
        for material in 0..component.atlas[1].max(1) {
            if material > 0 {
                ui.same_line();
            }
            let tip = if builtin {
                format!(
                    "{} (borders resolve themselves)",
                    terrain::builtin_material_label(material)
                )
            } else {
                format!("Material row {material}")
            };
            if crate::gui::icon(
                ui,
                &material.to_string(),
                &format!("terrain-material-{material}"),
                &tip,
                e.terrain_editor.brush.tile == material,
            ) {
                e.terrain_editor.brush.tile = material;
            }
        }
        crate::gui::muted(
            ui,
            if builtin {
                "0 grass, 1 dirt, 2 stone, 3 water. Edges and corners are chosen from the neighbours; brush 7 paints."
            } else {
                "One material per atlas row. Edges and corners are chosen from the neighbours; brush 7 paints."
            },
        );
        return;
    }
    let cols = component.atlas[0].max(1);
    let rows = component.atlas[1].max(1);
    ui.text("Paint tile");
    for row in 0..rows {
        for col in 0..cols {
            if col > 0 {
                ui.same_line();
            }
            let tile = row * cols + col;
            if crate::gui::icon(
                ui,
                &tile.to_string(),
                &format!("terrain-tile-{tile}"),
                &format!("Tile {tile}"),
                e.terrain_editor.brush.tile == tile,
            ) {
                e.terrain_editor.brush.tile = tile;
            }
        }
    }
    crate::gui::muted(
        ui,
        "Tiles are row-major in the texture. Brush 7 paints the selected tile.",
    );
}

fn grid_panel(ui: &Ui, e: &mut Editor, doc: &Document) {
    let cells = doc.cells();
    let span = doc.span();
    ui.text_wrapped(format!(
        "{} x {} cells of {:.2} units = {:.1} x {:.1} units. Heights {:.2} to {:.2}.",
        cells[0],
        cells[1],
        doc.cell_size(),
        span[0],
        span[1],
        doc.lowest(),
        doc.highest()
    ));
    let mut resize = [i32::from(cells[0]), i32::from(cells[1])];
    crate::gui::Drag::new(crate::gui::field(ui, "Cells X / Z"))
        .speed(0.25)
        .range(1, i32::from(terrain::MAX_CELLS))
        .build_array(ui, &mut resize);
    if button(ui, "Resize grid") {
        let target = [
            resize[0].clamp(1, i32::from(terrain::MAX_CELLS)) as u16,
            resize[1].clamp(1, i32::from(terrain::MAX_CELLS)) as u16,
        ];
        let result = edit(e, |doc| {
            doc.resize(target);
            Ok(())
        });
        report(e, result);
    }
    let mut size = doc.cell_size();
    crate::gui::Drag::new(crate::gui::field(ui, "Cell size"))
        .speed(0.05)
        .range(terrain::MIN_CELL_SIZE, terrain::MAX_CELL_SIZE)
        .build(ui, &mut size);
    if button(ui, "Apply cell size") {
        let result = edit(e, |doc| {
            doc.set_cell_size(size);
            Ok(())
        });
        report(e, result);
    }
    if button(ui, "Flatten to zero") {
        let result = edit(e, |doc| {
            let cells = doc.cells();
            for j in 0..=cells[1] {
                for i in 0..=cells[0] {
                    doc.set_height(i, j, 0.);
                }
            }
            Ok(())
        });
        report(e, result);
    }
    ui.text_wrapped(format!(
        "{} cells per draw chunk on each axis gives {} chunks before merging. A terrain spans at most {:.0} units per axis: beyond that a vertex leaves the local window the chunk compiler encodes.",
        doc.block(),
        doc.chunk_count(),
        terrain::MAX_SPAN
    ));
    ui.input_text(
        crate::gui::field(ui, "Asset path"),
        &mut e.terrain_editor.path,
    )
    .build();
    if button(ui, "Move / Rename asset") {
        let record = e.terrain_editor.record.clone();
        if let Some(record) = record {
            match assets::move_asset(&e.root, &record, &e.terrain_editor.path.clone()) {
                Ok(()) => {
                    if let Ok(path) = assets::inside(&e.root, &e.terrain_editor.path)
                        && let Some(open) = e.terrain_editor.record.as_mut()
                    {
                        open.path = path;
                    }
                    e.assets.refresh();
                }
                Err(error) => e.terrain_editor.error = Some(error),
            }
        }
    }
}

fn budget_panel(ui: &Ui, e: &mut Editor, doc: &Document) {
    let component = e
        .terrain_editor
        .target
        .and_then(|i| e.scene.actors.get(i))
        .and_then(|a| a.terrain.clone());
    let mut probe = crate::scene::Actor::cube("Budget".into());
    let mut binding = component.unwrap_or_else(|| {
        terrain::Component::new(
            e.terrain_editor
                .record
                .as_ref()
                .map(|r| r.meta.id)
                .unwrap_or_default(),
        )
    });
    binding.document = Some(Arc::new(doc.clone()));
    probe.terrain = Some(binding.clone());
    let quads = crate::lighting::quads(&probe);
    let authored = doc.quad_count();
    ui.text_wrapped(format!(
        "{authored} cells -> {} compiled quads ({} triangles) after merging.",
        quads.len(),
        quads.len() * 2
    ));
    match crate::terrain_compile::chunks(&binding, &quads) {
        Ok(chunks) => ui.text_wrapped(format!(
            "{} draw chunks of at most {} cells per axis / {} indexed positions.",
            chunks.len(),
            doc.block(),
            chunks.iter().map(|c| c.vertices.len()).sum::<usize>()
        )),
        Err(error) => ui.text_wrapped(format!("Chunking failed: {error}")),
    }
    let scene_quads: usize = e.scene.actors.iter().map(crate::lighting::quad_count).sum();
    let ratio = scene_quads as f32 / terrain::MAX_QUADS as f32;
    let color = if ratio > 1. {
        [1., 0.4, 0.3, 1.]
    } else if ratio > 0.8 {
        [1., 0.8, 0.3, 1.]
    } else {
        [0.6, 0.9, 0.6, 1.]
    };
    let style = ui.push_style_color(imgui::StyleColor::Text, color);
    ui.text_wrapped(format!(
        "Scene total {scene_quads} / {} resident quads ({:.0}% of the 7000-triangle budget).",
        terrain::MAX_QUADS,
        ratio * 100.
    ));
    drop(style);
    ui.text_wrapped(
        "Chunk count, not triangle count, is what the per-chunk bounds test pays for each frame. Measure in Play; these are build-time figures.",
    );
    if let Some(field) = crate::terrain_compile::heightfield(&probe) {
        ui.text_wrapped(format!(
            "Heightfield collider: {} corner heights = {} bytes of RAM.",
            field.heights.len(),
            field.heights.len() * 2
        ));
    }
}

#[cfg(test)]
pub fn verify_interactions(context: &mut imgui::Context) {
    let root = std::env::temp_dir().join(format!("epok-terrain-ui-{}", Uuid::new_v4()));
    std::fs::create_dir_all(root.join("assets")).unwrap();
    let mut e = Editor::new(root.clone());
    e.auto_build = false;
    e.terrain_editor.create_cells = [8, 8];
    e.terrain_editor.create_size = 2.;
    create(&mut e).expect("create terrain");
    assert!(e.terrain_editor.open);
    // The tool targets the actor carrying the terrain, wherever it sits: an
    // unlit scene also gains a sun, so the terrain is not the last actor.
    let target = e.terrain_editor.target.expect("create targets its terrain");
    let component = e.scene.actors[target]
        .terrain
        .as_ref()
        .expect("the targeted actor carries the terrain");
    // Out of the box the terrain draws the engine's ground atlas.
    assert_eq!(component.material.texture, Some(terrain::BUILTIN_ATLAS_ID));
    assert_eq!(component.atlas, terrain::BUILTIN_ATLAS_GRID);
    assert!(
        e.scene.actors.iter().any(|a| a.light.is_some()),
        "a terrain in an unlit scene must get a light, or it renders black"
    );
    // A second terrain must not add a second sun.
    let lights = e.scene.actors.iter().filter(|a| a.light.is_some()).count();
    create(&mut e).expect("create a second terrain");
    assert_eq!(
        e.scene.actors.iter().filter(|a| a.light.is_some()).count(),
        lights
    );

    fn frame(context: &mut imgui::Context, e: &mut Editor, panel: usize) {
        let ui = context.frame();
        ui.window("Terrain interaction")
            .position([10., 10.], Condition::Always)
            .size([700., 800.], Condition::Always)
            .build(|| {
                let doc = e.terrain_editor.doc.clone().unwrap();
                match panel {
                    0 => sculpt_panel(ui, e),
                    1 => surface_panel(ui, e),
                    2 => grid_panel(ui, e, &doc),
                    _ => budget_panel(ui, e, &doc),
                }
            });
        context.render();
    }
    fn click(context: &mut imgui::Context, e: &mut Editor, panel: usize, label: &str) {
        frame(context, e, panel);
        frame(context, e, panel);
        let p = BUTTONS.with(|buttons| buttons.borrow()[label]);
        context.io_mut().add_mouse_pos_event(p);
        frame(context, e, panel);
        context
            .io_mut()
            .add_mouse_button_event(imgui::MouseButton::Left, true);
        frame(context, e, panel);
        context
            .io_mut()
            .add_mouse_button_event(imgui::MouseButton::Left, false);
        frame(context, e, panel);
        assert!(
            e.terrain_editor.error.is_none(),
            "{}: {:?}",
            label,
            e.terrain_editor.error
        );
    }

    // Every panel renders without touching the grid.
    for panel in 0..4 {
        frame(context, &mut e, panel);
    }

    // A stroke applied through the same path the viewport uses: press,
    // several moves, release. One undo entry for the whole stroke.
    e.terrain_editor.brush = Brush {
        mode: Mode::Raise,
        radius: 3.,
        strength: 1.,
        ..Default::default()
    };
    let before = e.terrain_editor.doc.clone().unwrap();
    e.terrain_editor.pending = Some(before.clone());
    let samples = e.terrain_editor.stroke.begin([0., 0., 0.]);
    apply(&mut e, &samples);
    let more = e.terrain_editor.stroke.extend([2., 0., 0.], 1.);
    apply(&mut e, &more);
    finish(&mut e);
    let after = e.terrain_editor.doc.clone().unwrap();
    assert_ne!(before, after, "the stroke did not change the grid");
    assert_eq!(e.terrain_editor.undo.len(), 1, "a stroke is one undo entry");
    assert!(e.terrain_editor.error.is_none());

    undo(&mut e, false).unwrap();
    assert_eq!(e.terrain_editor.doc.as_ref().unwrap(), &before);
    undo(&mut e, true).unwrap();
    assert_eq!(e.terrain_editor.doc.as_ref().unwrap(), &after);

    // The saved package reflects the stroke, not the grid it started from.
    let record = e.terrain_editor.record.clone().unwrap();
    assert_eq!(terrain::document(&record).unwrap(), after);

    // A real button click through imgui, not a direct call.
    click(context, &mut e, 2, "Flatten to zero");
    assert_eq!(e.terrain_editor.doc.as_ref().unwrap().highest(), 0.);
    // The panel button and the keyboard path share `undo`, which the window
    // chrome wires to its own Undo button.
    undo(&mut e, false).unwrap();
    assert_eq!(e.terrain_editor.doc.as_ref().unwrap(), &after);

    click(context, &mut e, 2, "Resize grid");
    assert!(e.terrain_editor.doc.as_ref().unwrap().validate().is_ok());

    // Undo is bounded, so a long session cannot grow without limit.
    for _ in 0..(UNDO_DEPTH + 8) {
        let doc = e.terrain_editor.doc.clone().unwrap();
        push_undo(&mut e, doc);
    }
    assert_eq!(e.terrain_editor.undo.len(), UNDO_DEPTH);

    drop(e);
    std::fs::remove_dir_all(root).unwrap();
}

/// Inspector section for a terrain actor, beside the Mesh Renderer one the
/// blockout tool contributes. Without it a terrain actor shows the legacy cube
/// renderer, which has nothing to do with the surface it actually draws.
pub fn component(ui: &Ui, e: &mut Editor, actor: &mut crate::scene::Actor) {
    if actor.terrain.is_none() {
        return;
    }
    ui.separator();
    let mut remove = false;
    let expanded = crate::gui::section(ui, "Terrain", || {
        remove = ui.menu_item("Remove Terrain");
    });
    if remove {
        actor.terrain = None;
        actor.kind = "Empty".into();
        return;
    }
    if !expanded {
        return;
    }
    let Some(t) = actor.terrain.as_mut() else {
        return;
    };
    if let Some(error) = &t.error {
        ui.text_wrapped(error);
    }
    let label = e
        .assets
        .index
        .resolve(t.asset)
        .map(|r| r.meta.source.clone())
        .unwrap_or_else(|error| error);
    let _ = crate::gui::field(ui, "Grid");
    ui.text_wrapped(label);
    if let Some(doc) = &t.document {
        let cells = doc.cells();
        let span = doc.span();
        crate::gui::muted(
            ui,
            format!(
                "{} x {} cells of {:.2} units · {:.1} x {:.1} units · {} quads",
                cells[0],
                cells[1],
                doc.cell_size(),
                span[0],
                span[1],
                doc.quad_count()
            ),
        );
    }
    let mut atlas = [i32::from(t.atlas[0]), i32::from(t.atlas[1])];
    if crate::gui::Drag::new(crate::gui::field(ui, "Atlas columns / rows"))
        .speed(0.1)
        .range(1, 8)
        .build_array(ui, &mut atlas)
    {
        t.atlas = [atlas[0].clamp(1, 8) as u8, atlas[1].clamp(1, 8) as u8];
    }
    let mut merge = i32::from(t.merge);
    if crate::gui::Drag::new(crate::gui::field(ui, "Merge flat cells"))
        .speed(0.05)
        .range(0, 3)
        .build(ui, &mut merge)
    {
        t.merge = merge.clamp(0, 3) as u8;
    }
    ui.checkbox(
        crate::gui::field(ui, "Heightfield collision"),
        &mut t.collision,
    );
    let _ = crate::gui::field(ui, "Material");
    ui.text("PSX Material (instance)");
    ui.align_text_to_frame_padding();
    let _ = crate::gui::field(ui, "Color");
    ui.set_next_item_width(-1.);
    ui.color_edit3("##terrain-color", &mut t.material.color);
    ui.checkbox(crate::gui::field(ui, "Unlit"), &mut t.material.unlit);
    crate::texture::picker(ui, &e.assets.index, &mut t.material);
    crate::lighting_editor::mesh(ui, actor);
    let asset = actor.terrain.as_ref().map(|t| t.asset);
    if button(ui, "Edit terrain...")
        && let Some(asset) = asset
    {
        match e.assets.index.resolve(asset).cloned() {
            Ok(record) => open(e, record, e.selected),
            Err(error) => e.log(error),
        }
    }
}
