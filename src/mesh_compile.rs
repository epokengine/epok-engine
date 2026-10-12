//! Deterministic spatial partitioning for the authored mesh. Author groups never become draw units.
use crate::{
    lighting::Quad,
    mesh::{Component, Document},
    scene::Actor,
};
use std::collections::BTreeMap;

pub fn quads(e: &Actor, mesh: &Component) -> Vec<Quad> {
    let Some(doc) = &mesh.document else {
        return vec![];
    };
    let mut out = vec![];
    for f in &doc.faces {
        let p = doc.points(f);
        let normal = crate::mesh::face_normal(p);
        let mat = crate::mesh::material(e, f.material);
        // Storage range must not change authored topology or UV interpolation.
        // Wide chunks preserve large faces at the same Q12 precision. Explicit
        // modelling subdivisions already live in the source document.
        out.push(Quad {
            uv: f.uv,
            vertex_colors: f.vertex_colors,
            face: 0,
            points: p.map(|v| v.map(|c| (c * 4096.).round() / 4096.)),
            normal,
            material: mat,
            id: Some(f.id),
        });
    }
    out
}
pub struct Chunk {
    pub origin: [f32; 3],
    pub vertices: Vec<[i32; 3]>,
    pub faces: Vec<([u16; 4], usize)>,
    pub center: [i32; 3],
    pub extent: [i32; 3],
}
impl Chunk {
    /// On-disc and resident vertex width; not the host cooker's working type.
    pub fn vertex_stride(&self) -> usize {
        if self.vertices.iter().flatten().all(|v| i16::try_from(*v).is_ok()) { 6 } else { 12 }
    }
}
#[path = "mesh_visibility.rs"]
mod visibility;
pub fn chunks(quads: &[Quad]) -> Result<Vec<Chunk>, String> {
    // Culling cells need not force 4 m draw batches. Eight-metre cells retain
    // useful arena-scale culling while sharing vertices across adjacent panels.
    // Payloads are bounded independently below, including skewed convex faces.
    const CELL: f32 = 8.;
    let mut cells: BTreeMap<[i32; 3], Vec<usize>> = BTreeMap::new();
    for (i, q) in quads.iter().enumerate() {
        let key = std::array::from_fn(|c| {
            (q.points.iter().map(|p| p[c]).sum::<f32>() / (4. * CELL)).floor() as i32
        });
        cells.entry(key).or_default().push(i);
    }
    let mut chunks = vec![];
    for (_, faces) in cells {
        for part in faces.chunks(96) {
            encode_chunks(quads, part, &mut chunks)?;
        }
    }
    Ok(chunks)
}

pub(crate) fn encode_chunks(
    quads: &[Quad],
    part: &[usize],
    chunks: &mut Vec<Chunk>,
) -> Result<(), String> {
    let mut world_low = [i32::MAX; 3];
    let mut world_high = [i32::MIN; 3];
    for &i in part {
        for p in quads[i].points {
            for c in 0..3 {
                let v = (p[c] * 4096.).round() as i32;
                world_low[c] = world_low[c].min(v);
                world_high[c] = world_high[c].max(v);
            }
        }
    }
    // Retain compact partitions when all constituent faces fit narrow Q12.
    // If any face itself needs wide coordinates, widening its chunk preserves
    // topology; subdividing the polygon would add work and alter affine UVs.
    let needs_wide_face = part.iter().any(|&i| (0..3).any(|c| {
        let (mut low, mut high) = (i32::MAX, i32::MIN);
        for p in quads[i].points {
            let q = (p[c] * 4096.).round() as i32;
            low = low.min(q); high = high.max(q);
        }
        i64::from(high) - i64::from(low) > 65504
    }));
    if part.len() > 1 && !needs_wide_face
        && (0..3).any(|c| i64::from(world_high[c]) - i64::from(world_low[c]) > 65504) {
        let (a, b) = part.split_at(part.len() / 2);
        encode_chunks(quads, a, chunks)?;
        encode_chunks(quads, b, chunks)?;
        return Ok(());
    }
    // Snap origins to Q8 so origin>>4 + local>>4 is bit-identical to
    // global>>4. A changed partition must never introduce GTE seams.
    let origin_raw: [i32; 3] = std::array::from_fn(|c| {
        (((i64::from(world_low[c]) + i64::from(world_high[c])) / 2).div_euclid(16) * 16) as i32
    });
    let origin = origin_raw.map(|v| v as f32 / 4096.);
    let mut vertices = vec![];
    let mut lookup = BTreeMap::new();
    let mut encoded = vec![];
    let mut low = [i32::MAX; 3];
    let mut high = [i32::MIN; 3];
    for &i in part {
        let mut indices = [0_u16; 4];
        for (j, p) in quads[i].points.iter().enumerate() {
            let mut local = [0_i32; 3];
            for c in 0..3 {
                let v = ((p[c] * 4096.).round() as i32).checked_sub(origin_raw[c])
                    .ok_or("Chunk coordinate overflow")?;
                local[c] = v;
                low[c] = low[c].min(v);
                high[c] = high[c].max(v);
            }
            indices[j] = *lookup.entry(local).or_insert_with(|| {
                let id = vertices.len() as u16;
                vertices.push(local);
                id
            });
        }
        encoded.push((indices, i));
    }
    chunks.push(Chunk {
        origin,
        vertices,
        faces: encoded,
        center: std::array::from_fn(|c| ((i64::from(high[c]) + i64::from(low[c])) / 2) as i32),
        extent: std::array::from_fn(|c| ((i64::from(high[c]) - i64::from(low[c]) + 1) / 2) as i32),
    });
    Ok(())
}
/// A local mesh point in world space, rounded once to Q12. The ordered draw
/// and static-world cooks share it, so coincident corners of different
/// actors land on the same Q12 point.
pub fn world_q12(transform: &crate::transform::Matrix, point: [f32; 3]) -> Result<[i32; 3], String> {
    let mut out = [0; 3];
    for axis in 0..3 {
        let world = f64::from(transform.0[axis][3])
            + (0..3).map(|c| f64::from(transform.0[axis][c]) * f64::from(point[c])).sum::<f64>();
        let fixed = (world * 4096.).round();
        if !fixed.is_finite() || fixed < f64::from(i32::MIN) || fixed > f64::from(i32::MAX) {
            return Err("World coordinate exceeds Q12 range".into());
        }
        out[axis] = fixed as i32;
    }
    Ok(out)
}
/// A cooked chunk's vertices moved to world space: same vertex order, faces
/// and indices, so visibility bits and topology are unchanged. Each vertex is
/// computed from the authored point (not the local Q12 rounding). The origin
/// is snapped to Q8 like a local chunk's.
pub struct WorldChunk {
    pub vertices: Vec<[i32; 3]>,
    pub origin: [i32; 3],
    pub center: [i32; 3],
    pub extent: [i32; 3],
}
pub fn world_chunk(doc: &Document, transform: &crate::transform::Matrix, chunk: &Chunk) -> Result<WorldChunk, String> {
    let mut world = vec![None; chunk.vertices.len()];
    for (indices, face) in &chunk.faces {
        let raw = doc.points(doc.faces.get(*face).ok_or("Static world face out of range")?);
        for (corner, &vertex) in indices.iter().enumerate() {
            let slot = &mut world[usize::from(vertex)];
            if slot.is_none() {
                *slot = Some(world_q12(transform, raw[corner])?);
            }
        }
    }
    let world: Vec<[i32; 3]> = world.into_iter().collect::<Option<_>>().ok_or("Static world vertex without a face")?;
    let mut low = [i32::MAX; 3];
    let mut high = [i32::MIN; 3];
    for v in &world {
        for c in 0..3 { low[c] = low[c].min(v[c]); high[c] = high[c].max(v[c]); }
    }
    let origin: [i32; 3] = std::array::from_fn(|c| {
        ((i64::from(low[c]) + i64::from(high[c])) / 2).div_euclid(16) as i32 * 16
    });
    let vertices: Vec<[i32; 3]> = world.iter().map(|v| std::array::from_fn(|c| v[c] - origin[c])).collect();
    let (mut low, mut high) = ([i32::MAX; 3], [i32::MIN; 3]);
    for v in &vertices {
        for c in 0..3 { low[c] = low[c].min(v[c]); high[c] = high[c].max(v[c]); }
    }
    Ok(WorldChunk {
        vertices,
        origin,
        center: std::array::from_fn(|c| ((i64::from(high[c]) + i64::from(low[c])) / 2) as i32),
        extent: std::array::from_fn(|c| ((i64::from(high[c]) - i64::from(low[c]) + 1) / 2) as i32),
    })
}
/// Texture page geometry (width, height, VRAM y) used to bake 8-bit page UVs.
pub type PageLookup<'a> = &'a dyn Fn(uuid::Uuid) -> Option<crate::texture_cook::Page>;
#[cfg(test)]
pub fn header(e: &Actor, index: usize) -> Result<String, String> {
    header_with_pages(e, index, &|_| None)
}
/// Packed page UVs replicate the runtime's texture_uv mapping so the renderer
/// copies them instead of scaling four Q12 pairs per drawn quad.
pub fn packed_uv(uv: [f32; 2], page: crate::texture_cook::Page) -> u16 {
    let crate::texture_cook::Page { width, height, u: base_u, v: base_v } = page;
    let q = |v: f32| (v.clamp(0., 1.) * 4096.).round() as i32;
    let u = (i32::from(base_u) + q(uv[0]) * (i32::from(width) - 1) / 4096) & 255;
    let v = (i32::from(base_v) + q(uv[1]) * (i32::from(height) - 1) / 4096) & 255;
    (u | (v << 8)) as u16
}
pub fn header_with_pages(e: &Actor, index: usize, pages: PageLookup) -> Result<String, String> {
    let palette = crate::vertex_colors::Palette::quads(&crate::lighting::quads(e))?;
    header_with_colors(e, index, pages, &palette)
}
pub fn header_with_colors(e: &Actor, index: usize, pages: PageLookup, palette: &crate::vertex_colors::Palette) -> Result<String, String> {
    let mesh = e.editable_mesh.as_ref().unwrap();
    let _doc: &Document = mesh.document.as_deref().ok_or_else(|| {
        mesh.error
            .clone()
            .unwrap_or("EditableMesh not resolved".into())
    })?;
    let qs = crate::lighting::quads(e);
    if qs.len() > 3500 {
        return Err(
            "Compiled mesh exceeds 3500 faces. Split the authored mesh into separate assets.".into(),
        );
    }
    let chunks = chunks(&qs)?;
    Ok(header_for_colors(index, &qs, &chunks, pages, palette))
}

/// Emit the cooked chunk arrays for one actor. Terrain partitions its quads
/// differently but stores them in exactly this layout, so it shares the
/// emitter rather than growing a second symbol namespace the runtime,
/// streaming archive and visibility bake would all have to learn.
pub fn header_for(index: usize, qs: &[Quad], chunks: &[Chunk], pages: PageLookup) -> String {
    header_for_colors(index, qs, chunks, pages, &crate::vertex_colors::Palette::quads(qs).expect("validated mesh palette"))
}
fn header_for_colors(index: usize, qs: &[Quad], chunks: &[Chunk], pages: PageLookup, palette: &crate::vertex_colors::Palette) -> String {
    let mut text = visibility::header(chunks, index);
    for (c, chunk) in chunks.iter().enumerate().rev() {
        let key = format!("editable_{index}_{c}");
        let vertex_type = if chunk.vertex_stride() == 6 { "int16_t" } else { "int32_t" };
        text.push_str(&format!(
            "inline constexpr {vertex_type} {key}_vertices[][3]={{{}}};\n",
            chunk
                .vertices
                .iter()
                .map(|v| format!("{{{},{},{}}}", v[0], v[1], v[2]))
                .collect::<Vec<_>>()
                .join(",")
        ));
        text.push_str(&format!(
            "inline constexpr MeshQuad {key}_faces[]={{{}}};\n",
            chunk
                .faces
                .iter()
                .map(|(v, i)| {
                    let q = &qs[*i];
                    let n = q.normal.map(|v| (v * 4096.).round() as i32);
                    let page = q.material.texture.and_then(pages);
                    let packed = page.map_or("false,{0,0,0,0}".to_string(), |page| {
                        format!(
                            "true,{{{}}}",
                            q.uv.iter()
                                .map(|uv| packed_uv(*uv, page).to_string())
                                .collect::<Vec<_>>()
                                .join(",")
                        )
                    });
                    let color_indices = q.vertex_colors.map(|c| palette.indices(c)).unwrap_or([0; 4]);
                    let colors = format!("{},{{{}}}", q.vertex_colors.is_some(), color_indices.map(|i| i.to_string()).join(","));
                    format!(
                        "{{{{{},{},{},{}}},0,{{{},{},{}}},{},{},{},{},{}}}",
                        v[0],
                        v[1],
                        v[2],
                        v[3],
                        n[0],
                        n[1],
                        n[2],
                        crate::texture::material_cpp(&q.material),
                        i * 4,
                        crate::texture::uv_cpp(q.uv),
                        packed,
                        colors
                    )
                })
                .collect::<Vec<_>>()
                .join(",")
        ));
        let next = if c + 1 < chunks.len() {
            format!("&editable_{index}_{}", c + 1)
        } else {
            "nullptr".into()
        };
        let o = chunk.origin.map(|v| (v * 4096.) as i32);
        let center = chunk.center;
        let ext = chunk.extent;
        let visibility = if c == 0 && chunks.len() > 1 {
            format!("precomputed_visibility ? &editable_{index}_visibility : nullptr")
        } else {
            "nullptr".into()
        };
        text.push_str(&format!("inline constexpr MeshGeometry {key}={{{key}_vertices,{}, {key}_faces,{},true,{{{},{},{}}},{{{},{},{}}},{{{},{},{}}},{next},{visibility}}};\n",chunk.vertices.len(),chunk.faces.len(),o[0],o[1],o[2],center[0],center[1],center[2],ext[0],ext[1],ext[2]));
    }
    if chunks.is_empty() {
        text.push_str(&format!(
            "inline constexpr MeshGeometry editable_{index}_0={{nullptr,0,nullptr,0,true}};\n"
        ));
    }
    text
}

#[cfg(test)]
mod audit_tests {
    /// Opt-in measurement of the real cooker, without launching the editor or
    /// changing a map. A fresh output path preserves before/after evidence.
    #[test]
    #[ignore = "set EPOK_COOK_SCENE and EPOK_COOK_REPORT to audit a local scene"]
    fn audit_scene_cooking() {
        use sha2::{Digest, Sha256};
        use std::{io::Write, path::Path};
        let path = std::env::var("EPOK_COOK_SCENE").unwrap();
        let report = std::env::var("EPOK_COOK_REPORT").unwrap();
        let source = std::fs::read(&path).unwrap();
        let scene = crate::scene::Scene::load(Path::new(&path)).unwrap();
        let mut actors = vec![];
        let mut totals = [0_usize; 6];
        for (i, actor) in scene.actors.iter().enumerate() {
            if !scene.is_active(i) { continue; }
            let Some(mesh) = &actor.editable_mesh else { continue; };
            let doc = mesh.document.as_ref().expect("resolved source mesh");
            let quads = crate::lighting::quads(actor);
            let chunks = super::chunks(&quads).unwrap();
            let vertices: usize = chunks.iter().map(|c| c.vertices.len()).sum();
            let vertex_bytes: usize = chunks.iter().map(|c| c.vertices.len() * c.vertex_stride()).sum();
            let triangles: usize = quads.iter().map(|q| if q.points[2] == q.points[3] { 1 } else { 2 }).sum();
            let values = [doc.faces.len(), quads.len(), triangles, chunks.len(), vertices, vertex_bytes];
            for (total, value) in totals.iter_mut().zip(values) { *total += value; }
            actors.push(serde_json::json!({"id": actor.id, "name": actor.name,
                "source_faces": values[0], "cooked_faces": values[1], "cooked_triangles": values[2],
                "chunks": values[3], "vertices": values[4], "vertex_bytes": values[5],
                "wide_chunks": chunks.iter().filter(|c| c.vertex_stride() == 12).count()}));
        }
        let data = serde_json::json!({"scene": path, "source_sha256": format!("{:x}", Sha256::digest(source)),
            "source_faces": totals[0], "cooked_faces": totals[1], "cooked_triangles": totals[2],
            "chunks": totals[3], "vertices": totals[4], "vertex_bytes": totals[5], "actors": actors});
        let mut file = std::fs::OpenOptions::new().create_new(true).write(true).open(report).unwrap();
        file.write_all(serde_json::to_string_pretty(&data).unwrap().as_bytes()).unwrap();
        println!("cook totals: {totals:?}");
    }
}
