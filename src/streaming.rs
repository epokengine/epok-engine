//! Deterministic CD pages for immutable editable-mesh payloads. Bounds and links
//! remain resident, so visibility tests never require a disc read.
use crate::{lighting::Quad, scene::Scene, texture::BlendMode};
use std::{collections::BTreeMap, path::Path};

pub const PAGE_BYTES: usize = 65_536;
pub const QUAD_BYTES: usize = 72;
pub const ARCHIVE: &str = "GEOMETRY.BIN";

/// FNV-1a32 over every byte, including deterministic zero page padding. The
/// runtime checks this before making a DMA-filled page available to consumers.
fn page_hash(bytes: &[u8]) -> u32 {
    bytes.iter().fold(2_166_136_261_u32, |hash, byte| {
        (hash ^ u32::from(*byte)).wrapping_mul(16_777_619)
    })
}

#[derive(Clone, Debug)]
struct Location {
    page: u32,
    vertices: u16,
    quads: u16,
}
#[derive(Default)]
pub struct Bundle {
    bytes: Vec<u8>,
    chunks: Vec<BTreeMap<String, Location>>,
    pool_pages: usize,
    triangle_budget: usize,
}
impl Bundle {
    pub fn page_count(&self) -> usize {
        self.bytes.len() / PAGE_BYTES
    }
    pub fn triangle_budget(&self) -> usize {
        self.triangle_budget
    }
    pub fn write(&self, build: &Path) -> Result<(), String> {
        if self.bytes.is_empty() {
            return clear(build);
        }
        crate::project::write_changed(&build.join(ARCHIVE), &self.bytes)
    }
    pub fn globals(&self) -> String {
        let hashes = if self.bytes.is_empty() {
            "0".into()
        } else {
            self.bytes
                .chunks_exact(PAGE_BYTES)
                .map(|page| format!("{}u", page_hash(page)))
                .collect::<Vec<_>>()
                .join(",")
        };
        format!(
            "inline constexpr uint32_t stream_page_count={};\ninline constexpr size_t stream_pool_pages={};\ninline constexpr const char* stream_archive_path=\"GEOMETRY.BIN;1\";\ninline constexpr uint32_t stream_page_hashes[]={{{hashes}}};\n",
            self.page_count(),
            if self.bytes.is_empty() {
                0
            } else {
                self.pool_pages
            }
        )
    }
    /// Replace only the payload declarations emitted by mesh_compile; visibility
    /// metadata and all immutable bounds/counts/linked-list order are preserved.
    pub fn rewrite(&self, bank: usize, header: &str) -> Result<String, String> {
        let chunks = self
            .chunks
            .get(bank)
            .ok_or("Streaming bank index out of range")?;
        let mut out = String::new();
        let mut replaced = 0;
        for line in header.lines() {
            let mut line = line.to_string();
            let mut omit = false;
            for (key, location) in chunks {
                if line.starts_with(&format!("inline constexpr int16_t {key}_vertices"))
                    || line.starts_with(&format!("inline constexpr MeshQuad {key}_faces"))
                {
                    omit = true;
                    break;
                }
                if line.starts_with(&format!("inline constexpr MeshGeometry {key}=")) {
                    line = line
                        .replace(&format!("{key}_vertices"), "nullptr")
                        .replace(&format!("{key}_faces"), "nullptr");
                    let base = line
                        .strip_suffix("};")
                        .ok_or("Unexpected mesh descriptor syntax")?;
                    line = format!(
                        "{base},{},{},{}}};",
                        location.page, location.vertices, location.quads
                    );
                    replaced += 1;
                    break;
                }
            }
            if !omit {
                out.push_str(&line);
                out.push('\n');
            }
        }
        if replaced != chunks.len() {
            return Err("Generated streaming mesh descriptors do not match compiled chunks".into());
        }
        Ok(out)
    }
}
pub fn clear(build: &Path) -> Result<(), String> {
    match std::fs::remove_file(build.join(ARCHIVE)) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}
fn i16_at(out: &mut [u8], offset: usize, value: i16) {
    out[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}
fn u16_at(out: &mut [u8], offset: usize, value: u16) {
    out[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}
fn i32_at(out: &mut [u8], offset: usize, value: i32) {
    out[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

/// PSX ABI layout, verified with C++ static_asserts by the runtime. No native
/// host struct layout, pointers, or endian assumptions enter the file format.
fn encode_quad(
    q: &Quad,
    indices: [u16; 4],
    index: usize,
    texture: i32,
    page: Option<(u16, u16, u16)>,
) -> [u8; QUAD_BYTES] {
    let mut out = [0; QUAD_BYTES];
    for (i, value) in indices.into_iter().enumerate() {
        u16_at(&mut out, i * 2, value);
    }
    for (i, value) in q.normal.into_iter().enumerate() {
        i16_at(&mut out, 10 + i * 2, (value * 4096.).round() as i16);
    }
    for (i, value) in q.material.color.into_iter().enumerate() {
        out[16 + i] = (value * 255.).round() as u8;
    }
    out[19] = u8::from(q.material.unlit);
    i32_at(&mut out, 20, texture);
    i32_at(
        &mut out,
        24,
        match q.material.blend {
            BlendMode::Cutout => 0,
            BlendMode::Average => 1,
            BlendMode::Add => 2,
            BlendMode::Subtract => 3,
            BlendMode::AddQuarter => 4,
        },
    );
    i16_at(&mut out, 28, q.material.depth_bias);
    for (i, value) in q.material.uv_scroll.into_iter().enumerate() {
        i32_at(&mut out, 32 + i * 4, (value * 4096.).round() as i32);
    }
    out[40..44].copy_from_slice(&((index * 4) as u32).to_le_bytes());
    for (i, uv) in q.uv.into_iter().enumerate() {
        for (axis, value) in uv.into_iter().enumerate() {
            i16_at(
                &mut out,
                44 + i * 4 + axis * 2,
                (value.clamp(0., 1.) * 4096.).round() as i16,
            );
        }
        if let Some(page) = page {
            u16_at(
                &mut out,
                62 + i * 2,
                crate::mesh_compile::packed_uv(uv, page),
            );
        }
    }
    out[60] = u8::from(page.is_some());
    out
}

#[cfg(test)]
pub fn compile(scenes: &[Scene], pool_pages: usize) -> Result<Bundle, String> {
    compile_with_budget(scenes, pool_pages, 4096)
}
pub fn compile_with_budget(
    scenes: &[Scene],
    pool_pages: usize,
    triangle_budget: usize,
) -> Result<Bundle, String> {
    if scenes.is_empty() {
        return Err("Streaming compilation needs at least one scene".into());
    }
    if !(2..=8).contains(&pool_pages) {
        return Err("Geometry streaming pool requires 2..8 pages".into());
    }
    if !(512..=8192).contains(&triangle_budget) {
        return Err("Streaming per-frame triangle budget requires 512..8192 triangles".into());
    }
    let shared = crate::scene_bank::resources(scenes);
    let ids = crate::texture::ids(&shared);
    let mut bundle = Bundle {
        pool_pages,
        triangle_budget,
        ..Default::default()
    };
    let mut used = PAGE_BYTES;
    for scene in scenes {
        let layout = crate::texture::layout(scene)?.textures;
        let mut locations = BTreeMap::new();
        for (entity, e) in scene.actors.iter().enumerate().filter(|(_, e)| {
            e.kind == "Mesh"
                && (e.editable_mesh.is_some() || e.terrain.is_some())
                && e.skeletal_mesh.is_none()
        }) {
            let qs = crate::lighting::quads(e);
            if qs.len() > 3500 {
                return Err("Compiled mesh exceeds 7000 triangles".into());
            }
            // Terrain bins on cell indices, meshes on world centroids. Both
            // must partition here exactly as the header emitter does, or the
            // patched stream offsets would point at the wrong chunk.
            let parts = match &e.terrain {
                Some(terrain) => crate::terrain_compile::chunks(terrain, &qs)?,
                None => crate::mesh_compile::chunks(&qs)?,
            };
            for (chunk_index, chunk) in parts.into_iter().enumerate() {
                let vertex_bytes = chunk.vertices.len() * 6;
                let quad_start = (vertex_bytes + 3) & !3;
                let length = quad_start + chunk.faces.len() * QUAD_BYTES;
                if length > PAGE_BYTES {
                    return Err("Mesh chunk exceeds a 64 KiB streaming page".into());
                }
                used = (used + 3) & !3;
                if used + length > PAGE_BYTES {
                    bundle.bytes.resize(bundle.bytes.len() + PAGE_BYTES, 0);
                    used = 0;
                }
                let page = bundle.page_count() - 1;
                let start = page * PAGE_BYTES + used;
                for (i, v) in chunk.vertices.iter().enumerate() {
                    for (axis, value) in v.iter().enumerate() {
                        i16_at(&mut bundle.bytes, start + i * 6 + axis * 2, *value);
                    }
                }
                for (face, (indices, index)) in chunk.faces.iter().enumerate() {
                    let q = &qs[*index];
                    let (texture, placement) = if let Some(id) = q.material.texture {
                        let texture = ids
                            .iter()
                            .position(|candidate| *candidate == id)
                            .ok_or("Streamed material texture not found in shared bank")?
                            as i32;
                        let t = scene
                            .textures
                            .get(&id)
                            .ok_or("Streamed texture unresolved")?;
                        let p = layout
                            .iter()
                            .find(|(candidate, _)| *candidate == id)
                            .ok_or("Streamed texture not placed")?;
                        (texture, Some((t.width, t.height, p.1.y)))
                    } else {
                        (-1, None)
                    };
                    let encoded = encode_quad(q, *indices, *index, texture, placement);
                    let offset = start + quad_start + face * QUAD_BYTES;
                    bundle.bytes[offset..offset + QUAD_BYTES].copy_from_slice(&encoded);
                }
                locations.insert(
                    format!("editable_{entity}_{chunk_index}"),
                    Location {
                        page: u32::try_from(page).map_err(|_| "Too many streaming pages")?,
                        vertices: used as u16,
                        quads: (used + quad_start) as u16,
                    },
                );
                used += length;
            }
        }
        bundle.chunks.push(locations);
    }
    Ok(bundle)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn scene(cubes: usize) -> Scene {
        let mut doc = crate::mesh::Document::default();
        doc.materials[0].material.color = [0.2, 0.4, 0.6];
        doc.materials[0].material.unlit = true;
        doc.materials[0].material.blend = BlendMode::Add;
        doc.materials[0].material.depth_bias = -17;
        doc.materials[0].material.uv_scroll = [0.25, -0.5];
        for i in 0..cubes {
            doc.primitive(
                "Cube",
                [(i % 20) as f32 * 4., (i / 20) as f32 * 4., 0.],
                [1.; 3],
                1,
                doc.groups[0].id,
                doc.materials[0].id,
            );
        }
        let mut e = crate::scene::Actor::cube("Streamed".into());
        let mut component = crate::mesh::Component::new(uuid::Uuid::new_v4());
        component.document = Some(std::sync::Arc::new(doc));
        e.editable_mesh = Some(component);
        e.lighting.static_geometry = true;
        Scene {
            actors: vec![e],
            ..Default::default()
        }
    }
    fn u16_at(bytes: &[u8], offset: usize) -> u16 {
        u16::from_le_bytes(bytes[offset..offset + 2].try_into().unwrap())
    }
    fn i32_at(bytes: &[u8], offset: usize) -> i32 {
        i32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
    }
    #[test]
    fn pages_roundtrip_vertices_materials_and_chunk_indices() {
        let scene = scene(350);
        let bundle = compile(std::slice::from_ref(&scene), 2).unwrap();
        assert!(bundle.page_count() > 2);
        assert_eq!(bundle.bytes.len() % PAGE_BYTES, 0);
        assert_eq!(
            bundle.bytes,
            compile(std::slice::from_ref(&scene), 2).unwrap().bytes
        );
        let qs = crate::lighting::quads(&scene.actors[0]);
        for (i, chunk) in crate::mesh_compile::chunks(&qs).unwrap().iter().enumerate() {
            let loc = &bundle.chunks[0][&format!("editable_0_{i}")];
            assert_eq!(loc.vertices % 4, 0);
            assert_eq!(loc.quads % 4, 0);
            let page =
                &bundle.bytes[loc.page as usize * PAGE_BYTES..(loc.page as usize + 1) * PAGE_BYTES];
            assert!(loc.quads as usize + chunk.faces.len() * QUAD_BYTES <= PAGE_BYTES);
            for (v, xyz) in chunk.vertices.iter().enumerate() {
                for (axis, value) in xyz.iter().enumerate() {
                    assert_eq!(
                        u16_at(page, loc.vertices as usize + v * 6 + axis * 2) as i16,
                        *value
                    );
                }
            }
            for (face, (indices, index)) in chunk.faces.iter().enumerate() {
                let offset = loc.quads as usize + face * QUAD_BYTES;
                for (j, value) in indices.iter().enumerate() {
                    assert_eq!(u16_at(page, offset + j * 2), *value);
                }
                assert_eq!(&page[offset + 16..offset + 20], &[51, 102, 153, 1]);
                assert_eq!(i32_at(page, offset + 20), -1);
                assert_eq!(i32_at(page, offset + 24), 2);
                assert_eq!(u16_at(page, offset + 28) as i16, -17);
                assert_eq!(i32_at(page, offset + 32), 1024);
                assert_eq!(i32_at(page, offset + 36), -2048);
                assert_eq!(i32_at(page, offset + 40), (*index * 4) as i32);
            }
        }
    }
    #[test]
    fn bank_rewrite_removes_payload_but_preserves_bounds_and_links() {
        let first = scene(2);
        let mut second = first.clone();
        second.name = "Second".into();
        let bundle = compile(&[first.clone(), second.clone()], 4).unwrap();
        let header =
            crate::scene_bank::header_with_streaming(&[first, second], &[], Some(&bundle)).unwrap();
        assert!(!header.contains("inline constexpr int16_t editable_0_0_vertices"));
        assert!(!header.contains("inline constexpr MeshQuad editable_0_0_faces"));
        assert!(header.contains("MeshGeometry editable_0_0={nullptr,"));
        assert!(header.contains("&editable_0_1"));
        assert!(header.contains("stream_pool_pages=4"));
        let mut dynamic = scene(1);
        dynamic.actors[0].lighting.static_geometry = false;
        // Runtime transforms may change; only immutable payload is streamed.
        assert_eq!(compile(&[dynamic], 2).unwrap().page_count(), 1);
        assert_eq!(compile(&[Scene::default()], 2).unwrap().page_count(), 0);
        assert!(compile(&[], 2).is_err());
        assert!(compile(&[scene(1)], 1).is_err());
    }
    #[test]
    fn packed_uv_and_global_texture_index_have_native_offsets() {
        let scene = scene(1);
        let q = &crate::lighting::quads(&scene.actors[0])[0];
        let encoded = encode_quad(q, [1, 2, 3, 4], 7, 12, Some((64, 32, 300)));
        assert_eq!(i32_at(&encoded, 20), 12);
        assert_eq!(encoded[60], 1);
        for i in 0..4 {
            assert_eq!(
                u16_at(&encoded, 62 + i * 2),
                crate::mesh_compile::packed_uv(q.uv[i], (64, 32, 300))
            );
        }
    }
    #[test]
    fn streamed_world_size_does_not_size_gpu_or_retained_buffers() {
        let mut scene = scene(350);
        let mut other = scene.actors[0].clone();
        let ids = crate::actor_document::fresh_identities(std::slice::from_ref(&other));
        crate::actor_document::remap_actor(&mut other, &ids);
        other.name = "Second mesh".into();
        scene.actors.push(other);
        let world_triangles = scene
            .actors
            .iter()
            .map(crate::lighting::quad_count)
            .sum::<usize>()
            * 2;
        assert!(world_triangles > 8192);
        let bundle = compile_with_budget(&[scene.clone()], 2, 1024).unwrap();
        let header =
            crate::scene_bank::header_with_streaming(&[scene], &[], Some(&bundle)).unwrap();
        assert!(header.contains("inline constexpr size_t render_capacity=1024;"));
        assert!(header.contains("inline constexpr size_t retained_quad_capacity=512;"));
        assert!(compile_with_budget(&[Scene::default()], 2, 511).is_err());
        assert!(compile_with_budget(&[Scene::default()], 2, 8193).is_err());
    }
    #[test]
    fn checksums_cover_same_size_corruption_and_full_padding() {
        // Published FNV-1a32 reference vector also pins the algorithm's seed.
        assert_eq!(page_hash(b"hello"), 0x4f9f2cab);
        let scene = scene(2);
        let bundle = compile(std::slice::from_ref(&scene), 2).unwrap();
        let identical = compile(&[scene], 2).unwrap();
        assert_eq!(bundle.globals(), identical.globals());
        assert!(bundle.globals().contains(&format!(
            "stream_page_hashes[]={{{}u}}",
            page_hash(&bundle.bytes)
        )));
        let original_hash = page_hash(&bundle.bytes);
        let mut corrupt = bundle.bytes.clone();
        corrupt[0] ^= 1;
        assert_eq!(corrupt.len(), bundle.bytes.len());
        assert_ne!(page_hash(&corrupt), original_hash);
        corrupt.clone_from(&bundle.bytes);
        *corrupt.last_mut().unwrap() ^= 1;
        assert_ne!(page_hash(&corrupt), original_hash);
        assert!(
            Bundle::default()
                .globals()
                .contains("stream_page_hashes[]={0}")
        );
    }
}
