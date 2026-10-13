//! Dungeon Styles: what each spot of a generated dungeon becomes.
//!
//! A style is a small graph. Each spot node (Floor, Wall, Door, ...) owns an ordered
//! list of piece nodes, ordered left to right on the canvas. For every spot of that
//! kind the pieces are tried in order: a piece places when its filter matches, its
//! chance rolls true and its limit is not used up; a placed piece that *consumes* the
//! spot ends the list for that spot. Pieces are built-in blockout geometry, meshes
//! merged into the dungeon's sections, actors of any placeable class, baked lights,
//! and in 2D terrains and single tiles.
use crate::assets;
use crate::dungeon::{Layout, Spot, SpotKind};
use crate::dungeon_random::Random;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

pub const VERSION: u32 = 1;
pub const MAX_NODES: usize = 256;

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum Filter {
    #[default]
    Anywhere,
    Rooms,
    Corridors,
}
impl Filter {
    pub const ALL: [Filter; 3] = [Filter::Anywhere, Filter::Rooms, Filter::Corridors];
    pub fn label(self) -> &'static str {
        match self {
            Filter::Anywhere => "Anywhere",
            Filter::Rooms => "Rooms only",
            Filter::Corridors => "Corridors only",
        }
    }
    fn accepts(self, spot: &Spot) -> bool {
        match self {
            Filter::Anywhere => true,
            Filter::Rooms => spot.room.is_some(),
            Filter::Corridors => spot.corridor,
        }
    }
}

/// What a piece puts at a spot.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "piece")]
pub enum Piece {
    /// The generator's own geometry for the spot: a floor tile, a wall panel, a
    /// wall top, a doorway beam or a corner pillar, in one colour or texture.
    Blockout {
        color: [f32; 3],
        #[serde(default, skip_serializing_if = "Option::is_none")]
        texture: Option<Uuid>,
    },
    /// An EditableMesh merged into the dungeon's geometry, so it costs no actor.
    Mesh {
        #[serde(default)]
        asset: Option<Uuid>,
    },
    /// An actor of a placeable class (C++, Blueprint or Lua), one per placement.
    Actor {
        #[serde(default)]
        class: Option<String>,
    },
    /// A baked point light, one actor per placement.
    Light {
        color: [f32; 3],
        intensity: f32,
        range: f32,
    },
    /// 2D: the terrain a Floor or Solid cell is painted with.
    Terrain {
        #[serde(default)]
        tile_set: Option<Uuid>,
        terrain: String,
    },
    /// 2D: one tile on a decoration layer above the floor.
    Tile {
        #[serde(default)]
        tile_set: Option<Uuid>,
        atlas: [u8; 2],
    },
}
impl Piece {
    pub fn label(&self) -> &'static str {
        match self {
            Piece::Blockout { .. } => "Blockout",
            Piece::Mesh { .. } => "Mesh",
            Piece::Actor { .. } => "Actor",
            Piece::Light { .. } => "Light",
            Piece::Terrain { .. } => "Terrain",
            Piece::Tile { .. } => "Tile",
        }
    }
    /// Whether a piece belongs to 3D or 2D dungeons (actors fit both).
    pub fn fits(&self, two_d: bool) -> bool {
        match self {
            Piece::Blockout { .. } | Piece::Mesh { .. } | Piece::Light { .. } => !two_d,
            Piece::Terrain { .. } | Piece::Tile { .. } => two_d,
            Piece::Actor { .. } => true,
        }
    }
    /// The pieces a new node can start as, for a 3D or 2D style.
    pub fn defaults(two_d: bool) -> Vec<Piece> {
        if two_d {
            vec![
                Piece::Terrain {
                    tile_set: None,
                    terrain: crate::dungeon_bake::FLOOR_TERRAIN.into(),
                },
                Piece::Tile {
                    tile_set: None,
                    atlas: [0, 0],
                },
                Piece::Actor { class: None },
            ]
        } else {
            vec![
                Piece::Blockout {
                    color: [0.6, 0.56, 0.5],
                    texture: None,
                },
                Piece::Mesh { asset: None },
                Piece::Actor { class: None },
                Piece::Light {
                    color: [1., 0.72, 0.42],
                    intensity: 1.,
                    range: 6.,
                },
            ]
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Visual {
    pub piece: Piece,
    /// Percentage of eligible spots that get this piece.
    pub chance: u8,
    /// A placed piece ends the list for its spot.
    pub consume: bool,
    pub filter: Filter,
    /// Most placements in one dungeon; 0 places everywhere the chance allows.
    pub limit: u16,
    /// Item spots: only items with this name (any case); empty takes every item.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub item: String,
    /// Local offset, rotation (degrees) and scale relative to the spot.
    pub offset: [f32; 3],
    pub rotation: [f32; 3],
    pub scale: [f32; 3],
}
impl Default for Visual {
    fn default() -> Self {
        Self {
            piece: Piece::Mesh { asset: None },
            chance: 100,
            consume: true,
            filter: Filter::Anywhere,
            limit: 0,
            item: String::new(),
            offset: [0.; 3],
            rotation: [0.; 3],
            scale: [1.; 3],
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "node")]
pub enum NodeKind {
    Spot { spot: SpotKind },
    Piece { parent: Uuid, visual: Visual },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Node {
    pub id: Uuid,
    /// Canvas position; pieces of one spot are tried from left to right.
    pub position: [f32; 2],
    pub kind: NodeKind,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Style {
    pub version: u32,
    /// Whether the style was made for 2D dungeons.
    pub two_d: bool,
    pub nodes: Vec<Node>,
}
impl Default for Style {
    fn default() -> Self {
        Self {
            version: VERSION,
            two_d: false,
            nodes: Vec::new(),
        }
    }
}

impl Style {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let style: Self = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        style.validate()?;
        Ok(style)
    }
    pub fn encode(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        serde_json::to_vec_pretty(self).map_err(|e| e.to_string())
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version != VERSION {
            return Err(format!("Dungeon Style version {VERSION} required"));
        }
        if self.nodes.len() > MAX_NODES {
            return Err(format!("A Dungeon Style holds at most {MAX_NODES} nodes"));
        }
        let mut ids = std::collections::BTreeSet::new();
        for node in &self.nodes {
            if node.id.is_nil() || !ids.insert(node.id) {
                return Err("Dungeon Style node ids must be unique".into());
            }
            if node.position.iter().any(|v| !v.is_finite()) {
                return Err("Dungeon Style node positions must be finite".into());
            }
        }
        for node in &self.nodes {
            if let NodeKind::Piece { parent, visual } = &node.kind {
                if !self
                    .nodes
                    .iter()
                    .any(|n| n.id == *parent && matches!(n.kind, NodeKind::Spot { .. }))
                {
                    return Err("Every piece must belong to a spot node".into());
                }
                if visual.chance > 100 {
                    return Err("Chance is a percentage, 0-100".into());
                }
                let finite = visual
                    .offset
                    .iter()
                    .chain(&visual.rotation)
                    .chain(&visual.scale)
                    .all(|v| v.is_finite());
                if !finite || visual.offset.iter().any(|v| v.abs() > 64.) {
                    return Err("Piece offsets must be finite and within ±64".into());
                }
                if visual.scale.iter().any(|v| *v <= 0. || *v > 16.) {
                    return Err("Piece scale must be above 0 and at most 16".into());
                }
                if let Piece::Blockout { color, .. } | Piece::Light { color, .. } = &visual.piece
                    && color.iter().any(|c| !(0. ..=1.).contains(c))
                {
                    return Err("Colours are 0-1 per channel".into());
                }
                if let Piece::Light {
                    intensity, range, ..
                } = &visual.piece
                    && (!(0. ..=8.).contains(intensity) || !(0.25..=64.).contains(range))
                {
                    return Err("Light intensity is 0-8 and range 0.25-64".into());
                }
            }
        }
        Ok(())
    }
    pub fn spot_node(&self, kind: SpotKind) -> Option<&Node> {
        self.nodes
            .iter()
            .find(|n| matches!(n.kind, NodeKind::Spot { spot } if spot == kind))
    }
    /// The pieces of a spot node in the order they are tried: left to right, then top
    /// to bottom.
    pub fn pieces(&self, spot: Uuid) -> Vec<(&Node, &Visual)> {
        let mut pieces: Vec<(&Node, &Visual)> = self
            .nodes
            .iter()
            .filter_map(|n| match &n.kind {
                NodeKind::Piece { parent, visual } if *parent == spot => Some((n, visual)),
                _ => None,
            })
            .collect();
        pieces.sort_by(|a, b| {
            a.0.position[0]
                .total_cmp(&b.0.position[0])
                .then(a.0.position[1].total_cmp(&b.0.position[1]))
                .then(a.0.id.cmp(&b.0.id))
        });
        pieces
    }
    /// Assets and classes the style refers to, for dependency checks and the editor.
    pub fn references(&self) -> Vec<Uuid> {
        let mut out = Vec::new();
        for node in &self.nodes {
            if let NodeKind::Piece { visual, .. } = &node.kind {
                match &visual.piece {
                    Piece::Blockout {
                        texture: Some(id), ..
                    }
                    | Piece::Mesh { asset: Some(id) }
                    | Piece::Terrain {
                        tile_set: Some(id), ..
                    }
                    | Piece::Tile {
                        tile_set: Some(id), ..
                    } => out.push(*id),
                    _ => {}
                }
            }
        }
        out.sort();
        out.dedup();
        out
    }
}

/// A short digest of a style's content.
pub fn digest(style: &Style) -> String {
    use sha2::{Digest, Sha256};
    let bytes = serde_json::to_vec(style).expect("style serializes");
    Sha256::digest(bytes)[..8]
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

// ---- asset storage -----------------------------------------------------------------------------

pub const FOLDER: &str = "assets/Dungeons/Styles";

/// A fresh, unused path for a style called `name`.
pub fn fresh_path(root: &std::path::Path, name: &str) -> String {
    let stem: String = name
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-' || *c == ' ')
        .collect::<String>()
        .trim()
        .replace(' ', "");
    let stem = if stem.is_empty() {
        "DungeonStyle".into()
    } else {
        stem
    };
    let mut path = format!("{FOLDER}/{stem}.epokasset");
    let mut n = 2;
    while root.join(&path).exists() {
        path = format!("{FOLDER}/{stem}{n}.epokasset");
        n += 1;
    }
    path
}
pub fn create(root: &std::path::Path, path: &str, style: &Style) -> Result<Uuid, String> {
    let id = Uuid::new_v4();
    let source = style.encode()?;
    let package = assets::Package {
        meta: assets::Metadata {
            version: 1,
            id,
            kind: assets::Kind::DungeonStyle,
            importer_version: 1,
            source: path.into(),
            source_hash: assets::hash(&source),
            settings: crate::import_settings::Settings::Authored,
            extra: Default::default(),
        },
        source,
    };
    assets::atomic_write(&assets::inside(root, path)?, &package.bytes()?, None)?;
    Ok(id)
}
pub fn document(record: &assets::Record) -> Result<Style, String> {
    if record.meta.kind != assets::Kind::DungeonStyle {
        return Err("Select a Dungeon Style asset".into());
    }
    Style::parse(&assets::Package::load(&record.path)?.source)
}
/// Saves `style` over `record`, refusing a package changed or replaced since it was read.
pub fn save(record: &assets::Record, style: &Style) -> Result<String, String> {
    let mut package = assets::Package::load(&record.path)?;
    if package.meta.kind != assets::Kind::DungeonStyle || package.meta.id != record.meta.id {
        return Err("The Dungeon Style was replaced; reopen it before editing".into());
    }
    package.source = style.encode()?;
    package.meta.source_hash = assets::hash(&package.source);
    let bytes = package.bytes()?;
    assets::atomic_write(&record.path, &bytes, Some(&record.revision))?;
    Ok(assets::hash(&bytes))
}

/// Stable node identities for built-in styles.
fn node_id(seed: &str) -> Uuid {
    crate::dungeon::derived_id(
        Uuid::from_u128(0xd0d0_5ce9_e000_0000_0000_0000_0000_0001),
        seed,
    )
}

fn spot_node(spot: SpotKind, row: usize) -> Node {
    Node {
        id: node_id(&format!("spot-{}", spot.label())),
        position: [0., row as f32 * 140.],
        kind: NodeKind::Spot { spot },
    }
}
fn piece_node(spot: SpotKind, row: usize, column: usize, visual: Visual) -> Node {
    Node {
        id: node_id(&format!("piece-{}-{column}", spot.label())),
        position: [260. + column as f32 * 230., row as f32 * 140.],
        kind: NodeKind::Piece {
            parent: node_id(&format!("spot-{}", spot.label())),
            visual,
        },
    }
}

pub const ROOM_FLOOR: [f32; 3] = [0.46, 0.43, 0.40];
pub const CORRIDOR_FLOOR: [f32; 3] = [0.36, 0.34, 0.33];
pub const WALL: [f32; 3] = [0.66, 0.61, 0.54];
pub const WALL_TOP: [f32; 3] = [0.30, 0.28, 0.30];
pub const DOORWAY: [f32; 3] = [0.48, 0.33, 0.21];
pub const KEY: [f32; 3] = [0.92, 0.74, 0.24];
pub const LOCK: [f32; 3] = [0.72, 0.18, 0.16];
pub const ONE_WAY: [f32; 3] = [0.22, 0.46, 0.78];
pub const ITEM: [f32; 3] = [0.30, 0.66, 0.44];
/// Built-in dungeon tiles drawn as markers for recipe spots.
pub const MARKER_TILES: [(SpotKind, [u8; 2]); 4] = [
    (SpotKind::Key, [4, 0]),
    (SpotKind::Lock, [5, 0]),
    (SpotKind::OneWay, [6, 0]),
    (SpotKind::Item, [7, 0]),
];

/// The style a generator without one uses: blockout geometry in 3D, the built-in
/// dungeon tiles (or `tile_set`) in 2D. It is also what a new style starts from.
/// Recipe spots get markers: blockout pedestals, gates and plates in 3D, marker tiles
/// of the built-in dungeon tiles in 2D (left out when `tile_set` replaces them).
///
/// Its pieces do not consume their spots: a piece added to their right decorates on
/// top of them, and one added to their left that consumes replaces them.
pub fn builtin(two_d: bool, tile_set: Option<Uuid>) -> Style {
    let blockout = |color: [f32; 3], filter: Filter| Visual {
        piece: Piece::Blockout {
            color,
            texture: None,
        },
        filter,
        consume: false,
        ..Visual::default()
    };
    let mut nodes = Vec::new();
    let rows: Vec<(SpotKind, Vec<Visual>)> = if two_d {
        let terrain = |name: &str| Visual {
            piece: Piece::Terrain {
                tile_set,
                terrain: name.into(),
            },
            consume: false,
            ..Visual::default()
        };
        let mut rows = vec![
            (
                SpotKind::Floor,
                vec![terrain(crate::dungeon_bake::FLOOR_TERRAIN)],
            ),
            (
                SpotKind::Solid,
                vec![terrain(crate::dungeon_bake::WALL_TERRAIN)],
            ),
        ];
        if tile_set.is_none() {
            for (spot, atlas) in MARKER_TILES {
                rows.push((
                    spot,
                    vec![Visual {
                        piece: Piece::Tile {
                            tile_set: None,
                            atlas,
                        },
                        consume: false,
                        ..Visual::default()
                    }],
                ));
            }
        }
        rows
    } else {
        vec![
            (
                SpotKind::Floor,
                vec![
                    blockout(ROOM_FLOOR, Filter::Rooms),
                    blockout(CORRIDOR_FLOOR, Filter::Corridors),
                ],
            ),
            (SpotKind::Wall, vec![blockout(WALL, Filter::Anywhere)]),
            (SpotKind::Solid, vec![blockout(WALL_TOP, Filter::Anywhere)]),
            (SpotKind::Doorway, vec![blockout(DOORWAY, Filter::Anywhere)]),
            (SpotKind::Key, vec![blockout(KEY, Filter::Anywhere)]),
            (SpotKind::Lock, vec![blockout(LOCK, Filter::Anywhere)]),
            (SpotKind::OneWay, vec![blockout(ONE_WAY, Filter::Anywhere)]),
            (SpotKind::Item, vec![blockout(ITEM, Filter::Anywhere)]),
        ]
    };
    for (row, (spot, visuals)) in rows.into_iter().enumerate() {
        nodes.push(spot_node(spot, row));
        for (column, visual) in visuals.into_iter().enumerate() {
            nodes.push(piece_node(spot, row, column, visual));
        }
    }
    Style {
        version: VERSION,
        two_d,
        nodes,
    }
}

/// Points a 2D style's built-in marker tiles at the project's copy of the built-in
/// dungeon tiles, so they keep drawing the markers when a generator uses its own TileSet.
pub fn pin_markers(style: &mut Style, tiles: Uuid) {
    let spots: Vec<Uuid> = MARKER_TILES
        .iter()
        .filter_map(|(kind, _)| style.spot_node(*kind).map(|n| n.id))
        .collect();
    for node in &mut style.nodes {
        if let NodeKind::Piece { parent, visual } = &mut node.kind
            && spots.contains(parent)
            && let Piece::Tile { tile_set, atlas } = &mut visual.piece
            && tile_set.is_none()
            && MARKER_TILES.iter().any(|(_, a)| a == atlas)
        {
            *tile_set = Some(tiles);
        }
    }
}

/// Removes the built-in marker tile pieces, for TileSets without them.
pub fn strip_markers(style: &mut Style) {
    let spots: Vec<Uuid> = MARKER_TILES
        .iter()
        .filter_map(|(kind, _)| style.spot_node(*kind).map(|n| n.id))
        .collect();
    style.nodes.retain(|node| {
        !matches!(
            &node.kind,
            NodeKind::Piece { parent, visual: Visual { piece: Piece::Tile { tile_set: None, atlas }, .. } }
                if spots.contains(parent) && MARKER_TILES.iter().any(|(_, a)| a == atlas)
        )
    });
}

// ---- evaluation ------------------------------------------------------------------------------

/// One piece placed at one spot.
#[derive(Clone, Debug, PartialEq)]
pub struct Placement {
    pub spot: Spot,
    pub node: Uuid,
    pub visual: Visual,
}

/// Decides every placement of `style` over `layout`. Each piece draws from its own
/// stream derived from the seed and its node id, so editing one piece never reshuffles
/// another.
pub fn evaluate(style: &Style, layout: &Layout, seed: u32) -> Vec<Placement> {
    let spots = crate::dungeon::spots(layout);
    let mut out = Vec::new();
    for kind in SpotKind::ALL {
        let Some(spot_node) = style.spot_node(kind) else {
            continue;
        };
        let candidates: Vec<&Spot> = spots.iter().filter(|s| s.kind == kind).collect();
        let mut consumed = vec![false; candidates.len()];
        for (node, visual) in style.pieces(spot_node.id) {
            if !visual.piece.fits(style.two_d) {
                continue;
            }
            let salt = node.id.as_u128();
            let mut random = Random::stream(seed, (salt ^ (salt >> 64) ^ (salt >> 32)) as u32);
            let mut chosen: Vec<usize> = Vec::new();
            for (index, spot) in candidates.iter().enumerate() {
                // Every candidate draws, so the outcome at one spot does not depend on
                // which other spots were consumed.
                let roll = random.chance(u32::from(visual.chance));
                if !consumed[index]
                    && visual.filter.accepts(spot)
                    && named(visual, spot, layout)
                    && roll
                {
                    chosen.push(index);
                }
            }
            if visual.limit > 0 && chosen.len() > usize::from(visual.limit) {
                // Spread a limited piece over the dungeon instead of its first cells.
                for i in (1..chosen.len()).rev() {
                    let j = random.below(i as u32 + 1) as usize;
                    chosen.swap(i, j);
                }
                chosen.truncate(usize::from(visual.limit));
                chosen.sort_unstable();
            }
            for index in chosen {
                if visual.consume {
                    consumed[index] = true;
                }
                out.push(Placement {
                    spot: *candidates[index],
                    node: node.id,
                    visual: visual.clone(),
                });
            }
        }
    }
    out
}

/// Whether a piece naming an item accepts this spot.
fn named(visual: &Visual, spot: &Spot, layout: &Layout) -> bool {
    visual.item.is_empty()
        || spot.kind != SpotKind::Item
        || layout
            .items
            .get(usize::from(spot.tag))
            .is_some_and(|i| i.name.eq_ignore_ascii_case(visual.item.trim()))
}

/// Placement counts per piece node, for the editor.
pub fn counts(placements: &[Placement]) -> BTreeMap<Uuid, usize> {
    let mut out = BTreeMap::new();
    for p in placements {
        *out.entry(p.node).or_insert(0) += 1;
    }
    out
}

// ---- 3D frames ----------------------------------------------------------------------------

/// The local frame of a spot in root-local world units: +Y up, +Z facing into the floor
/// (walls, doors, corners) or into the room (doorways), origin at floor level.
pub fn frame(spot: &Spot, grid: &crate::dungeon_bake::Frame) -> crate::transform::Matrix {
    let c = grid.cell;
    let [x, y] = spot.cell;
    let center = grid.center(spot.cell);
    let (origin, forward) = match (spot.kind, spot.side) {
        (SpotKind::Wall | SpotKind::Door | SpotKind::Lock | SpotKind::OneWay, Some(side)) => {
            let d = side.delta();
            (
                [
                    center[0] + d[0] as f32 * c * 0.5,
                    0.,
                    center[2] + d[1] as f32 * c * 0.5,
                ],
                [-d[0] as f32, -d[1] as f32],
            )
        }
        (SpotKind::Doorway, Some(side)) => {
            let d = side.delta();
            (center, [d[0] as f32, d[1] as f32])
        }
        (SpotKind::Corner, Some(side)) => {
            let (a, b) = (side.delta(), side.next().delta());
            (
                [
                    grid.x(x) + c * 0.5 * (1 + a[0] + b[0]) as f32,
                    0.,
                    grid.z(y) + c * 0.5 * (1 + a[1] + b[1]) as f32,
                ],
                [-a[0] as f32, -a[1] as f32],
            )
        }
        _ => (center, [0., 1.]),
    };
    // Right-handed: X = Y × Z.
    let z = [forward[0], 0., forward[1]];
    let xaxis = [z[2], 0., -z[0]];
    let mut m = crate::transform::Matrix::IDENTITY;
    for row in 0..3 {
        m.0[row][0] = xaxis[row];
        m.0[row][1] = if row == 1 { 1. } else { 0. };
        m.0[row][2] = z[row];
        m.0[row][3] = origin[row];
    }
    m
}

/// The full transform of a placement: spot frame, then the piece's own offset,
/// rotation and scale.
pub fn transform(
    placement: &Placement,
    grid: &crate::dungeon_bake::Frame,
) -> crate::transform::Matrix {
    let v = &placement.visual;
    frame(&placement.spot, grid)
        .compose(crate::transform::Matrix::trs(v.offset, v.rotation, v.scale))
}

/// Which side a 2D tile at a wall spot lands on: the solid cell behind the wall.
pub fn tile_cell(spot: &Spot) -> [i32; 2] {
    match (spot.kind, spot.side) {
        (SpotKind::Wall, Some(side)) => {
            let d = side.delta();
            [spot.cell[0] + d[0], spot.cell[1] + d[1]]
        }
        _ => spot.cell,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dungeon::Generator;

    fn layout() -> Layout {
        crate::dungeon::layout(
            &Generator {
                seed: 3,
                ..Generator::default()
            },
            &Default::default(),
        )
        .unwrap()
    }

    #[test]
    fn builtin_styles_round_trip_and_validate() {
        for two_d in [false, true] {
            let style = builtin(two_d, None);
            let bytes = style.encode().unwrap();
            assert_eq!(Style::parse(&bytes).unwrap(), style);
        }
        let mut broken = builtin(false, None);
        if let NodeKind::Piece { parent, .. } = &mut broken.nodes[1].kind {
            *parent = Uuid::from_u128(9);
        }
        assert!(broken.validate().unwrap_err().contains("belong to a spot"));
    }

    #[test]
    fn pieces_try_in_canvas_order_and_consume_spots() {
        let layout = layout();
        let style = builtin(false, None);
        let placements = evaluate(&style, &layout, 1);
        // Every floor cell gets exactly one floor piece: rooms one colour, corridors
        // the other.
        let floors = placements
            .iter()
            .filter(|p| p.spot.kind == SpotKind::Floor)
            .count();
        assert_eq!(floors, layout.walkable_cells());
        for p in placements.iter().filter(|p| p.spot.kind == SpotKind::Floor) {
            let Piece::Blockout { color, .. } = p.visual.piece else {
                panic!()
            };
            assert_eq!(color == CORRIDOR_FLOOR, p.spot.corridor);
        }
        // A first piece with chance 100 that consumes hides everything after it.
        let mut style = builtin(false, None);
        let wall = style.spot_node(SpotKind::Wall).unwrap().id;
        style.nodes.push(Node {
            id: Uuid::from_u128(77),
            position: [-100., 0.],
            kind: NodeKind::Piece {
                parent: wall,
                visual: Visual {
                    piece: Piece::Mesh { asset: None },
                    ..Visual::default()
                },
            },
        });
        let placements = evaluate(&style, &layout, 1);
        assert!(
            placements
                .iter()
                .filter(|p| p.spot.kind == SpotKind::Wall)
                .all(|p| p.node == Uuid::from_u128(77))
        );
    }

    #[test]
    fn chance_and_limit_are_seeded_and_bounded() {
        let layout = layout();
        let mut style = builtin(false, None);
        let corner = Node {
            id: Uuid::from_u128(5),
            position: [0., 900.],
            kind: NodeKind::Spot {
                spot: SpotKind::Corner,
            },
        };
        style.nodes.push(corner);
        style.nodes.push(Node {
            id: Uuid::from_u128(6),
            position: [300., 900.],
            kind: NodeKind::Piece {
                parent: Uuid::from_u128(5),
                visual: Visual {
                    piece: Piece::Light {
                        color: [1.; 3],
                        intensity: 1.,
                        range: 6.,
                    },
                    chance: 50,
                    limit: 3,
                    ..Visual::default()
                },
            },
        });
        style.validate().unwrap();
        let a = evaluate(&style, &layout, 9);
        assert_eq!(a, evaluate(&style, &layout, 9));
        let lights = a.iter().filter(|p| p.node == Uuid::from_u128(6)).count();
        assert!(lights > 0 && lights <= 3, "{lights}");
        // Other pieces keep their placements when this one changes.
        let floors = |p: &[Placement]| {
            p.iter()
                .filter(|p| p.spot.kind == SpotKind::Floor)
                .cloned()
                .collect::<Vec<_>>()
        };
        if let NodeKind::Piece { visual, .. } = &mut style.nodes.last_mut().unwrap().kind {
            visual.chance = 10;
        }
        assert_eq!(floors(&a), floors(&evaluate(&style, &layout, 9)));
    }

    #[test]
    fn frames_face_into_the_floor() {
        let layout = layout();
        let grid = crate::dungeon_bake::Frame {
            size: layout.size,
            cell: 2.,
        };
        for spot in crate::dungeon::spots(&layout) {
            let m = frame(&spot, &grid);
            let ahead = m.point([0., 0., 0.5]);
            let center = grid.center(spot.cell);
            match spot.kind {
                SpotKind::Wall | SpotKind::Door => {
                    // Half a unit ahead of a wall is closer to its cell's centre.
                    let before = m.point([0.; 3]);
                    let d = |p: [f32; 3]| (p[0] - center[0]).abs() + (p[2] - center[2]).abs();
                    assert!(d(ahead) < d(before), "{spot:?}");
                }
                SpotKind::Floor | SpotKind::Solid => {
                    assert_eq!(m.point([0.; 3]), center);
                }
                _ => {}
            }
            // Rotations keep +Y up and unit axes.
            assert_eq!(m.vector([0., 1., 0.]), [0., 1., 0.]);
        }
    }
}
