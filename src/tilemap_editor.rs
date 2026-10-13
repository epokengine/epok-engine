//! Tile painting in the 2D view (World2D initiative, phase 2): the tile tools a selected
//! TileMapLayer gets (select, paint, line, rectangle, bucket fill, picker, eraser, random
//! scatter, flips and quarter turns), terrain painting, saved patterns, the palette panel
//! under the view and the TileMapLayer Inspector band.
//!
//! Painting edits the layer's `cells` record (see `tilemap.rs`); every stroke is one undo
//! step. Right-dragging with a painting tool erases with that tool's shape, as the
//! eraser would.
use crate::{
    editor::Editor,
    tilemap::{self, Cell, Cells, Stamp},
    tileset::{self, PatternCell, TileSet},
    world2d::{self, TileMapLayer},
};
use imgui::{MouseButton, Ui};
use std::{
    collections::{BTreeMap, BTreeSet},
    hash::{Hash, Hasher},
    sync::Arc,
};
use uuid::Uuid;

// ---- icons (FontAwesome solid; listed in platform::EDITOR_FA_GLYPHS) --------------------
pub const ICON_TILEMAP: &str = "\u{f84c}";
pub const ICON_TILESET: &str = "\u{f00a}";
pub const ICON_SELECT: &str = "\u{f125}";
pub const ICON_PAINT: &str = "\u{f1fc}";
pub const ICON_LINE: &str = "\u{f715}";
pub const ICON_RECT: &str = "\u{f0c8}";
pub const ICON_BUCKET: &str = "\u{f576}";
pub const ICON_PICKER: &str = "\u{f1fb}";
pub const ICON_ERASER: &str = "\u{f12d}";
pub const ICON_RANDOM: &str = "\u{f522}";
pub const ICON_ROTATE_LEFT: &str = "\u{f2ea}";
pub const ICON_ROTATE_RIGHT: &str = "\u{f2f9}";
pub const ICON_TERRAIN: &str = "\u{f6fc}";
pub const ICON_PATTERN: &str = "\u{f247}";
pub const ICON_EDIT: &str = "\u{f304}";

const ACCENT: [f32; 4] = [1.0, 0.62, 0.20, 1.];
const ERASE: [f32; 4] = [0.95, 0.35, 0.35, 1.];
const GRID: [f32; 4] = [1., 1., 1., 0.13];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tool {
    Select,
    Paint,
    Line,
    Rect,
    Bucket,
    Picker,
    Eraser,
}
impl Tool {
    pub const ALL: [(Tool, &'static str, &'static str, &'static str); 7] = [
        (
            Tool::Select,
            ICON_SELECT,
            "Select cells (S)",
            "Drag a rectangle of cells. Ctrl+C copies it to the brush, Delete erases it",
        ),
        (
            Tool::Paint,
            ICON_PAINT,
            "Paint (D)",
            "Click or drag to paint the brush. Right-drag erases. Shift+click draws a line from the last cell",
        ),
        (
            Tool::Line,
            ICON_LINE,
            "Line (L)",
            "Drag to paint a straight line. Right-drag erases one",
        ),
        (
            Tool::Rect,
            ICON_RECT,
            "Rectangle (R)",
            "Drag to fill a rectangle with the brush. Right-drag erases one",
        ),
        (
            Tool::Bucket,
            ICON_BUCKET,
            "Bucket fill (B)",
            "Fill the connected area of the same tile. Right-click erases it",
        ),
        (
            Tool::Picker,
            ICON_PICKER,
            "Pick tile (P)",
            "Click a painted cell to load its tile into the brush",
        ),
        (
            Tool::Eraser,
            ICON_ERASER,
            "Eraser (E)",
            "Click or drag to erase cells",
        ),
    ];
    fn label(self) -> &'static str {
        Self::ALL.iter().find(|t| t.0 == self).map_or("", |t| t.2)
    }
    fn paints(self) -> bool {
        matches!(self, Tool::Paint | Tool::Line | Tool::Rect | Tool::Bucket)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tab {
    #[default]
    Tiles,
    Patterns,
    Terrains,
}

/// What painting places: a stamp of tiles, or a terrain the tiles are solved for.
#[derive(Clone, Debug, PartialEq)]
pub enum Brush {
    Tiles(Stamp),
    /// `terrain` None erases the set's terrain.
    Terrain {
        set: usize,
        terrain: Option<u8>,
    },
}
impl Default for Brush {
    fn default() -> Self {
        Brush::Tiles(Stamp::default())
    }
}

struct Stroke {
    start: [i32; 2],
    last: [i32; 2],
    erase: bool,
    /// Cells already painted by this stroke (paint and eraser drags).
    done: BTreeSet<[i32; 2]>,
}

/// The "New TileSet" form for one layer.
#[derive(Clone, Debug)]
pub struct NewTileSet {
    pub actor: Uuid,
    pub component: Uuid,
    pub texture: Option<Uuid>,
    pub tile: [u16; 2],
}

pub struct State {
    pub tool: Option<Tool>,
    pub brush: Brush,
    pub random: bool,
    pub contiguous: bool,
    pub tab: Tab,
    pub selection: BTreeSet<[i32; 2]>,
    pub palette_zoom: f32,
    pub panel_height: f32,
    pub show_grid: bool,
    pub new_tileset: Option<NewTileSet>,
    palette_anchor: Option<[u8; 2]>,
    palette_drag: Option<([u8; 2], [u8; 2])>,
    stroke: Option<Stroke>,
    cache: BTreeMap<Uuid, (u64, Arc<Cells>)>,
    hovered: Option<[i32; 2]>,
    last_cell: Option<[i32; 2]>,
    /// The layer actor the tools last switched on for.
    layer: Option<Uuid>,
    pattern_name: String,
    renaming: Option<usize>,
}
impl State {
    /// The cell under the mouse in the 2D view, while painting.
    pub fn hovered_cell(&self) -> Option<[i32; 2]> {
        self.hovered
    }
}
impl Default for State {
    fn default() -> Self {
        Self {
            tool: Some(Tool::Paint),
            brush: Brush::default(),
            random: false,
            contiguous: true,
            tab: Tab::Tiles,
            selection: BTreeSet::new(),
            palette_zoom: 2.,
            panel_height: 220.,
            show_grid: true,
            new_tileset: None,
            palette_anchor: None,
            palette_drag: None,
            stroke: None,
            cache: BTreeMap::new(),
            hovered: None,
            last_cell: None,
            layer: None,
            pattern_name: String::new(),
            renaming: None,
        }
    }
}

// ---- the selected layer ----------------------------------------------------------------

/// One TileMapLayer as the tools see it.
#[derive(Clone)]
pub struct Layer {
    pub actor: Uuid,
    pub component: Uuid,
    pub record: TileMapLayer,
    pub cells: Arc<Cells>,
    pub tileset: Option<Arc<TileSet>>,
    /// World position of cell (0, 0)'s top-left corner.
    pub origin: [f32; 2],
}
impl Layer {
    pub fn tile_size(&self) -> [f32; 2] {
        self.tileset
            .as_ref()
            .map_or([16., 16.], |t| t.tile_size.map(|v| f32::from(v.max(1))))
    }
    pub fn cell_at(&self, world: [f32; 2]) -> [i32; 2] {
        let size = self.tile_size();
        [0, 1].map(|k| ((world[k] - self.origin[k]) / size[k]).floor() as i32)
    }
    pub fn cell_origin(&self, cell: [i32; 2]) -> [f32; 2] {
        let size = self.tile_size();
        [0, 1].map(|k| self.origin[k] + cell[k] as f32 * size[k])
    }
}
/// Decoded cells of a layer record, cached by the record's content.
pub fn cells_of(state: &mut State, component: Uuid, text: &str) -> Arc<Cells> {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    text.hash(&mut hasher);
    let key = hasher.finish();
    if let Some((cached, cells)) = state.cache.get(&component)
        && *cached == key
    {
        return cells.clone();
    }
    let cells = Arc::new(tilemap::decode(text).unwrap_or_default());
    state.cache.insert(component, (key, cells.clone()));
    cells
}
/// The TileSet `id`, loading it into the scene when a new asset is not resolved yet.
pub fn tileset(e: &mut Editor, id: Uuid) -> Option<Arc<TileSet>> {
    if let Some(set) = e.scene.tilesets.get(&id) {
        return Some(set.clone());
    }
    let set = Arc::new(
        e.assets
            .index
            .resolve(id)
            .ok()
            .and_then(|r| tileset::document(r).ok())?,
    );
    e.scene.tilesets.insert(id, set.clone());
    Some(set)
}
pub fn layer(e: &mut Editor, actor: Uuid) -> Option<Layer> {
    let index = e.scene.actor_index(actor)?;
    let component = world2d::component(&e.scene.actors[index], world2d::TILEMAP_LAYER)?;
    let (component_id, record) = (component.id, world2d::read::<TileMapLayer>(component));
    let origin = world2d::world_position(&e.scene, index);
    let cells = cells_of(&mut e.world2d.tiles, component_id, &record.cells);
    let tileset = record.tile_set.and_then(|id| tileset(e, id));
    Some(Layer {
        actor,
        component: component_id,
        record,
        cells,
        tileset,
        origin,
    })
}
pub fn selected_layer(e: &mut Editor) -> Option<Layer> {
    let actor = e.selected_actor?;
    layer(e, actor)
}
/// Whether the view's clicks paint tiles instead of moving nodes.
pub fn painting(e: &mut Editor) -> bool {
    let current = e.selected_actor.filter(|id| {
        e.scene.actor_index(*id).is_some_and(|i| {
            world2d::component(&e.scene.actors[i], world2d::TILEMAP_LAYER).is_some()
        })
    });
    // Selecting another layer switches the tile tools back on.
    if current != e.world2d.tiles.layer {
        e.world2d.tiles.layer = current;
        e.world2d.tiles.selection.clear();
        e.world2d.tiles.last_cell = None;
        if current.is_some() && e.world2d.tiles.tool.is_none() {
            e.world2d.tiles.tool = Some(Tool::Paint);
        }
    }
    current.is_some() && e.world2d.tiles.tool.is_some() && !e.playing
}

/// Writes a layer's cells back as one coalesced edit.
fn commit(e: &mut Editor, layer: &Layer, cells: &Cells) {
    let Some(index) = e.scene.actor_index(layer.actor) else {
        return;
    };
    let Some(component) = e.scene.actors[index]
        .components
        .iter_mut()
        .find(|c| c.id == layer.component)
    else {
        return;
    };
    let mut record: TileMapLayer = world2d::read(component);
    record.cells = tilemap::encode(cells);
    world2d::write(component, &record);
    e.scene.actors[index].refresh_components();
    cells_of(&mut e.world2d.tiles, layer.component, &record.cells);
    e.changed_coalesced("tile-paint");
}
fn edit_record(e: &mut Editor, actor: Uuid, component: Uuid, edit: impl FnOnce(&mut TileMapLayer)) {
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
    let mut record: TileMapLayer = world2d::read(c);
    edit(&mut record);
    world2d::write(c, &record);
    e.scene.actors[index].refresh_components();
}

// ---- painting --------------------------------------------------------------------------

/// The cells a tool affects for a gesture from `a` to `b`.
fn shape(tool: Tool, layer: &Layer, a: [i32; 2], b: [i32; 2], contiguous: bool) -> Vec<[i32; 2]> {
    match tool {
        Tool::Line => tilemap::line(a, b),
        Tool::Rect | Tool::Select => tilemap::rect(a, b),
        Tool::Bucket => {
            let (lo, hi) = tilemap::bounds(&layer.cells).unwrap_or((b, b));
            let margin = 16;
            let limit = (
                [lo[0].min(b[0]) - margin, lo[1].min(b[1]) - margin],
                [hi[0].max(b[0]) + margin, hi[1].max(b[1]) + margin],
            );
            tilemap::flood(&layer.cells, b, contiguous, limit)
        }
        _ => vec![b],
    }
}
/// Applies the brush (or erasing) to `targets`. `origin` anchors a repeating stamp.
fn apply(
    brush: &Brush,
    random: bool,
    tileset: Option<&TileSet>,
    cells: &mut Cells,
    targets: &[[i32; 2]],
    origin: [i32; 2],
    erase: bool,
) {
    let targets: Vec<[i32; 2]> = targets
        .iter()
        .copied()
        .filter(|p| tilemap::in_range(*p))
        .collect();
    match brush {
        Brush::Terrain { set, terrain } if tileset.is_some() => {
            let tileset = tileset.unwrap();
            let painted = if erase { None } else { *terrain };
            let area: BTreeSet<[i32; 2]> = targets.iter().copied().collect();
            let set_index = *set as u8;
            let terrain_of = |p: [i32; 2]| {
                if area.contains(&p) {
                    painted
                } else {
                    cells
                        .get(&p)
                        .and_then(|c| tileset.tile(c.atlas))
                        .filter(|t| t.terrain_set == Some(set_index))
                        .and_then(|t| t.terrain)
                }
            };
            let solved = tileset::solve(tileset, *set, &area, &terrain_of);
            if painted.is_none() {
                for p in &area {
                    cells.remove(p);
                }
            }
            for (p, atlas) in solved {
                cells.insert(p, Cell::new(atlas));
            }
        }
        Brush::Tiles(stamp) if !erase && !stamp.is_empty() => {
            for p in targets {
                if let Some(cell) = stamp.cell_at(tileset, origin, p, random) {
                    cells.insert(p, cell);
                }
            }
        }
        _ => {
            for p in &targets {
                cells.remove(p);
            }
        }
    }
}
/// The cells a paint stroke covers when it reaches `cell`: a single-tile brush paints the
/// path; a larger stamp is laid on its own grid from the stroke's start.
fn stamp_targets(
    stamp: &Stamp,
    start: [i32; 2],
    from: [i32; 2],
    to: [i32; 2],
) -> Vec<([i32; 2], [i32; 2])> {
    let size = stamp.size();
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    for p in tilemap::line(from, to) {
        let anchor = [
            start[0] + (p[0] - start[0]).div_euclid(size[0]) * size[0],
            start[1] + (p[1] - start[1]).div_euclid(size[1]) * size[1],
        ];
        if !seen.insert(anchor) {
            continue;
        }
        for c in &stamp.cells {
            out.push((
                anchor,
                [
                    anchor[0] + i32::from(c.offset[0]),
                    anchor[1] + i32::from(c.offset[1]),
                ],
            ));
        }
    }
    out
}

/// Viewport input while painting. Returns true when it consumed the mouse.
#[allow(clippy::too_many_arguments)]
pub fn input(ui: &Ui, e: &mut Editor, hovered: bool, world: [f32; 2]) {
    let Some(layer) = selected_layer(e) else {
        return;
    };
    let Some(tool) = e.world2d.tiles.tool else {
        return;
    };
    let cell = layer.cell_at(world);
    e.world2d.tiles.hovered = hovered.then_some(cell);
    let space = hovered && !crate::gui::text_input_active(ui) && ui.is_key_down(imgui::Key::Space);
    if space {
        return;
    }
    let shift = ui.io().key_shift;
    // Begin a gesture.
    for (button, erase) in [(MouseButton::Left, false), (MouseButton::Right, true)] {
        if !(hovered && ui.is_mouse_clicked(button)) || e.world2d.tiles.stroke.is_some() {
            continue;
        }
        let erase = erase || tool == Tool::Eraser;
        match tool {
            Tool::Picker => {
                if let Some(c) = layer.cells.get(&cell) {
                    e.world2d.tiles.brush = Brush::Tiles(Stamp {
                        cells: vec![PatternCell {
                            offset: [0, 0],
                            atlas: c.atlas,
                            flags: c.flags,
                        }],
                    });
                    e.world2d.tiles.tool = Some(Tool::Paint);
                    e.world2d.tiles.tab = Tab::Tiles;
                }
                return;
            }
            Tool::Bucket => {
                let mut cells = (*layer.cells).clone();
                let targets = shape(tool, &layer, cell, cell, e.world2d.tiles.contiguous);
                let brush = e.world2d.tiles.brush.clone();
                apply(
                    &brush,
                    e.world2d.tiles.random,
                    layer.tileset.as_deref(),
                    &mut cells,
                    &targets,
                    cell,
                    erase,
                );
                commit(e, &layer, &cells);
                e.end_coalesced();
                return;
            }
            _ => {}
        }
        let start = if shift && tool == Tool::Paint {
            e.world2d.tiles.last_cell.unwrap_or(cell)
        } else {
            cell
        };
        e.world2d.tiles.stroke = Some(Stroke {
            start,
            last: start,
            erase,
            done: BTreeSet::new(),
        });
        if tool == Tool::Select {
            e.world2d.tiles.selection = [cell].into_iter().collect();
        }
    }
    let Some(mut stroke) = e.world2d.tiles.stroke.take() else {
        return;
    };
    let button = if stroke.erase && tool != Tool::Eraser {
        MouseButton::Right
    } else {
        MouseButton::Left
    };
    let released = !ui.is_mouse_down(button);
    match tool {
        Tool::Paint | Tool::Eraser => {
            let mut cells = (*layer.cells).clone();
            let brush = if tool == Tool::Eraser {
                Brush::Tiles(Stamp::default())
            } else {
                e.world2d.tiles.brush.clone()
            };
            let erase = stroke.erase || tool == Tool::Eraser;
            let mut changed = false;
            match &brush {
                Brush::Tiles(stamp) if !erase && stamp.cells.len() > 1 => {
                    for (anchor, p) in stamp_targets(stamp, stroke.start, stroke.last, cell) {
                        if stroke.done.insert(p) {
                            apply(
                                &brush,
                                e.world2d.tiles.random,
                                layer.tileset.as_deref(),
                                &mut cells,
                                &[p],
                                anchor,
                                false,
                            );
                            changed = true;
                        }
                    }
                }
                _ => {
                    let path: Vec<_> = tilemap::line(stroke.last, cell)
                        .into_iter()
                        .filter(|p| stroke.done.insert(*p))
                        .collect();
                    if !path.is_empty() {
                        apply(
                            &brush,
                            e.world2d.tiles.random,
                            layer.tileset.as_deref(),
                            &mut cells,
                            &path,
                            stroke.start,
                            erase,
                        );
                        changed = true;
                    }
                }
            }
            if changed && cells != *layer.cells {
                commit(e, &layer, &cells);
            }
            stroke.last = cell;
            e.world2d.tiles.last_cell = Some(cell);
        }
        Tool::Select => {
            stroke.last = cell;
            e.world2d.tiles.selection = tilemap::rect(stroke.start, cell).into_iter().collect();
        }
        Tool::Line | Tool::Rect => {
            stroke.last = cell;
            if released {
                let mut cells = (*layer.cells).clone();
                let targets = shape(tool, &layer, stroke.start, cell, true);
                let origin = [stroke.start[0].min(cell[0]), stroke.start[1].min(cell[1])];
                let brush = e.world2d.tiles.brush.clone();
                apply(
                    &brush,
                    e.world2d.tiles.random,
                    layer.tileset.as_deref(),
                    &mut cells,
                    &targets,
                    origin,
                    stroke.erase,
                );
                if cells != *layer.cells {
                    commit(e, &layer, &cells);
                }
                e.world2d.tiles.last_cell = Some(cell);
            }
        }
        _ => {}
    }
    if released {
        e.end_coalesced();
    } else {
        e.world2d.tiles.stroke = Some(stroke);
    }
}

/// Keyboard shortcuts of the tile tools. Returns true when a key was handled, so the node
/// shortcuts of the 2D view skip it.
pub fn shortcuts(ui: &Ui, e: &mut Editor) -> bool {
    if crate::gui::text_input_active(ui) || e.playing {
        return false;
    }
    let io = ui.io();
    let command = io.key_ctrl || io.key_super;
    let state = &mut e.world2d.tiles;
    if command {
        if ui.is_key_pressed_no_repeat(imgui::Key::C) && !state.selection.is_empty() {
            copy_selection(e);
            return true;
        }
        return false;
    }
    for (key, tool) in [
        (imgui::Key::S, Tool::Select),
        (imgui::Key::D, Tool::Paint),
        (imgui::Key::L, Tool::Line),
        (imgui::Key::R, Tool::Rect),
        (imgui::Key::B, Tool::Bucket),
        (imgui::Key::P, Tool::Picker),
        (imgui::Key::E, Tool::Eraser),
    ] {
        if ui.is_key_pressed_no_repeat(key) {
            state.tool = Some(tool);
            return true;
        }
    }
    if let Brush::Tiles(stamp) = &mut state.brush {
        let mut handled = true;
        if ui.is_key_pressed_no_repeat(imgui::Key::Z) {
            stamp.rotate_counterclockwise();
        } else if ui.is_key_pressed_no_repeat(imgui::Key::X) {
            stamp.rotate_clockwise();
        } else if ui.is_key_pressed_no_repeat(imgui::Key::C) {
            stamp.flip_h();
        } else if ui.is_key_pressed_no_repeat(imgui::Key::V) {
            stamp.flip_v();
        } else {
            handled = false;
        }
        if handled {
            return true;
        }
    }
    if ui.is_key_pressed_no_repeat(imgui::Key::G) {
        state.random = !state.random;
        return true;
    }
    if (ui.is_key_pressed_no_repeat(imgui::Key::Delete)
        || ui.is_key_pressed_no_repeat(imgui::Key::Backspace))
        && !state.selection.is_empty()
    {
        if let Some(layer) = selected_layer(e) {
            let mut cells = (*layer.cells).clone();
            for p in &e.world2d.tiles.selection {
                cells.remove(p);
            }
            commit(e, &layer, &cells);
            e.end_coalesced();
        }
        return true;
    }
    if ui.is_key_pressed_no_repeat(imgui::Key::Escape) {
        let state = &mut e.world2d.tiles;
        if !state.selection.is_empty() {
            state.selection.clear();
        } else {
            state.tool = None;
        }
        return true;
    }
    if ui.is_key_pressed_no_repeat(imgui::Key::Q) || ui.is_key_pressed_no_repeat(imgui::Key::W) {
        // The node tools take the view back.
        e.world2d.tiles.tool = None;
        return false;
    }
    false
}
fn copy_selection(e: &mut Editor) {
    let Some(layer) = selected_layer(e) else {
        return;
    };
    let pattern = tilemap::pattern_from(&layer.cells, &e.world2d.tiles.selection, "");
    if pattern.cells.is_empty() {
        return;
    }
    e.world2d.tiles.brush = Brush::Tiles(Stamp::from_pattern(&pattern));
    e.world2d.tiles.tool = Some(Tool::Paint);
    e.log(format!(
        "Copied {} cells to the brush.",
        pattern.cells.len()
    ));
}

// ---- drawing ---------------------------------------------------------------------------

/// Draws cells of a tile layer: `to_screen` maps world pixels, `visible` (min, max world)
/// culls, `tint` multiplies, `seconds` animates.
#[allow(clippy::too_many_arguments)]
pub fn draw_cells<'a>(
    draw: &imgui::DrawListMut,
    texture: Option<(imgui::TextureId, [u16; 2])>,
    tileset: &TileSet,
    cells: impl Iterator<Item = ([i32; 2], &'a Cell)>,
    origin: [f32; 2],
    to_screen: &dyn Fn([f32; 2]) -> [f32; 2],
    visible: ([f32; 2], [f32; 2]),
    tint: [f32; 4],
    seconds: f32,
) -> usize {
    let size = tileset.tile_size.map(|v| f32::from(v.max(1)));
    let mut drawn = 0;
    for (p, cell) in cells {
        let lo = [
            origin[0] + p[0] as f32 * size[0],
            origin[1] + p[1] as f32 * size[1],
        ];
        let hi = [lo[0] + size[0], lo[1] + size[1]];
        if hi[0] < visible.0[0]
            || hi[1] < visible.0[1]
            || lo[0] > visible.1[0]
            || lo[1] > visible.1[1]
        {
            continue;
        }
        let corners = [lo, [hi[0], lo[1]], hi, [lo[0], hi[1]]].map(to_screen);
        let tile = tileset.tile(cell.atlas);
        match (tile, texture) {
            (Some(tile), Some((id, texture_size))) => {
                let frame = if tile.frames > 1 && tile.fps > 0. {
                    ((seconds * tile.fps) as u32 % u32::from(tile.frames)) as u8
                } else {
                    0
                };
                let texel = tileset.texel(tile.frame_atlas(frame));
                let (w, h) = (f32::from(texture_size[0]), f32::from(texture_size[1]));
                let uv = [[0., 0.], [1., 0.], [1., 1.], [0., 1.]].map(|c| {
                    let s = tilemap::sample(cell.flags, c);
                    [
                        (texel[0] as f32 + s[0] * size[0]) / w,
                        (texel[1] as f32 + s[1] * size[1]) / h,
                    ]
                });
                draw.add_image_quad(id, corners[0], corners[1], corners[2], corners[3])
                    .uv(uv[0], uv[1], uv[2], uv[3])
                    .col(tint)
                    .build();
            }
            (None, _) => {
                // A tile the TileSet no longer has.
                draw.add_rect(corners[0], corners[2], [0.8, 0.2, 0.2, 0.35 * tint[3]])
                    .filled(true)
                    .build();
                draw.add_line(corners[0], corners[2], [1., 0.4, 0.4, 0.8 * tint[3]])
                    .build();
                draw.add_line(corners[1], corners[3], [1., 0.4, 0.4, 0.8 * tint[3]])
                    .build();
            }
            _ => {
                draw.add_rect(corners[0], corners[2], [0.6, 0.6, 0.65, 0.3 * tint[3]])
                    .filled(true)
                    .build();
            }
        }
        drawn += 1;
    }
    drawn
}
/// Cells of `cells` inside the visible cell range, iterating whichever is smaller.
pub fn visible_cells(
    cells: &Cells,
    origin: [f32; 2],
    size: [f32; 2],
    visible: ([f32; 2], [f32; 2]),
) -> Vec<([i32; 2], &Cell)> {
    let lo = [0, 1].map(|k| ((visible.0[k] - origin[k]) / size[k]).floor() as i64 - 1);
    let hi = [0, 1].map(|k| ((visible.1[k] - origin[k]) / size[k]).ceil() as i64 + 1);
    let area = (hi[0] - lo[0] + 1).max(0) * (hi[1] - lo[1] + 1).max(0);
    if area as usize >= cells.len() {
        return cells.iter().map(|(p, c)| (*p, c)).collect();
    }
    let mut out = Vec::new();
    for y in lo[1]..=hi[1] {
        for x in lo[0]..=hi[0] {
            let p = [x as i32, y as i32];
            if let Some(c) = cells.get(&p) {
                out.push((p, c));
            }
        }
    }
    out
}

/// The grid, hover cell, brush preview and selection of the layer being painted.
pub fn overlay(
    ui: &Ui,
    e: &mut Editor,
    draw: &imgui::DrawListMut,
    origin: [f32; 2],
    size: [f32; 2],
) {
    let Some(layer) = selected_layer(e) else {
        return;
    };
    let view = e.view_2d;
    let to_screen = |p: [f32; 2]| view.world_to_screen(p, origin, size);
    let corner = [origin[0] + size[0], origin[1] + size[1]];
    let visible = (
        view.screen_to_world(origin, origin, size),
        view.screen_to_world(corner, origin, size),
    );
    let tile = layer.tile_size();
    // Cell grid, while it stays readable.
    if e.world2d.tiles.show_grid && tile[0] * view.zoom >= 6. && tile[1] * view.zoom >= 6. {
        let first = layer.cell_at(visible.0);
        let last = layer.cell_at(visible.1);
        for x in first[0]..=last[0] + 1 {
            let sx = to_screen(layer.cell_origin([x, 0]))[0];
            draw.add_line([sx, origin[1]], [sx, corner[1]], GRID)
                .build();
        }
        for y in first[1]..=last[1] + 1 {
            let sy = to_screen(layer.cell_origin([0, y]))[1];
            draw.add_line([origin[0], sy], [corner[0], sy], GRID)
                .build();
        }
    }
    let cell_rect = |p: [i32; 2]| {
        let lo = layer.cell_origin(p);
        (to_screen(lo), to_screen([lo[0] + tile[0], lo[1] + tile[1]]))
    };
    // Selection.
    for p in &e.world2d.tiles.selection {
        let (a, b) = cell_rect(*p);
        draw.add_rect(a, b, [0.35, 0.65, 1.0, 0.22])
            .filled(true)
            .build();
    }
    if let (Some(lo), Some(hi)) = (
        e.world2d
            .tiles
            .selection
            .iter()
            .copied()
            .reduce(|a, b| [a[0].min(b[0]), a[1].min(b[1])]),
        e.world2d
            .tiles
            .selection
            .iter()
            .copied()
            .reduce(|a, b| [a[0].max(b[0]), a[1].max(b[1])]),
    ) {
        let (a, _) = cell_rect(lo);
        let (_, b) = cell_rect(hi);
        draw.add_rect(a, b, [0.45, 0.75, 1.0, 0.9])
            .thickness(1.5)
            .build();
    }
    let Some(tool) = e.world2d.tiles.tool else {
        return;
    };
    let Some(hovered) = e.world2d.tiles.hovered else {
        return;
    };
    // What the gesture would change, as a ghost.
    let stroke = e.world2d.tiles.stroke.as_ref().map(|s| (s.start, s.erase));
    let erase = tool == Tool::Eraser || stroke.is_some_and(|s| s.1);
    let targets: Vec<([i32; 2], [i32; 2])> = match (tool, stroke) {
        (Tool::Line | Tool::Rect, Some((start, _))) => {
            let origin = [start[0].min(hovered[0]), start[1].min(hovered[1])];
            shape(tool, &layer, start, hovered, true)
                .into_iter()
                .map(|p| (origin, p))
                .collect()
        }
        (Tool::Paint, None) => match &e.world2d.tiles.brush {
            Brush::Tiles(stamp) if !stamp.is_empty() && !e.world2d.tiles.random => stamp
                .cells
                .iter()
                .map(|c| {
                    (
                        hovered,
                        [
                            hovered[0] + i32::from(c.offset[0]),
                            hovered[1] + i32::from(c.offset[1]),
                        ],
                    )
                })
                .collect(),
            _ => vec![(hovered, hovered)],
        },
        (Tool::Select, _) => Vec::new(),
        _ => vec![(hovered, hovered)],
    };
    let ghost: Cells = match (&e.world2d.tiles.brush, erase) {
        (Brush::Tiles(stamp), false) if tool.paints() && !e.world2d.tiles.random => targets
            .iter()
            .filter_map(|(anchor, p)| {
                stamp
                    .cell_at(layer.tileset.as_deref(), *anchor, *p, false)
                    .map(|c| (*p, c))
            })
            .collect(),
        _ => Cells::new(),
    };
    if let Some(tileset) = &layer.tileset
        && !ghost.is_empty()
    {
        let texture = tileset.texture.and_then(|t| e.world2d.textures.get(t));
        draw_cells(
            draw,
            texture,
            tileset,
            ghost.iter().map(|(p, c)| (*p, c)),
            layer.origin,
            &to_screen,
            visible,
            [1., 1., 1., 0.6],
            0.,
        );
    }
    let colour = match (&e.world2d.tiles.brush, erase) {
        (_, true) => ERASE,
        (
            Brush::Terrain {
                set,
                terrain: Some(t),
            },
            false,
        ) => layer
            .tileset
            .as_ref()
            .and_then(|s| s.terrain_sets.get(*set))
            .and_then(|s| s.terrains.get(usize::from(*t)))
            .map_or(ACCENT, |t| {
                let c = t.color.map(|v| f32::from(v) / 255.);
                [c[0], c[1], c[2], 1.]
            }),
        _ => ACCENT,
    };
    for (_, p) in &targets {
        let (a, b) = cell_rect(*p);
        if erase || matches!(e.world2d.tiles.brush, Brush::Terrain { .. }) {
            draw.add_rect(a, b, [colour[0], colour[1], colour[2], 0.25])
                .filled(true)
                .build();
        }
        draw.add_rect(a, b, colour).thickness(1.).build();
    }
    let _ = ui;
}

// ---- toolbar and palette ---------------------------------------------------------------

fn tool_button(ui: &Ui, glyph: &str, id: &str, tip: &str, detail: &str, active: bool) -> bool {
    let pressed = crate::gui::icon(ui, glyph, id, "", active);
    if ui.is_item_hovered() {
        ui.tooltip(|| {
            ui.text(tip);
            if !detail.is_empty() {
                ui.text_disabled(detail);
            }
        });
    }
    pressed
}
/// The tile tools row, shown under the 2D toolbar while a TileMapLayer is selected.
pub fn toolbar(ui: &Ui, e: &mut Editor) {
    let _spacing = ui.push_style_var(imgui::StyleVar::ItemSpacing([2., 4.]));
    let draw = ui.get_window_draw_list();
    let start = ui.cursor_screen_pos();
    let width = ui.content_region_avail()[0];
    draw.add_rect(
        [start[0] - 4., start[1] - 2.],
        [start[0] + width + 4., start[1] + 25.],
        [0.98, 0.62, 0.20, 0.07],
    )
    .filled(true)
    .rounding(4.)
    .build();
    ui.align_text_to_frame_padding();
    ui.text_colored(ACCENT, format!("{ICON_TILEMAP} Tiles"));
    ui.same_line_with_spacing(0., 8.);
    let current = e.world2d.tiles.tool;
    for (tool, glyph, tip, detail) in Tool::ALL {
        if tool_button(
            ui,
            glyph,
            &format!("tile-{tip}"),
            tip,
            detail,
            current == Some(tool),
        ) {
            e.world2d.tiles.tool = Some(tool);
        }
        ui.same_line();
    }
    ui.same_line_with_spacing(0., 10.);
    let state = &mut e.world2d.tiles;
    if tool_button(
        ui,
        ICON_RANDOM,
        "tile-random",
        "Random tile (G)",
        "Paint a random tile of the brush per cell, weighted by tile probability",
        state.random,
    ) {
        state.random = !state.random;
    }
    ui.same_line();
    let mut transform = None;
    for (glyph, id, tip, kind) in [
        (ICON_ROTATE_LEFT, "rotate-left", "Rotate left (Z)", 0),
        (ICON_ROTATE_RIGHT, "rotate-right", "Rotate right (X)", 1),
        (
            crate::world2d_editor::ICON_FLIP_H,
            "flip-h",
            "Flip horizontally (C)",
            2,
        ),
        (
            crate::world2d_editor::ICON_FLIP_V,
            "flip-v",
            "Flip vertically (V)",
            3,
        ),
    ] {
        if tool_button(ui, glyph, id, tip, "Transforms the brush", false) {
            transform = Some(kind);
        }
        ui.same_line();
    }
    if let (Some(kind), Brush::Tiles(stamp)) = (transform, &mut state.brush) {
        match kind {
            0 => stamp.rotate_counterclockwise(),
            1 => stamp.rotate_clockwise(),
            2 => stamp.flip_h(),
            _ => stamp.flip_v(),
        }
    }
    ui.same_line_with_spacing(0., 10.);
    if tool_button(
        ui,
        crate::world2d_editor::ICON_GRID,
        "tile-grid",
        "Show cell grid",
        "",
        state.show_grid,
    ) {
        state.show_grid = !state.show_grid;
    }
    if current == Some(Tool::Bucket) {
        ui.same_line();
        ui.checkbox("Contiguous", &mut state.contiguous);
        if ui.is_item_hovered() {
            ui.tooltip_text("Off: replace every cell of the clicked tile in the layer");
        }
    }
    ui.same_line_with_spacing(0., 10.);
    if current.is_some() {
        if ui.small_button("Done") {
            state.tool = None;
        }
        if ui.is_item_hovered() {
            ui.tooltip_text("Stop painting and move nodes again (Q or Esc)");
        }
    } else if ui.small_button(format!("{ICON_PAINT} Paint tiles")) {
        state.tool = Some(Tool::Paint);
    }
    ui.dummy([0., 0.]);
}

/// The palette panel under the 2D view: tiles, patterns and terrains of the layer's TileSet.
pub fn palette(ui: &Ui, e: &mut Editor, height: f32) {
    let Some(layer) = selected_layer(e) else {
        return;
    };
    let width = ui.content_region_avail()[0];
    ui.child_window("##tile-palette")
        .size([width, height])
        .border(true)
        .build(|| {
            let Some(tileset) = layer.tileset.clone() else {
                no_tileset(ui, e, &layer);
                return;
            };
            // Header: tabs, the TileSet, its editor.
            let tabs = [
                (Tab::Tiles, format!("{ICON_TILESET} Tiles")),
                (
                    Tab::Patterns,
                    format!("{ICON_PATTERN} Patterns ({})", tileset.patterns.len()),
                ),
                (Tab::Terrains, format!("{ICON_TERRAIN} Terrains")),
            ];
            for (tab, label) in tabs {
                let active = e.world2d.tiles.tab == tab;
                let _colour = ui.push_style_color(
                    imgui::StyleColor::Button,
                    if active {
                        [0.19, 0.37, 0.52, 1.]
                    } else {
                        [0.19, 0.19, 0.21, 1.]
                    },
                );
                if ui.button(&label) {
                    e.world2d.tiles.tab = tab;
                }
                ui.same_line();
            }
            ui.same_line_with_spacing(0., 12.);
            let name = layer
                .record
                .tile_set
                .and_then(|id| e.assets.index.resolve(id).ok())
                .map(|r| asset_name(&r.meta.source))
                .unwrap_or_else(|| "TileSet".into());
            ui.text_disabled(format!(
                "{name}  ·  {}x{} px  ·  {} tiles",
                tileset.tile_size[0],
                tileset.tile_size[1],
                tileset.tiles.len()
            ));
            ui.same_line();
            if ui.small_button(format!("{ICON_EDIT} Edit TileSet"))
                && let Some(id) = layer.record.tile_set
            {
                crate::tileset_editor::open(e, id);
            }
            if e.world2d.tiles.tab == Tab::Tiles {
                ui.same_line();
                ui.set_next_item_width(90.);
                crate::gui::Drag::new("##palette-zoom")
                    .range(0.5, 8.)
                    .speed(0.02)
                    .display_format("Zoom %.1fx")
                    .build(ui, &mut e.world2d.tiles.palette_zoom);
            }
            ui.separator();
            match e.world2d.tiles.tab {
                Tab::Tiles => tiles_tab(ui, e, &tileset),
                Tab::Patterns => patterns_tab(ui, e, &layer, &tileset),
                Tab::Terrains => terrains_tab(ui, e, &layer, &tileset),
            }
        });
}
fn asset_name(source: &str) -> String {
    std::path::Path::new(source)
        .file_stem()
        .map(|s| {
            s.to_string_lossy()
                .split('.')
                .next()
                .unwrap_or("")
                .to_owned()
        })
        .unwrap_or_default()
}
fn no_tileset(ui: &Ui, e: &mut Editor, layer: &Layer) {
    ui.text(format!("{ICON_TILESET}  This layer has no TileSet yet"));
    ui.text_disabled(
        "A TileSet cuts a texture into tiles and stores their collision, terrains and animation.",
    );
    ui.spacing();
    if ui.button(format!(
        "{} New TileSet from a texture...",
        crate::world2d_editor::ICON_PLUS
    )) {
        e.world2d.tiles.new_tileset = Some(NewTileSet {
            actor: layer.actor,
            component: layer.component,
            texture: None,
            tile: [16, 16],
        });
    }
    ui.same_line();
    pick_tileset(ui, e, layer.actor, layer.component, None);
}
/// A combo of the project's TileSets assigning the chosen one to a layer.
pub fn pick_tileset(ui: &Ui, e: &mut Editor, actor: Uuid, component: Uuid, current: Option<Uuid>) {
    let mut sets: Vec<_> = e
        .assets
        .index
        .usable()
        .filter(|r| r.meta.kind == crate::assets::Kind::TileSet)
        .map(|r| (r.meta.id, asset_name(&r.meta.source)))
        .collect();
    sets.sort_by(|a, b| a.1.cmp(&b.1));
    let label = current
        .and_then(|id| sets.iter().find(|s| s.0 == id))
        .map_or_else(|| "Use an existing TileSet".to_string(), |s| s.1.clone());
    ui.set_next_item_width(220.);
    if let Some(_combo) = ui.begin_combo("##pick-tileset", label) {
        if sets.is_empty() {
            ui.text_disabled("No TileSet in this project yet");
        }
        for (id, name) in sets {
            if ui
                .selectable_config(&name)
                .selected(current == Some(id))
                .build()
            {
                edit_record(e, actor, component, |r| r.tile_set = Some(id));
                let _ = crate::texture::resolve(&mut e.scene, &e.assets.index);
                e.changed();
            }
        }
    }
}
fn tiles_tab(ui: &Ui, e: &mut Editor, tileset: &TileSet) {
    let Some(texture_id) = tileset.texture else {
        ui.text_disabled("The TileSet has no texture. Open the TileSet editor to choose one.");
        return;
    };
    let Some((texture, texture_size)) = e.world2d.textures.get(texture_id) else {
        ui.text_disabled("Loading the atlas preview...");
        return;
    };
    let zoom = e.world2d.tiles.palette_zoom;
    let available = ui.content_region_avail();
    let side = 230_f32.min(available[0] * 0.35);
    ui.child_window("##palette-atlas")
        .size([available[0] - side - 8., available[1]])
        .horizontal_scrollbar(true)
        .build(|| {
            let canvas = [
                f32::from(texture_size[0]) * zoom,
                f32::from(texture_size[1]) * zoom,
            ];
            let start = ui.cursor_screen_pos();
            ui.invisible_button("##atlas", canvas);
            let hovered = ui.is_item_hovered();
            let draw = ui.get_window_draw_list();
            crate::world2d_editor::checker(
                &draw,
                start,
                [start[0] + canvas[0], start[1] + canvas[1]],
                6.,
            );
            draw.add_image(texture, start, [start[0] + canvas[0], start[1] + canvas[1]])
                .build();
            let grid = tileset.grid(texture_size);
            let size = tileset.tile_size.map(|v| f32::from(v) * zoom);
            let cell_rect = |atlas: [u8; 2]| {
                let t = tileset.texel([u16::from(atlas[0]), u16::from(atlas[1])]);
                let a = [start[0] + t[0] as f32 * zoom, start[1] + t[1] as f32 * zoom];
                (a, [a[0] + size[0], a[1] + size[1]])
            };
            for row in 0..grid[1] {
                for column in 0..grid[0] {
                    let atlas = [column as u8, row as u8];
                    let (a, b) = cell_rect(atlas);
                    if tileset.tile(atlas).is_some() {
                        draw.add_rect(a, b, [1., 1., 1., 0.18]).build();
                    } else {
                        draw.add_rect(a, b, [0., 0., 0., 0.55]).filled(true).build();
                    }
                }
            }
            for tile in &tileset.tiles {
                let (a, _) = cell_rect(tile.atlas);
                if tile.frames > 1 {
                    draw.add_text(
                        [a[0] + 2., a[1] + 1.],
                        [1., 1., 1., 0.85],
                        crate::world2d_editor::ICON_FILM,
                    );
                }
            }
            let mouse = ui.io().mouse_pos;
            let texel = [(mouse[0] - start[0]) / zoom, (mouse[1] - start[1]) / zoom];
            let under = hovered
                .then(|| tileset.atlas_at(texel, texture_size))
                .flatten();
            if let Some(atlas) = under {
                let (a, b) = cell_rect(atlas);
                draw.add_rect(a, b, [1., 1., 1., 0.8])
                    .thickness(1.5)
                    .build();
                if let Some(tile) = tileset.tile(atlas) {
                    ui.tooltip(|| {
                        ui.text(format!("Tile ({}, {})", atlas[0], atlas[1]));
                        ui.text_disabled(describe(tileset, tile));
                    });
                }
            }
            let state = &mut e.world2d.tiles;
            if let Some(atlas) = under
                && ui.is_mouse_clicked(MouseButton::Left)
            {
                state.palette_anchor = Some(atlas);
                state.palette_drag = Some((atlas, atlas));
            }
            if let (Some(anchor), true) =
                (state.palette_anchor, ui.is_mouse_down(MouseButton::Left))
            {
                if let Some(atlas) = tileset.atlas_at(texel, texture_size) {
                    state.palette_drag = Some((anchor, atlas));
                }
            } else if let Some((a, b)) = state.palette_drag.take() {
                state.palette_anchor = None;
                let lo = [a[0].min(b[0]), a[1].min(b[1])];
                let hi = [a[0].max(b[0]), a[1].max(b[1])];
                let mut cells = Vec::new();
                for row in lo[1]..=hi[1] {
                    for column in lo[0]..=hi[0] {
                        if tileset.tile([column, row]).is_some() {
                            cells.push(PatternCell {
                                offset: [column - lo[0], row - lo[1]],
                                atlas: [column, row],
                                flags: 0,
                            });
                        }
                    }
                }
                if !cells.is_empty() {
                    state.brush = Brush::Tiles(Stamp { cells });
                    if !state.tool.is_some_and(Tool::paints) {
                        state.tool = Some(Tool::Paint);
                    }
                }
            }
            // The brush's tiles, outlined.
            let selected: Vec<[u8; 2]> = match (&state.brush, state.palette_drag) {
                (_, Some((a, b))) => {
                    let lo = [a[0].min(b[0]), a[1].min(b[1])];
                    let hi = [a[0].max(b[0]), a[1].max(b[1])];
                    (lo[1]..=hi[1])
                        .flat_map(|r| (lo[0]..=hi[0]).map(move |c| [c, r]))
                        .collect()
                }
                (Brush::Tiles(stamp), None) => stamp.cells.iter().map(|c| c.atlas).collect(),
                _ => Vec::new(),
            };
            for atlas in selected {
                let (a, b) = cell_rect(atlas);
                draw.add_rect(a, b, [ACCENT[0], ACCENT[1], ACCENT[2], 0.25])
                    .filled(true)
                    .build();
                draw.add_rect(a, b, ACCENT).thickness(2.).build();
            }
            if hovered && ui.io().mouse_wheel != 0. && ui.io().key_ctrl {
                state.palette_zoom =
                    (state.palette_zoom * 1.2_f32.powf(ui.io().mouse_wheel)).clamp(0.5, 8.);
            }
        });
    ui.same_line();
    ui.child_window("##palette-help")
        .size([side, available[1]])
        .build(|| {
            brush_summary(ui, e, tileset);
        });
}
fn describe(tileset: &TileSet, tile: &tileset::Tile) -> String {
    let mut parts = Vec::new();
    match tile.collision {
        tileset::Collision::None => {}
        tileset::Collision::Full => parts.push("solid".to_string()),
        tileset::Collision::Rect => parts.push("solid (shape)".to_string()),
    }
    if tile.one_way {
        parts.push("one-way".into());
    }
    if tile.frames > 1 {
        parts.push(format!("{} frames at {:.0} fps", tile.frames, tile.fps));
    }
    if let (Some(set), Some(t)) = (tile.terrain_set, tile.terrain)
        && let Some(terrain) = tileset
            .terrain_sets
            .get(usize::from(set))
            .and_then(|s| s.terrains.get(usize::from(t)))
    {
        parts.push(format!("terrain {}", terrain.name));
    }
    for (layer, value) in tileset.custom_layers.iter().zip(&tile.custom) {
        if *value != 0 {
            parts.push(format!("{} = {value}", layer.name));
        }
    }
    if parts.is_empty() {
        "decoration (no collision)".into()
    } else {
        parts.join(", ")
    }
}
fn brush_summary(ui: &Ui, e: &mut Editor, tileset: &TileSet) {
    let state = &e.world2d.tiles;
    ui.text("Brush");
    match &state.brush {
        Brush::Tiles(stamp) if stamp.is_empty() => crate::gui::muted(
            ui,
            "Click a tile on the left, or drag across several to paint them as a block.",
        ),
        Brush::Tiles(stamp) => {
            let size = stamp.size();
            crate::gui::muted(
                ui,
                if stamp.cells.len() == 1 {
                    let c = stamp.cells[0];
                    format!(
                        "Tile ({}, {}){}",
                        c.atlas[0],
                        c.atlas[1],
                        flags_text(c.flags)
                    )
                } else {
                    format!("{} tiles, {}x{} cells", stamp.cells.len(), size[0], size[1])
                },
            );
        }
        Brush::Terrain { set, terrain } => {
            let name = terrain
                .and_then(|t| {
                    tileset
                        .terrain_sets
                        .get(*set)
                        .and_then(|s| s.terrains.get(usize::from(t)))
                })
                .map_or("Erase terrain".to_string(), |t| t.name.clone());
            crate::gui::muted(ui, format!("{ICON_TERRAIN} {name}"));
        }
    }
    ui.spacing();
    if let Some(tool) = state.tool {
        ui.text(tool.label());
        let detail = Tool::ALL.iter().find(|t| t.0 == tool).map_or("", |t| t.3);
        crate::gui::muted(ui, detail);
    } else {
        crate::gui::muted(ui, "Painting is off. Press D or Paint tiles.");
    }
    if state.random {
        ui.text_colored(ACCENT, format!("{ICON_RANDOM} Random scatter on"));
    }
}
fn flags_text(flags: u8) -> String {
    let mut out = String::new();
    if flags & tileset::TRANSPOSE != 0 {
        out += ", transposed";
    }
    if flags & tileset::FLIP_H != 0 {
        out += ", flipped H";
    }
    if flags & tileset::FLIP_V != 0 {
        out += ", flipped V";
    }
    out
}
fn patterns_tab(ui: &Ui, e: &mut Editor, layer: &Layer, tileset: &TileSet) {
    let Some(id) = layer.record.tile_set else {
        return;
    };
    let texture = tileset.texture.and_then(|t| e.world2d.textures.get(t));
    let can_save = !e.world2d.tiles.selection.is_empty();
    ui.disabled(!can_save, || {
        ui.set_next_item_width(160.);
        ui.input_text("##pattern-name", &mut e.world2d.tiles.pattern_name)
            .hint("Pattern name")
            .build();
        ui.same_line();
        if ui.button(format!(
            "{} Save selection as pattern",
            crate::world2d_editor::ICON_PLUS
        )) {
            let name = if e.world2d.tiles.pattern_name.trim().is_empty() {
                format!("Pattern {}", tileset.patterns.len() + 1)
            } else {
                e.world2d.tiles.pattern_name.trim().to_owned()
            };
            let pattern = tilemap::pattern_from(&layer.cells, &e.world2d.tiles.selection, &name);
            if !pattern.cells.is_empty() {
                crate::tileset_editor::modify(e, id, |set| set.patterns.push(pattern));
                e.world2d.tiles.pattern_name.clear();
            }
        }
    });
    if !can_save {
        ui.same_line();
        ui.text_disabled("Select cells with the Select tool (S) to save them as a pattern.");
    }
    ui.separator();
    if tileset.patterns.is_empty() {
        ui.text_disabled(
            "No patterns yet. Patterns are reusable blocks of tiles: trees, houses, platforms.",
        );
        return;
    }
    let card = 112.;
    let per_row = ((ui.content_region_avail()[0] / (card + 8.)).floor() as usize).max(1);
    let mut remove = None;
    for (index, pattern) in tileset.patterns.iter().enumerate() {
        if index % per_row != 0 {
            ui.same_line();
        }
        let _id = ui.push_id_usize(index);
        let start = ui.cursor_screen_pos();
        let selected =
            matches!(&e.world2d.tiles.brush, Brush::Tiles(s) if s.cells == pattern.cells);
        if ui.invisible_button("##pattern", [card, card]) {
            e.world2d.tiles.brush = Brush::Tiles(Stamp::from_pattern(pattern));
            if !e.world2d.tiles.tool.is_some_and(Tool::paints) {
                e.world2d.tiles.tool = Some(Tool::Paint);
            }
        }
        if ui.is_item_hovered() {
            ui.tooltip_text(format!(
                "{} ({} tiles). Right-click to rename or delete.",
                pattern.name,
                pattern.cells.len()
            ));
        }
        if ui.is_item_clicked_with_button(MouseButton::Right) {
            ui.open_popup("pattern-menu");
        }
        ui.popup("pattern-menu", || {
            if ui.menu_item("Rename") {
                e.world2d.tiles.renaming = Some(index);
                e.world2d.tiles.pattern_name = pattern.name.clone();
            }
            if ui.menu_item("Delete") {
                remove = Some(index);
            }
        });
        let draw = ui.get_window_draw_list();
        draw.add_rect(
            start,
            [start[0] + card, start[1] + card],
            [0.16, 0.17, 0.2, 1.],
        )
        .filled(true)
        .rounding(4.)
        .build();
        if selected {
            draw.add_rect(start, [start[0] + card, start[1] + card], ACCENT)
                .thickness(2.)
                .rounding(4.)
                .build();
        }
        let size = pattern.size();
        let tile = tileset.tile_size.map(f32::from);
        let extent = [
            f32::from(size[0].max(1)) * tile[0],
            f32::from(size[1].max(1)) * tile[1],
        ];
        let scale = ((card - 30.) / extent[0].max(extent[1])).min(4.);
        let origin = [
            start[0] + (card - extent[0] * scale) * 0.5,
            start[1] + 4. + (card - 26. - extent[1] * scale) * 0.5,
        ];
        let cells: Cells = pattern
            .cells
            .iter()
            .map(|c| {
                (
                    [i32::from(c.offset[0]), i32::from(c.offset[1])],
                    Cell {
                        atlas: c.atlas,
                        flags: c.flags,
                    },
                )
            })
            .collect();
        let to_screen = |p: [f32; 2]| [origin[0] + p[0] * scale, origin[1] + p[1] * scale];
        draw_cells(
            &draw,
            texture,
            tileset,
            cells.iter().map(|(p, c)| (*p, c)),
            [0., 0.],
            &to_screen,
            ([f32::MIN; 2], [f32::MAX; 2]),
            [1., 1., 1., 1.],
            0.,
        );
        let label = &pattern.name;
        let label_size = ui.calc_text_size(label);
        draw.add_text(
            [
                start[0] + (card - label_size[0]).max(4.) * 0.5,
                start[1] + card - 20.,
            ],
            [0.85, 0.86, 0.9, 1.],
            label,
        );
    }
    if let Some(index) = e.world2d.tiles.renaming {
        ui.open_popup("rename-pattern");
        if let Some(_popup) = ui.begin_popup("rename-pattern") {
            ui.input_text("Name", &mut e.world2d.tiles.pattern_name)
                .build();
            if ui.button("Rename") {
                let name = e.world2d.tiles.pattern_name.trim().to_owned();
                crate::tileset_editor::modify(e, id, |set| {
                    if let Some(p) = set.patterns.get_mut(index) {
                        p.name = name;
                    }
                });
                e.world2d.tiles.renaming = None;
                e.world2d.tiles.pattern_name.clear();
                ui.close_current_popup();
            }
            ui.same_line();
            if ui.button("Cancel") {
                e.world2d.tiles.renaming = None;
                ui.close_current_popup();
            }
        }
    }
    if let Some(index) = remove {
        crate::tileset_editor::modify(e, id, |set| {
            if index < set.patterns.len() {
                set.patterns.remove(index);
            }
        });
    }
}
fn terrains_tab(ui: &Ui, e: &mut Editor, layer: &Layer, tileset: &TileSet) {
    if tileset.terrain_sets.is_empty() {
        ui.text_disabled("This TileSet has no terrains. Terrains pick the right edge and corner tiles for you as you paint.");
        if ui.button(format!(
            "{ICON_TERRAIN} Set up terrains in the TileSet editor"
        )) && let Some(id) = layer.record.tile_set
        {
            crate::tileset_editor::open(e, id);
            if let Some(window) = &mut e.world2d.tileset_window {
                window.mode = crate::tileset_editor::Mode::Terrains;
            }
        }
        return;
    }
    ui.text_disabled("Pick a terrain, then paint, draw lines or rectangles, or fill: edges and corners connect automatically.");
    for (set_index, set) in tileset.terrain_sets.iter().enumerate() {
        ui.text(format!("{}  ·  {}", set.name, set.mode.label()));
        let _id = ui.push_id_usize(set_index);
        for (index, terrain) in set.terrains.iter().enumerate() {
            let selected = e.world2d.tiles.brush
                == Brush::Terrain {
                    set: set_index,
                    terrain: Some(index as u8),
                };
            let usable = tileset
                .tiles
                .iter()
                .filter(|t| {
                    t.terrain_set == Some(set_index as u8) && t.terrain == Some(index as u8)
                })
                .count();
            let start = ui.cursor_screen_pos();
            if ui
                .selectable_config(format!(
                    "      {}  ({usable} tiles)##terrain{index}",
                    terrain.name
                ))
                .selected(selected)
                .size([280., 0.])
                .build()
            {
                e.world2d.tiles.brush = Brush::Terrain {
                    set: set_index,
                    terrain: Some(index as u8),
                };
                if !e.world2d.tiles.tool.is_some_and(Tool::paints) {
                    e.world2d.tiles.tool = Some(Tool::Paint);
                }
            }
            let c = terrain.color.map(|v| f32::from(v) / 255.);
            ui.get_window_draw_list()
                .add_rect(
                    [start[0] + 4., start[1] + 2.],
                    [start[0] + 18., start[1] + 14.],
                    [c[0], c[1], c[2], 1.],
                )
                .filled(true)
                .rounding(2.)
                .build();
        }
        let erasing = e.world2d.tiles.brush
            == Brush::Terrain {
                set: set_index,
                terrain: None,
            };
        if ui
            .selectable_config(format!("{ICON_ERASER}  Erase terrain##erase{set_index}"))
            .selected(erasing)
            .size([280., 0.])
            .build()
        {
            e.world2d.tiles.brush = Brush::Terrain {
                set: set_index,
                terrain: None,
            };
        }
        ui.spacing();
    }
}

/// The "New TileSet" dialog: pick a texture and a tile size; tiles are created for every
/// cell that has visible texels and the TileSet editor opens on it.
pub fn dialogs(ui: &Ui, e: &mut Editor) {
    let Some(mut form) = e.world2d.tiles.new_tileset.clone() else {
        return;
    };
    let mut open = true;
    let mut create = false;
    ui.window("New TileSet")
        .opened(&mut open)
        .size([420., 0.], imgui::Condition::Appearing)
        .collapsible(false)
        .build(|| {
            ui.text_disabled("Choose the texture that holds your tiles, laid out on a grid.");
            let mut textures: Vec<_> = e
                .assets
                .index
                .usable()
                .filter(|r| r.meta.kind == crate::assets::Kind::Texture)
                .map(|r| (r.meta.id, r.meta.source.clone()))
                .collect();
            textures.sort_by(|a, b| a.1.cmp(&b.1));
            let label = form
                .texture
                .and_then(|id| textures.iter().find(|t| t.0 == id))
                .map_or_else(|| "Choose a texture".into(), |t| t.1.clone());
            let id = crate::gui::field(ui, "Texture");
            if let Some(_combo) = ui.begin_combo(id, label) {
                if textures.is_empty() {
                    ui.text_disabled("Import a PNG first (drag it into the Project panel).");
                }
                for (texture, name) in &textures {
                    if ui.selectable_config(name).selected(form.texture == Some(*texture)).build() {
                        form.texture = Some(*texture);
                    }
                }
            }
            let mut size = form.tile.map(i32::from);
            if crate::gui::Drag::new(crate::gui::field(ui, "Tile size (px)"))
                .range(1, 256)
                .speed(0.2)
                .build_array(ui, &mut size)
            {
                form.tile = size.map(|v| v.clamp(1, 256) as u16);
            }
            let _ = crate::gui::field(ui, "Presets");
            for preset in [8u16, 16, 24, 32] {
                if ui.small_button(format!("{preset}x{preset}")) {
                    form.tile = [preset, preset];
                }
                ui.same_line();
            }
            ui.new_line();
            ui.text_disabled("16x16 tiles draw as the console's cheapest sprites. Tiles are created for every cell with visible pixels.");
            ui.spacing();
            ui.disabled(form.texture.is_none(), || {
                if ui.button_with_size("Create TileSet", [160., 0.]) {
                    create = true;
                }
            });
        });
    if create {
        match create_tileset(e, &form) {
            Ok(_) => open = false,
            Err(error) => {
                e.last_error = Some(error.clone());
                e.log(error);
            }
        }
    }
    e.world2d.tiles.new_tileset = open.then_some(form);
}
/// Creates a TileSet asset from a texture, assigns it to the form's layer and opens it.
pub fn create_tileset(e: &mut Editor, form: &NewTileSet) -> Result<Uuid, String> {
    let texture = form.texture.ok_or("Choose a texture")?;
    let record = e.assets.index.resolve(texture)?.clone();
    let package = crate::assets::Package::load(&record.path)?;
    let data = crate::texture::decode(&package.source)?;
    let mut set = TileSet {
        texture: Some(texture),
        tile_size: form.tile,
        ..Default::default()
    };
    set.create_opaque_tiles(&data);
    let path = tileset::fresh_path(&asset_name(&record.meta.source));
    let id = tileset::create(&e.root, &path, &set)?;
    e.assets.index = crate::assets::scan(&e.root, &mut Default::default());
    e.assets.refresh();
    e.scene.tilesets.insert(id, Arc::new(set.clone()));
    edit_record(e, form.actor, form.component, |r| r.tile_set = Some(id));
    let _ = crate::texture::resolve(&mut e.scene, &e.assets.index);
    e.changed();
    e.log(format!(
        "Created TileSet {path} with {} tiles.",
        set.tiles.len()
    ));
    e.world2d.tiles.tool = Some(Tool::Paint);
    crate::tileset_editor::open(e, id);
    Ok(id)
}

// ---- inspector -------------------------------------------------------------------------

/// The TileMapLayer band of the 2D Inspector.
pub fn band(
    ui: &Ui,
    e: &mut Editor,
    actor: Uuid,
    component: &crate::actor_document::ComponentInstance,
) -> bool {
    let mut layer: TileMapLayer = world2d::read(component);
    let before = layer.clone();
    let cells = cells_of(&mut e.world2d.tiles, component.id, &layer.cells);
    let _ = crate::gui::field(ui, "TileSet");
    pick_tileset(ui, e, actor, component.id, layer.tile_set);
    if ui.small_button(format!("{} New...", crate::world2d_editor::ICON_PLUS)) {
        e.world2d.tiles.new_tileset = Some(NewTileSet {
            actor,
            component: component.id,
            texture: None,
            tile: [16, 16],
        });
    }
    if let Some(id) = layer.tile_set {
        ui.same_line();
        if ui.small_button(format!("{ICON_EDIT} Edit...")) {
            crate::tileset_editor::open(e, id);
        }
    }
    let painted = cells.len();
    let extent = tilemap::bounds(&cells).map_or_else(
        || "empty".to_string(),
        |(lo, hi)| format!("{} x {} cells", hi[0] - lo[0] + 1, hi[1] - lo[1] + 1),
    );
    ui.text_disabled(format!("{painted} tiles painted, {extent}"));
    if ui.button(format!("{ICON_PAINT}  Paint Tiles")) {
        e.world2d.tiles.tool = Some(Tool::Paint);
        e.set_scene_view_mode(crate::scene_view_mode::SceneViewMode::TwoD);
    }
    if ui.is_item_hovered() {
        ui.tooltip_text("Opens the tile tools and the palette under the 2D view");
    }
    let mut z = i32::from(layer.z_index);
    if crate::gui::Drag::new(crate::gui::field(ui, "Z Index"))
        .range(-4096, 4096)
        .speed(0.2)
        .build(ui, &mut z)
    {
        layer.z_index = z.clamp(-4096, 4096) as i16;
    }
    let mut colour = layer.modulate.map(|c| f32::from(c) / 255.);
    if ui.color_edit3(crate::gui::field(ui, "Modulate"), &mut colour) {
        layer.modulate = colour.map(|c| (c * 255.).round().clamp(0., 255.) as u8);
    }
    let modes = [
        crate::texture::BlendMode::Cutout,
        crate::texture::BlendMode::Average,
        crate::texture::BlendMode::Add,
        crate::texture::BlendMode::Subtract,
        crate::texture::BlendMode::AddQuarter,
    ];
    let mut mode = modes.iter().position(|m| *m == layer.blend).unwrap_or(0);
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
        layer.blend = modes[mode];
    }
    crate::gui::toggle(ui, "Visible", &mut layer.visible);
    crate::gui::toggle(ui, "Collision", &mut layer.collision_enabled);
    if ui.is_item_hovered() {
        ui.tooltip_text("Solid and one-way tiles block CharacterBody2D, rays and overlap queries");
    }
    if layer.collision_enabled {
        let mut value = layer.collision_layer;
        if crate::world2d_editor::layer_grid(ui, "Collision layer", &mut value) {
            layer.collision_layer = value;
        }
    }
    ui.text_disabled("Tile layers use their node's position; rotation and scale do not apply.");
    layer != before && {
        let tileset_changed = layer.tile_set != before.tile_set;
        edit_record(e, actor, component.id, |r| *r = layer);
        if tileset_changed {
            let _ = crate::texture::resolve(&mut e.scene, &e.assets.index);
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layer_with(cells: Cells, tileset: TileSet) -> Layer {
        Layer {
            actor: Uuid::nil(),
            component: Uuid::nil(),
            record: TileMapLayer::default(),
            cells: Arc::new(cells),
            tileset: Some(Arc::new(tileset)),
            origin: [-8., 4.],
        }
    }

    #[test]
    fn cells_map_to_world_through_the_layer_origin() {
        let layer = layer_with(Cells::new(), TileSet::default());
        assert_eq!(layer.cell_at([-8., 4.]), [0, 0]);
        assert_eq!(layer.cell_at([-9., 4.]), [-1, 0]);
        assert_eq!(layer.cell_at([8., 19.9]), [1, 0]);
        assert_eq!(layer.cell_origin([2, -1]), [24., -12.]);
    }

    #[test]
    fn painting_applies_stamps_patterns_random_and_erasing() {
        let mut set = TileSet::default();
        for x in 0..2 {
            set.tiles.push(tileset::Tile::new([x, 0]));
        }
        let mut cells = Cells::new();
        let stamp = Stamp {
            cells: vec![
                PatternCell {
                    offset: [0, 0],
                    atlas: [0, 0],
                    flags: 0,
                },
                PatternCell {
                    offset: [1, 0],
                    atlas: [1, 0],
                    flags: 0,
                },
            ],
        };
        // A rectangle repeats the 2 x 1 stamp from its top-left corner.
        let targets = tilemap::rect([0, 0], [3, 1]);
        apply(
            &Brush::Tiles(stamp.clone()),
            false,
            Some(&set),
            &mut cells,
            &targets,
            [0, 0],
            false,
        );
        assert_eq!(cells.len(), 8);
        assert_eq!(cells[&[2, 1]].atlas, [0, 0]);
        assert_eq!(cells[&[3, 1]].atlas, [1, 0]);
        // A paint drag lays a wide stamp on its own grid, never overlapping itself.
        let laid = stamp_targets(&stamp, [0, 5], [0, 5], [3, 5]);
        assert_eq!(laid.len(), 4);
        assert_eq!(laid[2], ([2, 5], [2, 5]));
        // Random scatter only uses the brush's tiles.
        let mut scattered = Cells::new();
        let row: Vec<_> = (0..40).map(|x| [x, 9]).collect();
        apply(
            &Brush::Tiles(stamp.clone()),
            true,
            Some(&set),
            &mut scattered,
            &row,
            [0, 9],
            false,
        );
        let kinds: BTreeSet<_> = scattered.values().map(|c| c.atlas).collect();
        assert_eq!(kinds.len(), 2);
        // Erasing removes cells whatever the brush.
        apply(
            &Brush::Tiles(stamp),
            false,
            Some(&set),
            &mut cells,
            &[[0, 0], [9, 9]],
            [0, 0],
            true,
        );
        assert!(!cells.contains_key(&[0, 0]) && cells.len() == 7);
    }

    #[test]
    fn terrain_painting_connects_and_erasing_updates_neighbours() {
        let mut set = TileSet {
            terrain_sets: vec![tileset::TerrainSet {
                mode: tileset::TerrainMode::Sides,
                ..Default::default()
            }],
            ..Default::default()
        };
        // Sixteen tiles: one per combination of connected sides.
        for mask in 0u8..16 {
            let mut tile = tileset::Tile::new([mask, 0]);
            tile.terrain_set = Some(0);
            tile.terrain = Some(0);
            for side in 0..4 {
                if mask & (1 << side) != 0 {
                    tile.peering[side * 2] = Some(0);
                }
            }
            set.tiles.push(tile);
        }
        let brush = Brush::Terrain {
            set: 0,
            terrain: Some(0),
        };
        let mut cells = Cells::new();
        let line = tilemap::line([0, 0], [2, 0]);
        apply(&brush, false, Some(&set), &mut cells, &line, [0, 0], false);
        // Ends connect inwards, the middle both ways.
        assert_eq!(cells[&[0, 0]].atlas, [1, 0]);
        assert_eq!(cells[&[1, 0]].atlas, [1 | 4, 0]);
        assert_eq!(cells[&[2, 0]].atlas, [4, 0]);
        // Erasing the right end turns the middle into an end.
        apply(
            &brush,
            false,
            Some(&set),
            &mut cells,
            &[[2, 0]],
            [2, 0],
            true,
        );
        assert!(!cells.contains_key(&[2, 0]));
        assert_eq!(cells[&[1, 0]].atlas, [4, 0]);
        // Painting below the middle connects it downwards.
        apply(
            &brush,
            false,
            Some(&set),
            &mut cells,
            &[[1, 1]],
            [1, 1],
            false,
        );
        assert_eq!(cells[&[1, 0]].atlas, [4 | 2, 0]);
        assert_eq!(cells[&[1, 1]].atlas, [8, 0]);
    }
}
