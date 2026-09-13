//! Conservative object-local plane masks, independent of camera sampling.
use super::Chunk;
use std::collections::BTreeMap;

const SLABS: usize = 32;
const DIRECTIONS: usize = 54;
fn direction_ranges(direction: usize) -> [(i64, i64); 3] {
    let dominant = direction / 18;
    let sign = if (direction / 9).is_multiple_of(2) {
        -2
    } else {
        2
    };
    let mut digits = direction % 9;
    let mut ranges = [(sign, sign); 3];
    for axis in (0..3).rev() {
        if axis == dominant {
            continue;
        }
        ranges[axis] = match digits % 3 {
            0 => (-2, -1),
            1 => (-1, 1),
            _ => (1, 2),
        };
        digits /= 3;
    }
    ranges
}
struct Grid {
    rows: Vec<u16>,
    masks: Vec<Vec<u32>>,
    min: i32,
    step: i32,
}
fn build(chunks: &[Chunk]) -> Grid {
    let bounds: Vec<([i64; 3], [i64; 3])> = chunks
        .iter()
        .map(|c| {
            let center: [i64; 3] =
                std::array::from_fn(|i| (c.origin[i] * 4096.) as i64 + i64::from(c.center[i]));
            // Two Q8 ticks cover truncation of local vertices and chunk origins.
            (
                std::array::from_fn(|i| center[i] - i64::from(c.extent[i]) - 32),
                std::array::from_fn(|i| center[i] + i64::from(c.extent[i]) + 32),
            )
        })
        .collect();
    let radius = bounds
        .iter()
        .map(|(lo, hi)| (0..3).map(|i| lo[i].abs().max(hi[i].abs())).sum::<i64>())
        .max()
        .unwrap_or(4096);
    // Power-of-two local slabs. The exporter already restricts representable
    // chunk origins; capping the grid only reduces optimization coverage.
    let step = ((((radius + 15) / 16).max(4096) as u64)
        .next_power_of_two()
        .min(1 << 26)) as i32;
    let min = -16 * step;
    let mut rows = vec![];
    let mut masks = vec![];
    let mut unique = BTreeMap::new();
    for direction in 0..DIRECTIONS {
        let ranges = direction_ranges(direction);
        for slab in 0..SLABS {
            let upper = i64::from(min) + (slab as i64 + 1) * i64::from(step);
            let mut mask = vec![0_u32; chunks.len().div_ceil(32)];
            for (i, (lo, hi)) in bounds.iter().enumerate() {
                let support = (0..3)
                    .map(|c| {
                        let (a, b) = ranges[c];
                        [a * lo[c], a * hi[c], b * lo[c], b * hi[c]]
                            .into_iter()
                            .max()
                            .unwrap()
                    })
                    .sum::<i64>();
                // Include touching boundaries. Products use exact integers;
                // uncorrelated interval axes deliberately overestimate support.
                if support + 2 * upper >= 0 {
                    mask[i / 32] |= 1 << (i % 32);
                }
            }
            let id = *unique.entry(mask.clone()).or_insert_with(|| {
                let id = masks.len() as u16;
                masks.push(mask);
                id
            });
            rows.push(id);
        }
    }
    Grid {
        rows,
        masks,
        min,
        step,
    }
}
pub(super) fn header(chunks: &[Chunk], index: usize) -> String {
    if chunks.len() < 2 {
        return String::new();
    }
    let g = build(chunks);
    let key = format!("editable_{index}_visibility");
    let rows = g
        .rows
        .iter()
        .map(u16::to_string)
        .collect::<Vec<_>>()
        .join(",");
    let masks = g
        .masks
        .iter()
        .flatten()
        .map(|v| format!("{v}u"))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "inline constexpr uint16_t {key}_rows[]={{{rows}}};\ninline constexpr uint32_t {key}_masks[]={{{masks}}};\ninline uint32_t {key}_combined[{words}]={{}},{key}_bounds_valid[{words}]={{}},{key}_bounds_result[{words}]={{}},{key}_basis_valid[{words}]={{}};\ninline ChunkBasisBounds {key}_basis_bounds[{chunks}]={{}};\ninline ChunkVisibilityCache {key}_cache={{{key}_combined,{key}_bounds_valid,{key}_bounds_result,{key}_basis_bounds,{key}_basis_valid}};\ninline constexpr ChunkVisibility {key}={{{key}_rows,{key}_masks,{},{},{},{},{SLABS},&{key}_cache}};\n",
        chunks.len(),
        chunks.len().div_ceil(32),
        g.min,
        g.step,
        words = chunks.len().div_ceil(32),
        chunks = chunks.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn chunk(center: [i32; 3], extent: [i32; 3]) -> Chunk {
        Chunk {
            origin: [0.; 3],
            vertices: vec![],
            faces: vec![],
            center,
            extent,
        }
    }
    #[test]
    fn masks_include_all_points_at_direction_and_offset_boundaries() {
        let chunks = vec![
            chunk([-48 * 4096, 0, 0], [4096; 3]),
            chunk([48 * 4096, 0, 0], [4096; 3]),
            chunk([0, 0, 0], [8192; 3]),
        ];
        let g = build(&chunks);
        let mut excluded = 0;
        for direction in 0..DIRECTIONS {
            let samples = direction_ranges(direction).map(|(lo, hi)| [lo * 2, lo + hi, hi * 2]);
            for slab in 0..SLABS {
                let mask = &g.masks[g.rows[direction * SLABS + slab] as usize];
                for (i, c) in chunks.iter().enumerate() {
                    if mask[i / 32] & (1 << (i % 32)) != 0 {
                        continue;
                    }
                    excluded += 1;
                    for nx in samples[0] {
                        for ny in samples[1] {
                            for nz in samples[2] {
                                for edge in [0, 1, 2] {
                                    let d = i64::from(g.min)
                                        + slab as i64 * i64::from(g.step)
                                        + i64::from(g.step) * edge / 2;
                                    for corner in 0..8 {
                                        let p: [i64; 3] = std::array::from_fn(|axis| {
                                            i64::from(c.center[axis])
                                                + if corner & (1 << axis) == 0 {
                                                    -i64::from(c.extent[axis])
                                                } else {
                                                    i64::from(c.extent[axis])
                                                }
                                        });
                                        assert!(
                                            nx * p[0] + ny * p[1] + nz * p[2] + 4 * d < 0,
                                            "excluded visible point"
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        assert!(excluded > 0);
        assert!(g.masks.len() < g.rows.len());
    }
}
