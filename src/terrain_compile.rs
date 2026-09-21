//! Bakes the authored height grid into the editable-mesh chunk format.
//!
//! One quad per cell, alabeado allowed: the runtime splits every quad into two
//! GT3 triangles anyway, so a bent cell costs the same as a flat one and half
//! of what two triangle faces would. Planarity is only an invariant of the
//! blockout document, which terrain never builds.
use crate::{
    lighting::Quad,
    mesh_compile::{Chunk, PageLookup},
    scene::Actor,
    terrain::{Component, Document},
};

/// Half a texel of a 256-pixel atlas, in normalized atlas space. Without it
/// adjacent tiles bleed into each other: the PSX sampler has no clamp.
const INSET: f32 = 1. / 512.;
/// Largest height deviation, in world units, still treated as coplanar when
/// merging cells. One Q8 step is 1/256, so this keeps merges imperceptible.
const FLATNESS: f32 = 1. / 64.;
/// Quads per draw chunk, matching the blockout compiler.
const CHUNK_QUADS: usize = 96;

/// Corner UVs for an atlas tile, wound like `Document::corners`.
pub fn tile_uv(atlas: [u8; 2], tile: u8, rotation: u8) -> [[f32; 2]; 4] {
    let cols = f32::from(atlas[0].max(1));
    let rows = f32::from(atlas[1].max(1));
    let index = u32::from(tile);
    let col = (index % atlas[0].max(1) as u32) as f32;
    let row = ((index / atlas[0].max(1) as u32) % atlas[1].max(1) as u32) as f32;
    let inset = if atlas[0] > 1 || atlas[1] > 1 {
        INSET
    } else {
        0.
    };
    let u0 = col / cols + inset;
    let u1 = (col + 1.) / cols - inset;
    let v0 = row / rows + inset;
    let v1 = (row + 1.) / rows - inset;
    let base = [[u0, v0], [u0, v1], [u1, v1], [u1, v0]];
    let mut out = [[0.; 2]; 4];
    for (k, slot) in out.iter_mut().enumerate() {
        *slot = base[(k + usize::from(rotation & 3)) % 4];
    }
    out
}

/// A baked cell, possibly covering `size` x `size` source cells after merging.
struct Patch {
    i: u16,
    j: u16,
    size: u16,
}

/// Merge coplanar neighbours that share a material. This is the only LOD the
/// format allows: geometry is cooked into fixed arrays, so nothing can be
/// decimated at runtime. Merging stretches the tile across the merged cells,
/// which is why it is opt-in per terrain.
fn patches(doc: &Document, merge: u8) -> Vec<Patch> {
    let cells = doc.cells();
    let mut taken = vec![false; doc.quad_count()];
    let mut out = Vec::new();
    let block = doc.block().max(1);
    let levels = merge.min(3);
    // Largest squares first, so a flat plain collapses instead of being
    // consumed piecemeal by its own smallest blocks.
    for level in (0..=levels).rev() {
        let size = 1_u16 << level;
        if size == 1 {
            break;
        }
        let mut j = 0;
        while j + size <= cells[1] {
            let mut i = 0;
            while i + size <= cells[0] {
                if mergeable(doc, &taken, i, j, size, block) {
                    for dj in 0..size {
                        for di in 0..size {
                            taken[usize::from(j + dj) * usize::from(cells[0])
                                + usize::from(i + di)] = true;
                        }
                    }
                    out.push(Patch { i, j, size });
                }
                i += size;
            }
            j += size;
        }
    }
    for j in 0..cells[1] {
        for i in 0..cells[0] {
            if !taken[usize::from(j) * usize::from(cells[0]) + usize::from(i)] {
                out.push(Patch { i, j, size: 1 });
            }
        }
    }
    // Row-major order keeps chunk binning and baked colours deterministic.
    out.sort_by_key(|p| (p.j, p.i));
    out
}

fn mergeable(doc: &Document, taken: &[bool], i: u16, j: u16, size: u16, block: u16) -> bool {
    let cells = doc.cells();
    // A merged quad must not straddle two draw chunks, or its chunk would
    // exceed the 16-unit span that i16 Q12 chunk-local positions encode.
    if i / block != (i + size - 1) / block || j / block != (j + size - 1) / block {
        return false;
    }
    let material = doc.tile(i, j) | (doc.rotation(i, j) << 6);
    for dj in 0..size {
        for di in 0..size {
            let index = usize::from(j + dj) * usize::from(cells[0]) + usize::from(i + di);
            if taken[index] {
                return false;
            }
            if doc.tile(i + di, j + dj) | (doc.rotation(i + di, j + dj) << 6) != material {
                return false;
            }
        }
    }
    // Every interior corner must sit on the bilinear surface of the four outer
    // corners, otherwise merging would visibly flatten a bump.
    let (a, b) = (i32::from(i), i32::from(j));
    let s = i32::from(size);
    let h00 = doc.height(a, b);
    let h10 = doc.height(a + s, b);
    let h01 = doc.height(a, b + s);
    let h11 = doc.height(a + s, b + s);
    for dj in 0..=s {
        for di in 0..=s {
            let u = di as f32 / s as f32;
            let v = dj as f32 / s as f32;
            let expected =
                h00 * (1. - u) * (1. - v) + h10 * u * (1. - v) + h01 * (1. - u) * v + h11 * u * v;
            if (doc.height(a + di, b + dj) - expected).abs() > FLATNESS {
                return false;
            }
        }
    }
    true
}

/// Draw geometry for a terrain actor, in the order the lighting bake and the
/// `color_offset` of every cooked quad depend on.
pub fn quads(terrain: &Component) -> Vec<Quad> {
    let Some(doc) = &terrain.document else {
        return vec![];
    };
    let mut out = Vec::with_capacity(doc.quad_count());
    for patch in patches(doc, terrain.merge) {
        let (i, j, size) = (patch.i, patch.j, patch.size);
        let (fi, fj, fs) = (f32::from(i), f32::from(j), f32::from(size));
        let (x0, x1) = (doc.local_x(fi), doc.local_x(fi + fs));
        let (z0, z1) = (doc.local_z(fj), doc.local_z(fj + fs));
        let (a, b, s) = (i32::from(i), i32::from(j), i32::from(size));
        let points = [
            [x0, doc.height(a, b), z0],
            [x0, doc.height(a, b + s), z1],
            [x1, doc.height(a + s, b + s), z1],
            [x1, doc.height(a + s, b), z0],
        ];
        // Average the four corner normals rather than taking the geometric
        // face normal: adjacent cells then vary continuously and the baked
        // corner colours read as a slope instead of a staircase of facets.
        // True per-corner shading would need a normal per corner, which
        // MeshQuad does not carry.
        let mut normal = [0.; 3];
        for (di, dj) in [(0, 0), (0, s), (s, s), (s, 0)] {
            let n = doc.corner_normal(a + di, b + dj);
            for c in 0..3 {
                normal[c] += n[c] / 4.;
            }
        }
        out.push(Quad {
            uv: tile_uv(terrain.atlas, doc.tile(i, j), doc.rotation(i, j)),
            face: 0,
            points,
            normal: crate::lighting::unit(normal),
            material: terrain.material.clone(),
            id: None,
        });
    }
    out
}

/// Group quads into draw chunks by grid block. Binning on cell indices rather
/// than on world centroids keeps every chunk exactly within one block, so the
/// 16-unit chunk span can never be exceeded by a rounding accident.
pub fn chunks(terrain: &Component, qs: &[Quad]) -> Result<Vec<Chunk>, String> {
    let Some(doc) = &terrain.document else {
        return Ok(vec![]);
    };
    let block = f32::from(doc.block().max(1)) * doc.cell_size();
    let span = doc.span();
    let mut cells: std::collections::BTreeMap<[i32; 2], Vec<usize>> = Default::default();
    for (index, q) in qs.iter().enumerate() {
        let centre: [f32; 3] =
            std::array::from_fn(|c| q.points.iter().map(|p| p[c]).sum::<f32>() / 4.);
        let key = [
            ((centre[0] + span[0] / 2.) / block).floor() as i32,
            ((centre[2] + span[1] / 2.) / block).floor() as i32,
        ];
        cells.entry(key).or_default().push(index);
    }
    let mut out = vec![];
    for (_, part) in cells {
        for run in part.chunks(CHUNK_QUADS) {
            crate::mesh_compile::encode_chunks(qs, run, &mut out)?;
        }
    }
    Ok(out)
}

pub fn header_with_pages(e: &Actor, index: usize, pages: PageLookup) -> Result<String, String> {
    let terrain = e.terrain.as_ref().unwrap();
    if terrain.document.is_none() {
        return Err(terrain
            .error
            .clone()
            .unwrap_or_else(|| "Terrain not resolved".into()));
    }
    let qs = crate::lighting::quads(e);
    if qs.len() > crate::terrain::MAX_QUADS {
        return Err(format!(
            "Compiled terrain exceeds {} triangles. Reduce the cell count, raise Merge Flat Cells, or enable geometry streaming.",
            crate::terrain::MAX_QUADS * 2
        ));
    }
    let chunks = chunks(terrain, &qs)?;
    Ok(crate::mesh_compile::header_for(index, &qs, &chunks, pages))
}

/// Heights for the runtime heightfield collider, in Q8 relative to the world
/// bottom of the terrain's bounding box, plus the box itself. Returns nothing
/// when the actor carries no collidable terrain.
pub struct Heightfield {
    pub cells: [u16; 2],
    /// World units per cell.
    pub cell_size: f32,
    /// Local-space centre and half extents of the collider box.
    pub center: [f32; 3],
    pub half_extents: [f32; 3],
    /// Corner heights above the box bottom, in Q8 local units.
    pub heights: Vec<i16>,
}
pub fn heightfield(e: &Actor) -> Option<Heightfield> {
    let terrain = e.terrain.as_ref()?;
    if !terrain.collision {
        return None;
    }
    let doc = terrain.document.as_ref()?;
    let cells = doc.cells();
    let span = doc.span();
    let (low, high) = (doc.lowest(), doc.highest());
    // A zero-thickness box cannot be swept against, so keep a floor slab under
    // the lowest corner. It is also what a character standing on a flat
    // terrain rests on.
    let bottom = low - 1.;
    let height = (high - bottom).max(2. / 4096.);
    let mut heights = Vec::with_capacity(doc.corner_count());
    for j in 0..=cells[1] {
        for i in 0..=cells[0] {
            let value =
                (doc.height(i32::from(i), i32::from(j)) - bottom) / crate::terrain::HEIGHT_UNIT;
            heights.push(value.round().clamp(0., 32767.) as i16);
        }
    }
    Some(Heightfield {
        cells,
        cell_size: doc.cell_size(),
        center: [0., bottom + height / 2., 0.],
        half_extents: [span[0] / 2., height / 2., span[1] / 2.],
        heights,
    })
}

/// Append inside initialize_components(), after `collision::cpp_setup`, so a
/// terrain collider can replace the box fields it leaves untouched.
pub fn collider_cpp(scene: &crate::scene::Scene) -> String {
    use std::fmt::Write;
    let mut output = String::new();
    for (index, e) in scene.actors.iter().enumerate() {
        let Some(field) = heightfield(e) else {
            continue;
        };
        let fixed = |v: f32| (v * 4096.).round() as i32;
        writeln!(
            output,
            "static constexpr int16_t terrain_heights_{index}[]={{{}}};",
            field
                .heights
                .iter()
                .map(i16::to_string)
                .collect::<Vec<_>>()
                .join(",")
        )
        .unwrap();
        writeln!(
            output,
            "objects[{index}].collider.enabled=true;objects[{index}].collider.trigger=false;objects[{index}].collider.layer=1u;objects[{index}].collider.mask=0xffffffffu;"
        )
        .unwrap();
        for axis in 0..3 {
            writeln!(
                output,
                "objects[{index}].collider.center[{axis}]=Fixed({},Fixed::RAW);objects[{index}].collider.half_extents[{axis}]=Fixed({},Fixed::RAW);",
                fixed(field.center[axis]),
                fixed(field.half_extents[axis])
            )
            .unwrap();
        }
        writeln!(
            output,
            "objects[{index}].collider.heights=terrain_heights_{index};objects[{index}].collider.height_cells[0]={}u;objects[{index}].collider.height_cells[1]={}u;objects[{index}].collider.height_step=Fixed({},Fixed::RAW);",
            field.cells[0],
            field.cells[1],
            fixed(field.cell_size)
        )
        .unwrap();
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{scene::Actor, terrain::Document};
    use std::sync::Arc;

    fn actor(doc: Document, merge: u8) -> Actor {
        let mut a = Actor::cube("Terrain".into());
        a.position = [0.; 3];
        let mut component = Component::new(uuid::Uuid::new_v4());
        component.document = Some(Arc::new(doc));
        component.merge = merge;
        a.terrain = Some(component);
        a
    }
    fn bumpy(cells: [u16; 2], size: f32) -> Document {
        let mut doc = Document::new(cells, size);
        for j in 0..=cells[1] {
            for i in 0..=cells[0] {
                doc.set_height(i, j, f32::from(i % 3) * 0.75 + f32::from(j % 2) * 0.5);
            }
        }
        doc
    }

    #[test]
    fn one_quad_per_cell_without_merging() {
        let a = actor(bumpy([8, 6], 2.), 0);
        let qs = crate::lighting::quads(&a);
        assert_eq!(qs.len(), 48);
        // Alabeado quads are the point: two triangle faces per cell would cost
        // twice the MeshQuad bytes and twice the retained packet slots.
        assert!(qs.iter().all(|q| q.points[2] != q.points[3]));
    }

    #[test]
    fn quads_are_wound_upwards_and_normals_are_smoothed() {
        let a = actor(bumpy([4, 4], 2.), 0);
        for q in crate::lighting::quads(&a) {
            assert!(
                crate::mesh::face_normal(q.points)[1] > 0.3,
                "downward winding: {:?}",
                q.points
            );
            assert!(q.normal[1] > 0.3, "downward normal: {:?}", q.normal);
            let length = crate::lighting::dot(q.normal, q.normal);
            assert!((length - 1.).abs() < 1e-3, "unnormalized normal: {length}");
        }
    }

    #[test]
    fn merging_collapses_flat_ground_and_spares_bumps() {
        let mut doc = Document::new([8, 8], 2.);
        let flat = actor(doc.clone(), 2);
        // A wholly flat grid collapses to the largest blocks that fit one
        // chunk; the chunk block for 2-unit cells is 7 cells, so 4x4 merges
        // survive and 8x8 would straddle.
        let merged = crate::lighting::quads(&flat).len();
        assert!(merged < 64, "flat terrain did not merge: {merged}");
        // One raised corner must keep its own cells.
        doc.set_height(3, 3, 5.);
        let bumped = actor(doc, 2);
        assert!(crate::lighting::quads(&bumped).len() > merged);
        // Merging off reproduces one quad per cell exactly.
        let plain = actor(Document::new([8, 8], 2.), 0);
        assert_eq!(crate::lighting::quads(&plain).len(), 64);
    }

    #[test]
    fn merging_respects_painted_tiles() {
        let mut doc = Document::new([8, 8], 2.);
        for j in 0..8 {
            for i in 0..8 {
                doc.set_tile(i, j, u8::from(i % 2 == 0), 0);
            }
        }
        // Alternating tiles are coplanar but must not merge: a merged quad
        // carries one tile, and stretching it would repaint the neighbour.
        assert_eq!(crate::lighting::quads(&actor(doc, 3)).len(), 64);
    }

    #[test]
    fn chunks_stay_within_the_encodable_span_and_quad_run() {
        for (cells, size, merge) in [([32, 32], 4., 0), ([48, 48], 2., 0), ([16, 16], 8., 2)] {
            let a = actor(bumpy(cells, size), merge);
            let terrain = a.terrain.clone().unwrap();
            let qs = crate::lighting::quads(&a);
            let chunks = chunks(&terrain, &qs).unwrap();
            assert!(!chunks.is_empty());
            for chunk in &chunks {
                assert!(chunk.faces.len() <= CHUNK_QUADS);
                for axis in 0..3 {
                    assert!(
                        chunk.extent[axis] * 2 <= 65504,
                        "chunk spans {} ticks on axis {axis}",
                        chunk.extent[axis] * 2
                    );
                }
                for v in &chunk.vertices {
                    assert!(v.iter().all(|c| i32::from(*c).abs() <= 32767));
                }
            }
            // Every quad lands in exactly one chunk.
            let placed: usize = chunks.iter().map(|c| c.faces.len()).sum();
            assert_eq!(placed, qs.len());
        }
    }

    #[test]
    fn atlas_uvs_stay_inside_their_tile() {
        let uv = tile_uv([4, 4], 5, 0);
        // Tile 5 is column 1, row 1 of a 4x4 atlas.
        for corner in uv {
            assert!(
                (0.25 - INSET..=0.5 + INSET).contains(&corner[0]),
                "{corner:?}"
            );
            assert!(
                (0.25 - INSET..=0.5 + INSET).contains(&corner[1]),
                "{corner:?}"
            );
        }
        // A single-tile atlas uses the whole texture, with no inset.
        assert_eq!(
            tile_uv([1, 1], 0, 0),
            [[0., 0.], [0., 1.], [1., 1.], [1., 0.]]
        );
        // Rotation permutes the corners without leaving the tile.
        let turned = tile_uv([4, 4], 5, 1);
        assert_ne!(turned, uv);
        let mut sorted_a = uv.to_vec();
        let mut sorted_b = turned.to_vec();
        let key = |v: &[f32; 2]| (v[0].to_bits(), v[1].to_bits());
        sorted_a.sort_by_key(key);
        sorted_b.sort_by_key(key);
        assert_eq!(sorted_a, sorted_b);
    }

    #[test]
    fn heightfield_covers_the_grid_and_stays_non_negative() {
        let mut doc = Document::new([8, 8], 2.);
        doc.set_height(4, 4, 6.);
        doc.set_height(1, 1, -3.);
        let a = actor(doc.clone(), 0);
        let field = heightfield(&a).unwrap();
        assert_eq!(field.cells, [8, 8]);
        assert_eq!(field.heights.len(), doc.corner_count());
        assert!(field.heights.iter().all(|h| *h >= 0));
        let bottom = field.center[1] - field.half_extents[1];
        // Q8 heights above the box bottom must reproduce the authored surface.
        let peak = bottom + f32::from(field.heights[4 * 9 + 4]) * crate::terrain::HEIGHT_UNIT;
        assert!((peak - 6.).abs() < 0.02, "{peak}");
        assert!((field.half_extents[0] - 8.).abs() < 1e-4);
        // Collision off produces no collider at all.
        let mut without = a.clone();
        without.terrain.as_mut().unwrap().collision = false;
        assert!(heightfield(&without).is_none());
    }

    #[test]
    fn collider_cpp_emits_function_scope_storage() {
        let mut scene = crate::scene::Scene::default();
        scene.actors.push(actor(Document::new([4, 4], 2.), 0));
        let cpp = collider_cpp(&scene);
        // The setup block is appended inside initialize_components(), where
        // `inline constexpr` would not compile.
        assert!(cpp.contains("static constexpr int16_t terrain_heights_"));
        assert!(!cpp.contains("inline constexpr int16_t terrain_heights_"));
        assert!(cpp.contains(".collider.heights=terrain_heights_"));
        assert!(cpp.contains(".collider.height_cells[0]=4u"));
        assert!(cpp.contains(".collider.height_step=Fixed(8192,Fixed::RAW)"));
    }
}

#[cfg(test)]
mod preview_tests {
    use super::*;
    use crate::{scene::Actor, terrain::Document};
    use std::sync::Arc;

    /// Terrain has to reach the editor preview through the same predicates the
    /// authored surfaces use. Missing one of them does not fail a build: it
    /// silently shades every cell with a cube face's colour, or draws the
    /// underside of the ground.
    #[test]
    fn preview_predicates_treat_terrain_as_an_authored_surface() {
        let mut a = Actor::cube("Terrain".into());
        let mut component = Component::new(uuid::Uuid::new_v4());
        component.document = Some(Arc::new(Document::new([4, 4], 2.)));
        a.terrain = Some(component);
        // The seam every preview path reads.
        assert_eq!(crate::lighting::quads(&a).len(), 16);
        assert_eq!(crate::lighting::quad_count(&a), 16);
        // Terrain quads carry their own normal; only the built-in cube may be
        // shaded from the six precomputed face colours.
        assert!(crate::lighting::quads(&a).iter().all(|q| q.face == 0));
        assert!(a.editable_mesh.is_none() && a.skeletal_mesh.is_none());
        let authored =
            a.editable_mesh.is_some() || a.terrain.is_some() || a.skeletal_mesh.is_some();
        assert!(authored, "terrain must count as an authored surface");
        // Baked lighting applies to terrain even though it has no material of
        // its own on the actor.
        a.lighting.receive = crate::lighting::Receive::Baked;
        assert!(crate::lighting::baked(&a));
        // The flat-slab lighting heuristic must not also claim it.
        assert!(!crate::lighting::tiled(&a));
    }
}
