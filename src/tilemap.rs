//! TileMapLayer cells on the editor side (World2D initiative, phase 2): the compact
//! document encoding, the painting operations the 2D view's tile tools perform, and the
//! chunked tables a bank cooks (see `runtime/world2d_tiles.hpp`).
//!
//! A layer stores its cells as one base64 string (eight bytes per cell: x and y as
//! little-endian i16, atlas column and row, flip flags and a reserved byte), so a large
//! map stays one line in the map document and decodes in microseconds.
use crate::tileset::{self, Pattern, PatternCell, TileSet};
use base64::Engine;
use std::collections::{BTreeMap, BTreeSet};

/// One painted cell: an atlas tile and its flip flags (`tileset::FLIP_H`, ...).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Cell {
    pub atlas: [u8; 2],
    pub flags: u8,
}
impl Cell {
    pub fn new(atlas: [u8; 2]) -> Self {
        Self { atlas, flags: 0 }
    }
}
/// Cells by [x, y].
pub type Cells = BTreeMap<[i32; 2], Cell>;

pub const CHUNK: i32 = 16;
pub const MAX_CELLS: usize = 65536;
const MAGIC: &[u8; 4] = b"EPT1";

pub fn encode(cells: &Cells) -> String {
    if cells.is_empty() {
        return String::new();
    }
    let mut bytes = Vec::with_capacity(4 + cells.len() * 8);
    bytes.extend_from_slice(MAGIC);
    for (p, cell) in cells {
        bytes.extend_from_slice(&(p[0] as i16).to_le_bytes());
        bytes.extend_from_slice(&(p[1] as i16).to_le_bytes());
        bytes.extend_from_slice(&[cell.atlas[0], cell.atlas[1], cell.flags, 0]);
    }
    base64::engine::general_purpose::STANDARD.encode(bytes)
}
pub fn decode(text: &str) -> Result<Cells, String> {
    let mut cells = Cells::new();
    if text.is_empty() {
        return Ok(cells);
    }
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(text.trim())
        .map_err(|e| format!("tile cells: {e}"))?;
    if bytes.len() < 4 || &bytes[..4] != MAGIC || (bytes.len() - 4) % 8 != 0 {
        return Err("tile cells: not a tile layer payload".into());
    }
    if (bytes.len() - 4) / 8 > MAX_CELLS {
        return Err(format!("tile cells: at most {MAX_CELLS} cells per layer"));
    }
    for record in bytes[4..].chunks_exact(8) {
        let x = i16::from_le_bytes([record[0], record[1]]);
        let y = i16::from_le_bytes([record[2], record[3]]);
        cells.insert(
            [i32::from(x), i32::from(y)],
            Cell {
                atlas: [record[4], record[5]],
                flags: record[6] & (tileset::FLIP_H | tileset::FLIP_V | tileset::TRANSPOSE),
            },
        );
    }
    Ok(cells)
}
pub fn in_range(p: [i32; 2]) -> bool {
    p.iter().all(|v| (-32768..=32767).contains(v))
}

/// Inclusive bounding box (min, max) of the painted cells.
pub fn bounds(cells: &Cells) -> Option<([i32; 2], [i32; 2])> {
    let mut iter = cells.keys();
    let first = *iter.next()?;
    Some(iter.fold((first, first), |(lo, hi), p| {
        (
            [lo[0].min(p[0]), lo[1].min(p[1])],
            [hi[0].max(p[0]), hi[1].max(p[1])],
        )
    }))
}

// ---- painting --------------------------------------------------------------------------

/// The cells of a straight line between two cells (Bresenham), both ends included.
pub fn line(a: [i32; 2], b: [i32; 2]) -> Vec<[i32; 2]> {
    let (dx, dy) = ((b[0] - a[0]).abs(), -(b[1] - a[1]).abs());
    let (sx, sy) = (
        if a[0] < b[0] { 1 } else { -1 },
        if a[1] < b[1] { 1 } else { -1 },
    );
    let mut error = dx + dy;
    let mut p = a;
    let mut out = Vec::new();
    for _ in 0..4096 {
        out.push(p);
        if p == b {
            break;
        }
        let twice = 2 * error;
        if twice >= dy {
            error += dy;
            p[0] += sx;
        }
        if twice <= dx {
            error += dx;
            p[1] += sy;
        }
    }
    out
}
/// Every cell of the rectangle spanned by two corners, row by row.
pub fn rect(a: [i32; 2], b: [i32; 2]) -> Vec<[i32; 2]> {
    let lo = [a[0].min(b[0]), a[1].min(b[1])];
    let hi = [a[0].max(b[0]), a[1].max(b[1])];
    let mut out = Vec::new();
    for y in lo[1]..=hi[1] {
        for x in lo[0]..=hi[0] {
            out.push([x, y]);
            if out.len() >= MAX_CELLS {
                return out;
            }
        }
    }
    out
}
/// The connected region (4-neighbourhood) of cells holding the same content as `start`,
/// bounded to `limit` so filling an open area outside the map stops instead of running
/// away. When `contiguous` is false, every cell of the layer with that content.
pub fn flood(
    cells: &Cells,
    start: [i32; 2],
    contiguous: bool,
    limit: ([i32; 2], [i32; 2]),
) -> Vec<[i32; 2]> {
    let target = cells.get(&start).copied();
    if !contiguous && target.is_some() {
        return cells
            .iter()
            .filter(|(_, c)| Some(**c) == target)
            .map(|(p, _)| *p)
            .collect();
    }
    let inside = |p: [i32; 2]| {
        p[0] >= limit.0[0] && p[1] >= limit.0[1] && p[0] <= limit.1[0] && p[1] <= limit.1[1]
    };
    if !inside(start) {
        return Vec::new();
    }
    let mut seen = BTreeSet::from([start]);
    let mut queue = vec![start];
    let mut out = Vec::new();
    while let Some(p) = queue.pop() {
        out.push(p);
        if out.len() >= MAX_CELLS / 4 {
            break;
        }
        for d in [[1, 0], [-1, 0], [0, 1], [0, -1]] {
            let n = [p[0] + d[0], p[1] + d[1]];
            if inside(n) && cells.get(&n).copied() == target && seen.insert(n) {
                queue.push(n);
            }
        }
    }
    out.sort_by_key(|p| (p[1], p[0]));
    out
}

/// What a paint tool stamps: a pattern of cells (one cell for a single tile), optionally
/// replaced per cell by a random choice among them.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Stamp {
    pub cells: Vec<PatternCell>,
}
impl Stamp {
    pub fn from_pattern(pattern: &Pattern) -> Self {
        Self {
            cells: pattern.cells.clone(),
        }
    }
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }
    pub fn size(&self) -> [i32; 2] {
        let size = Pattern {
            name: String::new(),
            cells: self.cells.clone(),
        }
        .size();
        [i32::from(size[0].max(1)), i32::from(size[1].max(1))]
    }
    /// Mirrors the stamp horizontally: the layout and every cell's texels.
    pub fn flip_h(&mut self) {
        let width = self.size()[0] as u8;
        for c in &mut self.cells {
            c.offset[0] = width - 1 - c.offset[0];
            // In a transposed cell the texture's rows run horizontally on screen.
            c.flags ^= if c.flags & tileset::TRANSPOSE != 0 {
                tileset::FLIP_V
            } else {
                tileset::FLIP_H
            };
        }
    }
    pub fn flip_v(&mut self) {
        let height = self.size()[1] as u8;
        for c in &mut self.cells {
            c.offset[1] = height - 1 - c.offset[1];
            c.flags ^= if c.flags & tileset::TRANSPOSE != 0 {
                tileset::FLIP_H
            } else {
                tileset::FLIP_V
            };
        }
    }
    /// Turns the stamp a quarter clockwise on screen (a transpose, then a horizontal flip).
    pub fn rotate_clockwise(&mut self) {
        let height = self.size()[1] as u8;
        for c in &mut self.cells {
            c.offset = [height - 1 - c.offset[1], c.offset[0]];
            c.flags = rotate_flags(c.flags);
        }
    }
    pub fn rotate_counterclockwise(&mut self) {
        for _ in 0..3 {
            self.rotate_clockwise();
        }
    }
    /// The cell stamped at `p` when the stamp's top-left lands on `origin`, repeating the
    /// stamp over larger areas; with `random`, a weighted pick among all stamp cells.
    pub fn cell_at(
        &self,
        tileset: Option<&TileSet>,
        origin: [i32; 2],
        p: [i32; 2],
        random: bool,
    ) -> Option<Cell> {
        if self.cells.is_empty() {
            return None;
        }
        if random {
            let options: Vec<(PatternCell, f32)> = self
                .cells
                .iter()
                .map(|c| {
                    let weight = tileset
                        .and_then(|t| t.tile(c.atlas))
                        .map_or(1., |t| t.probability);
                    (*c, weight)
                })
                .collect();
            return tileset::weighted(&options, p, 0x51).map(|c| Cell {
                atlas: c.atlas,
                flags: c.flags,
            });
        }
        let size = self.size();
        let local = [
            (p[0] - origin[0]).rem_euclid(size[0]),
            (p[1] - origin[1]).rem_euclid(size[1]),
        ];
        self.cells
            .iter()
            .find(|c| [i32::from(c.offset[0]), i32::from(c.offset[1])] == local)
            .map(|c| Cell {
                atlas: c.atlas,
                flags: c.flags,
            })
    }
}
/// Flags after turning a cell a quarter clockwise on screen. With the sampling rule of
/// `sample`, the turned cell samples the old one at (y, 1 - x): an upright cell becomes
/// transposed with its rows mirrored, and a transposed one upright with its columns
/// mirrored.
pub fn rotate_flags(flags: u8) -> u8 {
    let (h, v) = (flags & tileset::FLIP_H != 0, flags & tileset::FLIP_V != 0);
    let (transpose, h, v) = if flags & tileset::TRANSPOSE != 0 {
        (false, !h, v)
    } else {
        (true, h, !v)
    };
    (if transpose { tileset::TRANSPOSE } else { 0 })
        | (if h { tileset::FLIP_H } else { 0 })
        | (if v { tileset::FLIP_V } else { 0 })
}
/// Where a screen-space point inside a cell samples its tile, as fractions (0..1) of the
/// tile: a transpose swaps the axes, then flips mirror the texels. Shared by the editor
/// preview so it draws exactly what the console draws.
pub fn sample(flags: u8, p: [f32; 2]) -> [f32; 2] {
    let mut s = if flags & tileset::TRANSPOSE != 0 {
        [p[1], p[0]]
    } else {
        p
    };
    if flags & tileset::FLIP_H != 0 {
        s[0] = 1. - s[0];
    }
    if flags & tileset::FLIP_V != 0 {
        s[1] = 1. - s[1];
    }
    s
}

/// A pattern from a rectangular selection of a layer, relative to its top-left cell.
pub fn pattern_from(cells: &Cells, selection: &BTreeSet<[i32; 2]>, name: &str) -> Pattern {
    let lo = selection
        .iter()
        .fold([i32::MAX; 2], |lo, p| [lo[0].min(p[0]), lo[1].min(p[1])]);
    Pattern {
        name: name.into(),
        cells: selection
            .iter()
            .filter_map(|p| {
                let cell = cells.get(p)?;
                let offset = [p[0] - lo[0], p[1] - lo[1]];
                (offset[0] < 256 && offset[1] < 256).then_some(PatternCell {
                    offset: [offset[0] as u8, offset[1] as u8],
                    atlas: cell.atlas,
                    flags: cell.flags,
                })
            })
            .collect(),
    }
}

// ---- cooking ---------------------------------------------------------------------------

/// The cooked shape of one layer: its chunk grid and the cells of every non-empty chunk
/// (`CHUNK * CHUNK` values each, row-major), using `tileset`'s cooked tile indices.
pub struct Cooked {
    pub grid_origin: [i32; 2],
    pub grid_size: [u32; 2],
    pub grid: Vec<i16>,
    pub cells: Vec<u16>,
}
pub fn cook(cells: &Cells, tileset: &TileSet) -> Result<Cooked, String> {
    let index: BTreeMap<[u8; 2], usize> = tileset
        .tiles
        .iter()
        .enumerate()
        .map(|(i, t)| (t.atlas, i))
        .collect();
    let mut chunks: BTreeMap<[i32; 2], Vec<u16>> = BTreeMap::new();
    for (p, cell) in cells {
        // A tile the TileSet no longer has cooks as an empty cell.
        let Some(tile) = index.get(&cell.atlas) else {
            continue;
        };
        let chunk = [p[0].div_euclid(CHUNK), p[1].div_euclid(CHUNK)];
        let local = [p[0].rem_euclid(CHUNK), p[1].rem_euclid(CHUNK)];
        let mut value = (*tile as u16 + 1) & 0x1fff;
        if cell.flags & tileset::FLIP_H != 0 {
            value |= 0x2000;
        }
        if cell.flags & tileset::FLIP_V != 0 {
            value |= 0x4000;
        }
        if cell.flags & tileset::TRANSPOSE != 0 {
            value |= 0x8000;
        }
        chunks
            .entry(chunk)
            .or_insert_with(|| vec![0; (CHUNK * CHUNK) as usize])
            [(local[1] * CHUNK + local[0]) as usize] = value;
    }
    let Some(lo) = chunks
        .keys()
        .copied()
        .reduce(|a, b| [a[0].min(b[0]), a[1].min(b[1])])
    else {
        return Ok(Cooked {
            grid_origin: [0, 0],
            grid_size: [0, 0],
            grid: Vec::new(),
            cells: Vec::new(),
        });
    };
    let hi = chunks
        .keys()
        .copied()
        .reduce(|a, b| [a[0].max(b[0]), a[1].max(b[1])])
        .unwrap();
    let size = [(hi[0] - lo[0] + 1) as u32, (hi[1] - lo[1] + 1) as u32];
    if u64::from(size[0]) * u64::from(size[1]) > 16384 || chunks.len() > 4096 {
        return Err(
            "tile layer spans too many chunks; keep a layer within 2048 x 2048 cells".into(),
        );
    }
    let mut grid = vec![-1i16; (size[0] * size[1]) as usize];
    let mut out = Vec::with_capacity(chunks.len() * (CHUNK * CHUNK) as usize);
    for (n, (chunk, values)) in chunks.iter().enumerate() {
        grid[((chunk[1] - lo[1]) as u32 * size[0] + (chunk[0] - lo[0]) as u32) as usize] = n as i16;
        out.extend_from_slice(values);
    }
    Ok(Cooked {
        grid_origin: lo,
        grid_size: size,
        grid,
        cells: out,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cells_round_trip_through_the_document_encoding() {
        let mut cells = Cells::new();
        cells.insert([-3, 7], Cell::new([1, 2]));
        cells.insert(
            [32767, -32768],
            Cell {
                atlas: [255, 0],
                flags: tileset::FLIP_H | tileset::TRANSPOSE,
            },
        );
        let text = encode(&cells);
        assert!(!text.contains('\n'));
        assert_eq!(decode(&text).unwrap(), cells);
        assert!(decode("").unwrap().is_empty());
        assert_eq!(encode(&Cells::new()), "");
        assert!(decode("bm90IGNlbGxz").is_err());
        assert_eq!(bounds(&cells), Some(([-3, -32768], [32767, 7])));
    }

    #[test]
    fn lines_rectangles_and_fills_cover_the_expected_cells() {
        assert_eq!(line([0, 0], [3, 1]), vec![[0, 0], [1, 0], [2, 1], [3, 1]]);
        assert_eq!(line([2, 2], [2, 2]), vec![[2, 2]]);
        assert_eq!(rect([1, 1], [0, 2]).len(), 4);
        let mut cells = Cells::new();
        for x in 0..5 {
            cells.insert([x, 0], Cell::new([0, 0]));
        }
        cells.insert([2, 1], Cell::new([1, 0]));
        // Contiguous fill of the empty area under the row stays inside the limit box.
        let empty = flood(&cells, [0, 1], true, ([0, 1], [4, 2]));
        assert_eq!(empty.len(), 9);
        assert!(!empty.contains(&[2, 1]));
        // Replacing every matching tile ignores connectivity.
        cells.insert([9, 9], Cell::new([0, 0]));
        assert_eq!(flood(&cells, [0, 0], false, ([0, 0], [0, 0])).len(), 6);
        assert_eq!(flood(&cells, [0, 0], true, ([-10, -10], [10, 10])).len(), 5);
    }

    #[test]
    fn stamps_repeat_flip_and_rotate() {
        let mut stamp = Stamp {
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
        assert_eq!(stamp.size(), [2, 1]);
        assert_eq!(
            stamp.cell_at(None, [0, 0], [3, 5], false).unwrap().atlas,
            [1, 0]
        );
        stamp.flip_h();
        assert_eq!(
            stamp.cell_at(None, [0, 0], [0, 0], false).unwrap().atlas,
            [1, 0]
        );
        assert_eq!(stamp.cells[0].flags, tileset::FLIP_H);
        stamp.rotate_clockwise();
        assert_eq!(stamp.size(), [1, 2]);
        // Four quarter turns give back the original cells.
        let mut turned = stamp.clone();
        for _ in 0..4 {
            turned.rotate_clockwise();
        }
        assert_eq!(turned, stamp);
        // The editor preview samples exactly like the runtime: transpose, then flips.
        assert_eq!(
            sample(tileset::TRANSPOSE | tileset::FLIP_H, [0.25, 0.75]),
            [0.25, 0.25]
        );
        // A clockwise turn of an upright cell: screen top-left samples the texel column's
        // bottom (the tile's left edge appears at the top).
        let rotated = rotate_flags(0);
        assert_eq!(sample(rotated, [0., 0.]), [0., 1.]);
        assert_eq!(sample(rotated, [1., 0.]), [0., 0.]);
    }

    #[test]
    fn cooking_builds_a_sparse_chunk_grid() {
        let set = TileSet {
            tiles: vec![tileset::Tile::new([0, 0]), tileset::Tile::new([3, 1])],
            ..Default::default()
        };
        let mut cells = Cells::new();
        cells.insert([0, 0], Cell::new([0, 0]));
        cells.insert(
            [-1, 40],
            Cell {
                atlas: [3, 1],
                flags: tileset::FLIP_V,
            },
        );
        cells.insert([5, 5], Cell::new([9, 9]));
        let cooked = cook(&cells, &set).unwrap();
        assert_eq!(cooked.grid_origin, [-1, 0]);
        assert_eq!(cooked.grid_size, [2, 3]);
        // Only two of the six chunk slots hold data; the unknown tile cooks as empty.
        assert_eq!(cooked.grid.iter().filter(|g| **g >= 0).count(), 2);
        assert_eq!(cooked.cells.len(), 2 * 256);
        let slot = cooked.grid[(2 * 2) as usize];
        assert!(slot >= 0);
        assert_eq!(cooked.cells[slot as usize * 256 + 8 * 16 + 15], 2 | 0x4000);
        let slot = cooked.grid[1];
        assert_eq!(cooked.cells[slot as usize * 256], 1);
    }
}
