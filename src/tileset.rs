//! TileSet assets (World2D initiative, phase 2): one texture atlas cut into a grid of
//! tiles, with per-tile collision, one-way flags, animation, terrain peering bits, custom
//! data and saved patterns. A TileSet is an authored `.epokasset` (like a Terrain): maps
//! reference it from TileMapLayer components, and every bank that draws one cooks its
//! tile table next to the cells (see `runtime/world2d_tiles.hpp`).
//!
//! Units are texels and cells. A tile is addressed by its atlas column and row; its frames
//! (when animated) continue to the right of it, `tile_size + separation` texels apart.
use crate::assets;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    sync::Arc,
};
use uuid::Uuid;

pub const VERSION: u32 = 1;
/// Cooked cells keep the tile index in 13 bits.
pub const MAX_TILES: usize = 4096;
pub const MAX_CUSTOM_LAYERS: usize = 16;
pub const MAX_TERRAIN_SETS: usize = 8;
pub const MAX_TERRAINS: usize = 16;
pub const MAX_PATTERNS: usize = 64;

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum Collision {
    #[default]
    None,
    /// The whole cell.
    Full,
    /// `Tile::rect` inside the cell.
    Rect,
}

/// Which neighbours a terrain tile matches.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum TerrainMode {
    /// Sides and corners: the 47-tile "blob" layout.
    #[default]
    CornersAndSides,
    /// Sides only: 16 tiles.
    Sides,
    /// Corners only: 16 tiles.
    Corners,
}
impl TerrainMode {
    pub const ALL: [TerrainMode; 3] = [
        TerrainMode::CornersAndSides,
        TerrainMode::Sides,
        TerrainMode::Corners,
    ];
    pub fn label(self) -> &'static str {
        match self {
            TerrainMode::CornersAndSides => "Corners and sides",
            TerrainMode::Sides => "Sides",
            TerrainMode::Corners => "Corners",
        }
    }
    /// Whether the peering bit in direction `d` takes part in matching.
    pub fn uses(self, d: usize) -> bool {
        match self {
            TerrainMode::CornersAndSides => true,
            TerrainMode::Sides => d.is_multiple_of(2),
            TerrainMode::Corners => d % 2 == 1,
        }
    }
}

/// Neighbour directions of a cell, in peering-bit order: sides on even indices, corners on
/// odd ones, clockwise from the right side.
pub const DIRECTIONS: [[i32; 2]; 8] = [
    [1, 0],
    [1, 1],
    [0, 1],
    [-1, 1],
    [-1, 0],
    [-1, -1],
    [0, -1],
    [1, -1],
];
pub const DIRECTION_NAMES: [&str; 8] = [
    "right side",
    "bottom-right corner",
    "bottom side",
    "bottom-left corner",
    "left side",
    "top-left corner",
    "top side",
    "top-right corner",
];

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Terrain {
    pub name: String,
    pub color: [u8; 3],
}
impl Default for Terrain {
    fn default() -> Self {
        Self {
            name: "Terrain".into(),
            color: [96, 196, 96],
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct TerrainSet {
    pub name: String,
    pub mode: TerrainMode,
    pub terrains: Vec<Terrain>,
}
impl Default for TerrainSet {
    fn default() -> Self {
        Self {
            name: "Terrains".into(),
            mode: TerrainMode::CornersAndSides,
            terrains: vec![Terrain::default()],
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum CustomKind {
    #[default]
    Int,
    Bool,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct CustomLayer {
    pub name: String,
    pub kind: CustomKind,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Tile {
    /// Column and row in the atlas grid.
    pub atlas: [u8; 2],
    pub collision: Collision,
    /// Collision rectangle x, y, width, height in texels, when `collision` is `Rect`.
    pub rect: [u8; 4],
    /// Blocks only bodies falling onto its top edge.
    pub one_way: bool,
    /// Relative weight when painting randomly or picking between equal terrain matches.
    pub probability: f32,
    /// Terrain set and the terrain of the tile's centre.
    pub terrain_set: Option<u8>,
    pub terrain: Option<u8>,
    /// The terrain each neighbour direction connects to (see `DIRECTIONS`).
    pub peering: [Option<u8>; 8],
    /// Animation frames (1 = still), frames per row before wrapping (0 = never) and speed.
    pub frames: u8,
    pub columns: u8,
    pub fps: f32,
    /// One value per custom data layer.
    pub custom: Vec<i32>,
}
impl Default for Tile {
    fn default() -> Self {
        Self {
            atlas: [0, 0],
            collision: Collision::None,
            rect: [0, 0, 16, 16],
            one_way: false,
            probability: 1.,
            terrain_set: None,
            terrain: None,
            peering: [None; 8],
            frames: 1,
            columns: 0,
            fps: 8.,
            custom: Vec::new(),
        }
    }
}
impl Tile {
    pub fn new(atlas: [u8; 2]) -> Self {
        Self {
            atlas,
            ..Default::default()
        }
    }
    /// Atlas cell of animation frame `frame`.
    pub fn frame_atlas(&self, frame: u8) -> [u16; 2] {
        let frame = u16::from(frame);
        if self.columns == 0 {
            [u16::from(self.atlas[0]) + frame, u16::from(self.atlas[1])]
        } else {
            let columns = u16::from(self.columns);
            [
                u16::from(self.atlas[0]) + frame % columns,
                u16::from(self.atlas[1]) + frame / columns,
            ]
        }
    }
}

/// Flag bits of a painted cell, shared by maps and patterns.
pub const FLIP_H: u8 = 1;
pub const FLIP_V: u8 = 2;
pub const TRANSPOSE: u8 = 4;

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PatternCell {
    pub offset: [u8; 2],
    pub atlas: [u8; 2],
    pub flags: u8,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Pattern {
    pub name: String,
    pub cells: Vec<PatternCell>,
}
impl Pattern {
    pub fn size(&self) -> [u8; 2] {
        self.cells.iter().fold([0, 0], |size, c| {
            [
                size[0].max(c.offset[0].saturating_add(1)),
                size[1].max(c.offset[1].saturating_add(1)),
            ]
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct TileSet {
    pub version: u32,
    pub texture: Option<Uuid>,
    pub tile_size: [u16; 2],
    /// Texels before the first column and row.
    pub margin: [u16; 2],
    /// Texels between two columns and two rows.
    pub separation: [u16; 2],
    pub tiles: Vec<Tile>,
    pub terrain_sets: Vec<TerrainSet>,
    pub custom_layers: Vec<CustomLayer>,
    pub patterns: Vec<Pattern>,
}
impl Default for TileSet {
    fn default() -> Self {
        Self {
            version: VERSION,
            texture: None,
            tile_size: [16, 16],
            margin: [0, 0],
            separation: [0, 0],
            tiles: Vec::new(),
            terrain_sets: Vec::new(),
            custom_layers: Vec::new(),
            patterns: Vec::new(),
        }
    }
}
impl TileSet {
    /// Top-left texel of an atlas cell.
    pub fn texel(&self, atlas: [u16; 2]) -> [u32; 2] {
        [0, 1].map(|k| {
            u32::from(self.margin[k])
                + u32::from(atlas[k])
                    * (u32::from(self.tile_size[k]) + u32::from(self.separation[k]))
        })
    }
    /// Columns and rows of whole tiles that fit in a texture of `size`.
    pub fn grid(&self, size: [u16; 2]) -> [u16; 2] {
        [0, 1].map(|k| {
            let step = u32::from(self.tile_size[k].max(1)) + u32::from(self.separation[k]);
            let room = u32::from(size[k]).saturating_sub(u32::from(self.margin[k]))
                + u32::from(self.separation[k]);
            (room / step).min(255) as u16
        })
    }
    /// The atlas cell under texel `p`, if it is inside a tile.
    pub fn atlas_at(&self, p: [f32; 2], size: [u16; 2]) -> Option<[u8; 2]> {
        let grid = self.grid(size);
        let mut out = [0u8; 2];
        for k in 0..2 {
            let local = p[k] - f32::from(self.margin[k]);
            if local < 0. {
                return None;
            }
            let step = f32::from(self.tile_size[k].max(1)) + f32::from(self.separation[k]);
            let cell = (local / step).floor();
            if cell >= f32::from(grid[k]) || local - cell * step >= f32::from(self.tile_size[k]) {
                return None;
            }
            out[k] = cell as u8;
        }
        Some(out)
    }
    pub fn tile(&self, atlas: [u8; 2]) -> Option<&Tile> {
        self.tiles.iter().find(|t| t.atlas == atlas)
    }
    pub fn tile_mut(&mut self, atlas: [u8; 2]) -> Option<&mut Tile> {
        self.tiles.iter_mut().find(|t| t.atlas == atlas)
    }
    /// Keeps tiles in row-major atlas order, so cooked indices are stable.
    pub fn sort(&mut self) {
        self.tiles.sort_by_key(|t| (t.atlas[1], t.atlas[0]));
    }
    /// Adds a tile for every atlas cell of `texture` holding at least one visible texel,
    /// keeping existing tiles. Returns how many were added.
    pub fn create_opaque_tiles(&mut self, texture: &crate::texture::Data) -> usize {
        let grid = self.grid([texture.width, texture.height]);
        let mut added = 0;
        for row in 0..grid[1] {
            for column in 0..grid[0] {
                let atlas = [column as u8, row as u8];
                if self.tile(atlas).is_some() || self.tiles.len() >= MAX_TILES {
                    continue;
                }
                let origin = self.texel([column, row]);
                let opaque = (0..u32::from(self.tile_size[1])).any(|y| {
                    (0..u32::from(self.tile_size[0])).any(|x| {
                        let at = ((origin[1] + y) * u32::from(texture.width) + origin[0] + x)
                            as usize
                            * 4
                            + 3;
                        texture.rgba.get(at).is_some_and(|a| *a > 0)
                    })
                });
                if opaque {
                    let mut tile = Tile::new(atlas);
                    tile.rect = [
                        0,
                        0,
                        self.tile_size[0].min(255) as u8,
                        self.tile_size[1].min(255) as u8,
                    ];
                    tile.custom = vec![0; self.custom_layers.len()];
                    self.tiles.push(tile);
                    added += 1;
                }
            }
        }
        self.sort();
        added
    }
    /// Keeps per-tile custom values aligned with the declared layers.
    pub fn normalize(&mut self) {
        let layers = self.custom_layers.len();
        for tile in &mut self.tiles {
            tile.custom.resize(layers, 0);
            if tile.frames == 0 {
                tile.frames = 1;
            }
        }
        self.sort();
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version > VERSION {
            return Err(format!(
                "TileSet version {} is newer than this editor",
                self.version
            ));
        }
        if !(1..=256).contains(&self.tile_size[0]) || !(1..=256).contains(&self.tile_size[1]) {
            return Err("Tile size must be between 1 and 256 texels".into());
        }
        if self.tiles.len() > MAX_TILES {
            return Err(format!("A TileSet holds at most {MAX_TILES} tiles"));
        }
        if self.custom_layers.len() > MAX_CUSTOM_LAYERS {
            return Err(format!(
                "A TileSet holds at most {MAX_CUSTOM_LAYERS} custom data layers"
            ));
        }
        if self.terrain_sets.len() > MAX_TERRAIN_SETS
            || self
                .terrain_sets
                .iter()
                .any(|s| s.terrains.len() > MAX_TERRAINS)
        {
            return Err(format!(
                "A TileSet holds at most {MAX_TERRAIN_SETS} terrain sets of {MAX_TERRAINS} terrains"
            ));
        }
        if self.patterns.len() > MAX_PATTERNS {
            return Err(format!("A TileSet holds at most {MAX_PATTERNS} patterns"));
        }
        let mut seen = BTreeSet::new();
        for tile in &self.tiles {
            if !seen.insert(tile.atlas) {
                return Err(format!(
                    "Tile ({}, {}) is listed twice",
                    tile.atlas[0], tile.atlas[1]
                ));
            }
            if tile.frames == 0 || !tile.fps.is_finite() || tile.fps < 0. {
                return Err(format!(
                    "Tile ({}, {}) has an invalid animation",
                    tile.atlas[0], tile.atlas[1]
                ));
            }
            if !tile.probability.is_finite() || tile.probability < 0. {
                return Err(format!(
                    "Tile ({}, {}) has an invalid probability",
                    tile.atlas[0], tile.atlas[1]
                ));
            }
            if let Some(set) = tile.terrain_set {
                let Some(terrains) = self.terrain_sets.get(usize::from(set)) else {
                    return Err(format!(
                        "Tile ({}, {}) uses a missing terrain set",
                        tile.atlas[0], tile.atlas[1]
                    ));
                };
                let count = terrains.terrains.len();
                if tile
                    .terrain
                    .iter()
                    .chain(tile.peering.iter().flatten())
                    .any(|t| usize::from(*t) >= count)
                {
                    return Err(format!(
                        "Tile ({}, {}) uses a missing terrain",
                        tile.atlas[0], tile.atlas[1]
                    ));
                }
            }
        }
        Ok(())
    }
    /// Checks the tiles against the atlas texture they cut: every frame must fit.
    pub fn validate_texture(&self, size: [u16; 2]) -> Result<(), String> {
        for tile in &self.tiles {
            for frame in 0..tile.frames {
                let atlas = tile.frame_atlas(frame);
                let texel = self.texel(atlas);
                if texel[0] + u32::from(self.tile_size[0]) > u32::from(size[0])
                    || texel[1] + u32::from(self.tile_size[1]) > u32::from(size[1])
                {
                    return Err(format!(
                        "Tile ({}, {}) frame {} lies outside the {}x{} texture",
                        tile.atlas[0], tile.atlas[1], frame, size[0], size[1]
                    ));
                }
            }
        }
        Ok(())
    }
    pub fn encode(&self) -> Result<Vec<u8>, String> {
        serde_json::to_vec_pretty(self).map_err(|e| e.to_string())
    }
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let mut set: TileSet =
            serde_json::from_slice(bytes).map_err(|e| format!("TileSet: {e}"))?;
        set.normalize();
        set.validate()?;
        Ok(set)
    }
}

// ---- asset storage -------------------------------------------------------------------

pub fn fresh_path(name: &str) -> String {
    let stem: String = name
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .collect();
    let stem = if stem.is_empty() {
        "TileSet".into()
    } else {
        stem
    };
    format!(
        "assets/TileSets/{stem}-{}.epokasset",
        &Uuid::new_v4().simple().to_string()[..8]
    )
}
pub fn create(root: &Path, path: &str, set: &TileSet) -> Result<Uuid, String> {
    set.validate()?;
    let id = Uuid::new_v4();
    let source = set.encode()?;
    let package = assets::Package {
        meta: assets::Metadata {
            version: 1,
            id,
            kind: assets::Kind::TileSet,
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
pub fn document(record: &assets::Record) -> Result<TileSet, String> {
    if record.meta.kind != assets::Kind::TileSet {
        return Err("Select a TileSet asset".into());
    }
    TileSet::parse(&assets::Package::load(&record.path)?.source)
}
/// Saves an edited TileSet over its asset. Returns the new revision.
pub fn save(record: &assets::Record, set: &TileSet) -> Result<String, String> {
    set.validate()?;
    let mut package = assets::Package::load(&record.path)?;
    if package.meta.kind != assets::Kind::TileSet || package.meta.id != record.meta.id {
        return Err("The TileSet asset was replaced; reload before editing".into());
    }
    package.source = set.encode()?;
    package.meta.source_hash = assets::hash(&package.source);
    let bytes = package.bytes()?;
    assets::atomic_write(&record.path, &bytes, None)?;
    Ok(assets::hash(&bytes))
}
/// Every TileSet the scene's tile layers reference.
pub fn ids(scene: &crate::scene::Scene) -> BTreeSet<Uuid> {
    scene
        .actors
        .iter()
        .flat_map(crate::world2d::tileset_ids)
        .collect()
}
/// Loads the TileSets the scene references into `scene.tilesets`.
pub fn resolve(scene: &mut crate::scene::Scene, index: &assets::Index) -> Result<(), String> {
    let wanted = ids(scene);
    scene.tilesets.retain(|id, _| wanted.contains(id));
    let mut errors = Vec::new();
    for id in wanted {
        match index.resolve(id).and_then(document) {
            Ok(set) => {
                scene.tilesets.insert(id, Arc::new(set));
            }
            Err(error) => {
                scene.tilesets.remove(&id);
                errors.push(format!("TileSet {id}: {error}"));
            }
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}

// ---- terrains ----------------------------------------------------------------------------

/// The peering bits a cell of terrain `centre` needs, given its neighbours' terrains
/// (`around[d]` in `DIRECTIONS` order). A side connects to the neighbour's terrain; a
/// corner is the cell's own terrain only when both sides and the diagonal share it, and
/// otherwise the terrain that breaks it (the diagonal first).
pub fn desired_bits(centre: u8, around: [Option<u8>; 8]) -> [Option<u8>; 8] {
    let mut bits = [None; 8];
    for d in 0..8 {
        if d % 2 == 0 {
            bits[d] = around[d];
        } else {
            let sides = [around[d - 1], around[(d + 1) % 8]];
            bits[d] = if around[d] == Some(centre) && sides.iter().all(|s| *s == Some(centre)) {
                Some(centre)
            } else if around[d] != Some(centre) {
                around[d]
            } else {
                sides.into_iter().find(|s| *s != Some(centre)).flatten()
            };
        }
    }
    bits
}
/// Deterministic per-cell noise for weighted choices.
fn noise(cell: [i32; 2], salt: u32) -> f32 {
    let mut h = (cell[0] as u32).wrapping_mul(0x9e37_79b1)
        ^ (cell[1] as u32).wrapping_mul(0x85eb_ca77)
        ^ salt.wrapping_mul(0xc2b2_ae3d);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2c1b_3c6d);
    h ^= h >> 12;
    (h & 0xffff) as f32 / 65536.
}
/// Picks one of `candidates` (atlas, weight) by weight, deterministically per cell.
pub fn weighted<T: Copy>(candidates: &[(T, f32)], cell: [i32; 2], salt: u32) -> Option<T> {
    let total: f32 = candidates.iter().map(|c| c.1.max(0.)).sum();
    if candidates.is_empty() {
        return None;
    }
    if total <= 0. {
        return Some(candidates[0].0);
    }
    let mut pick = noise(cell, salt) * total;
    for (value, weight) in candidates {
        if pick < weight.max(0.) {
            return Some(*value);
        }
        pick -= weight.max(0.);
    }
    Some(candidates[candidates.len() - 1].0)
}
/// The tile of terrain set `set` that best connects terrain `centre` to `around`: the most
/// matching peering bits wins (only the bits the set's mode uses), ties pick by tile
/// probability. None when the set has no tile of that terrain.
pub fn best_tile(
    tileset: &TileSet,
    set: usize,
    centre: u8,
    around: [Option<u8>; 8],
    cell: [i32; 2],
) -> Option<[u8; 2]> {
    let mode = tileset.terrain_sets.get(set)?.mode;
    let wanted = desired_bits(centre, around);
    let mut best = i32::MIN;
    let mut candidates = Vec::new();
    for tile in &tileset.tiles {
        if tile.terrain_set.map(usize::from) != Some(set) || tile.terrain != Some(centre) {
            continue;
        }
        let score: i32 = (0..8)
            .filter(|d| mode.uses(*d))
            .map(|d| if tile.peering[d] == wanted[d] { 2 } else { -1 })
            .sum();
        if score > best {
            best = score;
            candidates.clear();
        }
        if score == best {
            candidates.push((tile.atlas, tile.probability));
        }
    }
    weighted(&candidates, cell, 0x7e44)
}
/// Re-solves the terrain tiles of `cells` and their neighbours.
///
/// `terrain_of(cell)` reports the terrain a cell holds after the edit (None for cells
/// without a terrain of this set, which are never changed here). Returns the new tile of
/// every terrain cell in the affected area.
pub fn solve(
    tileset: &TileSet,
    set: usize,
    cells: &BTreeSet<[i32; 2]>,
    terrain_of: &dyn Fn([i32; 2]) -> Option<u8>,
) -> BTreeMap<[i32; 2], [u8; 2]> {
    let mut area = BTreeSet::new();
    for cell in cells {
        area.insert(*cell);
        for d in DIRECTIONS {
            area.insert([cell[0] + d[0], cell[1] + d[1]]);
        }
    }
    let mut out = BTreeMap::new();
    for cell in area {
        let Some(centre) = terrain_of(cell) else {
            continue;
        };
        let around = DIRECTIONS.map(|d| terrain_of([cell[0] + d[0], cell[1] + d[1]]));
        if let Some(atlas) = best_tile(tileset, set, centre, around, cell) {
            out.insert(cell, atlas);
        }
    }
    out
}
/// Assigns the peering bits of a 3 x 3 block of tiles (top-left at `corner`) as the outer
/// corners, edges and centre of an area of `terrain`: the quickest way to set up a basic
/// terrain. Tiles the block covers are created when missing.
pub fn assign_block(tileset: &mut TileSet, corner: [u8; 2], set: u8, terrain: u8) {
    for row in 0..3u8 {
        for column in 0..3u8 {
            let atlas = [corner[0] + column, corner[1] + row];
            if tileset.tile(atlas).is_none() {
                let mut tile = Tile::new(atlas);
                tile.custom = vec![0; tileset.custom_layers.len()];
                tileset.tiles.push(tile);
            }
            let tile = tileset.tile_mut(atlas).unwrap();
            tile.terrain_set = Some(set);
            tile.terrain = Some(terrain);
            for (d, offset) in DIRECTIONS.iter().enumerate() {
                // A neighbour inside the 3 x 3 block is the same terrain.
                let x = i32::from(column) + offset[0];
                let y = i32::from(row) + offset[1];
                let inside = (0..3).contains(&x) && (0..3).contains(&y);
                tile.peering[d] = inside.then_some(terrain);
            }
        }
    }
    tileset.sort();
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 47-tile-capable set built from every combination the blob layout needs.
    fn blob() -> TileSet {
        let mut set = TileSet {
            terrain_sets: vec![TerrainSet::default()],
            ..Default::default()
        };
        // Enumerate the 256 bit combinations and keep the distinct canonical ones (a
        // corner only counts when both of its sides do): that is the 47-tile blob.
        let mut seen = BTreeSet::new();
        for mask in 0u16..256 {
            let mut bits = [None; 8];
            for (d, bit) in bits.iter_mut().enumerate() {
                if mask & (1 << d) != 0 {
                    *bit = Some(0);
                }
            }
            for d in (1..8).step_by(2) {
                if bits[d - 1].is_none() || bits[(d + 1) % 8].is_none() {
                    bits[d] = None;
                }
            }
            if seen.insert(bits) {
                let index = seen.len() - 1;
                let mut tile = Tile::new([(index % 8) as u8, (index / 8) as u8]);
                tile.terrain_set = Some(0);
                tile.terrain = Some(0);
                tile.peering = bits;
                set.tiles.push(tile);
            }
        }
        assert_eq!(set.tiles.len(), 47);
        set
    }

    #[test]
    fn atlas_geometry_honours_margin_and_separation() {
        let set = TileSet {
            tile_size: [16, 8],
            margin: [2, 1],
            separation: [1, 2],
            ..Default::default()
        };
        assert_eq!(set.texel([1, 2]), [2 + 17, 1 + 20]);
        assert_eq!(set.grid([64, 32]), [3, 3]);
        assert_eq!(set.atlas_at([20., 12.], [64, 32]), Some([1, 1]));
        // The separation between tiles belongs to no tile.
        assert_eq!(set.atlas_at([18.5, 12.], [64, 32]), None);
        assert_eq!(set.atlas_at([1., 12.], [64, 32]), None);
        let tile = Tile {
            atlas: [1, 0],
            frames: 4,
            columns: 2,
            ..Default::default()
        };
        assert_eq!(tile.frame_atlas(3), [2, 1]);
    }

    #[test]
    fn opaque_tiles_are_created_and_documents_round_trip() {
        let mut rgba = vec![0u8; 32 * 16 * 4];
        // One visible texel in the right-hand tile only.
        rgba[(5 * 32 + 20) * 4 + 3] = 255;
        let texture = crate::texture::Data {
            width: 32,
            height: 16,
            rgba,
            words: Vec::new(),
            palette: Vec::new(),
        };
        let mut set = TileSet {
            custom_layers: vec![CustomLayer {
                name: "damage".into(),
                kind: CustomKind::Int,
            }],
            ..Default::default()
        };
        assert_eq!(set.create_opaque_tiles(&texture), 1);
        assert_eq!(set.tiles[0].atlas, [1, 0]);
        assert_eq!(set.tiles[0].custom, vec![0]);
        assert_eq!(set.create_opaque_tiles(&texture), 0);
        let back = TileSet::parse(&set.encode().unwrap()).unwrap();
        assert_eq!(back, set);
        set.tiles.push(Tile::new([1, 0]));
        assert!(set.validate().is_err());
        set.tiles.pop();
        set.tiles[0].frames = 3;
        assert!(set.validate_texture([32, 16]).is_err());
    }

    #[test]
    fn blob_terrain_connects_every_neighbourhood() {
        let set = blob();
        // A plus-shaped area: the centre is surrounded on its sides only.
        let area: BTreeSet<[i32; 2]> = [[0, 0], [1, 0], [-1, 0], [0, 1], [0, -1]]
            .into_iter()
            .collect();
        let terrain_of = |cell: [i32; 2]| area.contains(&cell).then_some(0u8);
        let tiles = solve(&set, 0, &area, &terrain_of);
        assert_eq!(tiles.len(), 5);
        let peering = |cell: [i32; 2]| set.tile(tiles[&cell]).unwrap().peering;
        // The centre connects on all four sides and on no corner.
        assert_eq!(
            peering([0, 0]),
            [Some(0), None, Some(0), None, Some(0), None, Some(0), None]
        );
        // The right arm connects only to its left.
        assert_eq!(
            peering([1, 0]),
            [None, None, None, None, Some(0), None, None, None]
        );
        // A filled 3 x 3 block gives its centre the all-connected tile.
        let block: BTreeSet<[i32; 2]> = (-1..=1)
            .flat_map(|y| (-1..=1).map(move |x| [x, y]))
            .collect();
        let terrain_of = |cell: [i32; 2]| block.contains(&cell).then_some(0u8);
        let tiles = solve(&set, 0, &block, &terrain_of);
        assert_eq!(set.tile(tiles[&[0, 0]]).unwrap().peering, [Some(0); 8]);
        // The top-left corner tile connects right, bottom-right and bottom.
        assert_eq!(
            set.tile(tiles[&[-1, -1]]).unwrap().peering,
            [Some(0), Some(0), Some(0), None, None, None, None, None]
        );
    }

    #[test]
    fn sides_mode_ignores_corners_and_blocks_assign_bits() {
        let mut set = TileSet {
            terrain_sets: vec![TerrainSet {
                mode: TerrainMode::Sides,
                ..Default::default()
            }],
            ..Default::default()
        };
        assign_block(&mut set, [0, 0], 0, 0);
        assert_eq!(set.tiles.len(), 9);
        // The block's centre is fully connected, its top-left corner only right and down.
        assert_eq!(set.tile([1, 1]).unwrap().peering, [Some(0); 8]);
        assert_eq!(
            set.tile([0, 0]).unwrap().peering,
            [Some(0), Some(0), Some(0), None, None, None, None, None]
        );
        // A lone cell has no exact match in a 3 x 3 block; sides mode still picks the
        // closest tile instead of nothing.
        let one: BTreeSet<[i32; 2]> = [[5, 5]].into_iter().collect();
        let terrain_of = |cell: [i32; 2]| one.contains(&cell).then_some(0u8);
        assert_eq!(solve(&set, 0, &one, &terrain_of).len(), 1);
        // Random picks are deterministic per cell.
        let options = [([0u8, 0u8], 1.), ([1, 0], 3.)];
        assert_eq!(weighted(&options, [3, 4], 1), weighted(&options, [3, 4], 1));
        let threes = (0..400)
            .filter(|i| weighted(&options, [*i, 0], 9) == Some([1, 0]))
            .count();
        assert!((240..360).contains(&threes), "{threes}");
    }
}
