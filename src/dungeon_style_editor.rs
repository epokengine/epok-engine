//! The Dungeon Style editor: a canvas of spot nodes and the pieces they become, a
//! details panel for the selected node, and live regeneration of the dungeons in the
//! open map that use the style.
//!
//! Pieces of one spot are tried from left to right, so moving a piece node changes its
//! priority; the canvas numbers each link with that order. Every finished edit is saved,
//! like the TileSet editor, and has its own undo history.
use crate::dungeon::SpotKind;
use crate::dungeon_style::{self, Filter, Node, NodeKind, Piece, Style, Visual};
use crate::editor::Editor;
use imgui::{MouseButton, Ui};
use std::sync::Arc;
use uuid::Uuid;

const SPOT_SIZE: [f32; 2] = [190., 62.];
const PIECE_SIZE: [f32; 2] = [222., 92.];
const PIN: f32 = 6.;
const CANVAS: [f32; 4] = [0.105, 0.11, 0.125, 1.];
const GRID: [f32; 4] = [1., 1., 1., 0.045];
const NODE: [f32; 4] = [0.16, 0.165, 0.185, 1.];
const SELECTED: [f32; 4] = [0.36, 0.62, 0.95, 1.];
const AMBER: [f32; 4] = [1., 0.72, 0.3, 1.];
const SPOT_COLOR: [f32; 4] = [0.17, 0.42, 0.45, 1.];
const LINK: [f32; 4] = [0.62, 0.66, 0.72, 0.9];
const ICON_PIECE: &str = "\u{f1b2}";
const ICON_LIGHT: &str = "\u{f0eb}";
const ICON_ACTOR: &str = "\u{f554}";
const ICON_TILE: &str = "\u{f84c}";
const ICON_KEY: &str = "\u{f084}";
const ICON_DOOR: &str = "\u{f023}";
const ICON_ADD: &str = "\u{f067}";

#[derive(Clone, Copy)]
enum Drag {
    Node { id: Uuid, grab: [f32; 2] },
    Link { spot: Uuid },
    Pan,
}

struct Add {
    /// Canvas position for the new node.
    at: [f32; 2],
    /// The spot a new piece belongs to, when the menu came from a spot.
    spot: Option<Uuid>,
    query: String,
    open: bool,
}

pub struct Window {
    pub asset: Uuid,
    pub style: Style,
    undo: Vec<Style>,
    redo: Vec<Style>,
    dirty: bool,
    status: String,
    selected: Option<Uuid>,
    pan: [f32; 2],
    zoom: f32,
    drag: Option<Drag>,
    /// The style before the drag in progress, recorded as one undo step on release.
    gesture: Option<Style>,
    add: Option<Add>,
    /// Regenerate the map's dungeons that use this style after each edit.
    pub live: bool,
    framed: bool,
}
impl Window {
    fn new(asset: Uuid, style: Style) -> Self {
        Self {
            asset,
            style,
            undo: Vec::new(),
            redo: Vec::new(),
            dirty: false,
            status: String::new(),
            selected: None,
            pan: [0.; 2],
            zoom: 1.,
            drag: None,
            gesture: None,
            add: None,
            live: true,
            framed: false,
        }
    }
    fn checkpoint(&mut self) {
        self.undo.push(self.style.clone());
        if self.undo.len() > 64 {
            self.undo.remove(0);
        }
        self.redo.clear();
        self.dirty = true;
    }
    fn node(&self, id: Uuid) -> Option<&Node> {
        self.style.nodes.iter().find(|n| n.id == id)
    }
}

// ---- opening and creating ------------------------------------------------------------------

/// Opens the editor on style `id`.
pub fn open(e: &mut Editor, id: Uuid) {
    if e.dungeon_style.as_ref().is_some_and(|w| w.asset == id) {
        return;
    }
    let style = e
        .assets
        .index
        .resolve(id)
        .ok()
        .and_then(|r| dungeon_style::document(r).ok())
        .or_else(|| {
            let index = crate::assets::scan(&e.root, &mut Default::default());
            index
                .resolve(id)
                .ok()
                .and_then(|r| dungeon_style::document(r).ok())
        });
    match style {
        Some(style) => e.dungeon_style = Some(Window::new(id, style)),
        None => {
            let error = format!("Dungeon Style {id} could not be opened");
            e.last_error = Some(error.clone());
            e.log(error);
        }
    }
}

/// Selects the open style's first piece, for screenshots.
pub fn select_first_piece(e: &mut Editor) {
    if let Some(w) = &mut e.dungeon_style {
        w.selected = w
            .style
            .nodes
            .iter()
            .find(|n| matches!(n.kind, NodeKind::Piece { .. }))
            .map(|n| n.id);
    }
}

/// Creates a style asset starting from `from` (or the built-in style) and opens it.
pub fn create(e: &mut Editor, name: &str, mut from: Style) -> Result<Uuid, String> {
    crate::dungeon_bake::pin_markers(&e.root, &mut from)?;
    let path = dungeon_style::fresh_path(&e.root, name);
    let id = dungeon_style::create(&e.root, &path, &from)?;
    e.assets.index = crate::assets::scan(&e.root, &mut Default::default());
    e.assets.refresh();
    open(e, id);
    e.log(format!("Created {path}."));
    Ok(id)
}

fn store(e: &mut Editor, w: &Window) -> Result<(), String> {
    let record = match e.assets.index.resolve(w.asset) {
        Ok(record) => record.clone(),
        Err(_) => {
            e.assets.index = crate::assets::scan(&e.root, &mut Default::default());
            e.assets.index.resolve(w.asset)?.clone()
        }
    };
    dungeon_style::save(&record, &w.style)?;
    e.assets.index = crate::assets::scan(&e.root, &mut Default::default());
    e.assets.refresh();
    Ok(())
}

// ---- window --------------------------------------------------------------------------------

pub fn window(ui: &Ui, e: &mut Editor) {
    let Some(mut w) = e.dungeon_style.take() else {
        return;
    };
    let name = e
        .assets
        .index
        .resolve(w.asset)
        .ok()
        .and_then(|r| r.path.file_stem().map(|s| s.to_string_lossy().into_owned()))
        .unwrap_or_else(|| "Dungeon Style".into());
    let mut open = true;
    ui.window(format!(
        "{} Dungeon Style: {name}###dungeon-style-editor",
        crate::dungeon_editor::ICON
    ))
    .opened(&mut open)
    .size([1040., 560.], imgui::Condition::FirstUseEver)
    .position([280., 380.], imgui::Condition::FirstUseEver)
    .build(|| body(ui, e, &mut w));
    // A finished edit (no drag in progress) is saved, then previewed.
    if w.dirty
        && w.drag.is_none()
        && !ui.is_mouse_down(MouseButton::Left)
        && !ui.is_any_item_active()
    {
        w.dirty = false;
        match w.style.validate().and_then(|_| store(e, &w)) {
            Ok(()) => {
                w.status = "Saved".into();
                if w.live {
                    let count = crate::dungeon_editor::regenerate_with_style(e, w.asset);
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
        }
    }
    if open {
        e.dungeon_style = Some(w);
    }
}

fn body(ui: &Ui, e: &mut Editor, w: &mut Window) {
    let _font = crate::inspector_theme::scope(ui, e.inspector_font);
    toolbar(ui, e, w);
    let available = ui.content_region_avail();
    let side = 330_f32.min(available[0] * 0.4);
    ui.child_window("##style-canvas")
        .size([available[0] - side - 8., available[1] - 24.])
        .border(true)
        .build(|| canvas(ui, e, w));
    ui.same_line();
    ui.child_window("##style-details")
        .size([side, available[1] - 24.])
        .build(|| details(ui, e, w));
    ui.text_disabled(
        "Pieces of a spot are tried left to right. Drag from a spot's dot to add a piece; drag a piece onto another spot to move it. Delete removes the selection.",
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
        w.redo.push(std::mem::replace(&mut w.style, previous));
        w.dirty = true;
    }
}
fn redo(w: &mut Window) {
    if let Some(next) = w.redo.pop() {
        w.undo.push(std::mem::replace(&mut w.style, next));
        w.dirty = true;
    }
}

fn toolbar(ui: &Ui, e: &mut Editor, w: &mut Window) {
    if ui.button(format!("{ICON_ADD} Add")) {
        w.add = Some(Add {
            at: [w.pan[0] + 40. / w.zoom, w.pan[1] + 40. / w.zoom],
            spot: None,
            query: String::new(),
            open: true,
        });
    }
    #[cfg(test)]
    crate::gui::record_script_control(ui, "Add style node");
    if ui.is_item_hovered() {
        ui.tooltip_text("Add a spot or a piece. Right-click the canvas does the same.");
    }
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
    if ui.button("\u{f065} Frame all") {
        w.framed = false;
    }
    ui.same_line();
    ui.checkbox("Live preview", &mut w.live);
    if ui.is_item_hovered() {
        let users = e
            .scene
            .dungeons
            .iter()
            .filter(|g| g.style == Some(w.asset))
            .count();
        ui.tooltip_text(format!(
            "Regenerate the dungeons of this map that use the style after every edit ({users} use it now)."
        ));
    }
    ui.same_line();
    ui.text_disabled(if w.style.two_d {
        "2D style"
    } else {
        "3D style"
    });
    if !w.status.is_empty() {
        ui.same_line();
        crate::gui::muted(ui, &w.status);
    }
}

// ---- canvas --------------------------------------------------------------------------------

fn size(node: &Node) -> [f32; 2] {
    match node.kind {
        NodeKind::Spot { .. } => SPOT_SIZE,
        NodeKind::Piece { .. } => PIECE_SIZE,
    }
}
fn piece_color(piece: &Piece) -> [f32; 4] {
    match piece {
        Piece::Blockout { color, .. } => [color[0] * 0.7, color[1] * 0.7, color[2] * 0.7, 1.],
        Piece::Mesh { .. } => [0.25, 0.42, 0.55, 1.],
        Piece::Actor { .. } => [0.42, 0.30, 0.58, 1.],
        Piece::Light { .. } => [0.62, 0.48, 0.20, 1.],
        Piece::Terrain { .. } => [0.30, 0.48, 0.28, 1.],
        Piece::Tile { .. } => [0.46, 0.36, 0.26, 1.],
        Piece::Key { .. } => [0.62, 0.50, 0.16, 1.],
        Piece::Door { .. } => [0.58, 0.20, 0.18, 1.],
    }
}
fn piece_icon(piece: &Piece) -> &'static str {
    match piece {
        Piece::Blockout { .. } | Piece::Mesh { .. } => ICON_PIECE,
        Piece::Actor { .. } => ICON_ACTOR,
        Piece::Light { .. } => ICON_LIGHT,
        Piece::Terrain { .. } | Piece::Tile { .. } => ICON_TILE,
        Piece::Key { .. } => ICON_KEY,
        Piece::Door { .. } => ICON_DOOR,
    }
}

/// Why a piece will not place anything, if it will not.
fn problem(e: &Editor, style: &Style, spot: SpotKind, visual: &Visual) -> Option<String> {
    if !visual.piece.fits(style.two_d) {
        return Some(format!(
            "{} pieces only work in {} styles",
            visual.piece.label(),
            if style.two_d { "3D" } else { "2D" }
        ));
    }
    if visual.chance == 0 {
        return Some("Chance is 0%: it never places".into());
    }
    match &visual.piece {
        Piece::Blockout { .. }
            if matches!(
                spot,
                SpotKind::Door | SpotKind::Room | SpotKind::Spawn | SpotKind::Goal
            ) =>
        {
            Some("Blockout draws nothing at this spot; use a Mesh or Actor".into())
        }
        Piece::Mesh { asset: None } => Some("Choose a mesh".into()),
        Piece::Mesh { asset: Some(id) } if e.assets.index.resolve(*id).is_err() => {
            Some("The mesh is missing".into())
        }
        Piece::Actor { class: None } => Some("Choose an actor class".into()),
        Piece::Terrain { .. } if !matches!(spot, SpotKind::Floor | SpotKind::Solid) => {
            Some("Terrains paint Floor and Solid spots only".into())
        }
        Piece::Key { .. } if spot != SpotKind::Key => {
            Some("Key pieces work on Key spots, which recipe layouts place".into())
        }
        Piece::Door { .. } if spot != SpotKind::Lock => {
            Some("Door pieces work on Locked door spots, which recipe layouts place".into())
        }
        _ => None,
    }
}

fn summary(e: &Editor, spot: Option<SpotKind>, piece: &Piece) -> String {
    let asset_name = |id: &Option<Uuid>| {
        id.and_then(|id| e.assets.index.resolve(id).ok())
            .and_then(|r| r.path.file_stem().map(|s| s.to_string_lossy().into_owned()))
    };
    match piece {
        Piece::Blockout { texture, .. } => {
            let shape = match spot {
                Some(SpotKind::Floor) => "floor tile",
                Some(SpotKind::Wall) => "wall panel",
                Some(SpotKind::Solid) => "wall top",
                Some(SpotKind::Doorway) => "doorway beam",
                Some(SpotKind::Corner) => "corner pillar",
                _ => "nothing here",
            };
            match asset_name(texture) {
                Some(t) => format!("{shape}, textured {t}"),
                None => shape.into(),
            }
        }
        Piece::Mesh { asset } => asset_name(asset).unwrap_or_else(|| "no mesh chosen".into()),
        Piece::Actor { class } => class
            .as_deref()
            .map(|c| {
                e.object_model()
                    .and_then(|m| m.class(c).map(|k| k.cpp_name.clone()))
                    .unwrap_or_else(|| c.to_owned())
            })
            .map(|c| crate::actor_document::short_class_name(&c).to_owned())
            .unwrap_or_else(|| "no class chosen".into()),
        Piece::Light {
            intensity, range, ..
        } => format!("intensity {intensity:.1} · range {range:.1}"),
        Piece::Terrain { tile_set, terrain } => format!(
            "{terrain} · {}",
            asset_name(tile_set).unwrap_or_else(|| "generator tiles".into())
        ),
        Piece::Tile { tile_set, atlas } => format!(
            "tile {},{} · {}",
            atlas[0],
            atlas[1],
            asset_name(tile_set).unwrap_or_else(|| "generator tiles".into())
        ),
        Piece::Key { .. } => "picked up on touch, opens its door".into(),
        Piece::Door { .. } => "blocks until its key is picked up".into(),
    }
}
fn badges(visual: &Visual) -> String {
    let mut parts = vec![format!("{}%", visual.chance)];
    if !visual.consume {
        parts.push("then continues".into());
    }
    match visual.filter {
        Filter::Rooms => parts.push("rooms".into()),
        Filter::Corridors => parts.push("corridors".into()),
        Filter::Anywhere => {}
    }
    if visual.limit > 0 {
        parts.push(format!("max {}", visual.limit));
    }
    if !visual.item.is_empty() {
        parts.push(format!("\"{}\" only", visual.item));
    }
    if !visual.path.is_empty() {
        parts.push(format!("on {}", visual.path));
    }
    parts.join(" · ")
}

fn canvas(ui: &Ui, e: &mut Editor, w: &mut Window) {
    let origin = ui.cursor_screen_pos();
    let area = ui.content_region_avail();
    if !w.framed {
        frame_all(w, area);
        w.framed = true;
    }
    let to_screen = |w: &Window, p: [f32; 2]| {
        [
            origin[0] + (p[0] - w.pan[0]) * w.zoom,
            origin[1] + (p[1] - w.pan[1]) * w.zoom,
        ]
    };
    let to_canvas = |w: &Window, p: [f32; 2]| {
        [
            (p[0] - origin[0]) / w.zoom + w.pan[0],
            (p[1] - origin[1]) / w.zoom + w.pan[1],
        ]
    };
    // Text follows the zoom like the boxes do.
    ui.set_window_font_scale(w.zoom);
    let z = w.zoom;
    ui.invisible_button("##style-canvas-input", area);
    let hovered = ui.is_item_hovered();
    let draw = ui.get_window_draw_list();
    let clip_max = [origin[0] + area[0], origin[1] + area[1]];
    draw.with_clip_rect_intersect(origin, clip_max, || {
        draw.add_rect(origin, clip_max, CANVAS).filled(true).build();
        let step = 32. * z;
        let mut x = origin[0] - (w.pan[0] * z).rem_euclid(step);
        while x < clip_max[0] {
            draw.add_line([x, origin[1]], [x, clip_max[1]], GRID)
                .build();
            x += step;
        }
        let mut y = origin[1] - (w.pan[1] * z).rem_euclid(step);
        while y < clip_max[1] {
            draw.add_line([origin[0], y], [clip_max[0], y], GRID)
                .build();
            y += step;
        }
        // Links; each piece's header shows its place in its spot's order.
        let mut orders = std::collections::BTreeMap::new();
        for spot in w
            .style
            .nodes
            .iter()
            .filter(|n| matches!(n.kind, NodeKind::Spot { .. }))
        {
            let from = to_screen(
                w,
                [
                    spot.position[0] + SPOT_SIZE[0],
                    spot.position[1] + SPOT_SIZE[1] * 0.5,
                ],
            );
            for (order, (piece, _)) in w.style.pieces(spot.id).into_iter().enumerate() {
                orders.insert(piece.id, order + 1);
                let to = to_screen(
                    w,
                    [piece.position[0], piece.position[1] + PIECE_SIZE[1] * 0.5],
                );
                let bend = ((to[0] - from[0]).abs() * 0.5).clamp(30. * z, 120. * z);
                draw.add_bezier_curve(
                    from,
                    [from[0] + bend, from[1]],
                    [to[0] - bend, to[1]],
                    to,
                    LINK,
                )
                .thickness(2.)
                .build();
            }
        }
        if let Some(Drag::Link { spot }) = w.drag
            && let Some(node) = w.node(spot)
        {
            let from = to_screen(
                w,
                [
                    node.position[0] + SPOT_SIZE[0],
                    node.position[1] + SPOT_SIZE[1] * 0.5,
                ],
            );
            let to = ui.io().mouse_pos;
            draw.add_bezier_curve(
                from,
                [from[0] + 60., from[1]],
                [to[0] - 60., to[1]],
                to,
                SELECTED,
            )
            .thickness(2.)
            .build();
        }
        // Nodes, spots first so pieces draw over them.
        let mut order: Vec<usize> = (0..w.style.nodes.len()).collect();
        order.sort_by_key(|i| matches!(w.style.nodes[*i].kind, NodeKind::Piece { .. }));
        for i in order {
            let node = &w.style.nodes[i];
            let a = to_screen(w, node.position);
            let s = size(node).map(|v| v * z);
            let b = [a[0] + s[0], a[1] + s[1]];
            let selected = w.selected == Some(node.id);
            draw.add_rect(a, b, NODE).filled(true).rounding(5.).build();
            let (header, title, line, extra, warning) = match &node.kind {
                NodeKind::Spot { spot } => (
                    SPOT_COLOR,
                    format!("{} {}", crate::dungeon_editor::ICON, spot.label()),
                    spot.description().to_owned(),
                    String::new(),
                    None,
                ),
                NodeKind::Piece { parent, visual } => {
                    let spot = w.node(*parent).and_then(|n| match n.kind {
                        NodeKind::Spot { spot } => Some(spot),
                        _ => None,
                    });
                    let order = orders
                        .get(&node.id)
                        .map(|o| format!("{o} · "))
                        .unwrap_or_default();
                    (
                        piece_color(&visual.piece),
                        format!(
                            "{order}{} {}",
                            piece_icon(&visual.piece),
                            visual.piece.label()
                        ),
                        summary(e, spot, &visual.piece),
                        badges(visual),
                        spot.and_then(|spot| problem(e, &w.style, spot, visual)),
                    )
                }
            };
            draw.add_rect(a, [b[0], a[1] + 22. * z], header)
                .filled(true)
                .rounding(5.)
                .round_top_left(true)
                .round_top_right(true)
                .round_bot_left(false)
                .round_bot_right(false)
                .build();
            draw.add_text([a[0] + 8. * z, a[1] + 3. * z], [1., 1., 1., 1.], &title);
            if let NodeKind::Piece { visual, .. } = &node.kind
                && let Piece::Blockout { color, .. } | Piece::Light { color, .. } = &visual.piece
            {
                draw.add_rect(
                    [b[0] - 22. * z, a[1] + 5. * z],
                    [b[0] - 8. * z, a[1] + 17. * z],
                    [color[0], color[1], color[2], 1.],
                )
                .filled(true)
                .build();
            }
            let wrap_width = s[0] - 16. * z;
            let mut text = line.clone();
            while !text.is_empty() && ui.calc_text_size(&text)[0] > wrap_width {
                text.pop();
            }
            if text.len() < line.len() {
                // Room for the dots; pop by characters so a name never splits.
                text.pop();
                text.pop();
                text.push_str("...");
            }
            draw.add_text(
                [a[0] + 8. * z, a[1] + 28. * z],
                [0.82, 0.84, 0.88, 1.],
                &text,
            );
            if !extra.is_empty() {
                let mut extra = extra.clone();
                let full = extra.len();
                while !extra.is_empty() && ui.calc_text_size(&extra)[0] > wrap_width {
                    extra.pop();
                }
                if extra.len() < full {
                    extra.pop();
                    extra.pop();
                    extra.push_str("...");
                }
                draw.add_text(
                    [a[0] + 8. * z, a[1] + 48. * z],
                    [0.6, 0.63, 0.68, 1.],
                    &extra,
                );
            }
            if let Some(warning) = &warning {
                let mut text = format!("\u{f071} {warning}");
                while !text.is_empty() && ui.calc_text_size(&text)[0] > wrap_width {
                    text.pop();
                }
                draw.add_text([a[0] + 8. * z, a[1] + 68. * z], AMBER, &text);
            }
            let outline = if selected {
                SELECTED
            } else if warning.is_some() {
                AMBER
            } else {
                [1., 1., 1., 0.08]
            };
            draw.add_rect(a, b, outline)
                .rounding(5.)
                .thickness(if selected { 2. } else { 1. })
                .build();
            match node.kind {
                NodeKind::Spot { .. } => {
                    let pin = [b[0], a[1] + s[1] * 0.5];
                    draw.add_circle(pin, PIN * z, [0.62, 0.86, 0.88, 1.])
                        .filled(true)
                        .build();
                }
                NodeKind::Piece { .. } => {
                    let pin = [a[0], a[1] + s[1] * 0.5];
                    draw.add_circle(pin, (PIN - 1.) * z, LINK)
                        .filled(true)
                        .build();
                }
            }
        }
        if w.style.nodes.is_empty() {
            ui.set_window_font_scale(1.);
            let text = "Empty style. Press Add, or right-click, to add a spot.";
            let width = ui.calc_text_size(text)[0];
            draw.add_text(
                [
                    origin[0] + (area[0] - width) * 0.5,
                    origin[1] + area[1] * 0.45,
                ],
                [0.7, 0.72, 0.76, 1.],
                text,
            );
        }
    });
    ui.set_window_font_scale(1.);
    // Interaction.
    let mouse = ui.io().mouse_pos;
    let at = to_canvas(w, mouse);
    let node_under = w
        .style
        .nodes
        .iter()
        .rev()
        .find(|n| {
            let s = size(n);
            at[0] >= n.position[0] - PIN
                && at[0] <= n.position[0] + s[0] + PIN
                && at[1] >= n.position[1]
                && at[1] <= n.position[1] + s[1]
        })
        .map(|n| n.id);
    let spot_pin = w
        .style
        .nodes
        .iter()
        .find(|n| {
            matches!(n.kind, NodeKind::Spot { .. }) && {
                let pin = [
                    n.position[0] + SPOT_SIZE[0],
                    n.position[1] + SPOT_SIZE[1] * 0.5,
                ];
                (at[0] - pin[0]).powi(2) + (at[1] - pin[1]).powi(2) <= (PIN * 2.).powi(2)
            }
        })
        .map(|n| n.id);
    if hovered && ui.is_mouse_clicked(MouseButton::Left) {
        if let Some(spot) = spot_pin {
            w.drag = Some(Drag::Link { spot });
        } else if let Some(id) = node_under {
            w.selected = Some(id);
            let position = w.node(id).map_or([0.; 2], |n| n.position);
            w.drag = Some(Drag::Node {
                id,
                grab: [at[0] - position[0], at[1] - position[1]],
            });
            w.gesture = Some(w.style.clone());
        } else {
            w.selected = None;
            w.drag = Some(Drag::Pan);
        }
    }
    if hovered && ui.is_mouse_clicked(MouseButton::Middle) {
        w.drag = Some(Drag::Pan);
    }
    // Wheel zoom keeps the point under the cursor in place.
    let wheel = ui.io().mouse_wheel;
    if hovered && wheel != 0. {
        let zoom = (w.zoom * 1.12_f32.powf(wheel)).clamp(0.35, 1.6);
        w.pan = [0, 1].map(|k| at[k] - (mouse[k] - origin[k]) / zoom);
        w.zoom = zoom;
    }
    match w.drag {
        Some(Drag::Pan) => {
            let delta = ui.io().mouse_delta;
            w.pan[0] -= delta[0] / w.zoom;
            w.pan[1] -= delta[1] / w.zoom;
            if !ui.is_mouse_down(MouseButton::Left) && !ui.is_mouse_down(MouseButton::Middle) {
                w.drag = None;
            }
        }
        Some(Drag::Node { id, grab }) => {
            if let Some(node) = w.style.nodes.iter_mut().find(|n| n.id == id) {
                let target = [(at[0] - grab[0]).round(), (at[1] - grab[1]).round()];
                if node.position != target {
                    node.position = target;
                }
            }
            if !ui.is_mouse_down(MouseButton::Left) {
                w.drag = None;
                // Dropping a piece onto another spot moves it there.
                let mut style = w.style.clone();
                if let Some(spot) = node_under_spot(&style, id, at)
                    && let Some(NodeKind::Piece { parent, .. }) = style
                        .nodes
                        .iter_mut()
                        .find(|n| n.id == id)
                        .map(|n| &mut n.kind)
                    && *parent != spot
                {
                    *parent = spot;
                    w.style = style;
                }
                if let Some(before) = w.gesture.take()
                    && before != w.style
                {
                    w.undo.push(before);
                    w.redo.clear();
                    w.dirty = true;
                }
            }
        }
        Some(Drag::Link { spot }) if !ui.is_mouse_down(MouseButton::Left) => {
            w.drag = None;
            match node_under {
                Some(id) if matches!(w.node(id).map(|n| &n.kind), Some(NodeKind::Piece { .. })) => {
                    w.checkpoint();
                    if let Some(NodeKind::Piece { parent, .. }) = w
                        .style
                        .nodes
                        .iter_mut()
                        .find(|n| n.id == id)
                        .map(|n| &mut n.kind)
                    {
                        *parent = spot;
                    }
                }
                Some(_) => {}
                None => {
                    w.add = Some(Add {
                        at,
                        spot: Some(spot),
                        query: String::new(),
                        open: true,
                    })
                }
            }
        }
        Some(Drag::Link { .. }) => {}
        None => {}
    }
    if hovered && ui.is_mouse_clicked(MouseButton::Right) {
        let spot = node_under
            .filter(|id| matches!(w.node(*id).map(|n| &n.kind), Some(NodeKind::Spot { .. })));
        w.add = Some(Add {
            at,
            spot,
            query: String::new(),
            open: true,
        });
    }
    if (hovered || ui.is_window_focused())
        && !crate::gui::text_input_active(ui)
        && (ui.is_key_pressed(imgui::Key::Delete) || ui.is_key_pressed(imgui::Key::Backspace))
        && let Some(id) = w.selected
    {
        remove(w, id);
    }
    if hovered && ui.is_key_pressed(imgui::Key::F) && !crate::gui::text_input_active(ui) {
        frame_all(w, area);
    }
    add_popup(ui, e, w);
}

/// The spot node under `at`, other than the dragged node itself.
fn node_under_spot(style: &Style, dragged: Uuid, at: [f32; 2]) -> Option<Uuid> {
    style
        .nodes
        .iter()
        .filter(|n| n.id != dragged && matches!(n.kind, NodeKind::Spot { .. }))
        .find(|n| {
            at[0] >= n.position[0]
                && at[0] <= n.position[0] + SPOT_SIZE[0]
                && at[1] >= n.position[1]
                && at[1] <= n.position[1] + SPOT_SIZE[1]
        })
        .map(|n| n.id)
}

fn frame_all(w: &mut Window, area: [f32; 2]) {
    if w.style.nodes.is_empty() {
        w.pan = [-40., -40.];
        w.zoom = 1.;
        return;
    }
    let lo = [0, 1].map(|k| {
        w.style
            .nodes
            .iter()
            .map(|n| n.position[k])
            .fold(f32::MAX, f32::min)
    });
    let hi = [0, 1].map(|k| {
        w.style
            .nodes
            .iter()
            .map(|n| n.position[k] + size(n)[k])
            .fold(f32::MIN, f32::max)
    });
    let span = [hi[0] - lo[0] + 60., hi[1] - lo[1] + 60.];
    w.zoom = (area[0] / span[0]).min(area[1] / span[1]).clamp(0.25, 1.);
    w.pan = [0, 1].map(|k| (lo[k] + hi[k]) * 0.5 - area[k] * 0.5 / w.zoom);
}

fn remove(w: &mut Window, id: Uuid) {
    w.checkpoint();
    w.style.nodes.retain(|n| {
        n.id != id && !matches!(n.kind, NodeKind::Piece { parent, .. } if parent == id)
    });
    w.selected = None;
}

// ---- adding ----------------------------------------------------------------------------------

fn add_popup(ui: &Ui, e: &mut Editor, w: &mut Window) {
    let Some(add) = &mut w.add else {
        return;
    };
    if add.open {
        ui.open_popup("##style-add");
        add.open = false;
    }
    let two_d = w.style.two_d;
    let present: Vec<(Uuid, SpotKind)> = w
        .style
        .nodes
        .iter()
        .filter_map(|n| match n.kind {
            NodeKind::Spot { spot } => Some((n.id, spot)),
            _ => None,
        })
        .collect();
    // (label, group, action)
    enum Action {
        Spot(SpotKind),
        Piece(Uuid, Piece),
    }
    let mut entries: Vec<(String, String, String, Action)> = Vec::new();
    let targets: Vec<(Uuid, SpotKind)> = match add.spot {
        Some(spot) => present
            .iter()
            .copied()
            .filter(|(id, _)| *id == spot)
            .collect(),
        None => present.clone(),
    };
    for (id, kind) in &targets {
        for piece in Piece::defaults(two_d) {
            let label = if add.spot.is_some() {
                format!("{} {} piece", piece_icon(&piece), piece.label())
            } else {
                format!(
                    "{} {} on {}",
                    piece_icon(&piece),
                    piece.label(),
                    kind.label()
                )
            };
            let tip = match &piece {
                Piece::Blockout { .. } => {
                    "The generator's own geometry for this spot, in a colour or texture."
                }
                Piece::Mesh { .. } => {
                    "An EditableMesh merged into the dungeon geometry: costs no actor."
                }
                Piece::Actor { .. } => {
                    "An actor of any placeable class at each placement; each costs an actor."
                }
                Piece::Light { .. } => {
                    "A baked point light at each placement; each costs an actor."
                }
                Piece::Terrain { .. } => "The terrain this cell is painted with.",
                Piece::Tile { .. } => "One tile on a decoration layer above the floor.",
                Piece::Key { .. } => {
                    "A key the player picks up by touching it; it opens the doors with its lock number. One actor each."
                }
                Piece::Door { .. } => {
                    "A locked door that blocks the doorway until its key is picked up. One actor each."
                }
            };
            entries.push((
                label,
                "Pieces".into(),
                tip.into(),
                Action::Piece(*id, piece),
            ));
        }
    }
    if add.spot.is_none() {
        for kind in SpotKind::ALL {
            if present.iter().any(|(_, k)| *k == kind) {
                continue;
            }
            entries.push((
                format!("{} {} spot", crate::dungeon_editor::ICON, kind.label()),
                "Spots".into(),
                kind.description().into(),
                Action::Spot(kind),
            ));
        }
    }
    let mut chosen = None;
    let mut close = false;
    let mut shown = false;
    ui.popup("##style-add", || {
        shown = true;
        if ui.is_window_appearing() {
            ui.set_keyboard_focus_here();
        }
        ui.set_next_item_width(300.);
        let enter = ui
            .input_text("##style-add-search", &mut add.query)
            .hint("Search spots and pieces")
            .enter_returns_true(true)
            .build();
        let query = add.query.to_lowercase();
        let matches: Vec<usize> = entries
            .iter()
            .enumerate()
            .filter(|(_, (label, group, tip, _))| {
                query.split_whitespace().all(|word| {
                    label.to_lowercase().contains(word)
                        || group.to_lowercase().contains(word)
                        || tip.to_lowercase().contains(word)
                })
            })
            .map(|(i, _)| i)
            .collect();
        if enter && let Some(first) = matches.first() {
            chosen = Some(*first);
            close = true;
        }
        let mut group = "";
        for i in &matches {
            let (label, g, tip, _) = &entries[*i];
            if g != group {
                group = g;
                ui.text_disabled(g);
            }
            if ui.selectable(label) {
                chosen = Some(*i);
                close = true;
            }
            #[cfg(test)]
            crate::gui::record_script_control(ui, &format!("Add style: {label}"));
            if ui.is_item_hovered() {
                ui.tooltip_text(tip);
            }
        }
        if matches.is_empty() {
            ui.text_disabled(if entries.is_empty() {
                "Add a spot first: pieces belong to spots."
            } else {
                "Nothing matches."
            });
        }
        if close {
            ui.close_current_popup();
        }
    });
    let at = add.at;
    if let Some(i) = chosen {
        let (_, _, _, action) = entries.swap_remove(i);
        w.checkpoint();
        let id = Uuid::new_v4();
        match action {
            Action::Spot(spot) => w.style.nodes.push(Node {
                id,
                position: at,
                kind: NodeKind::Spot { spot },
            }),
            Action::Piece(parent, piece) => {
                // A piece added from a spot lands right of that spot's last piece.
                let position = match w.add.as_ref().and_then(|a| a.spot) {
                    Some(_) => at,
                    None => {
                        let parent_node = w.node(parent).map_or(at, |n| n.position);
                        let right = w
                            .style
                            .pieces(parent)
                            .iter()
                            .map(|(n, _)| n.position[0] + PIECE_SIZE[0] + 20.)
                            .fold(parent_node[0] + SPOT_SIZE[0] + 60., f32::max);
                        [right, parent_node[1]]
                    }
                };
                w.style.nodes.push(Node {
                    id,
                    position,
                    kind: NodeKind::Piece {
                        parent,
                        visual: Visual {
                            piece,
                            ..Visual::default()
                        },
                    },
                });
            }
        }
        w.selected = Some(id);
        w.add = None;
        e.view_dirty = true;
    } else if !shown {
        w.add = None;
    }
}

// ---- details -------------------------------------------------------------------------------

fn details(ui: &Ui, e: &mut Editor, w: &mut Window) {
    let Some(id) = w.selected else {
        crate::gui::heading(ui, "Dungeon Style");
        crate::gui::muted(
            ui,
            "A style decides what every spot of a dungeon becomes. Select a node to edit it.",
        );
        ui.spacing();
        crate::gui::muted(
            ui,
            "Blockout pieces are the generator's own geometry; Mesh pieces merge EditableMeshes into it at no actor cost; Actor and Light pieces add one actor per placement.",
        );
        if let Some(counts) = placement_counts(e, w) {
            ui.spacing();
            crate::gui::muted(
                ui,
                format!(
                    "In this map: {} placements across {} pieces.",
                    counts.values().sum::<usize>(),
                    counts.len()
                ),
            );
        }
        return;
    };
    let Some(index) = w.style.nodes.iter().position(|n| n.id == id) else {
        w.selected = None;
        return;
    };
    let mut node = w.style.nodes[index].clone();
    let before = node.clone();
    let mut delete = false;
    match &mut node.kind {
        NodeKind::Spot { spot } => {
            crate::gui::heading(
                ui,
                &format!("{} {} spot", crate::dungeon_editor::ICON, spot.label()),
            );
            ui.text_wrapped(spot.description());
            let pieces = w.style.pieces(id).len();
            let _ = crate::gui::field(ui, "Pieces");
            ui.text(format!("{pieces}, tried left to right"));
            ui.spacing();
            if ui.button(format!("{ICON_ADD} Add piece")) {
                w.add = Some(Add {
                    at: [0.; 2],
                    spot: None,
                    query: spot.label().to_lowercase(),
                    open: true,
                });
            }
            ui.same_line();
            if ui.button("Remove spot") {
                delete = true;
            }
            if ui.is_item_hovered() {
                ui.tooltip_text("Removes the spot and its pieces (Delete).");
            }
        }
        NodeKind::Piece { parent, visual } => {
            let spot = w
                .node(*parent)
                .and_then(|n| match n.kind {
                    NodeKind::Spot { spot } => Some(spot),
                    _ => None,
                })
                .unwrap_or(SpotKind::Floor);
            crate::gui::heading(
                ui,
                &format!(
                    "{} {} piece",
                    piece_icon(&visual.piece),
                    visual.piece.label()
                ),
            );
            let count = placement_counts(e, w).and_then(|c| c.get(&id).copied());
            if let Some(count) = count {
                crate::gui::muted(
                    ui,
                    format!(
                        "On {} spots · placed {count} times in this map",
                        spot.label()
                    ),
                );
            } else {
                crate::gui::muted(ui, format!("On {} spots", spot.label()));
            }
            if let Some(problem) = problem(e, &w.style, spot, visual) {
                ui.text_colored(AMBER, format!("\u{f071} {problem}"));
            }
            ui.spacing();
            piece_fields(ui, e, w.style.two_d, &mut visual.piece);
            ui.spacing();
            crate::gui::muted(ui, "Placement");
            let mut chance = i32::from(visual.chance);
            if ui
                .slider_config(crate::gui::field(ui, "Chance"), 0, 100)
                .display_format("%d %%")
                .build(&mut chance)
            {
                visual.chance = chance.clamp(0, 100) as u8;
            }
            if ui.is_item_hovered() {
                ui.tooltip_text("Percentage of eligible spots that get this piece.");
            }
            let mut stop = visual.consume;
            if crate::gui::toggle(ui, "Stop here", &mut stop) {
                visual.consume = stop;
            }
            if ui.is_item_hovered() {
                ui.tooltip_text("When this piece places, pieces to its right are skipped at that spot. Off: they are tried too, so it decorates.");
            }
            let filter_label = visual.filter.label();
            if let Some(_combo) = ui.begin_combo(crate::gui::field(ui, "Where"), filter_label) {
                for filter in Filter::ALL {
                    if ui
                        .selectable_config(filter.label())
                        .selected(filter == visual.filter)
                        .build()
                    {
                        visual.filter = filter;
                    }
                }
            }
            let mut path = visual.path.clone();
            if ui
                .input_text(crate::gui::field(ui, "Path"), &mut path)
                .hint("any path")
                .build()
            {
                visual.path = path.chars().take(32).collect();
            }
            if ui.is_item_hovered() {
                ui.tooltip_text("Recipe layouts: only spots in rooms of this path, such as main, or a branch's name. Empty places everywhere; corridors belong to no path.");
            }
            if spot == SpotKind::Item {
                let mut name = visual.item.clone();
                if ui
                    .input_text(crate::gui::field(ui, "Item name"), &mut name)
                    .hint("any item")
                    .build()
                {
                    visual.item = name.chars().take(32).collect();
                }
                if ui.is_item_hovered() {
                    ui.tooltip_text(
                        "Only items a recipe's Items step names this way; empty takes every item.",
                    );
                }
            }
            if matches!(spot, SpotKind::Key | SpotKind::Lock)
                && matches!(visual.piece, Piece::Actor { .. })
            {
                crate::gui::muted(
                    ui,
                    "Actors placed here get the lock number in a number property named lock, on the actor or its components.",
                );
            }
            let mut limit = i32::from(visual.limit);
            if crate::gui::Drag::new(crate::gui::field(ui, "Limit"))
                .range(0, 999)
                .build(ui, &mut limit)
            {
                visual.limit = limit.clamp(0, 999) as u16;
            }
            if ui.is_item_hovered() {
                ui.tooltip_text(
                    "Most placements in one dungeon, spread at random; 0 has no limit.",
                );
            }
            if !w.style.two_d
                || matches!(
                    visual.piece,
                    Piece::Actor { .. } | Piece::Key { .. } | Piece::Door { .. }
                )
            {
                ui.spacing();
                crate::gui::muted(
                    ui,
                    if w.style.two_d {
                        "Offset (pixels)"
                    } else {
                        "Transform (relative to the spot)"
                    },
                );
                vector3(ui, "Offset", &mut visual.offset, 0.05, -64., 64.);
                if !w.style.two_d {
                    vector3(ui, "Rotation", &mut visual.rotation, 1., -360., 360.);
                    vector3(ui, "Scale", &mut visual.scale, 0.01, 0.01, 16.);
                }
            }
            ui.spacing();
            if ui.button("Remove piece") {
                delete = true;
            }
        }
    }
    if delete {
        remove(w, id);
        return;
    }
    if node != before {
        if !w.dirty || w.undo.last().is_none_or(|last| *last != w.style) {
            w.checkpoint();
        }
        w.style.nodes[index] = node;
        w.dirty = true;
    }
}

fn vector3(ui: &Ui, label: &str, value: &mut [f32; 3], speed: f32, min: f32, max: f32) {
    let id = crate::gui::field(ui, label);
    if crate::gui::Drag::new(id)
        .speed(speed)
        .range(min, max)
        .display_format("%.2f")
        .build_array(ui, value)
    {
        for v in value.iter_mut() {
            *v = v.clamp(min, max);
        }
    }
}

fn asset_combo(
    ui: &Ui,
    e: &Editor,
    label: &str,
    kind: crate::assets::Kind,
    value: &mut Option<Uuid>,
    none: &str,
) {
    let mut items: Vec<(Uuid, String)> = e
        .assets
        .index
        .usable()
        .filter(|r| r.meta.kind == kind)
        .map(|r| {
            (
                r.meta.id,
                r.path
                    .strip_prefix(&e.root)
                    .map(|p| p.display().to_string())
                    .unwrap_or_default(),
            )
        })
        .collect();
    items.sort_by(|a, b| a.1.cmp(&b.1));
    let current = value
        .and_then(|id| items.iter().find(|(i, _)| *i == id).map(|(_, n)| n.clone()))
        .unwrap_or_else(|| none.to_owned());
    if let Some(_combo) = ui.begin_combo(crate::gui::field(ui, label), &current) {
        if ui.selectable_config(none).selected(value.is_none()).build() {
            *value = None;
        }
        for (id, name) in &items {
            if ui
                .selectable_config(name)
                .selected(*value == Some(*id))
                .build()
            {
                *value = Some(*id);
            }
        }
        if items.is_empty() {
            ui.text_disabled("No asset of this kind in the project yet.");
        }
    }
}

fn piece_fields(ui: &Ui, e: &mut Editor, two_d: bool, piece: &mut Piece) {
    // Switching the kind keeps the node and its placement settings.
    let kinds = Piece::defaults(two_d);
    if let Some(_combo) = ui.begin_combo(crate::gui::field(ui, "Piece"), piece.label()) {
        for kind in kinds {
            if ui
                .selectable_config(kind.label())
                .selected(kind.label() == piece.label())
                .build()
                && kind.label() != piece.label()
            {
                *piece = kind;
            }
        }
    }
    match piece {
        Piece::Blockout { color, texture } => {
            let id = crate::gui::field(ui, "Colour");
            ui.color_edit3(id, color);
            asset_combo(
                ui,
                e,
                "Texture",
                crate::assets::Kind::Texture,
                texture,
                "None (colour only)",
            );
        }
        Piece::Mesh { asset } => {
            asset_combo(
                ui,
                e,
                "Mesh",
                crate::assets::Kind::EditableMesh,
                asset,
                "Choose a mesh",
            );
            crate::gui::muted(
                ui,
                "Model the mesh around the spot: floor level at 0, facing +Z into the room for walls and doors.",
            );
        }
        Piece::Actor { class } => {
            let model = e.object_model();
            let domain = if two_d {
                crate::reflection_schema::Domain::World2D
            } else {
                crate::reflection_schema::Domain::World3D
            };
            let mut classes: Vec<(String, String)> = model
                .as_deref()
                .map(|m| {
                    m.placeable()
                        .filter(|c| c.domain == domain)
                        .map(|c| (c.id.clone(), c.cpp_name.clone()))
                        .collect()
                })
                .unwrap_or_default();
            classes.sort_by(|a, b| a.1.cmp(&b.1));
            let current = class
                .as_deref()
                .and_then(|c| classes.iter().find(|(id, name)| id == c || name == c))
                .map(|(_, name)| name.clone())
                .unwrap_or_else(|| "Choose a class".into());
            if let Some(_combo) = ui.begin_combo(crate::gui::field(ui, "Class"), &current) {
                for (id, name) in &classes {
                    if ui
                        .selectable_config(name)
                        .selected(class.as_deref() == Some(id))
                        .build()
                    {
                        *class = Some(id.clone());
                    }
                }
                if classes.is_empty() {
                    ui.text_disabled("No placeable classes resolved for this project.");
                }
            }
        }
        Piece::Light {
            color,
            intensity,
            range,
        } => {
            let id = crate::gui::field(ui, "Colour");
            ui.color_edit3(id, color);
            if crate::gui::Drag::new(crate::gui::field(ui, "Intensity"))
                .speed(0.02)
                .range(0., 8.)
                .display_format("%.2f")
                .build(ui, intensity)
            {
                *intensity = intensity.clamp(0., 8.);
            }
            if crate::gui::Drag::new(crate::gui::field(ui, "Range"))
                .speed(0.05)
                .range(0.25, 64.)
                .display_format("%.2f")
                .build(ui, range)
            {
                *range = range.clamp(0.25, 64.);
            }
            crate::gui::muted(ui, "Baked: lights static geometry when lighting is built.");
        }
        Piece::Terrain { tile_set, terrain } => {
            asset_combo(
                ui,
                e,
                "TileSet",
                crate::assets::Kind::TileSet,
                tile_set,
                "Generator tiles",
            );
            let names = tileset_of(e, *tile_set)
                .map(|set| {
                    set.terrain_sets
                        .iter()
                        .flat_map(|s| s.terrains.iter().map(|t| t.name.clone()))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_else(|| {
                    vec![
                        crate::dungeon_bake::FLOOR_TERRAIN.into(),
                        crate::dungeon_bake::WALL_TERRAIN.into(),
                    ]
                });
            if let Some(_combo) = ui.begin_combo(crate::gui::field(ui, "Terrain"), terrain.as_str())
            {
                for name in &names {
                    if ui.selectable_config(name).selected(name == terrain).build() {
                        *terrain = name.clone();
                    }
                }
            }
        }
        Piece::Tile { tile_set, atlas } => {
            asset_combo(
                ui,
                e,
                "TileSet",
                crate::assets::Kind::TileSet,
                tile_set,
                "Generator tiles",
            );
            let _ = crate::gui::field(ui, "Tile");
            ui.text(format!("column {}, row {}", atlas[0], atlas[1]));
            if let Some(set) = tileset_of(e, *tile_set) {
                atlas_picker(ui, e, &set, atlas);
            } else {
                crate::gui::muted(
                    ui,
                    "Choose a TileSet, or generate once to create the built-in tiles.",
                );
            }
        }
        Piece::Key { color } | Piece::Door { color } => {
            let id = crate::gui::field(ui, if two_d { "Tint" } else { "Colour" });
            ui.color_edit3(id, color);
            crate::gui::muted(
                ui,
                "Works in game: the project gets the DungeonKey and DungeonDoor scripts on the first generation, and you can edit them. Actors get their lock number.",
            );
        }
    }
}

/// The TileSet a 2D piece uses: its own, or the built-in dungeon tiles.
fn tileset_of(e: &mut Editor, id: Option<Uuid>) -> Option<Arc<crate::tileset::TileSet>> {
    let id = match id {
        Some(id) => id,
        None => e
            .assets
            .index
            .usable()
            .find(|r| {
                r.meta.kind == crate::assets::Kind::TileSet
                    && r.path
                        .ends_with(crate::dungeon_bake::TILES_PATH.trim_start_matches("assets/"))
            })
            .map(|r| r.meta.id)?,
    };
    if let Some(set) = e.scene.tilesets.get(&id) {
        return Some(set.clone());
    }
    let set = Arc::new(crate::tileset::document(e.assets.index.resolve(id).ok()?).ok()?);
    // Cached on the map so the texture preview resolves like any tile layer's.
    e.scene.tilesets.insert(id, set.clone());
    Some(set)
}

fn atlas_picker(ui: &Ui, e: &mut Editor, set: &crate::tileset::TileSet, atlas: &mut [u8; 2]) {
    let Some(texture) = set.texture else {
        return;
    };
    let Some((image, texture_size)) = e.world2d.textures.get(texture) else {
        crate::gui::muted(ui, "Loading the atlas...");
        return;
    };
    let width = ui.content_region_avail()[0].min(300.);
    let zoom = (width / f32::from(texture_size[0].max(1))).clamp(0.5, 3.);
    let size = [
        f32::from(texture_size[0]) * zoom,
        f32::from(texture_size[1]) * zoom,
    ];
    let start = ui.cursor_screen_pos();
    ui.invisible_button("##dungeon-atlas", size);
    let draw = ui.get_window_draw_list();
    draw.add_image(image, start, [start[0] + size[0], start[1] + size[1]])
        .build();
    let tile = set.tile_size.map(|v| f32::from(v) * zoom);
    let cell = |a: [u8; 2]| {
        let t = set.texel([u16::from(a[0]), u16::from(a[1])]);
        let p = [start[0] + t[0] as f32 * zoom, start[1] + t[1] as f32 * zoom];
        (p, [p[0] + tile[0], p[1] + tile[1]])
    };
    let (a, b) = cell(*atlas);
    draw.add_rect(a, b, SELECTED).thickness(2.).build();
    if ui.is_item_hovered() {
        let mouse = ui.io().mouse_pos;
        let texel = [(mouse[0] - start[0]) / zoom, (mouse[1] - start[1]) / zoom];
        if let Some(under) = set.atlas_at(texel, texture_size) {
            let (a, b) = cell(under);
            draw.add_rect(a, b, [1., 1., 1., 0.7]).build();
            if ui.is_mouse_clicked(MouseButton::Left) {
                *atlas = under;
            }
        }
    }
}

/// Placements of each piece in the first dungeon of the map that uses this style.
fn placement_counts(e: &mut Editor, w: &Window) -> Option<std::collections::BTreeMap<Uuid, usize>> {
    let g = e
        .scene
        .dungeons
        .iter()
        .find(|g| g.style == Some(w.asset))?
        .clone();
    let layout = crate::dungeon::layout(&g, &e.assets.index).ok()?;
    Some(dungeon_style::counts(&dungeon_style::evaluate(
        &w.style, &layout, g.seed,
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removing_a_spot_removes_its_pieces_and_undo_restores_them() {
        let style = dungeon_style::builtin(false, None);
        let mut w = Window::new(Uuid::from_u128(1), style.clone());
        let floor = w.style.spot_node(SpotKind::Floor).unwrap().id;
        remove(&mut w, floor);
        assert!(w.style.spot_node(SpotKind::Floor).is_none());
        assert!(w.style.pieces(floor).is_empty());
        w.style.validate().unwrap();
        undo(&mut w);
        assert_eq!(w.style, style);
        redo(&mut w);
        assert!(w.style.spot_node(SpotKind::Floor).is_none());
    }

    #[test]
    fn framing_fits_every_node_in_the_canvas() {
        let mut w = Window::new(Uuid::from_u128(1), dungeon_style::builtin(false, None));
        let area = [600., 400.];
        frame_all(&mut w, area);
        for node in &w.style.nodes {
            let s = size(node);
            for k in 0..2 {
                let lo = (node.position[k] - w.pan[k]) * w.zoom;
                let hi = (node.position[k] + s[k] - w.pan[k]) * w.zoom;
                assert!(lo >= 0. && hi <= area[k], "{k}: {lo}..{hi}");
            }
        }
    }

    #[test]
    fn pieces_name_their_problems() {
        let e_style = dungeon_style::builtin(true, None);
        let blockout = Visual {
            piece: Piece::Blockout {
                color: [1.; 3],
                texture: None,
            },
            ..Visual::default()
        };
        // Without an editor the checks that need no project still apply.
        assert!(!blockout.piece.fits(e_style.two_d));
        assert_eq!(
            badges(&Visual {
                chance: 40,
                consume: false,
                filter: Filter::Rooms,
                limit: 3,
                ..Visual::default()
            }),
            "40% · then continues · rooms · max 3"
        );
    }
}
