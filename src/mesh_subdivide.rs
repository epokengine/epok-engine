//! Texture-correction subdivision for editable meshes.
//!
//! The PSX GPU interpolates texture coordinates affinely, so a large polygon
//! seen at an angle bends straight texture lines along its triangle seam. The
//! fix of the era was modelling: smaller polygons near the camera. This is an
//! authoring operation: it rewrites the mesh document, and everything the build
//! derives from it (chunks, streaming pages, packet capacity, budgets) follows.
//!
//! Splits never open cracks. Opposite edges of a quad, and all edges of a
//! triangle, are joined into edge rings (loops); every edge of a ring is cut
//! into the same number of pieces at the same points, so faces that share an
//! edge share every new vertex. A selection therefore extends along its rings,
//! exactly like inserting edge loops in a modelling package.
use crate::mesh::{Document, Face};
use std::collections::{BTreeSet, HashMap};
use uuid::Uuid;

/// Smallest edge length the tool accepts (local units). Smaller pieces risk
/// collapsing at the PSX's Q12 precision.
pub const MIN_EDGE: f32 = 0.25;

/// What a subdivision would do to a document.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Plan {
    /// Faces that are split (the selection plus the rings it reaches).
    pub split: usize,
    pub faces_before: usize,
    pub faces_after: usize,
    /// Faces the split reaches that the caller protected (hidden or locked).
    pub protected: usize,
}
impl Plan {
    /// Extra PSX triangles: quads draw as two, triangles as one.
    pub fn added_faces(&self) -> usize {
        self.faces_after.saturating_sub(self.faces_before)
    }
}

type Key = [i32; 3];

/// Positions are compared on the target's Q12 grid, so faces that share an
/// edge through different vertex indices still share it here.
fn key(p: [f32; 3]) -> Key {
    p.map(|c| (c * 4096.).round() as i32)
}
fn edge_key(a: [f32; 3], b: [f32; 3]) -> Option<(Key, Key)> {
    let (a, b) = (key(a), key(b));
    match a.cmp(&b) {
        std::cmp::Ordering::Less => Some((a, b)),
        std::cmp::Ordering::Greater => Some((b, a)),
        std::cmp::Ordering::Equal => None,
    }
}
fn triangle(f: &Face) -> bool {
    f.vertices[2] == f.vertices[3]
}
fn length(a: [f32; 3], b: [f32; 3]) -> f32 {
    crate::lighting::dot(crate::lighting::sub(b, a), crate::lighting::sub(b, a)).sqrt()
}

struct Rings {
    ids: HashMap<(Key, Key), usize>,
    parent: Vec<usize>,
    length: Vec<f32>,
}
impl Rings {
    fn edge(&mut self, a: [f32; 3], b: [f32; 3]) -> Option<usize> {
        let k = edge_key(a, b)?;
        let next = self.parent.len();
        let id = *self.ids.entry(k).or_insert(next);
        if id == next {
            self.parent.push(id);
            self.length.push(length(a, b));
        }
        Some(id)
    }
    fn find(&mut self, mut i: usize) -> usize {
        while self.parent[i] != i {
            self.parent[i] = self.parent[self.parent[i]];
            i = self.parent[i];
        }
        i
    }
    fn union(&mut self, a: Option<usize>, b: Option<usize>) {
        if let (Some(a), Some(b)) = (a, b) {
            let (a, b) = (self.find(a), self.find(b));
            if a != b {
                self.parent[a] = b;
            }
        }
    }
}

/// Edges of a face in corner order: 0-1, 1-2, 2-3, 3-0 (a triangle's third
/// edge is 2-0).
fn corners(f: &Face) -> usize {
    if triangle(f) { 3 } else { 4 }
}

/// The pieces each face is cut into: `[along 0-1, along 0-3]` for quads, and
/// `[n, n]` for a triangle cut into n x n.
fn counts(doc: &Document, scope: &BTreeSet<Uuid>, max_edge: f32) -> Vec<[usize; 2]> {
    let mut rings = Rings {
        ids: HashMap::new(),
        parent: Vec::new(),
        length: Vec::new(),
    };
    let mut face_edges = Vec::with_capacity(doc.faces.len());
    for f in &doc.faces {
        let p = doc.points(f);
        let n = corners(f);
        let edges: Vec<Option<usize>> = (0..n).map(|i| rings.edge(p[i], p[(i + 1) % n])).collect();
        if n == 4 {
            rings.union(edges[0], edges[2]);
            rings.union(edges[1], edges[3]);
        } else {
            rings.union(edges[0], edges[1]);
            rings.union(edges[1], edges[2]);
        }
        face_edges.push(edges);
    }
    let all = scope.is_empty();
    let mut wanted = vec![false; rings.parent.len()];
    for (f, edges) in doc.faces.iter().zip(&face_edges) {
        if all || scope.contains(&f.id) {
            for e in edges.iter().flatten() {
                let r = rings.find(*e);
                wanted[r] = true;
            }
        }
    }
    let mut pieces = vec![1_usize; rings.parent.len()];
    for e in 0..rings.parent.len() {
        let r = rings.find(e);
        if wanted[r] {
            let n = (rings.length[e] / max_edge).ceil().max(1.) as usize;
            pieces[r] = pieces[r].max(n);
        }
    }
    face_edges
        .iter()
        .zip(&doc.faces)
        .map(|(edges, f)| {
            let count =
                |e: Option<usize>, rings: &mut Rings| e.map_or(1, |e| pieces[rings.find(e)]);
            if triangle(f) {
                let n = count(edges[0], &mut rings);
                [n, n]
            } else {
                [count(edges[0], &mut rings), count(edges[3], &mut rings)]
            }
        })
        .collect()
}

/// What subdividing `scope` (every face when empty) to edges of at most
/// `max_edge` local units would do. `protected` faces must not change.
pub fn plan(
    doc: &Document,
    scope: &BTreeSet<Uuid>,
    protected: &BTreeSet<Uuid>,
    max_edge: f32,
) -> Plan {
    let max_edge = max_edge.max(MIN_EDGE);
    let mut plan = Plan {
        faces_before: doc.faces.len(),
        ..Plan::default()
    };
    for (f, [u, v]) in doc.faces.iter().zip(counts(doc, scope, max_edge)) {
        let pieces = u * v;
        plan.faces_after += pieces;
        if pieces > 1 {
            plan.split += 1;
            if protected.contains(&f.id) {
                plan.protected += 1;
            }
        }
    }
    plan
}

/// Cut `scope` (every face when empty) so no edge is longer than `max_edge`
/// local units. Split faces get new IDs; untouched faces keep theirs.
pub fn apply(
    doc: &mut Document,
    scope: &BTreeSet<Uuid>,
    protected: &BTreeSet<Uuid>,
    max_edge: f32,
) -> Result<Plan, String> {
    if !max_edge.is_finite() || max_edge < MIN_EDGE {
        return Err(format!(
            "Choose a maximum edge length of at least {MIN_EDGE} units"
        ));
    }
    let summary = plan(doc, scope, protected, max_edge);
    if summary.protected > 0 {
        return Err(format!(
            "The split would reach {} hidden or locked faces along their edge loops. Show and unlock them first.",
            summary.protected
        ));
    }
    if summary.faces_after > 3500 {
        return Err(format!(
            "The result would have {} faces; a mesh holds at most 3500. Use a larger edge length or select fewer faces.",
            summary.faces_after
        ));
    }
    if summary.split == 0 {
        return Ok(summary);
    }
    let pieces = counts(doc, scope, max_edge);
    let mut welded: HashMap<Key, u32> = HashMap::new();
    for (i, p) in doc.vertices.iter().enumerate() {
        welded.entry(key(*p)).or_insert(i as u32);
    }
    let mut vertices = std::mem::take(&mut doc.vertices);
    // Corners are read from the authored positions; new points are appended.
    let positions = vertices.clone();
    let mut vertex = |p: [f32; 3]| -> u32 {
        let p = p.map(|c| (c * 4096.).round() / 4096.);
        *welded.entry(key(p)).or_insert_with(|| {
            vertices.push(p);
            (vertices.len() - 1) as u32
        })
    };
    let source = std::mem::take(&mut doc.faces);
    let mut faces = Vec::with_capacity(summary.faces_after);
    for (f, [nu, nv]) in source.iter().zip(pieces) {
        if nu * nv <= 1 {
            faces.push(f.clone());
            continue;
        }
        let p = f.vertices.map(|v| positions[v as usize]);
        let piece = |vertices: [u32; 4], uv: [[f32; 2]; 4], colors| Face {
            id: Uuid::new_v4(),
            vertices,
            group: f.group,
            material: f.material,
            uv,
            vertex_colors: colors,
        };
        if triangle(f) {
            let n = nu;
            // Barycentric grid: i along corner 0 -> 1, j along corner 0 -> 2.
            let point = |i: usize, j: usize| -> [f32; 3] {
                if j == 0 {
                    edge_point(p[0], p[1], i, n)
                } else if i == 0 {
                    edge_point(p[0], p[2], j, n)
                } else if i + j == n {
                    edge_point(p[1], p[2], j, n)
                } else {
                    std::array::from_fn(|c| {
                        p[0][c]
                            + (p[1][c] - p[0][c]) * i as f32 / n as f32
                            + (p[2][c] - p[0][c]) * j as f32 / n as f32
                    })
                }
            };
            let weights = |i: usize, j: usize| {
                let (b, c) = (i as f32 / n as f32, j as f32 / n as f32);
                [1. - b - c, b, c]
            };
            let uv = |w: [f32; 3]| -> [f32; 2] {
                std::array::from_fn(|c| f.uv[0][c] * w[0] + f.uv[1][c] * w[1] + f.uv[2][c] * w[2])
            };
            let color = |w: [f32; 3]| {
                f.vertex_colors.map(|colors| {
                    std::array::from_fn(|c| {
                        (f32::from(colors[0][c]) * w[0]
                            + f32::from(colors[1][c]) * w[1]
                            + f32::from(colors[2][c]) * w[2])
                            .round()
                            .clamp(0., 255.) as u8
                    })
                })
            };
            for j in 0..n {
                for i in 0..n - j {
                    let mut triangles = vec![[(i, j), (i + 1, j), (i, j + 1)]];
                    if i + j + 1 < n {
                        triangles.push([(i + 1, j), (i + 1, j + 1), (i, j + 1)]);
                    }
                    for t in triangles {
                        let w = t.map(|(i, j)| weights(i, j));
                        let v = t.map(|(i, j)| vertex(point(i, j)));
                        let colors = color(w[0])
                            .zip(color(w[1]))
                            .zip(color(w[2]))
                            .map(|((a, b), c)| [a, b, c, c]);
                        faces.push(piece(
                            [v[0], v[1], v[2], v[2]],
                            [uv(w[0]), uv(w[1]), uv(w[2]), uv(w[2])],
                            colors,
                        ));
                    }
                }
            }
            continue;
        }
        // Quad grid: i along corners 0 -> 1 (and 3 -> 2), j along 0 -> 3 (and 1 -> 2).
        let point = |i: usize, j: usize| -> [f32; 3] {
            if j == 0 {
                edge_point(p[0], p[1], i, nu)
            } else if j == nv {
                edge_point(p[3], p[2], i, nu)
            } else if i == 0 {
                edge_point(p[0], p[3], j, nv)
            } else if i == nu {
                edge_point(p[1], p[2], j, nv)
            } else {
                let (u, v) = (i as f32 / nu as f32, j as f32 / nv as f32);
                bilinear(p, u, v)
            }
        };
        let uv = |i: usize, j: usize| -> [f32; 2] {
            let (u, v) = (i as f32 / nu as f32, j as f32 / nv as f32);
            std::array::from_fn(|c| {
                f.uv[0][c] * (1. - u) * (1. - v)
                    + f.uv[1][c] * u * (1. - v)
                    + f.uv[2][c] * u * v
                    + f.uv[3][c] * (1. - u) * v
            })
        };
        let color = |i: usize, j: usize| {
            f.vertex_colors.map(|colors| {
                let (u, v) = (i as f32 / nu as f32, j as f32 / nv as f32);
                std::array::from_fn(|c| {
                    (f32::from(colors[0][c]) * (1. - u) * (1. - v)
                        + f32::from(colors[1][c]) * u * (1. - v)
                        + f32::from(colors[2][c]) * u * v
                        + f32::from(colors[3][c]) * (1. - u) * v)
                        .round()
                        .clamp(0., 255.) as u8
                })
            })
        };
        for j in 0..nv {
            for i in 0..nu {
                let corners = [(i, j), (i + 1, j), (i + 1, j + 1), (i, j + 1)];
                let colors = f
                    .vertex_colors
                    .map(|_| corners.map(|(i, j)| color(i, j).unwrap()));
                faces.push(piece(
                    corners.map(|(i, j)| vertex(point(i, j))),
                    corners.map(|(i, j)| uv(i, j)),
                    colors,
                ));
            }
        }
    }
    doc.vertices = vertices;
    doc.faces = faces;
    doc.compact();
    if doc.vertices.len() > 16384 {
        return Err("The result would exceed 16384 vertices; use a larger edge length".into());
    }
    Ok(summary)
}

/// Point `k` of `n` along an edge, computed from its canonical end so both
/// faces sharing the edge produce bit-identical positions.
fn edge_point(a: [f32; 3], b: [f32; 3], k: usize, n: usize) -> [f32; 3] {
    let (from, to, k) = if key(a) <= key(b) {
        (a, b, k)
    } else {
        (b, a, n - k)
    };
    let t = k as f32 / n as f32;
    std::array::from_fn(|c| from[c] + (to[c] - from[c]) * t)
}
fn bilinear(p: [[f32; 3]; 4], u: f32, v: f32) -> [f32; 3] {
    std::array::from_fn(|c| {
        p[0][c] * (1. - u) * (1. - v)
            + p[1][c] * u * (1. - v)
            + p[2][c] * u * v
            + p[3][c] * (1. - u) * v
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn floor(cells: usize, size: f32) -> Document {
        let mut doc = Document::default();
        let (group, slot) = (doc.groups[0].id, doc.materials[0].id);
        for z in 0..cells {
            for x in 0..cells {
                let (x0, z0) = (x as f32 * size, z as f32 * size);
                doc.add_face(
                    [
                        [x0, 0., z0],
                        [x0, 0., z0 + size],
                        [x0 + size, 0., z0 + size],
                        [x0 + size, 0., z0],
                    ],
                    group,
                    slot,
                );
            }
        }
        doc
    }
    /// Every edge used by exactly one face must lie on the mesh's outline:
    /// an interior edge seen once is a crack (a T-junction).
    fn open_edges(doc: &Document) -> BTreeMap<(Key, Key), usize> {
        let mut uses = BTreeMap::new();
        for f in &doc.faces {
            let p = doc.points(f);
            let n = corners(f);
            for i in 0..n {
                if let Some(k) = edge_key(p[i], p[(i + 1) % n]) {
                    *uses.entry(k).or_insert(0) += 1;
                }
            }
        }
        uses.retain(|_, n| *n == 1);
        uses
    }
    fn on_outline(doc: &Document, (a, b): (Key, Key), low: f32, high: f32) -> bool {
        let _ = doc;
        let edge = |v: i32| {
            (v as f32 / 4096. - low).abs() < 1e-3 || (v as f32 / 4096. - high).abs() < 1e-3
        };
        (edge(a[0]) && edge(b[0]) && a[0] == b[0]) || (edge(a[2]) && edge(b[2]) && a[2] == b[2])
    }

    #[test]
    fn whole_mesh_splits_every_long_edge_without_cracks() {
        let mut doc = floor(3, 4.);
        let summary = apply(&mut doc, &BTreeSet::new(), &BTreeSet::new(), 1.).unwrap();
        assert_eq!(summary.faces_before, 9);
        assert_eq!(summary.faces_after, 9 * 16);
        assert_eq!(doc.faces.len(), 144);
        doc.validate().unwrap();
        // 12 x 12 grid of welded vertices.
        assert_eq!(doc.vertices.len(), 13 * 13);
        for edge in open_edges(&doc).into_keys() {
            assert!(on_outline(&doc, edge, 0., 12.), "crack at {edge:?}");
        }
    }

    #[test]
    fn a_selection_extends_along_its_edge_loops_and_stays_crack_free() {
        let mut doc = floor(3, 4.);
        let centre = doc.faces[4].id;
        let plan = plan(&doc, &BTreeSet::from([centre]), &BTreeSet::new(), 2.);
        // The centre's two rings cross the middle row and column: 5 faces,
        // the centre cut 2 x 2, its four neighbours cut 2 x 1.
        assert_eq!(plan.split, 5);
        assert_eq!(plan.faces_after, 4 + 4 + 4 * 2);
        apply(&mut doc, &BTreeSet::from([centre]), &BTreeSet::new(), 2.).unwrap();
        doc.validate().unwrap();
        assert_eq!(doc.faces.len(), 16);
        for edge in open_edges(&doc).into_keys() {
            assert!(on_outline(&doc, edge, 0., 12.), "crack at {edge:?}");
        }
    }

    #[test]
    fn untouched_faces_keep_their_ids_and_short_edges_are_left_alone() {
        let mut doc = floor(2, 1.);
        let before = doc.clone();
        let summary = apply(&mut doc, &BTreeSet::new(), &BTreeSet::new(), 2.).unwrap();
        assert_eq!(summary.split, 0);
        assert_eq!(doc, before, "nothing longer than the limit: no change");
    }

    #[test]
    fn uvs_and_colours_are_interpolated_across_the_pieces() {
        let mut doc = floor(1, 4.);
        doc.faces[0].uv = [[0., 0.], [0., 64.], [64., 64.], [64., 0.]];
        doc.faces[0].vertex_colors = Some([[0, 0, 0], [0, 0, 0], [255, 255, 255], [255, 255, 255]]);
        apply(&mut doc, &BTreeSet::new(), &BTreeSet::new(), 2.).unwrap();
        assert_eq!(doc.faces.len(), 4);
        let first = &doc.faces[0];
        assert_eq!(first.uv, [[0., 0.], [0., 32.], [32., 32.], [32., 0.]]);
        let colors = first.vertex_colors.unwrap();
        assert_eq!(colors[0], [0, 0, 0]);
        assert_eq!(colors[2], [128, 128, 128]);
        for f in &doc.faces {
            assert_eq!(
                (f.group, f.material),
                (before_group(&doc), before_slot(&doc))
            );
        }
    }
    fn before_group(doc: &Document) -> Uuid {
        doc.groups[0].id
    }
    fn before_slot(doc: &Document) -> Uuid {
        doc.materials[0].id
    }

    #[test]
    fn triangles_split_into_matching_triangles() {
        let mut doc = Document::default();
        let (group, slot) = (doc.groups[0].id, doc.materials[0].id);
        doc.add_face(
            [[0., 0., 0.], [0., 0., 4.], [4., 0., 0.], [4., 0., 0.]],
            group,
            slot,
        );
        doc.add_face(
            [[0., 0., 4.], [4., 0., 4.], [4., 0., 0.], [4., 0., 0.]],
            group,
            slot,
        );
        apply(&mut doc, &BTreeSet::new(), &BTreeSet::new(), 2.).unwrap();
        doc.validate().unwrap();
        // The diagonal is longer than the sides, so every ring takes 3 pieces.
        assert_eq!(doc.faces.len(), 2 * 9);
        assert!(doc.faces.iter().all(|f| f.vertices[2] == f.vertices[3]));
        for edge in open_edges(&doc).into_keys() {
            assert!(on_outline(&doc, edge, 0., 4.), "crack at {edge:?}");
        }
    }

    #[test]
    fn a_box_splits_all_around_and_validates() {
        let mut doc = Document::default();
        let (group, slot) = (doc.groups[0].id, doc.materials[0].id);
        doc.primitive("Box", [0.; 3], [8., 2., 4.], 1, group, slot);
        apply(&mut doc, &BTreeSet::new(), &BTreeSet::new(), 2.).unwrap();
        doc.validate().unwrap();
        // A closed box has no outline: every edge is shared by two faces.
        assert!(open_edges(&doc).is_empty(), "{:?}", open_edges(&doc));
        // x edges (8) take 4 pieces, z edges (4) take 2, y edges (2) stay whole:
        // front/back 4 x 1, top/bottom 4 x 2, left/right 2 x 1.
        assert_eq!(doc.faces.len(), 2 * (4 + 8 + 2));
    }

    #[test]
    fn protected_faces_and_limits_refuse_without_changing_the_mesh() {
        let mut doc = floor(3, 4.);
        let before = doc.clone();
        let locked = BTreeSet::from([doc.faces[1].id]);
        let centre = BTreeSet::from([doc.faces[4].id]);
        let error = apply(&mut doc, &centre, &locked, 2.).unwrap_err();
        assert!(error.contains("locked"), "{error}");
        assert_eq!(doc, before);
        assert!(apply(&mut doc, &BTreeSet::new(), &BTreeSet::new(), 0.1).is_err());
        let mut big = floor(4, 32.);
        let error = apply(&mut big, &BTreeSet::new(), &BTreeSet::new(), 0.5).unwrap_err();
        assert!(error.contains("3500"), "{error}");
    }
}
