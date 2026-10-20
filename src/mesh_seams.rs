//! Render-only coverage at nonconforming wall joins. Authored faces, bake corner
//! indices, normals, UVs and collision geometry stay unchanged.
use super::{Chunk, Quad};
use std::collections::BTreeMap;

const GUARD: i64 = 512; // 1/8 unit; capped to a quarter of a small panel's width.
type Point = [i32; 3];
struct Floor {
    face: usize,
    corner: usize,
    height: i32,
    center: [i64; 2],
    guard: i32,
}
#[derive(Clone, Copy)]
struct Edge {
    face: usize,
    corners: [usize; 2],
    axis: usize,
    normal: i64,
    line: i32,
    center: i64,
    lo: i32,
    hi: i32,
    guard: i32,
}
impl Edge {
    fn overlaps(self, other: Self) -> bool {
        self.lo.max(other.lo) < self.hi.min(other.hi)
    }
    fn opposite(self, other: Self) -> bool {
        (self.center - 4 * i64::from(self.line)).signum()
            != (other.center - 4 * i64::from(other.line)).signum()
    }
    fn outward(self) -> i32 {
        if 4 * i64::from(self.line) > self.center {
            self.guard
        } else {
            -self.guard
        }
    }
}

pub(super) fn close(quads: &[Quad], chunks: &mut [Chunk]) -> Result<(), String> {
    let points: Vec<[Point; 4]> = quads
        .iter()
        .map(|q| q.points.map(|p| p.map(|v| (v * 4096.).round() as i32)))
        .collect();
    let mut lines: BTreeMap<(usize, i32, i32, bool), Vec<Edge>> = BTreeMap::new();
    let mut corners: BTreeMap<[i32; 2], Vec<Edge>> = BTreeMap::new();
    let mut floors: BTreeMap<[i32; 2], Vec<Floor>> = BTreeMap::new();
    for (face, p) in points.iter().enumerate() {
        let material = &quads[face].material;
        // Additional coverage must never accumulate translucent layers or
        // interfere with the separate scrolling/clipping path.
        if material.blend != crate::texture::BlendMode::Cutout || material.uv_scroll != [0.; 2] {
            continue;
        }
        if quads[face].normal[1] > 0. && p.iter().all(|v| v[1] == p[0][1]) {
            let center = [
                p.iter().map(|v| i64::from(v[0])).sum(),
                p.iter().map(|v| i64::from(v[2])).sum(),
            ];
            let guard = GUARD.min(
                (0..=2)
                    .step_by(2)
                    .map(|a| {
                        i64::from(p.iter().map(|v| v[a]).max().unwrap())
                            - i64::from(p.iter().map(|v| v[a]).min().unwrap())
                    })
                    .min()
                    .unwrap()
                    / 4,
            ) as i32;
            for (corner, v) in p.iter().enumerate() {
                floors.entry([v[0], v[2]]).or_default().push(Floor {
                    face,
                    corner,
                    height: v[1],
                    center,
                    guard,
                });
            }
        }
        if p[0][1] != p[1][1] || p[2][1] != p[3][1] || p[0][1] == p[3][1] {
            continue;
        }
        let axis = if p.iter().all(|v| v[0] == p[0][0]) {
            0
        } else if p.iter().all(|v| v[2] == p[0][2]) {
            2
        } else {
            continue;
        };
        let tangent = 2 - axis;
        if p[0][tangent] != p[3][tangent]
            || p[1][tangent] != p[2][tangent]
            || p[0][tangent] == p[1][tangent]
        {
            continue;
        }
        let winding = (p[1][tangent] > p[0][tangent]) == (p[3][1] > p[0][1]);
        let normal = if (axis == 0) == winding { -1 } else { 1 };
        let width = (i64::from(p[1][tangent]) - i64::from(p[0][tangent])).abs();
        let guard = GUARD.min(width / 4) as i32;
        if guard == 0 {
            continue;
        }
        for pair in [[0, 3], [1, 2]] {
            let a = p[pair[0]];
            let b = p[pair[1]];
            let edge = Edge {
                face,
                corners: pair,
                axis,
                normal,
                line: a[tangent],
                center: p.iter().map(|v| i64::from(v[tangent])).sum(),
                lo: a[1].min(b[1]),
                hi: a[1].max(b[1]),
                guard,
            };
            lines
                .entry((axis, a[axis], a[tangent], winding))
                .or_default()
                .push(edge);
            corners.entry([a[0], a[2]]).or_default().push(edge);
        }
    }
    let mut shifts = vec![[[0_i32; 3]; 4]; quads.len()];
    let extend = |shifts: &mut Vec<[[i32; 3]; 4]>, edge: Edge| {
        for corner in edge.corners {
            shifts[edge.face][corner][2 - edge.axis] = edge.outward();
        }
    };
    // A longer edge meeting a shorter coplanar edge is a T junction. Overlap
    // only the longer panel into its neighbour; never grow an open doorway.
    for edges in lines.values() {
        for &edge in edges {
            if edges.iter().any(|&other| {
                other.face != edge.face
                    && edge.opposite(other)
                    && edge.lo <= other.lo
                    && other.hi <= edge.hi
                    && (edge.lo < other.lo || other.hi < edge.hi)
            }) {
                // At a grazing view, 1/8 unit projects to less than the GTE's
                // one-pixel edge disagreement. Coplanar T joins need a wider
                // overlap; retain the quarter-panel cap on narrow walls.
                let guard =
                    (GUARD * 4).min((edge.center - 4 * i64::from(edge.line)).abs() / 8) as i32;
                extend(&mut shifts, Edge { guard, ..edge });
            }
        }
    }
    // A floor wrapping around a convex solid corner can expose one pixel at
    // the three-surface join too. Exclude free-standing decoration and inward
    // room corners: there must be an authored floor corner at the wall's foot.
    for (at, edges) in &corners {
        for &edge in edges {
            if !floors.get(at).is_some_and(|ys| {
                ys.iter().any(|floor| {
                    i64::from(floor.height) >= i64::from(edge.lo)
                        && i64::from(floor.height) <= i64::from(edge.lo) + GUARD
                        && (floor.center[edge.axis / 2] - 4 * i64::from(at[edge.axis / 2]))
                            * edge.normal
                            > 0
                })
            }) {
                continue;
            }
            if edges.iter().any(|&other| {
                other.face != edge.face
                    && other.axis != edge.axis
                    && edge.overlaps(other)
                    && (edge.center - 4 * i64::from(edge.line)) * other.normal < 0
                    && (other.center - 4 * i64::from(other.line)) * edge.normal < 0
            }) {
                extend(&mut shifts, edge);
                // Carry the common floor corner just under the solid wall.
                // Otherwise the extended walls no longer share its projected
                // endpoint and integer rasterization can leave a pinhole.
                let joined: Vec<_> = floors[at]
                    .iter()
                    .filter(|floor| {
                        i64::from(floor.height) >= i64::from(edge.lo)
                            && i64::from(floor.height) <= i64::from(edge.lo) + GUARD
                    })
                    .collect();
                let guard = joined
                    .iter()
                    .map(|f| f.guard)
                    .min()
                    .unwrap_or(0)
                    .min(edge.guard);
                for floor in joined {
                    shifts[floor.face][floor.corner][edge.axis] = -edge.normal as i32 * guard;
                }
            }
        }
    }
    for chunk in chunks {
        if !chunk
            .faces
            .iter()
            .any(|(_, face)| shifts[*face] != [[0; 3]; 4])
        {
            continue;
        }
        let origin = chunk.origin.map(|v| (v * 4096.).round() as i32);
        let mut vertices = vec![];
        let mut lookup = BTreeMap::new();
        for (indices, face) in &mut chunk.faces {
            for (corner, index) in indices.iter_mut().enumerate() {
                let mut local = [0; 3];
                for (axis, out) in local.iter_mut().enumerate() {
                    *out = points[*face][corner][axis]
                        .checked_add(shifts[*face][corner][axis])
                        .and_then(|v| v.checked_sub(origin[axis]))
                        .ok_or("Mesh seam coordinate overflow")?;
                }
                *index = *lookup.entry(local).or_insert_with(|| {
                    let index = vertices.len() as u16;
                    vertices.push(local);
                    index
                });
            }
        }
        let low: Point =
            std::array::from_fn(|axis| vertices.iter().map(|p| p[axis]).min().unwrap());
        let high: Point =
            std::array::from_fn(|axis| vertices.iter().map(|p| p[axis]).max().unwrap());
        chunk.center =
            std::array::from_fn(|a| ((i64::from(low[a]) + i64::from(high[a])) / 2) as i32);
        chunk.extent =
            std::array::from_fn(|a| ((i64::from(high[a]) - i64::from(low[a]) + 1) / 2) as i32);
        chunk.vertices = vertices;
    }
    Ok(())
}

#[cfg(test)]
#[path = "mesh_seams_tests.rs"]
mod tests;
