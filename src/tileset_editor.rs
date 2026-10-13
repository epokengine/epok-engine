//! The TileSet editor (World2D initiative, phase 2): one window over a TileSet asset's
//! atlas, with modes to create and remove tiles, paint collision and one-way flags, set
//! up terrains by painting peering bits, define tile animations and edit custom data.
//! Every edit saves the asset at once, refreshes the open map's tile layers and can be
//! undone inside the window (Ctrl+Z / Ctrl+Shift+Z).
use crate::{
    editor::Editor,
    tileset::{self, Collision, CustomKind, CustomLayer, Terrain, TerrainSet, TileSet},
};
use imgui::{MouseButton, Ui};
use std::{collections::BTreeSet, sync::Arc};
use uuid::Uuid;

const ACCENT: [f32; 4] = [1.0, 0.62, 0.20, 1.];
const SOLID: [f32; 4] = [0.36, 0.66, 1.0, 1.];
const ONE_WAY: [f32; 4] = [0.98, 0.84, 0.30, 1.];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Tiles,
    Collision,
    Terrains,
    Animation,
    CustomData,
}
impl Mode {
    const ALL: [(Mode, &'static str, &'static str); 5] = [
        (
            Mode::Tiles,
            "Tiles",
            "Create, select and remove tiles; edit the selected tile",
        ),
        (
            Mode::Collision,
            "Collision",
            "Paint solid, one-way or partial collision onto tiles",
        ),
        (
            Mode::Terrains,
            "Terrains",
            "Paint which neighbours each tile connects to",
        ),
        (
            Mode::Animation,
            "Animation",
            "Animate a tile with the frames to its right",
        ),
        (
            Mode::CustomData,
            "Custom data",
            "Paint values gameplay reads with get_cell_custom_data",
        ),
    ];
}

/// What the Collision mode paints.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Shape {
    None,
    Full,
    Top,
    Bottom,
    Left,
    Right,
}
impl Shape {
    const ALL: [(Shape, &'static str); 6] = [
        (Shape::Full, "Full"),
        (Shape::Top, "Top half"),
        (Shape::Bottom, "Bottom half"),
        (Shape::Left, "Left half"),
        (Shape::Right, "Right half"),
        (Shape::None, "None"),
    ];
    fn apply(self, tile: &mut tileset::Tile, size: [u16; 2]) {
        let [w, h] = size.map(|v| v.min(255) as u8);
        let (collision, rect) = match self {
            Shape::None => (Collision::None, [0, 0, w, h]),
            Shape::Full => (Collision::Full, [0, 0, w, h]),
            Shape::Top => (Collision::Rect, [0, 0, w, (h / 2).max(1)]),
            Shape::Bottom => (Collision::Rect, [0, h / 2, w, h - h / 2]),
            Shape::Left => (Collision::Rect, [0, 0, (w / 2).max(1), h]),
            Shape::Right => (Collision::Rect, [w / 2, 0, w - w / 2, h]),
        };
        tile.collision = collision;
        tile.rect = rect;
    }
}

pub struct Window {
    pub asset: Uuid,
    pub doc: TileSet,
    pub mode: Mode,
    zoom: f32,
    selected: BTreeSet<[u8; 2]>,
    undo: Vec<TileSet>,
    redo: Vec<TileSet>,
    dirty: bool,
    status: String,
    shape: Shape,
    one_way: bool,
    terrain_set: usize,
    terrain: u8,
    block: bool,
    custom_layer: usize,
    custom_value: i32,
    /// Tiles already changed by the current drag.
    painted: BTreeSet<[u8; 2]>,
}
impl Window {
    fn new(asset: Uuid, doc: TileSet) -> Self {
        Self {
            asset,
            doc,
            mode: Mode::Tiles,
            zoom: 0.,
            selected: BTreeSet::new(),
            undo: Vec::new(),
            redo: Vec::new(),
            dirty: false,
            status: String::new(),
            shape: Shape::Full,
            one_way: false,
            terrain_set: 0,
            terrain: 0,
            block: false,
            custom_layer: 0,
            custom_value: 1,
            painted: BTreeSet::new(),
        }
    }
    /// Records the state before an edit.
    fn checkpoint(&mut self) {
        self.undo.push(self.doc.clone());
        if self.undo.len() > 64 {
            self.undo.remove(0);
        }
        self.redo.clear();
        self.dirty = true;
    }
}

/// Opens the editor on TileSet `id`.
pub fn open(e: &mut Editor, id: Uuid) {
    if e.world2d
        .tileset_window
        .as_ref()
        .is_some_and(|w| w.asset == id)
    {
        return;
    }
    let doc = e
        .scene
        .tilesets
        .get(&id)
        .map(|s| (**s).clone())
        .or_else(|| {
            e.assets
                .index
                .resolve(id)
                .ok()
                .and_then(|r| tileset::document(r).ok())
        });
    match doc {
        Some(doc) => e.world2d.tileset_window = Some(Window::new(id, doc)),
        None => {
            let error = format!("TileSet {id} could not be opened");
            e.last_error = Some(error.clone());
            e.log(error);
        }
    }
}
/// Creates an empty TileSet asset (from the Content Browser) and opens it.
pub fn create_empty(e: &mut Editor) -> Result<Uuid, String> {
    let path = tileset::fresh_path("TileSet");
    let id = tileset::create(&e.root, &path, &TileSet::default())?;
    e.assets.index = crate::assets::scan(&e.root, &mut Default::default());
    e.assets.refresh();
    open(e, id);
    e.log(format!(
        "Created {path}. Choose its texture in the TileSet editor."
    ));
    Ok(id)
}
/// Saves `doc` over asset `id` and refreshes the open map.
fn store(e: &mut Editor, id: Uuid, doc: &TileSet) -> Result<(), String> {
    let record = match e.assets.index.resolve(id) {
        Ok(record) => record.clone(),
        Err(_) => {
            e.assets.index = crate::assets::scan(&e.root, &mut Default::default());
            e.assets.index.resolve(id)?.clone()
        }
    };
    tileset::save(&record, doc)?;
    let texture_changed = e
        .scene
        .tilesets
        .get(&id)
        .is_none_or(|old| old.texture != doc.texture);
    e.scene.tilesets.insert(id, Arc::new(doc.clone()));
    if texture_changed {
        let _ = crate::texture::resolve(&mut e.scene, &e.assets.index);
    }
    e.assets.refresh();
    e.view_dirty = true;
    Ok(())
}
/// Applies `edit` to TileSet `id` (through the open window when it shows that set) and
/// saves it.
pub fn modify(e: &mut Editor, id: Uuid, edit: impl FnOnce(&mut TileSet)) {
    if let Some(mut window) = e.world2d.tileset_window.take() {
        if window.asset == id {
            window.checkpoint();
            edit(&mut window.doc);
            window.doc.normalize();
            window.dirty = false;
            let doc = window.doc.clone();
            e.world2d.tileset_window = Some(window);
            let result = store(e, id, &doc);
            report(e, result);
            return;
        }
        e.world2d.tileset_window = Some(window);
    }
    let Some(mut doc) = crate::tilemap_editor::tileset(e, id).map(|s| (*s).clone()) else {
        return;
    };
    edit(&mut doc);
    doc.normalize();
    let result = store(e, id, &doc);
    report(e, result);
}
fn report(e: &mut Editor, result: Result<(), String>) {
    if let Err(error) = result {
        e.last_error = Some(error.clone());
        e.log(error);
    }
}

/// Draws the window when it is open.
pub fn window(ui: &Ui, e: &mut Editor) {
    let Some(mut window) = e.world2d.tileset_window.take() else {
        return;
    };
    let name = e
        .assets
        .index
        .resolve(window.asset)
        .ok()
        .map(|r| {
            std::path::Path::new(&r.meta.source)
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default()
        })
        .unwrap_or_else(|| "TileSet".into());
    let mut open = true;
    ui.window(format!(
        "{} TileSet Editor: {name}###tileset-editor",
        crate::tilemap_editor::ICON_TILESET
    ))
    .opened(&mut open)
    .size([980., 640.], imgui::Condition::FirstUseEver)
    .build(|| body(ui, e, &mut window));
    // An edit finished (no drag in progress): save it.
    if window.dirty
        && !ui.is_mouse_down(MouseButton::Left)
        && !ui.is_mouse_down(MouseButton::Right)
        && !ui.is_any_item_active()
    {
        window.doc.normalize();
        window.dirty = false;
        window.painted.clear();
        match store(e, window.asset, &window.doc) {
            Ok(()) => window.status = "Saved".into(),
            Err(error) => {
                window.status = error.clone();
                e.last_error = Some(error);
            }
        }
    }
    if open {
        e.world2d.tileset_window = Some(window);
    }
}

fn body(ui: &Ui, e: &mut Editor, w: &mut Window) {
    header(ui, e, w);
    ui.separator();
    for (mode, label, tip) in Mode::ALL {
        let active = w.mode == mode;
        let _colour = ui.push_style_color(
            imgui::StyleColor::Button,
            if active {
                [0.19, 0.37, 0.52, 1.]
            } else {
                [0.19, 0.19, 0.21, 1.]
            },
        );
        if ui.button(label) {
            w.mode = mode;
        }
        if ui.is_item_hovered() {
            ui.tooltip_text(tip);
        }
        ui.same_line();
    }
    ui.new_line();
    let available = ui.content_region_avail();
    let side = 300_f32.min(available[0] * 0.4);
    ui.child_window("##tileset-atlas")
        .size([available[0] - side - 8., available[1] - 22.])
        .border(true)
        .horizontal_scrollbar(true)
        .build(|| atlas(ui, e, w));
    ui.same_line();
    ui.child_window("##tileset-side")
        .size([side, available[1] - 22.])
        .build(|| side_panel(ui, e, w));
    ui.text_disabled(hint(w));
    // Undo and redo while the window has focus.
    if ui.is_window_focused_with_flags(imgui::WindowFocusedFlags::ROOT_AND_CHILD_WINDOWS)
        && !crate::gui::text_input_active(ui)
        && (ui.io().key_ctrl || ui.io().key_super)
        && ui.is_key_pressed_no_repeat(imgui::Key::Z)
    {
        if ui.io().key_shift {
            redo(w);
        } else {
            undo(w);
        }
    }
}
fn undo(w: &mut Window) {
    if let Some(previous) = w.undo.pop() {
        w.redo.push(std::mem::replace(&mut w.doc, previous));
        w.dirty = true;
    }
}
fn redo(w: &mut Window) {
    if let Some(next) = w.redo.pop() {
        w.undo.push(std::mem::replace(&mut w.doc, next));
        w.dirty = true;
    }
}
fn hint(w: &Window) -> &'static str {
    match w.mode {
        Mode::Tiles => {
            "Click a tile to select it (Shift adds). Click an empty cell to create a tile there. Right-click a tile to remove it."
        }
        Mode::Collision => {
            "Pick a shape on the right, then click or drag across tiles. Right-click clears collision."
        }
        Mode::Terrains => {
            "Click a tile's centre or edges to paint the selected terrain; right-click clears. Shift+click fills a whole tile."
        }
        Mode::Animation => "Select a tile; its frames continue to the right of it in the atlas.",
        Mode::CustomData => {
            "Pick a layer and value on the right, then click or drag across tiles. Right-click resets to 0."
        }
    }
}

fn header(ui: &Ui, e: &mut Editor, w: &mut Window) {
    let mut textures: Vec<_> = e
        .assets
        .index
        .usable()
        .filter(|r| r.meta.kind == crate::assets::Kind::Texture)
        .map(|r| (r.meta.id, r.meta.source.clone()))
        .collect();
    textures.sort_by(|a, b| a.1.cmp(&b.1));
    ui.align_text_to_frame_padding();
    ui.text("Texture");
    ui.same_line();
    ui.set_next_item_width(220.);
    let label = w
        .doc
        .texture
        .and_then(|id| textures.iter().find(|t| t.0 == id))
        .map_or_else(|| "Choose a texture".into(), |t| t.1.clone());
    if let Some(_combo) = ui.begin_combo("##tileset-texture", label) {
        for (id, name) in &textures {
            if ui
                .selectable_config(name)
                .selected(w.doc.texture == Some(*id))
                .build()
            {
                w.checkpoint();
                w.doc.texture = Some(*id);
            }
        }
    }
    ui.same_line_with_spacing(0., 14.);
    for (label, min) in [("Tile", 1), ("Margin", 0), ("Separation", 0)] {
        let current = match label {
            "Tile" => w.doc.tile_size,
            "Margin" => w.doc.margin,
            _ => w.doc.separation,
        };
        ui.align_text_to_frame_padding();
        ui.text(label);
        ui.same_line();
        let mut v = current.map(i32::from);
        ui.set_next_item_width(84.);
        if crate::gui::Drag::new(format!("##{label}"))
            .range(min, 256)
            .speed(0.1)
            .build_array(ui, &mut v)
        {
            let next = v.map(|x| x.clamp(min, 256) as u16);
            if next != current {
                w.checkpoint();
                match label {
                    "Tile" => w.doc.tile_size = next,
                    "Margin" => w.doc.margin = next,
                    _ => w.doc.separation = next,
                }
            }
        }
        ui.same_line_with_spacing(0., 10.);
    }
    ui.new_line();
    let texture = w.doc.texture.and_then(|id| texture_data(e, id));
    ui.disabled(texture.is_none(), || {
        if ui.button(format!(
            "{} Create tiles in visible cells",
            crate::world2d_editor::ICON_WAND
        )) && let Some(texture) = &texture
        {
            w.checkpoint();
            let added = w.doc.create_opaque_tiles(texture);
            w.status = format!("Added {added} tiles");
        }
    });
    if ui.is_item_hovered() {
        ui.tooltip_text("Adds a tile for every grid cell holding at least one visible texel");
    }
    ui.same_line();
    if ui.button("Remove all tiles") && !w.doc.tiles.is_empty() {
        w.checkpoint();
        w.doc.tiles.clear();
        w.selected.clear();
    }
    ui.same_line_with_spacing(0., 14.);
    ui.disabled(w.undo.is_empty(), || {
        if ui.button("Undo") {
            undo(w);
        }
    });
    ui.same_line();
    ui.disabled(w.redo.is_empty(), || {
        if ui.button("Redo") {
            redo(w);
        }
    });
    ui.same_line_with_spacing(0., 14.);
    if let Some(size) = texture.as_ref().map(|t| [t.width, t.height]) {
        let grid = w.doc.grid(size);
        ui.text_disabled(format!(
            "{}x{} texels  ·  {}x{} grid  ·  {} tiles",
            size[0],
            size[1],
            grid[0],
            grid[1],
            w.doc.tiles.len()
        ));
        if let Err(error) = w.doc.validate_texture(size) {
            ui.same_line();
            ui.text_colored([1., 0.45, 0.4, 1.], error);
        }
    }
    if !w.status.is_empty() {
        ui.same_line();
        ui.text_disabled(format!("·  {}", w.status));
    }
}
fn texture_data(e: &Editor, id: Uuid) -> Option<Arc<crate::texture::Data>> {
    if let Some(data) = e.scene.textures.get(&id) {
        return Some(data.clone());
    }
    let record = e.assets.index.resolve(id).ok()?;
    let package = crate::assets::Package::load(&record.path).ok()?;
    crate::texture::decode(&package.source).ok().map(Arc::new)
}

/// The zoomable atlas with the current mode's overlay and painting.
fn atlas(ui: &Ui, e: &mut Editor, w: &mut Window) {
    let Some(texture_id) = w.doc.texture else {
        ui.text_disabled("Choose the texture that holds the tiles above.");
        return;
    };
    let Some((texture, size)) = e.world2d.textures.get(texture_id) else {
        ui.text_disabled("Loading the texture preview...");
        // The preview cache uploads textures the scene holds; make sure it holds this one.
        if !e.scene.textures.contains_key(&texture_id)
            && let Some(data) = texture_data(e, texture_id)
        {
            e.scene.textures.insert(texture_id, data);
        }
        return;
    };
    let available = ui.content_region_avail();
    if w.zoom <= 0. {
        let fit = ((available[0] - 16.) / f32::from(size[0]))
            .min((available[1] - 16.) / f32::from(size[1]));
        w.zoom = fit.floor().clamp(1., 8.);
    }
    let io = ui.io();
    let zoom = w.zoom;
    let canvas = [f32::from(size[0]) * zoom, f32::from(size[1]) * zoom];
    let start = ui.cursor_screen_pos();
    ui.invisible_button_flags(
        "##atlas",
        canvas,
        imgui::ButtonFlags::MOUSE_BUTTON_LEFT | imgui::ButtonFlags::MOUSE_BUTTON_RIGHT,
    );
    let hovered = ui.is_item_hovered();
    if hovered && io.mouse_wheel != 0. {
        w.zoom = (w.zoom * 1.25_f32.powf(io.mouse_wheel)).clamp(0.5, 16.);
    }
    let draw = ui.get_window_draw_list();
    crate::world2d_editor::checker(
        &draw,
        start,
        [start[0] + canvas[0], start[1] + canvas[1]],
        8.,
    );
    draw.add_image(texture, start, [start[0] + canvas[0], start[1] + canvas[1]])
        .build();
    let tile = w.doc.tile_size.map(|v| f32::from(v) * zoom);
    let rect = |atlas: [u8; 2]| {
        let t = w.doc.texel([u16::from(atlas[0]), u16::from(atlas[1])]);
        let a = [start[0] + t[0] as f32 * zoom, start[1] + t[1] as f32 * zoom];
        (a, [a[0] + tile[0], a[1] + tile[1]])
    };
    let grid = w.doc.grid(size);
    for row in 0..grid[1] {
        for column in 0..grid[0] {
            let atlas = [column as u8, row as u8];
            let (a, b) = rect(atlas);
            if w.doc.tile(atlas).is_some() {
                draw.add_rect(a, b, [1., 1., 1., 0.25]).build();
            } else {
                draw.add_rect(a, b, [0., 0., 0., 0.5]).filled(true).build();
                draw.add_rect(a, b, [1., 1., 1., 0.06]).build();
            }
        }
    }
    let mouse = io.mouse_pos;
    let texel = [(mouse[0] - start[0]) / zoom, (mouse[1] - start[1]) / zoom];
    let under = hovered.then(|| w.doc.atlas_at(texel, size)).flatten();
    // Overlays.
    for t in &w.doc.tiles {
        let (a, b) = rect(t.atlas);
        match w.mode {
            Mode::Collision => collision_overlay(&draw, t, a, zoom),
            Mode::Terrains => terrain_overlay(&draw, &w.doc, t, a, b),
            Mode::CustomData => {
                if let Some(value) = t.custom.get(w.custom_layer)
                    && *value != 0
                {
                    let text = match w.doc.custom_layers.get(w.custom_layer).map(|l| l.kind) {
                        Some(CustomKind::Bool) => "on".to_string(),
                        _ => value.to_string(),
                    };
                    draw.add_rect(a, b, [0.4, 0.8, 0.5, 0.25])
                        .filled(true)
                        .build();
                    draw.add_text([a[0] + 3., a[1] + 2.], [1., 1., 1., 1.], text);
                }
            }
            Mode::Animation if t.frames > 1 => {
                for frame in 1..t.frames {
                    let at = t.frame_atlas(frame);
                    if at[0] > 255 || at[1] > 255 {
                        break;
                    }
                    let (fa, fb) = rect([at[0] as u8, at[1] as u8]);
                    draw.add_rect(fa, fb, [0.78, 0.48, 1.0, 0.25])
                        .filled(true)
                        .build();
                    draw.add_text(
                        [fa[0] + 3., fa[1] + 2.],
                        [1., 1., 1., 0.9],
                        frame.to_string(),
                    );
                }
                draw.add_text(
                    [a[0] + 3., a[1] + 2.],
                    [0.9, 0.7, 1., 1.],
                    crate::world2d_editor::ICON_FILM,
                );
            }
            _ => {}
        }
        if w.selected.contains(&t.atlas) {
            draw.add_rect(a, b, ACCENT).thickness(2.).build();
        }
    }
    for atlas in &w.selected {
        if w.doc.tile(*atlas).is_none() {
            let (a, b) = rect(*atlas);
            crate::world2d_editor::dashed(&draw, a, [b[0], a[1]], ACCENT, 4.);
            crate::world2d_editor::dashed(&draw, [b[0], a[1]], b, ACCENT, 4.);
            crate::world2d_editor::dashed(&draw, b, [a[0], b[1]], ACCENT, 4.);
            crate::world2d_editor::dashed(&draw, [a[0], b[1]], a, ACCENT, 4.);
        }
    }
    let Some(atlas) = under else {
        return;
    };
    let (a, b) = rect(atlas);
    draw.add_rect(a, b, [1., 1., 1., 0.9]).thickness(1.).build();
    let left = ui.is_mouse_down(MouseButton::Left);
    let right = ui.is_mouse_down(MouseButton::Right);
    let clicked_left = ui.is_mouse_clicked(MouseButton::Left);
    let clicked_right = ui.is_mouse_clicked(MouseButton::Right);
    if clicked_left || clicked_right {
        w.painted.clear();
    }
    let size_t = w.doc.tile_size;
    match w.mode {
        Mode::Tiles => {
            if clicked_left {
                if !ui.io().key_shift {
                    w.selected.clear();
                }
                if w.doc.tile(atlas).is_none() {
                    w.checkpoint();
                    let mut tile = tileset::Tile::new(atlas);
                    tile.rect = [0, 0, size_t[0].min(255) as u8, size_t[1].min(255) as u8];
                    w.doc.tiles.push(tile);
                    w.doc.normalize();
                }
                w.selected.insert(atlas);
            }
            if clicked_right && w.doc.tile(atlas).is_some() {
                w.checkpoint();
                w.doc.tiles.retain(|t| t.atlas != atlas);
                w.selected.remove(&atlas);
            }
        }
        Mode::Collision => {
            if (left || right) && w.doc.tile(atlas).is_some() && w.painted.insert(atlas) {
                w.checkpoint();
                let shape = if right { Shape::None } else { w.shape };
                let one_way = !right && w.one_way;
                if let Some(t) = w.doc.tile_mut(atlas) {
                    shape.apply(t, size_t);
                    t.one_way = one_way && shape != Shape::None;
                }
            }
        }
        Mode::Terrains => {
            if (clicked_left || clicked_right) && w.doc.terrain_sets.get(w.terrain_set).is_some() {
                let local = [(mouse[0] - a[0]) / tile[0], (mouse[1] - a[1]) / tile[1]];
                let zone = [0, 1].map(|k| ((local[k] * 3.).floor() as i32).clamp(0, 2));
                if w.block && clicked_left {
                    w.checkpoint();
                    tileset::assign_block(&mut w.doc, atlas, w.terrain_set as u8, w.terrain);
                    w.block = false;
                } else {
                    w.checkpoint();
                    if w.doc.tile(atlas).is_none() {
                        w.doc.tiles.push(tileset::Tile::new(atlas));
                        w.doc.normalize();
                    }
                    let (set, terrain) = (w.terrain_set as u8, w.terrain);
                    let fill = ui.io().key_shift;
                    if let Some(t) = w.doc.tile_mut(atlas) {
                        paint_bit(t, set, terrain, zone, clicked_right, fill);
                    }
                }
            }
        }
        Mode::Animation => {
            if clicked_left && w.doc.tile(atlas).is_some() {
                w.selected.clear();
                w.selected.insert(atlas);
            }
        }
        Mode::CustomData => {
            if (left || right)
                && w.doc.tile(atlas).is_some()
                && w.custom_layer < w.doc.custom_layers.len()
                && w.painted.insert(atlas)
            {
                w.checkpoint();
                let (layer, value) = (w.custom_layer, if right { 0 } else { w.custom_value });
                if let Some(t) = w.doc.tile_mut(atlas) {
                    t.custom.resize(layer + 1, 0);
                    t.custom[layer] = value;
                }
            }
        }
    }
    if w.doc.tile(atlas).is_some() {
        let local = [(mouse[0] - a[0]) / tile[0], (mouse[1] - a[1]) / tile[1]];
        let zone = [0, 1].map(|k| ((local[k] * 3.).floor() as i32).clamp(0, 2));
        ui.tooltip(|| {
            ui.text(format!("Tile ({}, {})", atlas[0], atlas[1]));
            if w.mode == Mode::Terrains {
                let part = tileset::DIRECTIONS
                    .iter()
                    .position(|o| [o[0] + 1, o[1] + 1] == zone)
                    .map_or("centre: the tile's own terrain", |d| {
                        tileset::DIRECTION_NAMES[d]
                    });
                ui.text_disabled(format!("Under the mouse: {part}"));
            }
        });
    }
}
/// Sets or clears one terrain zone of a tile: the centre (1, 1) is the tile's own terrain,
/// the others its peering bits. `fill` paints the whole tile.
fn paint_bit(
    tile: &mut tileset::Tile,
    set: u8,
    terrain: u8,
    zone: [i32; 2],
    clear: bool,
    fill: bool,
) {
    if tile.terrain_set != Some(set) {
        tile.terrain_set = Some(set);
        tile.terrain = None;
        tile.peering = [None; 8];
    }
    let value = (!clear).then_some(terrain);
    if fill {
        tile.terrain = value;
        tile.peering = [value; 8];
    } else if zone == [1, 1] {
        tile.terrain = value;
    } else if let Some(d) = tileset::DIRECTIONS
        .iter()
        .position(|o| [o[0] + 1, o[1] + 1] == zone)
    {
        tile.peering[d] = value;
    }
    if tile.terrain.is_none() && tile.peering.iter().all(Option::is_none) {
        tile.terrain_set = None;
    }
}
fn collision_overlay(draw: &imgui::DrawListMut, t: &tileset::Tile, a: [f32; 2], zoom: f32) {
    if t.collision == Collision::None {
        return;
    }
    let r = t.rect.map(f32::from);
    let colour = if t.one_way { ONE_WAY } else { SOLID };
    let (lo, hi) = match t.collision {
        Collision::Rect => (
            [a[0] + r[0] * zoom, a[1] + r[1] * zoom],
            [a[0] + (r[0] + r[2]) * zoom, a[1] + (r[1] + r[3]) * zoom],
        ),
        _ => (a, [a[0] + r[2].max(1.) * zoom, a[1] + r[3].max(1.) * zoom]),
    };
    draw.add_rect(lo, hi, [colour[0], colour[1], colour[2], 0.3])
        .filled(true)
        .build();
    draw.add_rect(lo, hi, colour).thickness(1.5).build();
    if t.one_way {
        let mid = (lo[0] + hi[0]) * 0.5;
        draw.add_triangle(
            [mid - 4., lo[1] + 9.],
            [mid + 4., lo[1] + 9.],
            [mid, lo[1] + 3.],
            colour,
        )
        .filled(true)
        .build();
    }
}
fn terrain_overlay(
    draw: &imgui::DrawListMut,
    doc: &TileSet,
    t: &tileset::Tile,
    a: [f32; 2],
    b: [f32; 2],
) {
    let Some(set) = t
        .terrain_set
        .and_then(|s| doc.terrain_sets.get(usize::from(s)))
    else {
        return;
    };
    let colour = |terrain: Option<u8>| {
        terrain
            .and_then(|i| set.terrains.get(usize::from(i)))
            .map(|t| {
                let c = t.color.map(|v| f32::from(v) / 255.);
                [c[0], c[1], c[2], 0.6]
            })
    };
    let third = [(b[0] - a[0]) / 3., (b[1] - a[1]) / 3.];
    let zone = |z: [i32; 2]| {
        let lo = [a[0] + z[0] as f32 * third[0], a[1] + z[1] as f32 * third[1]];
        (lo, [lo[0] + third[0], lo[1] + third[1]])
    };
    if let Some(c) = colour(t.terrain) {
        let (lo, hi) = zone([1, 1]);
        draw.add_rect(lo, hi, c).filled(true).build();
    }
    for (d, offset) in tileset::DIRECTIONS.iter().enumerate() {
        if !set.mode.uses(d) {
            continue;
        }
        if let Some(c) = colour(t.peering[d]) {
            let (lo, hi) = zone([offset[0] + 1, offset[1] + 1]);
            draw.add_rect(lo, hi, c).filled(true).build();
        }
    }
}

fn side_panel(ui: &Ui, e: &mut Editor, w: &mut Window) {
    match w.mode {
        Mode::Tiles => tiles_panel(ui, w),
        Mode::Collision => {
            ui.text("Collision shape");
            for (shape, label) in Shape::ALL {
                if ui.radio_button_bool(label, w.shape == shape) {
                    w.shape = shape;
                }
            }
            ui.checkbox("One-way (only stops falls onto the top)", &mut w.one_way);
            ui.spacing();
            ui.text_disabled("Solid tiles block CharacterBody2D, rays and overlap queries. One-way tiles make jump-through platforms.");
            selected_tile(ui, w);
        }
        Mode::Terrains => terrains_panel(ui, w),
        Mode::Animation => animation_panel(ui, e, w),
        Mode::CustomData => custom_panel(ui, w),
    }
}
fn tiles_panel(ui: &Ui, w: &mut Window) {
    ui.text(format!("{} selected", w.selected.len()));
    if !w.selected.is_empty() {
        ui.same_line();
        if ui.small_button("Remove selected") {
            w.checkpoint();
            let selected = w.selected.clone();
            w.doc.tiles.retain(|t| !selected.contains(&t.atlas));
            w.selected.clear();
        }
    }
    selected_tile(ui, w);
}
/// The editable properties of the single selected tile.
fn selected_tile(ui: &Ui, w: &mut Window) {
    let Some(atlas) = (w.selected.len() == 1).then(|| *w.selected.first().unwrap()) else {
        return;
    };
    let Some(mut tile) = w.doc.tile(atlas).cloned() else {
        return;
    };
    let before = tile.clone();
    ui.separator();
    ui.text(format!("Tile ({}, {})", atlas[0], atlas[1]));
    let mut kind = match tile.collision {
        Collision::None => 0,
        Collision::Full => 1,
        Collision::Rect => 2,
    };
    if ui.combo_simple_string(
        crate::gui::field(ui, "Collision"),
        &mut kind,
        &["None", "Full", "Rectangle"],
    ) {
        tile.collision = [Collision::None, Collision::Full, Collision::Rect][kind];
    }
    if tile.collision == Collision::Rect {
        let mut r = tile.rect.map(i32::from);
        if crate::gui::Drag::new(crate::gui::field(ui, "Shape x/y/w/h"))
            .range(0, 255)
            .speed(0.1)
            .build_array(ui, &mut r)
        {
            tile.rect = r.map(|v| v.clamp(0, 255) as u8);
        }
    }
    if tile.collision != Collision::None {
        crate::gui::toggle(ui, "One-way", &mut tile.one_way);
    }
    crate::gui::Drag::new(crate::gui::field(ui, "Probability"))
        .range(0., 100.)
        .speed(0.01)
        .display_format("%.2f")
        .build(ui, &mut tile.probability);
    if ui.is_item_hovered() {
        ui.tooltip_text("Weight when painting randomly or choosing between equal terrain tiles");
    }
    for (index, layer) in w.doc.custom_layers.iter().enumerate() {
        tile.custom.resize(w.doc.custom_layers.len(), 0);
        match layer.kind {
            CustomKind::Int => {
                crate::gui::Drag::new(crate::gui::field(
                    ui,
                    &format!("{}##custom{index}", layer.name),
                ))
                .speed(0.1)
                .build(ui, &mut tile.custom[index]);
            }
            CustomKind::Bool => {
                let mut on = tile.custom[index] != 0;
                if crate::gui::toggle(ui, &format!("{}##custom{index}", layer.name), &mut on) {
                    tile.custom[index] = i32::from(on);
                }
            }
        }
    }
    if tile != before {
        w.checkpoint();
        if let Some(t) = w.doc.tile_mut(atlas) {
            *t = tile;
        }
    }
}
fn terrains_panel(ui: &Ui, w: &mut Window) {
    ui.text("Terrain sets");
    if ui.small_button(format!("{} Add set", crate::world2d_editor::ICON_PLUS))
        && w.doc.terrain_sets.len() < tileset::MAX_TERRAIN_SETS
    {
        w.checkpoint();
        w.doc.terrain_sets.push(TerrainSet {
            name: format!("Terrains {}", w.doc.terrain_sets.len() + 1),
            ..Default::default()
        });
        w.terrain_set = w.doc.terrain_sets.len() - 1;
        w.terrain = 0;
    }
    if w.doc.terrain_sets.is_empty() {
        ui.text_disabled("Add a terrain set to start. A set groups terrains that connect to each other, such as grass and dirt.");
        return;
    }
    let names: Vec<String> = w.doc.terrain_sets.iter().map(|s| s.name.clone()).collect();
    let names: Vec<&str> = names.iter().map(String::as_str).collect();
    w.terrain_set = w.terrain_set.min(names.len() - 1);
    ui.combo_simple_string(crate::gui::field(ui, "Set"), &mut w.terrain_set, &names);
    let mut set = w.doc.terrain_sets[w.terrain_set].clone();
    let before = set.clone();
    ui.input_text(crate::gui::field(ui, "Name"), &mut set.name)
        .build();
    let mut mode = tileset::TerrainMode::ALL
        .iter()
        .position(|m| *m == set.mode)
        .unwrap_or(0);
    let labels: Vec<&str> = tileset::TerrainMode::ALL
        .iter()
        .map(|m| m.label())
        .collect();
    if ui.combo_simple_string(crate::gui::field(ui, "Match"), &mut mode, &labels) {
        set.mode = tileset::TerrainMode::ALL[mode];
    }
    if ui.is_item_hovered() {
        ui.tooltip_text("Corners and sides: 47-tile blob layouts. Sides: 16-tile layouts. Corners: 16-tile corner layouts");
    }
    ui.spacing();
    ui.text("Terrains");
    let mut remove = None;
    for (index, terrain) in set.terrains.iter_mut().enumerate() {
        let _id = ui.push_id_usize(index);
        let mut colour = terrain.color.map(|c| f32::from(c) / 255.);
        if ui
            .color_edit3_config("##colour", &mut colour)
            .inputs(false)
            .build()
        {
            terrain.color = colour.map(|c| (c * 255.).round() as u8);
        }
        ui.same_line();
        if ui.radio_button_bool("##pick", w.terrain == index as u8) {
            w.terrain = index as u8;
        }
        ui.same_line();
        ui.set_next_item_width(140.);
        ui.input_text("##name", &mut terrain.name).build();
        ui.same_line();
        if ui.small_button("x") {
            remove = Some(index);
        }
    }
    if let Some(index) = remove {
        set.terrains.remove(index);
        w.terrain = 0;
    }
    if ui.small_button(format!("{} Add terrain", crate::world2d_editor::ICON_PLUS))
        && set.terrains.len() < tileset::MAX_TERRAINS
    {
        let palette = [
            [96, 196, 96],
            [196, 150, 90],
            [90, 150, 220],
            [220, 90, 90],
            [200, 200, 90],
        ];
        set.terrains.push(Terrain {
            name: format!("Terrain {}", set.terrains.len() + 1),
            color: palette[set.terrains.len() % palette.len()],
        });
    }
    ui.spacing();
    if ui.button("Assign a 3x3 block...") {
        w.block = true;
    }
    if ui.is_item_hovered() {
        ui.tooltip_text("Then click the top-left tile of a 3x3 block laid out as an area's corners, edges and centre");
    }
    if w.block {
        ui.text_colored(ACCENT, "Click the block's top-left tile.");
    }
    if ui.small_button("Remove set") {
        w.checkpoint();
        let removed = w.terrain_set as u8;
        w.doc.terrain_sets.remove(w.terrain_set);
        for tile in &mut w.doc.tiles {
            match tile.terrain_set {
                Some(s) if s == removed => {
                    tile.terrain_set = None;
                    tile.terrain = None;
                    tile.peering = [None; 8];
                }
                Some(s) if s > removed => tile.terrain_set = Some(s - 1),
                _ => {}
            }
        }
        w.terrain_set = 0;
        return;
    }
    if let Some(index) = remove {
        // Tiles that used the removed terrain lose it; later terrains shift down.
        let removed = index as u8;
        let set_index = w.terrain_set as u8;
        w.checkpoint();
        for tile in &mut w.doc.tiles {
            if tile.terrain_set != Some(set_index) {
                continue;
            }
            tile.terrain = shift_terrain(tile.terrain, removed);
            tile.peering = tile.peering.map(|v| shift_terrain(v, removed));
        }
        w.doc.terrain_sets[w.terrain_set] = set;
    } else if set != before {
        w.checkpoint();
        w.doc.terrain_sets[w.terrain_set] = set;
    }
    w.terrain = w.terrain.min(
        w.doc.terrain_sets[w.terrain_set]
            .terrains
            .len()
            .saturating_sub(1) as u8,
    );
}
/// A terrain reference after terrain `removed` is deleted: gone, or one lower when after it.
fn shift_terrain(value: Option<u8>, removed: u8) -> Option<u8> {
    match value {
        Some(t) if t == removed => None,
        Some(t) if t > removed => Some(t - 1),
        other => other,
    }
}
fn animation_panel(ui: &Ui, e: &mut Editor, w: &mut Window) {
    let Some(atlas) = (w.selected.len() == 1).then(|| *w.selected.first().unwrap()) else {
        ui.text_disabled("Select a tile to animate it.");
        return;
    };
    let Some(mut tile) = w.doc.tile(atlas).cloned() else {
        return;
    };
    let before = tile.clone();
    ui.text(format!("Tile ({}, {})", atlas[0], atlas[1]));
    let mut frames = i32::from(tile.frames);
    if crate::gui::Drag::new(crate::gui::field(ui, "Frames"))
        .range(1, 64)
        .speed(0.05)
        .build(ui, &mut frames)
    {
        tile.frames = frames.clamp(1, 64) as u8;
    }
    let mut columns = i32::from(tile.columns);
    if crate::gui::Drag::new(crate::gui::field(ui, "Frames per row"))
        .range(0, 64)
        .speed(0.05)
        .build(ui, &mut columns)
    {
        tile.columns = columns.clamp(0, 64) as u8;
    }
    if ui.is_item_hovered() {
        ui.tooltip_text("0 keeps every frame on the tile's row");
    }
    crate::gui::Drag::new(crate::gui::field(ui, "Speed (fps)"))
        .range(0.1, 60.)
        .speed(0.05)
        .display_format("%.1f")
        .build(ui, &mut tile.fps);
    if tile != before {
        w.checkpoint();
        if let Some(t) = w.doc.tile_mut(atlas) {
            *t = tile.clone();
        }
    }
    // A live preview of the animation.
    if let Some((texture, size)) = w.doc.texture.and_then(|t| e.world2d.textures.get(t)) {
        let frame = if tile.frames > 1 && tile.fps > 0. {
            ((ui.time() as f32 * tile.fps) as u32 % u32::from(tile.frames)) as u8
        } else {
            0
        };
        let texel = w.doc.texel(tile.frame_atlas(frame));
        let tw = f32::from(w.doc.tile_size[0]);
        let th = f32::from(w.doc.tile_size[1]);
        let scale = (96. / tw.max(th)).max(1.);
        let start = ui.cursor_screen_pos();
        ui.dummy([tw * scale, th * scale]);
        let draw = ui.get_window_draw_list();
        crate::world2d_editor::checker(
            &draw,
            start,
            [start[0] + tw * scale, start[1] + th * scale],
            6.,
        );
        draw.add_image(
            texture,
            start,
            [start[0] + tw * scale, start[1] + th * scale],
        )
        .uv_min([
            texel[0] as f32 / f32::from(size[0]),
            texel[1] as f32 / f32::from(size[1]),
        ])
        .uv_max([
            (texel[0] as f32 + tw) / f32::from(size[0]),
            (texel[1] as f32 + th) / f32::from(size[1]),
        ])
        .build();
    }
}
fn custom_panel(ui: &Ui, w: &mut Window) {
    ui.text("Custom data layers");
    let mut layers = w.doc.custom_layers.clone();
    let before = layers.clone();
    let mut remove = None;
    for (index, layer) in layers.iter_mut().enumerate() {
        let _id = ui.push_id_usize(index);
        if ui.radio_button_bool("##pick", w.custom_layer == index) {
            w.custom_layer = index;
        }
        ui.same_line();
        ui.set_next_item_width(120.);
        ui.input_text("##name", &mut layer.name).build();
        ui.same_line();
        let mut kind = usize::from(layer.kind == CustomKind::Bool);
        ui.set_next_item_width(60.);
        if ui.combo_simple_string("##kind", &mut kind, &["Int", "Bool"]) {
            layer.kind = if kind == 1 {
                CustomKind::Bool
            } else {
                CustomKind::Int
            };
        }
        ui.same_line();
        ui.text_disabled(format!("#{index}"));
        ui.same_line();
        if ui.small_button("x") {
            remove = Some(index);
        }
    }
    if ui.small_button(format!("{} Add layer", crate::world2d_editor::ICON_PLUS))
        && layers.len() < tileset::MAX_CUSTOM_LAYERS
    {
        layers.push(CustomLayer {
            name: format!("data{}", layers.len()),
            kind: CustomKind::Int,
        });
    }
    if let Some(index) = remove {
        layers.remove(index);
        w.checkpoint();
        for tile in &mut w.doc.tiles {
            if index < tile.custom.len() {
                tile.custom.remove(index);
            }
        }
        w.doc.custom_layers = layers;
        w.custom_layer = 0;
    } else if layers != before {
        w.checkpoint();
        w.doc.custom_layers = layers;
        w.doc.normalize();
    }
    if w.doc.custom_layers.is_empty() {
        ui.text_disabled("Custom data stores per-tile numbers gameplay reads: damage, friction, a footstep sound.");
        return;
    }
    ui.spacing();
    let kind = w.doc.custom_layers.get(w.custom_layer).map(|l| l.kind);
    match kind {
        Some(CustomKind::Bool) => {
            let mut on = w.custom_value != 0;
            crate::gui::toggle(ui, "Paint value", &mut on);
            w.custom_value = i32::from(on);
        }
        _ => {
            crate::gui::Drag::new(crate::gui::field(ui, "Paint value"))
                .speed(0.1)
                .build(ui, &mut w.custom_value);
        }
    }
    ui.text_disabled(format!(
        "Gameplay reads it with get_cell_custom_data(x, y, {}).",
        w.custom_layer
    ));
    selected_tile(ui, w);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shapes_cover_the_expected_half_and_terrain_zones_map_to_bits() {
        let mut tile = tileset::Tile::default();
        Shape::Bottom.apply(&mut tile, [16, 16]);
        assert_eq!(
            (tile.collision, tile.rect),
            (Collision::Rect, [0, 8, 16, 8])
        );
        Shape::Full.apply(&mut tile, [8, 16]);
        assert_eq!(
            (tile.collision, tile.rect),
            (Collision::Full, [0, 0, 8, 16])
        );
        Shape::None.apply(&mut tile, [8, 16]);
        assert_eq!(tile.collision, Collision::None);
        // Zone (2, 1) is the right side, (0, 0) the top-left corner, (1, 1) the centre.
        let mut tile = tileset::Tile::default();
        paint_bit(&mut tile, 0, 2, [2, 1], false, false);
        paint_bit(&mut tile, 0, 2, [0, 0], false, false);
        paint_bit(&mut tile, 0, 2, [1, 1], false, false);
        assert_eq!(tile.terrain_set, Some(0));
        assert_eq!(tile.terrain, Some(2));
        assert_eq!(tile.peering[0], Some(2));
        assert_eq!(tile.peering[5], Some(2));
        // Clearing everything detaches the tile from the set; Shift fills all zones.
        for zone in [[2, 1], [0, 0], [1, 1]] {
            paint_bit(&mut tile, 0, 2, zone, true, false);
        }
        assert_eq!(tile.terrain_set, None);
        paint_bit(&mut tile, 1, 0, [0, 0], false, true);
        assert_eq!(tile.peering, [Some(0); 8]);
        assert_eq!(tile.terrain, Some(0));
    }
}
