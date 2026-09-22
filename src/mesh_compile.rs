//! Deterministic spatial partitioning for the authored mesh. Author groups never become draw units.
use crate::{
    lighting::{Quad, dot, sub},
    mesh::{Component, Document},
    scene::Actor,
};
use std::collections::BTreeMap;

/// Bound each parametric edge separately. A 32 x 0.5 wall cap needs 8 x 1
/// pieces, not 8 x 8. Bilinear derivatives interpolate the opposing edges.
pub(crate) fn subdivisions(p: [[f32; 3]; 4]) -> [usize; 2] {
    let edge_span = |a: usize, b: usize| {
        (0..3)
            .map(|c| (p[a][c] - p[b][c]).abs())
            .fold(0_f32, f32::max)
    };
    [
        edge_span(0, 1).max(edge_span(3, 2)),
        edge_span(0, 3).max(edge_span(1, 2)),
    ]
    .map(|span| (span / 4.).ceil().max(1.) as usize)
}

pub fn quads(e: &Actor, mesh: &Component) -> Vec<Quad> {
    let Some(doc) = &mesh.document else {
        return vec![];
    };
    let mut out = vec![];
    for f in &doc.faces {
        let p = doc.points(f);
        let normal = crate::mesh::face_normal(p);
        let mat = crate::mesh::material(e, f.material);
        // Bound generated face extent so chunk-local Q12 coordinates cannot overflow.
        let [nx, ny] = subdivisions(p);
        for y in 0..ny {
            for x in 0..nx {
                let points = [(x, y), (x + 1, y), (x + 1, y + 1), (x, y + 1)].map(|(x, y)| {
                    let u = x as f32 / nx as f32;
                    let v = y as f32 / ny as f32;
                    std::array::from_fn(|c| {
                        ((p[0][c] * (1. - u) * (1. - v)
                            + p[1][c] * u * (1. - v)
                            + p[2][c] * u * v
                            + p[3][c] * (1. - u) * v)
                            * 4096.)
                            .round()
                            / 4096.
                    })
                });
                if dot(
                    crate::mesh::cross(sub(points[1], points[0]), sub(points[2], points[0])),
                    normal,
                )
                .abs()
                    < 1e-9
                    && points[2] != points[3]
                {
                    continue;
                }
                out.push(Quad {
                    uv: [(x, y), (x + 1, y), (x + 1, y + 1), (x, y + 1)].map(|(x, y)| {
                        let u = x as f32 / nx as f32;
                        let v = y as f32 / ny as f32;
                        std::array::from_fn(|c| {
                            f.uv[0][c] * (1. - u) * (1. - v)
                                + f.uv[1][c] * u * (1. - v)
                                + f.uv[2][c] * u * v
                                + f.uv[3][c] * (1. - u) * v
                        })
                    }),
                    face: 0,
                    points,
                    normal,
                    material: mat.clone(),
                    id: Some(f.id),
                });
            }
        }
    }
    out
}
pub struct Chunk {
    pub origin: [f32; 3],
    pub vertices: Vec<[i16; 3]>,
    pub faces: Vec<([u16; 4], usize)>,
    pub center: [i32; 3],
    pub extent: [i32; 3],
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
    // Snap origins to Q8 so origin>>4 + local>>4 is bit-identical to
    // global>>4. A changed partition must never introduce GTE seams.
    if (0..3).any(|c| world_high[c] - world_low[c] > 65504) {
        if part.len() < 2 {
            return Err("Chunk coordinate overflow".into());
        }
        let (a, b) = part.split_at(part.len() / 2);
        encode_chunks(quads, a, chunks)?;
        encode_chunks(quads, b, chunks)?;
        return Ok(());
    }
    let origin = std::array::from_fn(|c| {
        (((world_low[c] + world_high[c]) / 2).div_euclid(16) * 16) as f32 / 4096.
    });
    let mut vertices = vec![];
    let mut lookup = BTreeMap::new();
    let mut encoded = vec![];
    let mut low = [i32::MAX; 3];
    let mut high = [i32::MIN; 3];
    for &i in part {
        let mut indices = [0_u16; 4];
        for (j, p) in quads[i].points.iter().enumerate() {
            let mut local = [0_i16; 3];
            for c in 0..3 {
                let v = ((p[c] - origin[c]) * 4096.).round() as i32;
                local[c] = i16::try_from(v).map_err(|_| "Chunk coordinate overflow")?;
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
        center: std::array::from_fn(|c| (high[c] + low[c]) / 2),
        extent: std::array::from_fn(|c| (high[c] - low[c] + 1) / 2),
    });
    Ok(())
}
/// Texture page geometry (width, height, VRAM y) used to bake 8-bit page UVs.
pub type PageLookup<'a> = &'a dyn Fn(uuid::Uuid) -> Option<(u16, u16, u16)>;
#[cfg(test)]
pub fn header(e: &Actor, index: usize) -> Result<String, String> {
    header_with_pages(e, index, &|_| None)
}
/// Packed page UVs replicate the runtime's texture_uv mapping so the renderer
/// copies them instead of scaling four Q12 pairs per drawn quad.
pub fn packed_uv(uv: [f32; 2], page: (u16, u16, u16)) -> u16 {
    let (width, height, y) = page;
    let q = |v: f32| (v.clamp(0., 1.) * 4096.).round() as i32;
    let u = (q(uv[0]) * (i32::from(width) - 1) / 4096) & 255;
    let v = ((i32::from(y) % 256) + q(uv[1]) * (i32::from(height) - 1) / 4096) & 255;
    (u | (v << 8)) as u16
}
pub fn header_with_pages(e: &Actor, index: usize, pages: PageLookup) -> Result<String, String> {
    let mesh = e.editable_mesh.as_ref().unwrap();
    let _doc: &Document = mesh.document.as_deref().ok_or_else(|| {
        mesh.error
            .clone()
            .unwrap_or("EditableMesh not resolved".into())
    })?;
    let qs = crate::lighting::quads(e);
    if qs.len() > 3500 {
        return Err(
            "Compiled mesh exceeds 7000 triangles. Reduce surface sizes or face count.".into(),
        );
    }
    let chunks = chunks(&qs)?;
    Ok(header_for(index, &qs, &chunks, pages))
}

/// Emit the cooked chunk arrays for one actor. Terrain partitions its quads
/// differently but stores them in exactly this layout, so it shares the
/// emitter rather than growing a second symbol namespace the runtime,
/// streaming archive and visibility bake would all have to learn.
pub fn header_for(index: usize, qs: &[Quad], chunks: &[Chunk], pages: PageLookup) -> String {
    let mut text = visibility::header(chunks, index);
    for (c, chunk) in chunks.iter().enumerate().rev() {
        let key = format!("editable_{index}_{c}");
        text.push_str(&format!(
            "inline constexpr int16_t {key}_vertices[][3]={{{}}};\n",
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
                    format!(
                        "{{{{{},{},{},{}}},0,{{{},{},{}}},{},{},{},{}}}",
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
                        packed
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
