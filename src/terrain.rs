//! Heightmap terrain. The authored payload is a compact binary grid, never an
//! expanded mesh document: a brush stroke rewrites kilobytes instead of
//! megabytes, and undo can afford whole-grid snapshots.
//!
//! Draw geometry is baked into the editable-mesh chunk format by
//! `terrain_compile`, so terrain inherits chunk culling, retained packets,
//! precomputed visibility, geometry streaming and the lighting bake without a
//! second renderer path.
use crate::{
    assets,
    scene::{Material, Scene},
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path, sync::Arc};
use uuid::Uuid;

const MAGIC: &[u8; 4] = b"EPTR";
const VERSION: u16 = 1;
const HEADER: usize = 16;
/// Height quantization. Q8 in an i16 spans exactly the ±128 local units the
/// chunk compiler accepts, so a representable height is always a legal vertex.
pub const HEIGHT_UNIT: f32 = 1. / 256.;
pub const MIN_CELLS: u16 = 1;
pub const MAX_CELLS: u16 = 256;
pub const MIN_CELL_SIZE: f32 = 0.25;
/// A single cell has to fit one draw chunk, and chunk-local positions are i16
/// Q12: 65504 ticks, just under 16 units. Sixteen would overflow by one tick
/// with no way to split further, so the authoring limit stops below it.
pub const MAX_CELL_SIZE: f32 = 15.;
/// Largest footprint along one axis. Beyond this a vertex leaves the ±128
/// local-unit window that `mesh_compile` encodes.
pub const MAX_SPAN: f32 = 256.;
/// One quad per cell against the per-scene resident budget.
pub const MAX_QUADS: usize = 3500;
pub const MAX_TILES: u8 = 64;

/// The terrain atlas every project starts with, so a new terrain draws ground
/// instead of flat grey and a path can be painted without sourcing art first.
///
/// Sixteen 64-pixel tiles in one 256-pixel page, four per material, in rows:
/// grass, dirt, stone, water. Sixty-four pixels is what a PSX ground tile
/// actually was, and four variants per material give the paint scatter
/// something to work with.
const BUILTIN_ATLAS_PNG: &[u8] = include_bytes!("../resources/terrain/EpokTerrainAtlas.png");
/// Fixed so the atlas keeps one identity across every project that adopts it,
/// and so the editor can recognise it and name its rows.
pub const BUILTIN_ATLAS_ID: Uuid = Uuid::from_u128(0xe7a3_f1c2_5b48_4d96_9f10_2c6d_84b7_a350);
pub const BUILTIN_ATLAS_GRID: [u8; 2] = [4, 4];
pub const BUILTIN_ATLAS_SOURCE: &str = "assets/Terrain/EpokTerrainAtlas.png";
const BUILTIN_ATLAS_ASSET: &str = "assets/Terrain/EpokTerrainAtlas.epokasset";

/// Material a tile of the bundled atlas belongs to. Meaningless for a custom
/// atlas, which is why callers check the texture id first.
pub fn builtin_tile_label(tile: u8) -> String {
    let row = tile / BUILTIN_ATLAS_GRID[0];
    let name = match row {
        0 => "Grass",
        1 => "Dirt",
        2 => "Stone",
        3 => "Water",
        _ => "Tile",
    };
    format!("{name} {}", tile % BUILTIN_ATLAS_GRID[0] + 1)
}

/// Write the bundled atlas into a project the first time it is needed.
///
/// Both the PNG and its package are written, so the atlas behaves like any
/// imported texture afterwards: it can be inspected, reimported, or replaced
/// with the project's own art at the same path.
pub fn ensure_builtin_atlas(root: &Path) -> Result<Uuid, String> {
    let asset = assets::inside(root, BUILTIN_ATLAS_ASSET)?;
    if asset.exists() {
        let package = assets::Package::load(&asset)?;
        if package.meta.kind == assets::Kind::Texture {
            return Ok(package.meta.id);
        }
        return Err(format!("{BUILTIN_ATLAS_ASSET} is not a Texture asset"));
    }
    let source = assets::inside(root, BUILTIN_ATLAS_SOURCE)?;
    if let Some(parent) = source.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    if !source.exists() {
        assets::atomic_write(&source, BUILTIN_ATLAS_PNG, None)?;
    }
    let bytes = BUILTIN_ATLAS_PNG.to_vec();
    crate::texture::decode(&bytes)?;
    let package = assets::Package {
        meta: assets::Metadata {
            version: 2,
            id: BUILTIN_ATLAS_ID,
            kind: assets::Kind::Texture,
            importer_version: 1,
            source: BUILTIN_ATLAS_SOURCE.into(),
            source_hash: assets::hash(&bytes),
            settings: crate::import_settings::Settings::Texture,
            extra: Default::default(),
        },
        source: bytes,
    };
    assets::atomic_write(&asset, &package.bytes()?, None)?;
    Ok(BUILTIN_ATLAS_ID)
}
/// Chunk-local positions are i16 Q12, so a chunk may span just under 16 units.
const MAX_CHUNK_TICKS: f32 = 65504.;

/// The authored grid. Heights live on cell corners, materials on cells.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Document {
    cells: [u16; 2],
    /// World units per cell, quantized to Q12 on encode.
    cell_size_q12: i32,
    /// Corner heights in Q8, row-major with X varying fastest.
    heights: Vec<i16>,
    /// Per-cell atlas tile in the low six bits, quarter-turn rotation in the
    /// top two. Row-major with X varying fastest.
    materials: Vec<u8>,
}
impl Default for Document {
    fn default() -> Self {
        Self::new([32, 32], 4.)
    }
}
impl Document {
    pub fn new(cells: [u16; 2], cell_size: f32) -> Self {
        let cells = cells.map(|c| c.clamp(MIN_CELLS, MAX_CELLS));
        let corners = (usize::from(cells[0]) + 1) * (usize::from(cells[1]) + 1);
        Self {
            cells,
            cell_size_q12: quantize(cell_size.clamp(MIN_CELL_SIZE, MAX_CELL_SIZE)),
            heights: vec![0; corners],
            materials: vec![0; usize::from(cells[0]) * usize::from(cells[1])],
        }
    }
    pub fn cells(&self) -> [u16; 2] {
        self.cells
    }
    pub fn cell_size(&self) -> f32 {
        self.cell_size_q12 as f32 / 4096.
    }
    pub fn set_cell_size(&mut self, value: f32) {
        self.cell_size_q12 = quantize(value.clamp(MIN_CELL_SIZE, MAX_CELL_SIZE));
    }
    pub fn quad_count(&self) -> usize {
        usize::from(self.cells[0]) * usize::from(self.cells[1])
    }
    pub fn corner_count(&self) -> usize {
        (usize::from(self.cells[0]) + 1) * (usize::from(self.cells[1]) + 1)
    }
    /// Footprint in world units along X and Z.
    pub fn span(&self) -> [f32; 2] {
        [
            f32::from(self.cells[0]) * self.cell_size(),
            f32::from(self.cells[1]) * self.cell_size(),
        ]
    }
    fn corner_index(&self, i: u16, j: u16) -> usize {
        usize::from(j) * (usize::from(self.cells[0]) + 1) + usize::from(i)
    }
    fn cell_index(&self, i: u16, j: u16) -> usize {
        usize::from(j) * usize::from(self.cells[0]) + usize::from(i)
    }
    /// Corner height in world units. Out-of-range corners clamp to the edge so
    /// neighbour lookups at the border need no special case.
    pub fn height(&self, i: i32, j: i32) -> f32 {
        f32::from(self.raw_height(i, j)) * HEIGHT_UNIT
    }
    pub fn raw_height(&self, i: i32, j: i32) -> i16 {
        let i = i.clamp(0, i32::from(self.cells[0])) as u16;
        let j = j.clamp(0, i32::from(self.cells[1])) as u16;
        self.heights[self.corner_index(i, j)]
    }
    /// Returns whether the stored value changed, which is what tells a stroke
    /// it has to publish and invalidate the preview.
    pub fn set_height(&mut self, i: u16, j: u16, value: f32) -> bool {
        if i > self.cells[0] || j > self.cells[1] {
            return false;
        }
        let raw = quantize_height(value);
        let index = self.corner_index(i, j);
        let changed = self.heights[index] != raw;
        self.heights[index] = raw;
        changed
    }
    pub fn tile(&self, i: u16, j: u16) -> u8 {
        if i >= self.cells[0] || j >= self.cells[1] {
            return 0;
        }
        self.materials[self.cell_index(i, j)] & 0x3F
    }
    pub fn rotation(&self, i: u16, j: u16) -> u8 {
        if i >= self.cells[0] || j >= self.cells[1] {
            return 0;
        }
        self.materials[self.cell_index(i, j)] >> 6
    }
    pub fn set_tile(&mut self, i: u16, j: u16, tile: u8, rotation: u8) -> bool {
        if i >= self.cells[0] || j >= self.cells[1] {
            return false;
        }
        let packed = (tile & 0x3F) | ((rotation & 3) << 6);
        let index = self.cell_index(i, j);
        let changed = self.materials[index] != packed;
        self.materials[index] = packed;
        changed
    }
    /// Local X of a grid column. The grid is centred on the actor so the ±128
    /// local-unit budget is spent symmetrically.
    pub fn local_x(&self, i: f32) -> f32 {
        (i - f32::from(self.cells[0]) / 2.) * self.cell_size()
    }
    pub fn local_z(&self, j: f32) -> f32 {
        (j - f32::from(self.cells[1]) / 2.) * self.cell_size()
    }
    /// Continuous grid coordinates for a local position. Values outside
    /// 0..cells lie off the terrain.
    pub fn grid_of(&self, x: f32, z: f32) -> [f32; 2] {
        let size = self.cell_size();
        [
            x / size + f32::from(self.cells[0]) / 2.,
            z / size + f32::from(self.cells[1]) / 2.,
        ]
    }
    /// Bilinear height at a local XZ position, clamped to the footprint.
    pub fn sample(&self, x: f32, z: f32) -> f32 {
        let [gx, gz] = self.grid_of(x, z);
        let gx = gx.clamp(0., f32::from(self.cells[0]));
        let gz = gz.clamp(0., f32::from(self.cells[1]));
        let i = gx.floor().min(f32::from(self.cells[0]) - 1.).max(0.);
        let j = gz.floor().min(f32::from(self.cells[1]) - 1.).max(0.);
        let (u, v) = (gx - i, gz - j);
        let (i, j) = (i as i32, j as i32);
        let h00 = self.height(i, j);
        let h10 = self.height(i + 1, j);
        let h01 = self.height(i, j + 1);
        let h11 = self.height(i + 1, j + 1);
        h00 * (1. - u) * (1. - v) + h10 * u * (1. - v) + h01 * (1. - u) * v + h11 * u * v
    }
    /// The four corners of a cell in local space, wound so that
    /// `mesh::face_normal` points up. This matches the Plane primitive.
    pub fn corners(&self, i: u16, j: u16) -> [[f32; 3]; 4] {
        let (fi, fj) = (f32::from(i), f32::from(j));
        let (x0, x1) = (self.local_x(fi), self.local_x(fi + 1.));
        let (z0, z1) = (self.local_z(fj), self.local_z(fj + 1.));
        let (a, b) = (i32::from(i), i32::from(j));
        [
            [x0, self.height(a, b), z0],
            [x0, self.height(a, b + 1), z1],
            [x1, self.height(a + 1, b + 1), z1],
            [x1, self.height(a + 1, b), z0],
        ]
    }
    /// Averaged corner normal, used to bake smooth Gouraud shading instead of
    /// the faceted per-quad normal the renderer would otherwise light with.
    pub fn corner_normal(&self, i: i32, j: i32) -> [f32; 3] {
        let size = self.cell_size();
        let dx = self.height(i + 1, j) - self.height(i - 1, j);
        let dz = self.height(i, j + 1) - self.height(i, j - 1);
        crate::lighting::unit([-dx, 2. * size, -dz])
    }
    pub fn lowest(&self) -> f32 {
        f32::from(self.heights.iter().copied().min().unwrap_or(0)) * HEIGHT_UNIT
    }
    pub fn highest(&self) -> f32 {
        f32::from(self.heights.iter().copied().max().unwrap_or(0)) * HEIGHT_UNIT
    }
    /// Cells per chunk along one axis. Chunk-local positions are i16 Q12, so a
    /// chunk must stay just under 16 world units however small the cells are.
    pub fn block(&self) -> u16 {
        let size = self.cell_size();
        if size <= 0. {
            return 1;
        }
        let fit = (MAX_CHUNK_TICKS / 4096. / size).floor();
        if !fit.is_finite() || fit < 1. {
            1
        } else {
            (fit as u16).min(MAX_CELLS)
        }
    }
    /// Chunks the compiled terrain will produce, before decimation merges any
    /// cells away. Surfaced in the editor because chunk count, not triangle
    /// count, is what the per-chunk bounds test pays for.
    pub fn chunk_count(&self) -> usize {
        let block = usize::from(self.block().max(1));
        let along = |cells: u16| usize::from(cells).div_ceil(block);
        along(self.cells[0]) * along(self.cells[1])
    }
    /// Grow or shrink the grid, preserving the overlapping region.
    pub fn resize(&mut self, cells: [u16; 2]) {
        let cells = cells.map(|c| c.clamp(MIN_CELLS, MAX_CELLS));
        if cells == self.cells {
            return;
        }
        let mut heights = vec![0_i16; (usize::from(cells[0]) + 1) * (usize::from(cells[1]) + 1)];
        for j in 0..=cells[1] {
            for i in 0..=cells[0] {
                heights[usize::from(j) * (usize::from(cells[0]) + 1) + usize::from(i)] =
                    self.raw_height(i32::from(i), i32::from(j));
            }
        }
        let mut materials = vec![0_u8; usize::from(cells[0]) * usize::from(cells[1])];
        for j in 0..cells[1].min(self.cells[1]) {
            for i in 0..cells[0].min(self.cells[0]) {
                materials[usize::from(j) * usize::from(cells[0]) + usize::from(i)] =
                    self.materials[self.cell_index(i, j)];
            }
        }
        self.cells = cells;
        self.heights = heights;
        self.materials = materials;
    }
    /// First terrain hit along a local-space ray, as a local position. A 2D
    /// grid march rather than a scan of every cell: a sculpt brush raycasts on
    /// every mouse move, so this stays proportional to the cells crossed.
    pub fn raycast(&self, origin: [f32; 3], direction: [f32; 3]) -> Option<[f32; 3]> {
        let span = self.span();
        let (half_x, half_z) = (span[0] / 2., span[1] / 2.);
        // Clip to the footprint slab first so a distant camera does not march
        // empty space, and so a ray that misses entirely costs nothing.
        let mut enter = 0_f32;
        let mut leave = (span[0] + span[1] + 512.) / direction_scale(direction);
        for (axis, half) in [(0_usize, half_x), (2, half_z)] {
            let (o, d) = (origin[axis], direction[axis]);
            if d.abs() < 1e-6 {
                if o < -half || o > half {
                    return None;
                }
                continue;
            }
            let (mut lo, mut hi) = ((-half - o) / d, (half - o) / d);
            if lo > hi {
                std::mem::swap(&mut lo, &mut hi);
            }
            enter = enter.max(lo);
            leave = leave.min(hi);
            if enter > leave {
                return None;
            }
        }
        if !enter.is_finite() || !leave.is_finite() || leave < 0. {
            return None;
        }
        let size = self.cell_size();
        let start: [f32; 3] = std::array::from_fn(|c| origin[c] + direction[c] * enter);
        let [gx, gz] = self.grid_of(start[0], start[2]);
        let mut cell = [
            (gx.floor() as i32).clamp(0, i32::from(self.cells[0]) - 1),
            (gz.floor() as i32).clamp(0, i32::from(self.cells[1]) - 1),
        ];
        let step = [
            if direction[0] >= 0. { 1_i32 } else { -1 },
            if direction[2] >= 0. { 1_i32 } else { -1 },
        ];
        let mut next = [0_f32; 2];
        let mut delta = [f32::INFINITY; 2];
        for (k, axis) in [(0_usize, 0_usize), (1, 2)] {
            let d = direction[axis];
            if d.abs() < 1e-6 {
                next[k] = f32::INFINITY;
                continue;
            }
            delta[k] = (size / d).abs();
            let boundary = if step[k] > 0 {
                cell[k] as f32 + 1.
            } else {
                cell[k] as f32
            };
            let world = if k == 0 {
                self.local_x(boundary)
            } else {
                self.local_z(boundary)
            };
            next[k] = enter + (world - start[axis]) / d;
        }
        let budget = usize::from(self.cells[0]) + usize::from(self.cells[1]) + 2;
        for _ in 0..budget {
            if cell[0] < 0
                || cell[1] < 0
                || cell[0] >= i32::from(self.cells[0])
                || cell[1] >= i32::from(self.cells[1])
            {
                return None;
            }
            let p = self.corners(cell[0] as u16, cell[1] as u16);
            // The renderer splits every quad the same way; hit-test the same
            // two triangles so the cursor lands where the surface is drawn.
            for tri in [[p[0], p[1], p[2]], [p[0], p[2], p[3]]] {
                if let Some(t) = crate::mesh::hit(origin, direction, tri)
                    && t >= 0.
                {
                    return Some(std::array::from_fn(|c| origin[c] + direction[c] * t));
                }
            }
            let k = usize::from(next[1] < next[0]);
            if next[k] > leave {
                return None;
            }
            cell[k] += step[k];
            next[k] += delta[k];
        }
        None
    }
    /// Apply one brush sample at a local-space position. Returns whether any
    /// stored value changed, which is what tells a stroke it must publish.
    ///
    /// Reads are gathered before any write so a Smooth pass averages the
    /// heights the stroke started from rather than its own partial results,
    /// which would make a stroke depend on iteration order.
    pub fn sculpt(&mut self, brush: &crate::brush::Brush, local: [f32; 3]) -> bool {
        use crate::brush::Mode;
        if brush.mode == Mode::Paint {
            return self.paint(brush, local);
        }
        let size = self.cell_size();
        if size <= 0. || !brush.radius.is_finite() {
            return false;
        }
        let reach = (brush.radius / size).ceil() as i32 + 1;
        let [gx, gz] = self.grid_of(local[0], local[2]);
        let (ci, cj) = (gx.round() as i32, gz.round() as i32);
        let lo = [(ci - reach).max(0), (cj - reach).max(0)];
        let hi = [
            (ci + reach).min(i32::from(self.cells[0])),
            (cj + reach).min(i32::from(self.cells[1])),
        ];
        let mut updates: Vec<(u16, u16, f32)> = Vec::new();
        for j in lo[1]..=hi[1] {
            for i in lo[0]..=hi[0] {
                let x = self.local_x(i as f32);
                let z = self.local_z(j as f32);
                let weight = brush.weight(x - local[0], z - local[2]);
                if weight <= 0. {
                    continue;
                }
                let current = self.height(i, j);
                let next = match brush.mode {
                    Mode::Smooth => {
                        let average = (self.height(i - 1, j)
                            + self.height(i + 1, j)
                            + self.height(i, j - 1)
                            + self.height(i, j + 1))
                            / 4.;
                        current + (average - current) * weight * brush.strength.min(1.)
                    }
                    Mode::Noise => {
                        current + crate::brush::noise(i, j, brush.seed) * brush.strength * weight
                    }
                    _ => current + brush.delta(weight, current),
                };
                updates.push((i as u16, j as u16, next));
            }
        }
        let mut changed = false;
        for (i, j, value) in updates {
            changed |= self.set_height(i, j, value);
        }
        changed
    }
    /// Assign the brush tile to every cell whose centre falls inside the disc.
    /// Centre containment rather than a weight threshold keeps painting
    /// predictable: the cells under the ring are exactly the ones that change.
    pub fn paint(&mut self, brush: &crate::brush::Brush, local: [f32; 3]) -> bool {
        let size = self.cell_size();
        if size <= 0. || !brush.radius.is_finite() {
            return false;
        }
        let reach = (brush.radius / size).ceil() as i32 + 1;
        let [gx, gz] = self.grid_of(local[0], local[2]);
        let (ci, cj) = (gx.floor() as i32, gz.floor() as i32);
        let mut updates: Vec<(u16, u16)> = Vec::new();
        for j in (cj - reach).max(0)..=(cj + reach).min(i32::from(self.cells[1]) - 1) {
            for i in (ci - reach).max(0)..=(ci + reach).min(i32::from(self.cells[0]) - 1) {
                let x = self.local_x(i as f32 + 0.5);
                let z = self.local_z(j as f32 + 0.5);
                if brush.weight(x - local[0], z - local[2]) > 0. {
                    updates.push((i as u16, j as u16));
                }
            }
        }
        let mut changed = false;
        for (i, j) in updates {
            changed |= self.set_tile(i, j, brush.tile, brush.rotation_for(i, j));
        }
        changed
    }
    /// Well-formedness of the stored grid: enough to decode it safely.
    ///
    /// Limits that come from cooking rather than from storage live in
    /// `cook_limits`, so a grid that is merely too heavy for the console still
    /// loads and can be reported against the actor that uses it. Failing here
    /// makes the asset unusable, which the editor can only report as a missing
    /// file.
    pub fn validate(&self) -> Result<(), String> {
        if !(MIN_CELLS..=MAX_CELLS).contains(&self.cells[0])
            || !(MIN_CELLS..=MAX_CELLS).contains(&self.cells[1])
        {
            return Err(format!(
                "Terrain needs between {MIN_CELLS} and {MAX_CELLS} cells on each axis"
            ));
        }
        let size = self.cell_size();
        if !size.is_finite() || !(MIN_CELL_SIZE..=MAX_CELL_SIZE).contains(&size) {
            return Err(format!(
                "Terrain cell size must be between {MIN_CELL_SIZE} and {MAX_CELL_SIZE} units"
            ));
        }
        if self.heights.len() != self.corner_count() || self.materials.len() != self.quad_count() {
            return Err("Terrain grid payload does not match its dimensions".into());
        }
        // Tiles need no range check: the field is six bits, so a stored tile is
        // always below MAX_TILES. What can be wrong is a tile that exceeds the
        // atlas its instance declares, which `validate_scene` checks because
        // only the scene knows the atlas.
        Ok(())
    }

    /// What the console imposes on a grid that is otherwise well formed.
    /// Reported against the actor by `validate_scene`, and refused by the
    /// terrain editor before an edit is published.
    pub fn cook_limits(&self) -> Result<(), String> {
        let span = self.span();
        if span[0] > MAX_SPAN || span[1] > MAX_SPAN {
            return Err(format!(
                "terrain spans {:.1} x {:.1} units; the chunk compiler encodes at most {MAX_SPAN} per axis. Reduce the cell count or cell size.",
                span[0], span[1]
            ));
        }
        if self.quad_count() > MAX_QUADS {
            return Err(format!(
                "terrain compiles {} quads; the resident budget is {MAX_QUADS} ({} triangles) for the whole scene. Reduce the cell count or enable Engine > Streaming > Geometry Streaming.",
                self.quad_count(),
                MAX_QUADS * 2
            ));
        }
        // A cell is one quad, and one quad must fit one chunk on every axis.
        // The horizontal span is the cell size; the vertical span is whatever
        // a cliff was sculpted to, and nothing downstream can split it.
        let limit = MAX_CHUNK_TICKS / 4096.;
        for j in 0..i32::from(self.cells[1]) {
            for i in 0..i32::from(self.cells[0]) {
                let corners = [
                    self.height(i, j),
                    self.height(i + 1, j),
                    self.height(i, j + 1),
                    self.height(i + 1, j + 1),
                ];
                let low = corners.iter().copied().fold(f32::INFINITY, f32::min);
                let high = corners.iter().copied().fold(f32::NEG_INFINITY, f32::max);
                if high - low > limit {
                    return Err(format!(
                        "cell {i},{j} drops {:.1} units across one cell; a single cell must stay within {limit:.1}. Add cells across the cliff instead of one tall step.",
                        high - low
                    ));
                }
            }
        }
        Ok(())
    }
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(HEADER + self.heights.len() * 2 + self.materials.len());
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&VERSION.to_le_bytes());
        out.extend_from_slice(&self.cells[0].to_le_bytes());
        out.extend_from_slice(&self.cells[1].to_le_bytes());
        out.extend_from_slice(&self.cell_size_q12.to_le_bytes());
        out.extend_from_slice(&0_u16.to_le_bytes());
        for h in &self.heights {
            out.extend_from_slice(&h.to_le_bytes());
        }
        out.extend_from_slice(&self.materials);
        out
    }
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < HEADER || &bytes[0..4] != MAGIC {
            return Err("Not a terrain payload".into());
        }
        let u16_at = |at: usize| u16::from_le_bytes([bytes[at], bytes[at + 1]]);
        if u16_at(4) != VERSION {
            return Err("Unsupported terrain version".into());
        }
        let cells = [u16_at(6), u16_at(8)];
        if !(MIN_CELLS..=MAX_CELLS).contains(&cells[0])
            || !(MIN_CELLS..=MAX_CELLS).contains(&cells[1])
        {
            return Err("Terrain dimensions out of range".into());
        }
        let cell_size_q12 = i32::from_le_bytes([bytes[10], bytes[11], bytes[12], bytes[13]]);
        let corners = (usize::from(cells[0]) + 1) * (usize::from(cells[1]) + 1);
        let quads = usize::from(cells[0]) * usize::from(cells[1]);
        if bytes.len() != HEADER + corners * 2 + quads {
            return Err("Terrain payload length does not match its dimensions".into());
        }
        let mut heights = Vec::with_capacity(corners);
        for k in 0..corners {
            let at = HEADER + k * 2;
            heights.push(i16::from_le_bytes([bytes[at], bytes[at + 1]]));
        }
        let start = HEADER + corners * 2;
        let doc = Self {
            cells,
            cell_size_q12,
            heights,
            materials: bytes[start..].to_vec(),
        };
        doc.validate()?;
        Ok(doc)
    }
}

fn quantize(value: f32) -> i32 {
    (value * 4096.).round() as i32
}
fn quantize_height(value: f32) -> i16 {
    if !value.is_finite() {
        return 0;
    }
    (value / HEIGHT_UNIT).round().clamp(-32768., 32767.) as i16
}
fn direction_scale(direction: [f32; 3]) -> f32 {
    let length =
        (direction[0] * direction[0] + direction[1] * direction[1] + direction[2] * direction[2])
            .sqrt();
    if length.is_finite() && length > 1e-6 {
        length
    } else {
        1.
    }
}

/// Scene-side reference. The grid lives in the package; the surface material
/// and its atlas layout are per-instance, so one grid can be reused with
/// different ground textures.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Component {
    pub asset: Uuid,
    #[serde(default)]
    pub material: Material,
    /// Atlas subdivision as (columns, rows). `[1, 1]` uses the whole texture.
    #[serde(default = "default_atlas")]
    pub atlas: [u8; 2],
    /// Cook a heightfield collider alongside the draw geometry.
    #[serde(default = "default_collision")]
    pub collision: bool,
    /// Largest power-of-two block of coplanar same-material cells the baker may
    /// merge into one quad: 0 off, 1 up to 2x2, 2 up to 4x4, 3 up to 8x8.
    /// Merging is the only decimation the cooked format allows, and it
    /// stretches the tile across the merged cells.
    #[serde(default)]
    pub merge: u8,
    #[serde(skip)]
    pub document: Option<Arc<Document>>,
    #[serde(skip)]
    pub error: Option<String>,
}
fn default_atlas() -> [u8; 2] {
    [1, 1]
}
fn default_collision() -> bool {
    true
}
impl Component {
    pub fn new(asset: Uuid) -> Self {
        Self {
            asset,
            material: Material::default(),
            atlas: default_atlas(),
            collision: default_collision(),
            merge: 0,
            document: None,
            error: None,
        }
    }
    pub fn tiles(&self) -> u8 {
        u16::from(self.atlas[0].max(1))
            .saturating_mul(u16::from(self.atlas[1].max(1)))
            .min(u16::from(MAX_TILES)) as u8
    }
    pub fn validate(&self) -> Result<(), String> {
        if !(1..=8).contains(&self.atlas[0]) || !(1..=8).contains(&self.atlas[1]) {
            return Err("Terrain atlas columns and rows must be between 1 and 8".into());
        }
        if u16::from(self.atlas[0]) * u16::from(self.atlas[1]) > u16::from(MAX_TILES) {
            return Err(format!("Terrain atlas holds at most {MAX_TILES} tiles"));
        }
        if self.merge > 3 {
            return Err("Terrain merge level must be between 0 and 3".into());
        }
        crate::texture::validate_material(&self.material)?;
        Ok(())
    }
}

pub fn create(root: &Path, path: &str, document: &Document) -> Result<Uuid, String> {
    document.validate()?;
    let id = Uuid::new_v4();
    let source = document.encode();
    let package = assets::Package {
        meta: assets::Metadata {
            version: 1,
            id,
            kind: assets::Kind::Terrain,
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
pub fn document(record: &assets::Record) -> Result<Document, String> {
    if record.meta.kind != assets::Kind::Terrain {
        return Err("Select a Terrain asset".into());
    }
    Document::parse(&assets::Package::load(&record.path)?.source)
}
pub fn save(record: &assets::Record, doc: &Document) -> Result<String, String> {
    doc.validate()?;
    let mut package = assets::Package::load(&record.path)?;
    if package.meta.kind != assets::Kind::Terrain || package.meta.id != record.meta.id {
        return Err("The terrain asset was replaced; reload before editing".into());
    }
    package.source = doc.encode();
    package.meta.source_hash = assets::hash(&package.source);
    let bytes = package.bytes()?;
    assets::atomic_write(&record.path, &bytes, Some(&record.revision))?;
    Ok(assets::hash(&bytes))
}
pub fn resolve(scene: &mut Scene, index: &assets::Index) -> Result<(), String> {
    let mut errors = vec![];
    let mut loaded = BTreeMap::new();
    for e in &mut scene.actors {
        if let Some(t) = &mut e.terrain {
            let result = loaded
                .entry(t.asset)
                .or_insert_with(|| index.resolve(t.asset).and_then(document).map(Arc::new));
            match result {
                Ok(doc) => {
                    t.document = Some(doc.clone());
                    t.error = None;
                }
                Err(error) => {
                    t.document = None;
                    t.error = Some(error.clone());
                    errors.push(format!("{}: {error}", e.name));
                }
            }
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}
pub fn validate_scene(scene: &Scene) -> Result<(), String> {
    for (index, e) in scene.actors.iter().enumerate() {
        let Some(t) = &e.terrain else { continue };
        t.validate()
            .map_err(|error| format!("{}: {error}", e.name))?;
        if let Some(doc) = &t.document {
            doc.validate()
                .and_then(|()| doc.cook_limits())
                .map_err(|error| format!("{}: {error}", e.name))?;
            let tiles = t.tiles();
            let cells = doc.cells();
            for j in 0..cells[1] {
                for i in 0..cells[0] {
                    if doc.tile(i, j) >= tiles {
                        return Err(format!(
                            "{}: cell {i},{j} paints atlas tile {} but the atlas holds {tiles}",
                            e.name,
                            doc.tile(i, j)
                        ));
                    }
                }
            }
            if t.collision {
                // The heightfield is indexed from the collider box's world
                // minimum, so the grid only lines up under pure translation.
                // Reject anything else at cook time rather than shipping a
                // collider that silently samples the wrong cells.
                let world = scene.world_matrix(index);
                for row in 0..3 {
                    for axis in 0..3 {
                        let expected = if row == axis { 1. } else { 0. };
                        if (world.0[row][axis] - expected).abs() > 1e-3 {
                            return Err(format!(
                                "{}: terrain collision needs an unrotated, unscaled actor. Change the cell size instead of the scale, or turn Heightfield Collision off.",
                                e.name
                            ));
                        }
                    }
                }
                let span = doc.span();
                let origin = world.point([0.; 3]);
                let low = [
                    origin[0] - span[0] / 2.,
                    origin[1] + doc.lowest() - 1.,
                    origin[2] - span[1] / 2.,
                ];
                let high = [
                    origin[0] + span[0] / 2.,
                    origin[1] + doc.highest(),
                    origin[2] + span[1] / 2.,
                ];
                if low
                    .iter()
                    .chain(&high)
                    .any(|v| !v.is_finite() || v.abs() > 512.)
                {
                    return Err(format!(
                        "{}: terrain collider world bounds exceed ±512",
                        e.name
                    ));
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brush::{Brush, Mode};

    fn ramp(cells: [u16; 2], size: f32) -> Document {
        let mut doc = Document::new(cells, size);
        for j in 0..=cells[1] {
            for i in 0..=cells[0] {
                doc.set_height(i, j, f32::from(i) * 0.5);
            }
        }
        doc
    }

    #[test]
    fn payload_round_trips_and_rejects_corruption() {
        let doc = ramp([8, 6], 2.);
        let bytes = doc.encode();
        assert_eq!(Document::parse(&bytes).unwrap(), doc);
        assert!(Document::parse(&bytes[..bytes.len() - 1]).is_err());
        assert!(Document::parse(b"nope").is_err());
        let mut wrong = bytes.clone();
        wrong[4] = 9;
        assert!(Document::parse(&wrong).is_err());
        // The payload stays small enough that undo can snapshot whole grids.
        assert!(bytes.len() < 1024, "{} bytes", bytes.len());
        assert_eq!(
            Document::new([32, 32], 4.).encode().len(),
            16 + 33 * 33 * 2 + 32 * 32
        );
    }

    #[test]
    fn cooking_limits_bound_span_and_budget_without_breaking_the_payload() {
        let fine = Document::new([32, 32], 4.);
        fine.validate().unwrap();
        fine.cook_limits().unwrap();
        // 64 cells of 8 units is 512 across: double the local window a chunk
        // vertex can encode. The grid is still well formed, so the asset
        // loads and the error can name the actor that uses it instead of
        // surfacing as a missing file.
        let wide = Document::new([64, 64], 8.);
        wide.validate().unwrap();
        assert!(wide.cook_limits().unwrap_err().contains("spans"));
        // Inside the span limit but over the resident quad budget.
        let dense = Document::new([64, 64], 2.);
        assert_eq!(dense.quad_count(), 4096);
        dense.validate().unwrap();
        assert!(dense.cook_limits().unwrap_err().contains("resident budget"));
        Document::new([48, 48], 2.).cook_limits().unwrap();
    }

    #[test]
    fn heights_quantize_to_q8_and_clamp_to_the_local_window() {
        let mut doc = Document::new([2, 2], 1.);
        doc.set_height(1, 1, 1. / 256.);
        assert_eq!(doc.height(1, 1), 1. / 256.);
        doc.set_height(1, 1, 400.);
        assert!(doc.height(1, 1) <= 128.);
        doc.set_height(1, 1, -400.);
        assert!(doc.height(1, 1) >= -128.);
        doc.set_height(1, 1, f32::NAN);
        assert_eq!(doc.height(1, 1), 0.);
        // Out-of-range corners clamp to the edge so neighbour lookups at the
        // border need no special case.
        doc.set_height(0, 0, 3.);
        assert_eq!(doc.height(-5, -5), 3.);
    }

    #[test]
    fn chunk_blocks_stay_inside_the_encodable_span() {
        for size in [0.25, 0.5, 1., 2., 3., 4., 7.5, MAX_CELL_SIZE] {
            let doc = Document::new([64, 64], size);
            let block = doc.block();
            assert!(block >= 1);
            assert!(
                f32::from(block) * size * 4096. <= 65504.,
                "cell {size} gave block {block}"
            );
            // And it is the largest block that fits.
            assert!(f32::from(block + 1) * size * 4096. > 65504.);
        }
        assert_eq!(Document::new([32, 32], 4.).chunk_count(), 11 * 11);
    }

    #[test]
    fn sampling_is_bilinear_and_clamped() {
        let doc = ramp([4, 4], 2.);
        // Corner heights are i * 0.5 with cells 2 units wide, centred on zero.
        assert!((doc.sample(doc.local_x(0.), 0.) - 0.).abs() < 1e-4);
        assert!((doc.sample(doc.local_x(4.), 0.) - 2.).abs() < 1e-4);
        assert!((doc.sample(doc.local_x(2.5), 0.) - 1.25).abs() < 1e-3);
        // Outside the footprint clamps to the border rather than extrapolating.
        assert!((doc.sample(1000., 0.) - 2.).abs() < 1e-4);
        assert!((doc.sample(-1000., 0.) - 0.).abs() < 1e-4);
    }

    #[test]
    fn corner_winding_faces_up() {
        let doc = ramp([3, 3], 2.);
        for j in 0..3 {
            for i in 0..3 {
                let normal = crate::mesh::face_normal(doc.corners(i, j));
                assert!(normal[1] > 0.5, "cell {i},{j} wound downwards: {normal:?}");
            }
        }
    }

    #[test]
    fn raycast_hits_the_surface_and_misses_outside() {
        let mut doc = Document::new([8, 8], 2.);
        doc.set_height(4, 4, 6.);
        let down = [0., -1., 0.];
        // Straight down onto the raised corner.
        let hit = doc
            .raycast([doc.local_x(4.), 40., doc.local_z(4.)], down)
            .unwrap();
        assert!((hit[1] - 6.).abs() < 0.05, "{hit:?}");
        // Straight down onto flat ground.
        let flat = doc
            .raycast([doc.local_x(1.), 40., doc.local_z(1.)], down)
            .unwrap();
        assert!(flat[1].abs() < 0.05, "{flat:?}");
        // Beyond the footprint there is nothing to hit.
        assert!(doc.raycast([500., 40., 500.], down).is_none());
        // A ray pointing away from the terrain must not report a hit behind it.
        assert!(doc.raycast([0., 40., 0.], [0., 1., 0.]).is_none());
        // A shallow ray that crosses the grid still finds the surface.
        let slanted = doc.raycast([-40., 20., 0.], [1., -0.5, 0.]);
        assert!(slanted.is_some());
    }

    #[test]
    fn sculpt_raises_lowers_and_flattens() {
        let mut doc = Document::new([16, 16], 1.);
        let raise = Brush {
            mode: Mode::Raise,
            radius: 3.,
            strength: 1.,
            ..Default::default()
        };
        assert!(doc.sculpt(&raise, [0., 0., 0.]));
        let peak = doc.sample(0., 0.);
        assert!(peak > 0.9, "{peak}");
        // Outside the radius nothing moved.
        assert_eq!(doc.sample(doc.local_x(16.), doc.local_z(16.)), 0.);
        let lower = Brush {
            mode: Mode::Lower,
            ..raise
        };
        doc.sculpt(&lower, [0., 0., 0.]);
        assert!(doc.sample(0., 0.) < peak);
        let flatten = Brush {
            mode: Mode::Flatten,
            reference: 4.,
            strength: 1.,
            ..raise
        };
        for _ in 0..12 {
            doc.sculpt(&flatten, [0., 0., 0.]);
        }
        assert!((doc.sample(0., 0.) - 4.).abs() < 0.05);
    }

    #[test]
    fn smooth_averages_the_grid_it_started_from() {
        let mut doc = Document::new([8, 8], 1.);
        doc.set_height(4, 4, 8.);
        let spike = doc.height(4, 4);
        let smooth = Brush {
            mode: Mode::Smooth,
            radius: 3.,
            strength: 1.,
            ..Default::default()
        };
        assert!(doc.sculpt(&smooth, [doc.local_x(4.), 0., doc.local_z(4.)]));
        assert!(doc.height(4, 4) < spike);
        // Neighbours rose towards the spike rather than staying flat.
        assert!(doc.height(3, 4) > 0.);
    }

    #[test]
    fn noise_is_reproducible_across_strokes() {
        let brush = Brush {
            mode: Mode::Noise,
            radius: 4.,
            strength: 2.,
            seed: 42,
            ..Default::default()
        };
        let mut a = Document::new([8, 8], 1.);
        let mut b = Document::new([8, 8], 1.);
        a.sculpt(&brush, [0., 0., 0.]);
        b.sculpt(&brush, [0., 0., 0.]);
        assert_eq!(a, b);
    }

    #[test]
    fn paint_assigns_tiles_under_the_disc_only() {
        let mut doc = Document::new([8, 8], 1.);
        let brush = Brush {
            mode: Mode::Paint,
            radius: 2.,
            tile: 3,
            ..Default::default()
        };
        assert!(doc.sculpt(&brush, [doc.local_x(4.), 0., doc.local_z(4.)]));
        assert_eq!(doc.tile(4, 4), 3);
        assert_eq!(doc.tile(0, 0), 0);
        // Repainting the same cells is a no-op, so a stroke does not churn.
        assert!(!doc.sculpt(&brush, [doc.local_x(4.), 0., doc.local_z(4.)]));
    }

    #[test]
    fn resize_preserves_the_overlap() {
        let mut doc = ramp([8, 8], 1.);
        doc.set_tile(2, 3, 5, 1);
        let before = doc.height(3, 3);
        doc.resize([16, 4]);
        assert_eq!(doc.cells(), [16, 4]);
        assert_eq!(doc.height(3, 3), before);
        assert_eq!(doc.tile(2, 3), 5);
        assert_eq!(doc.rotation(2, 3), 1);
        doc.validate().unwrap();
        doc.cook_limits().unwrap();
        // Growing past the old grid extends the border rather than leaving a
        // hole, because out-of-range reads clamp.
        assert_eq!(doc.height(12, 0), doc.height(8, 0));
    }

    #[test]
    fn a_cell_taller_than_a_chunk_is_rejected() {
        let mut doc = Document::new([4, 4], 2.);
        doc.set_height(2, 2, 40.);
        // One corner 40 units above its neighbours cannot be encoded: chunk
        // positions are i16 Q12 and nothing downstream can split one quad.
        doc.validate().unwrap();
        let error = doc.cook_limits().unwrap_err();
        assert!(error.contains("across one cell"), "{error}");
        // Spreading the same drop over several cells is fine.
        let mut gentle = Document::new([8, 8], 2.);
        for j in 0..=8 {
            for i in 0..=8 {
                gentle.set_height(i, j, f32::from(i) * 5.);
            }
        }
        gentle.validate().unwrap();
        gentle.cook_limits().unwrap();
    }

    /// Opening a project must put the ground on screen.
    ///
    /// A terrain reaches the viewport only if its grid was resolved, and the
    /// editor resolves assets from a dozen places. Missing one of them does
    /// not fail any build: the scene simply loads with no floor. Round-trip a
    /// real scene through disk, which is the path `Scene::load` takes when a
    /// project is opened.
    #[test]
    fn a_scene_loaded_from_disk_resolves_its_terrain() {
        let root = std::env::temp_dir().join(format!("epok-terrain-load-{}", Uuid::new_v4()));
        std::fs::create_dir_all(root.join("assets/scenes")).unwrap();
        let mut doc = Document::new([4, 4], 2.);
        doc.set_height(2, 2, 3.);
        let id = create(&root, "assets/Ground.epokasset", &doc).unwrap();

        let mut actor = crate::scene::Actor::cube("Ground".into());
        actor.position = [0.; 3];
        actor.terrain = Some(Component::new(id));
        let scene = Scene {
            actors: vec![actor],
            ..Default::default()
        };
        let path = root.join("assets/scenes/Test.epokmap");
        scene.save(&path).unwrap();

        let loaded = Scene::load(&path).unwrap();
        let component = loaded.actors[0]
            .terrain
            .as_ref()
            .expect("the terrain reference survived the round trip");
        assert!(component.error.is_none(), "{:?}", component.error);
        let resolved = component
            .document
            .as_ref()
            .expect("loading a scene must resolve its terrain grid");
        assert_eq!(resolved.cells(), [4, 4]);
        assert_eq!(resolved.height(2, 2), 3.);
        // And the geometry seam every preview and cook reads produces a floor.
        let quads = crate::lighting::quads(&loaded.actors[0]);
        assert_eq!(quads.len(), 16, "an unresolved grid draws nothing at all");
        assert!(quads.iter().any(|q| q.points.iter().any(|p| p[1] > 0.)));

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn the_bundled_atlas_materializes_once_and_decodes() {
        let root = std::env::temp_dir().join(format!("epok-terrain-atlas-{}", Uuid::new_v4()));
        std::fs::create_dir_all(root.join("assets")).unwrap();
        let id = ensure_builtin_atlas(&root).unwrap();
        assert_eq!(id, BUILTIN_ATLAS_ID);
        // Both the PNG and its package land, so the atlas can be reimported
        // or replaced with the project's own art at the same path.
        assert!(root.join(BUILTIN_ATLAS_SOURCE).is_file());
        let package = assets::Package::load(&root.join(BUILTIN_ATLAS_ASSET)).unwrap();
        assert_eq!(package.meta.kind, assets::Kind::Texture);
        assert_eq!(package.meta.source_hash, assets::hash(&package.source));
        // The importer's own limits: one page, at most 256 px per axis.
        let decoded = crate::texture::decode(&package.source).unwrap();
        assert!(decoded.width <= 256 && decoded.height <= 256);
        assert_eq!(
            [decoded.width, decoded.height],
            [
                u16::from(BUILTIN_ATLAS_GRID[0]) * 64,
                u16::from(BUILTIN_ATLAS_GRID[1]) * 64
            ]
        );
        // Idempotent: a second terrain in the same project reuses it.
        assert_eq!(ensure_builtin_atlas(&root).unwrap(), id);
        // Rows are named so the tile grid can label them.
        assert_eq!(builtin_tile_label(0), "Grass 1");
        assert_eq!(builtin_tile_label(4), "Dirt 1");
        assert_eq!(builtin_tile_label(11), "Stone 4");
        assert_eq!(builtin_tile_label(15), "Water 4");
        // Every tile of the bundled grid is addressable by a brush.
        let component = Component {
            atlas: BUILTIN_ATLAS_GRID,
            ..Component::new(Uuid::new_v4())
        };
        component.validate().unwrap();
        assert_eq!(component.tiles(), 16);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn component_validation_bounds_the_atlas() {
        let mut component = Component::new(Uuid::new_v4());
        component.validate().unwrap();
        component.atlas = [9, 1];
        assert!(component.validate().is_err());
        component.atlas = [8, 8];
        assert_eq!(component.tiles(), 64);
        component.validate().unwrap();
        component.merge = 4;
        assert!(component.validate().is_err());
    }
}
