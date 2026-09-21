//! Topology operations on the authored mesh; callers publish only after validation succeeds.
use crate::{
    lighting::{dot, sub, unit},
    mesh::{Document, Face, face_normal},
};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

pub type Edge = [u32; 2];
pub fn edge(a: u32, b: u32) -> Edge {
    if a < b { [a, b] } else { [b, a] }
}
pub fn edges(f: &Face) -> Vec<Edge> {
    let n = if f.vertices[2] == f.vertices[3] { 3 } else { 4 };
    (0..n)
        .map(|i| edge(f.vertices[i], f.vertices[(i + 1) % n]))
        .collect()
}
#[derive(Clone, Copy)]
struct Corner {
    p: [f32; 3],
    uv: [f32; 2],
}

/// Clip a convex closed solid by a plane parallel to its selected edge.
/// This removes the corner and creates a material-bearing slope, rather than
/// collapsing the edge's endpoints or deleting a surface and leaving a hole.
pub fn bevel(doc: &mut Document, selection: &BTreeSet<Edge>, distance: f32) -> Result<(), String> {
    if selection.is_empty() {
        return Err("Select an edge to bevel".into());
    }
    if !distance.is_finite() || distance < 0.001 {
        return Err("Bevel distance must be positive".into());
    }
    let mut result = doc.clone();
    for selected in selection {
        bevel_one(&mut result, *selected, distance)?;
    }
    result.compact();
    result.validate()?;
    *doc = result;
    Ok(())
}
fn bevel_one(doc: &mut Document, selected: Edge, distance: f32) -> Result<(), String> {
    let mut adjacency: BTreeMap<Edge, Vec<usize>> = BTreeMap::new();
    for (i, f) in doc.faces.iter().enumerate() {
        for edge in edges(f) {
            adjacency.entry(edge).or_default().push(i);
        }
    }
    let pair = adjacency
        .get(&selected)
        .filter(|v| v.len() == 2)
        .ok_or("Bevel needs an edge shared by exactly two faces")?;
    if pair
        .iter()
        .any(|i| doc.faces[*i].vertices[2] == doc.faces[*i].vertices[3])
    {
        return Err("Bevel currently requires two quad faces, not triangles".into());
    }
    let mut solid = BTreeSet::new();
    let mut pending = pair.clone();
    while let Some(i) = pending.pop() {
        if !solid.insert(i) {
            continue;
        }
        for edge in edges(&doc.faces[i]) {
            let neighbors = &adjacency[&edge];
            if neighbors.len() != 2 {
                return Err("Bevel requires a closed solid without non-manifold edges".into());
            }
            pending.extend(neighbors.iter().filter(|i| !solid.contains(i)));
        }
    }
    let normals = pair
        .iter()
        .map(|i| face_normal(doc.points(&doc.faces[*i])))
        .collect::<Vec<_>>();
    if dot(normals[0], normals[1]).abs() > 0.999 {
        return Err("Choose a corner between non-coplanar faces".into());
    }
    // Plane clipping is unambiguous for a convex shell; reject concave geometry
    // atomically rather than slicing unrelated rooms behind the selected corner.
    let points = solid
        .iter()
        .flat_map(|i| doc.points(&doc.faces[*i]))
        .collect::<Vec<_>>();
    for i in &solid {
        let p = doc.points(&doc.faces[*i]);
        let normal = face_normal(p);
        if points.iter().any(|v| dot(sub(*v, p[0]), normal) > 0.001) {
            return Err("Bevel currently supports convex solids with outward faces".into());
        }
    }
    let normal = unit(std::array::from_fn(|c| normals[0][c] + normals[1][c]));
    let limit = dot(doc.vertices[selected[0] as usize], normal) - distance;
    if !points.iter().any(|p| dot(*p, normal) < limit - 0.001) {
        return Err("Bevel distance would remove the entire solid".into());
    }
    let source = doc.faces[pair[0]].clone();
    let mut cap = Vec::<[f32; 3]>::new();
    let mut output = Vec::new();
    for (i, f) in doc.faces.clone().into_iter().enumerate() {
        if !solid.contains(&i) {
            output.push(f);
            continue;
        }
        let n = if f.vertices[2] == f.vertices[3] { 3 } else { 4 };
        let input = (0..n)
            .map(|i| Corner {
                p: doc.vertices[f.vertices[i] as usize],
                uv: f.uv[i],
            })
            .collect::<Vec<_>>();
        let mut polygon = Vec::new();
        let mut previous = input[n - 1];
        let mut pd = dot(previous.p, normal) - limit;
        for current in input {
            let cd = dot(current.p, normal) - limit;
            if (pd <= 0.) != (cd <= 0.) {
                let t = pd / (pd - cd);
                let cut = Corner {
                    p: std::array::from_fn(|c| previous.p[c] + (current.p[c] - previous.p[c]) * t),
                    uv: std::array::from_fn(|c| {
                        previous.uv[c] + (current.uv[c] - previous.uv[c]) * t
                    }),
                };
                polygon.push(cut);
                if !cap
                    .iter()
                    .any(|p| dot(sub(*p, cut.p), sub(*p, cut.p)) < 1e-10)
                {
                    cap.push(cut.p);
                }
            }
            if cd <= 0. {
                polygon.push(current);
            }
            previous = current;
            pd = cd;
        }
        polygon.dedup_by(|a, b| dot(sub(a.p, b.p), sub(a.p, b.p)) < 1e-10);
        if polygon.len() > 1
            && dot(
                sub(polygon[0].p, polygon.last().unwrap().p),
                sub(polygon[0].p, polygon.last().unwrap().p),
            ) < 1e-10
        {
            polygon.pop();
        }
        emit(doc, &mut output, &polygon, &f);
    }
    if cap.len() < 3 {
        return Err("Bevel did not produce a valid slope; reduce its distance".into());
    }
    let center: [f32; 3] =
        std::array::from_fn(|c| cap.iter().map(|p| p[c]).sum::<f32>() / cap.len() as f32);
    let u = unit(sub(
        doc.vertices[selected[1] as usize],
        doc.vertices[selected[0] as usize],
    ));
    let v = crate::mesh::cross(normal, u);
    cap.sort_by(|a, b| {
        let a = sub(*a, center);
        let b = sub(*b, center);
        dot(a, v)
            .atan2(dot(a, u))
            .total_cmp(&dot(b, v).atan2(dot(b, u)))
    });
    let cap = cap
        .into_iter()
        .map(|p| Corner {
            p,
            uv: [dot(sub(p, center), u), dot(sub(p, center), v)],
        })
        .collect::<Vec<_>>();
    let mut material = source;
    material.id = Uuid::new_v4();
    emit(doc, &mut output, &cap, &material);
    doc.faces = output;
    Ok(())
}
fn emit(doc: &mut Document, output: &mut Vec<Face>, polygon: &[Corner], source: &Face) {
    if polygon.len() < 3 {
        return;
    }
    let pieces = if polygon.len() <= 4 {
        vec![(0..polygon.len()).collect::<Vec<_>>()]
    } else {
        (1..polygon.len() - 1).map(|i| vec![0, i, i + 1]).collect()
    };
    for (i, piece) in pieces.into_iter().enumerate() {
        let mut corners = piece.iter().map(|i| polygon[*i]).collect::<Vec<_>>();
        if corners.len() == 3 {
            corners.push(corners[2]);
        }
        let indices = std::array::from_fn(|c| {
            let p = corners[c].p;
            doc.vertices
                .iter()
                .position(|v| dot(sub(*v, p), sub(*v, p)) < 1e-10)
                .unwrap_or_else(|| {
                    doc.vertices.push(p);
                    doc.vertices.len() - 1
                }) as u32
        });
        output.push(Face {
            id: if i == 0 { source.id } else { Uuid::new_v4() },
            vertices: indices,
            group: source.group,
            material: source.material,
            uv: std::array::from_fn(|c| corners[c].uv),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn cube() -> Document {
        let mut d = Document::default();
        d.primitive(
            "Box",
            [0.; 3],
            [1.; 3],
            1,
            d.groups[0].id,
            d.materials[0].id,
        );
        d
    }
    fn front(d: &Document) -> Edge {
        let index = |p| d.vertices.iter().position(|v| *v == p).unwrap() as u32;
        edge(index([-0.5, 0.5, -0.5]), index([0.5, 0.5, -0.5]))
    }
    fn closed(d: &Document) {
        let mut incidence = BTreeMap::new();
        for f in &d.faces {
            for edge in edges(f) {
                *incidence.entry(edge).or_insert(0) += 1;
            }
        }
        assert!(
            incidence.values().all(|n| *n == 2),
            "Bevel must leave a watertight solid: {incidence:?}"
        );
    }
    #[test]
    fn corner_bevel_preserves_materials_and_builds_a_closed_ramp() {
        for distance in [0.25, std::f32::consts::FRAC_1_SQRT_2] {
            let mut doc = cube();
            let selected = BTreeSet::from([front(&doc)]);
            let group = doc.groups[0].id;
            let material = doc.materials[0].id;
            bevel(&mut doc, &selected, distance).unwrap();
            closed(&doc);
            doc.validate().unwrap();
            assert!(
                doc.faces
                    .iter()
                    .all(|f| f.group == group && f.material == material)
            );
            assert!(doc.faces.iter().any(|f| {
                let normal = face_normal(doc.points(f));
                normal[1] > 0.6 && normal[2] < -0.6
            }));
            if distance > 0.7 {
                assert_eq!(doc.faces.len(), 5);
                assert_eq!(doc.vertices.len(), 6);
            }
        }
    }
    #[test]
    fn rejects_open_shell_triangular_edges_and_overlarge_cut_without_changes() {
        let original = cube();
        let selected = BTreeSet::from([front(&original)]);
        let mut doc = original.clone();
        assert!(bevel(&mut doc, &selected, 4.).is_err());
        assert_eq!(doc, original);
        let mut doc = original.clone();
        doc.faces.pop();
        let before = doc.clone();
        assert!(bevel(&mut doc, &selected, 0.25).is_err());
        assert_eq!(doc, before);
        let mut doc = original.clone();
        let face = doc
            .faces
            .iter()
            .position(|f| edges(f).contains(&front(&original)))
            .unwrap();
        doc.faces[face].vertices[3] = doc.faces[face].vertices[2];
        let before = doc.clone();
        assert!(bevel(&mut doc, &selected, 0.25).is_err());
        assert_eq!(doc, before);
    }
}

/// Displacement tolerance at which a bent quad is split. The validator rejects
/// a face whose fourth corner leaves the plane by more than 0.001, so split a
/// little earlier and never hand it an invalid document.
const PLANAR: f32 = 0.0005;

/// Apply a height brush to the vertices under it, then split every quad the
/// displacement bent. Blockout faces must stay planar, so a sculpted quad
/// becomes two triangles rather than an invalid face.
///
/// This is the same brush the terrain tool uses. Sculpting a subdivided Plane
/// here and sculpting a height grid there feel identical because the falloff,
/// strength and mode arithmetic are literally the same code.
pub fn sculpt(
    doc: &mut Document,
    brush: &crate::brush::Brush,
    center: [f32; 3],
    selection: &BTreeSet<u32>,
) -> Result<(), String> {
    use crate::brush::Mode;
    brush.validate()?;
    if doc.vertices.is_empty() {
        return Err("The mesh has no vertices to sculpt".into());
    }
    let under: Vec<(usize, f32)> = doc
        .vertices
        .iter()
        .enumerate()
        .filter(|(index, _)| selection.is_empty() || selection.contains(&(*index as u32)))
        .filter_map(|(index, v)| {
            let weight = brush.weight(v[0] - center[0], v[2] - center[2]);
            (weight > 0.).then_some((index, weight))
        })
        .collect();
    if under.is_empty() {
        return Err("No vertices under the brush. Widen the radius or move the brush.".into());
    }
    // Smooth pulls towards the mean of what the brush covers, which needs no
    // adjacency and behaves the same whether the surface is a grid or not.
    let mean = under
        .iter()
        .map(|(index, _)| doc.vertices[*index][1])
        .sum::<f32>()
        / under.len() as f32;
    let mut moved = false;
    for (index, weight) in under {
        let current = doc.vertices[index][1];
        let next = match brush.mode {
            Mode::Smooth => current + (mean - current) * weight * brush.strength.min(1.),
            Mode::Noise => {
                let p = doc.vertices[index];
                current
                    + crate::brush::noise(
                        (p[0] * 16.).round() as i32,
                        (p[2] * 16.).round() as i32,
                        brush.seed,
                    ) * brush.strength
                        * weight
            }
            Mode::Paint => current,
            _ => current + brush.delta(weight, current),
        };
        let next = next.clamp(-128., 128.);
        if next != current && next.is_finite() {
            doc.vertices[index][1] = next;
            moved = true;
        }
    }
    if !moved {
        return Ok(());
    }
    triangulate_bent(doc);
    Ok(())
}

/// Split quads whose corners no longer share a plane. The first half keeps the
/// original face id so a selection survives the split.
fn triangulate_bent(doc: &mut Document) {
    let mut splits = vec![];
    for (index, f) in doc.faces.iter().enumerate() {
        if f.vertices[2] == f.vertices[3] {
            continue;
        }
        let p = doc.points(f);
        let normal = face_normal(p);
        if !normal.iter().all(|v| v.is_finite()) {
            continue;
        }
        if dot(sub(p[3], p[0]), normal).abs() <= PLANAR {
            continue;
        }
        let first = Face {
            id: f.id,
            vertices: [f.vertices[0], f.vertices[1], f.vertices[2], f.vertices[2]],
            group: f.group,
            material: f.material,
            uv: [f.uv[0], f.uv[1], f.uv[2], f.uv[2]],
        };
        let second = Face {
            id: Uuid::new_v4(),
            vertices: [f.vertices[0], f.vertices[2], f.vertices[3], f.vertices[3]],
            group: f.group,
            material: f.material,
            uv: [f.uv[0], f.uv[2], f.uv[3], f.uv[3]],
        };
        splits.push((index, first, second));
    }
    // Insert from the back so earlier indices stay valid.
    for (index, first, second) in splits.into_iter().rev() {
        doc.faces[index] = first;
        doc.faces.insert(index + 1, second);
    }
}

#[cfg(test)]
mod sculpt_tests {
    use super::*;
    use crate::brush::{Brush, Mode};

    fn plane(steps: usize) -> Document {
        let mut doc = Document::default();
        let group = doc.groups[0].id;
        let slot = doc.materials[0].id;
        doc.primitive("Plane", [0.; 3], [8., 1., 8.], steps, group, slot);
        // A single Plane face is one quad; subdivide it so the brush has
        // vertices to move, the way a terrain grid would.
        for _ in 0..steps.min(3) {
            let all = doc.faces.iter().map(|f| f.id).collect::<BTreeSet<_>>();
            doc.split(&all);
        }
        doc
    }

    #[test]
    fn sculpting_bends_a_plane_and_keeps_the_document_valid() {
        let mut doc = plane(2);
        doc.validate().unwrap();
        let before = doc.faces.len();
        let brush = Brush {
            mode: Mode::Raise,
            radius: 3.,
            strength: 1.5,
            ..Default::default()
        };
        sculpt(&mut doc, &brush, [0., 0., 0.], &BTreeSet::new()).unwrap();
        // The centre rose.
        assert!(doc.vertices.iter().any(|v| v[1] > 0.5));
        // Bent quads became triangle pairs rather than invalid faces.
        assert!(doc.faces.len() > before);
        assert!(doc.faces.iter().any(|f| f.vertices[2] == f.vertices[3]));
        doc.validate()
            .unwrap_or_else(|e| panic!("sculpt produced an invalid document: {e}"));
    }

    #[test]
    fn sculpting_only_touches_the_selection() {
        let mut doc = plane(2);
        let selection: BTreeSet<u32> = doc
            .vertices
            .iter()
            .enumerate()
            .filter(|(_, v)| v[0] < 0.)
            .map(|(i, _)| i as u32)
            .collect();
        assert!(!selection.is_empty());
        let brush = Brush {
            mode: Mode::Raise,
            radius: 64.,
            strength: 2.,
            falloff: crate::brush::Falloff::Constant,
            ..Default::default()
        };
        sculpt(&mut doc, &brush, [0., 0., 0.], &selection).unwrap();
        for (index, v) in doc.vertices.iter().enumerate() {
            if selection.contains(&(index as u32)) {
                assert!(v[1] > 0.5, "selected vertex {index} did not move");
            } else {
                assert_eq!(v[1], 0., "unselected vertex {index} moved");
            }
        }
    }

    #[test]
    fn a_brush_that_covers_nothing_reports_instead_of_silently_passing() {
        let mut doc = plane(1);
        let brush = Brush {
            radius: 0.5,
            ..Default::default()
        };
        let error = sculpt(&mut doc, &brush, [500., 0., 500.], &BTreeSet::new()).unwrap_err();
        assert!(error.contains("No vertices under the brush"), "{error}");
    }

    #[test]
    fn an_already_flat_smooth_leaves_the_topology_alone() {
        let mut doc = plane(2);
        let faces = doc.faces.len();
        let brush = Brush {
            mode: Mode::Smooth,
            radius: 32.,
            strength: 1.,
            ..Default::default()
        };
        sculpt(&mut doc, &brush, [0., 0., 0.], &BTreeSet::new()).unwrap();
        // Everything was already at the mean, so nothing moved and nothing
        // had to be triangulated.
        assert_eq!(doc.faces.len(), faces);
        doc.validate().unwrap();
    }
}
