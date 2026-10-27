//! Deterministic CD pages for immutable editable-mesh payloads. Bounds and links
//! remain resident, so visibility tests never require a disc read.
use crate::{lighting::Quad, scene::Scene, texture::BlendMode};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

pub const PAGE_BYTES: usize = 65_536;
pub const QUAD_BYTES: usize = 80;
pub const ARCHIVE: &str = "GEOMETRY.BIN";

/// FNV-1a32 over every byte, including deterministic zero page padding. The
/// runtime checks this before making a DMA-filled page available to consumers.
pub(crate) fn page_hash(bytes: &[u8]) -> u32 {
    bytes.iter().fold(2_166_136_261_u32, |hash, byte| {
        (hash ^ u32::from(*byte)).wrapping_mul(16_777_619)
    })
}

#[derive(Clone, Debug)]
struct Location {
    page: u32,
    vertices: u16,
    quads: u16,
    vertex_stride: u8,
    baked: bool,
    /// Static-world chunk: world-space origin, center and extent (Q12).
    world: Option<[[i32; 3]; 3]>,
}
#[derive(Default)]
struct PageTrack {
    sets: Vec<Vec<u32>>,
    samples: Vec<u16>,
}
impl PageTrack {
    fn cook(sets: &[BTreeMap<u32, ()>]) -> Result<Self, String> {
        let mut out = Self::default();
        let mut dictionary = BTreeMap::new();
        for set in sets {
            let pages = set.keys().copied().collect::<Vec<_>>();
            let index = if let Some(&index) = dictionary.get(&pages) {
                index
            } else {
                let index =
                    u16::try_from(out.sets.len()).map_err(|_| "Too many baked page sets")?;
                dictionary.insert(pages.clone(), index);
                out.sets.push(pages);
                index
            };
            out.samples.push(index);
        }
        Ok(out)
    }
    fn header(&self) -> String {
        let mut ids = Vec::new();
        let mut spans = Vec::new();
        for set in &self.sets {
            spans.push(format!("{{track_page_ids+{},{}}}", ids.len(), set.len()));
            ids.extend(set.iter().map(u32::to_string));
        }
        if ids.is_empty() {
            ids.push("0".into());
        }
        format!(
            "inline constexpr uint32_t track_page_ids[]={{{}}};\ninline constexpr VisibilityPageSet track_page_sets[]={{{}}};\ninline constexpr uint16_t track_page_samples[]={{{}}};\n",
            ids.join(","),
            spans.join(","),
            self.samples
                .iter()
                .map(u16::to_string)
                .collect::<Vec<_>>()
                .join(",")
        )
    }
}
#[derive(Default)]
pub struct Bundle {
    vertex_colors: crate::vertex_colors::Palette,
    bytes: Vec<u8>,
    chunks: Vec<BTreeMap<String, Location>>,
    pool_pages: usize,
    triangle_budget: usize,
    previews: Vec<crate::baked_preview::Bake>,
    page_tracks: Vec<Option<PageTrack>>,
    ordered: Vec<Option<crate::ordered_draw_cook::Cooked>>,
    texture_pixels: BTreeMap<uuid::Uuid, (u32, u16)>,
    geometry_pages: Option<usize>,
}
impl Bundle {
    pub fn vertex_colors(&self) -> &crate::vertex_colors::Palette {
        &self.vertex_colors
    }
    pub fn texture_pixels(&self, id: uuid::Uuid) -> Option<(u32, u16)> {
        self.texture_pixels.get(&id).copied()
    }
    /// Append upload-only pixels after geometry packing. Existing page IDs,
    /// visibility sets and checksums for geometry remain unchanged. Palettes
    /// stay in CPU memory for animation/reset; pixels live in VRAM after load.
    pub fn append_texture_pixels(&mut self, resources: &Scene) -> Result<(), String> {
        if !self.texture_pixels.is_empty() {
            return Err("Texture upload payloads were already packed".into());
        }
        self.geometry_pages = Some(self.page_count());
        let mut used = PAGE_BYTES;
        for id in crate::texture::ids(resources) {
            let data = resources
                .textures
                .get(&id)
                .ok_or("Unresolved streamed texture")?;
            let cooked = crate::texture_cook::cook(data);
            let length = cooked.words.len() * 2;
            if length == 0 || length > PAGE_BYTES || !length.is_multiple_of(4) {
                return Err("Texture upload must be 4-byte aligned and fit one 64 KiB page".into());
            }
            if used + length > PAGE_BYTES {
                self.bytes.resize(self.bytes.len() + PAGE_BYTES, 0);
                used = 0;
            }
            let page = self.page_count() - 1;
            let start = page * PAGE_BYTES + used;
            for (i, word) in cooked.words.iter().enumerate() {
                u16_at(&mut self.bytes, start + i * 2, *word);
            }
            self.texture_pixels.insert(
                id,
                (
                    u32::try_from(page).map_err(|_| "Too many texture upload pages")?,
                    u16::try_from(used).map_err(|_| "Texture upload offset overflow")?,
                ),
            );
            used += length;
        }
        Ok(())
    }
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
        crate::project::write_changed(&build.join(ARCHIVE), &self.bytes)?;
        crate::project::write_changed(
            &build.join(crate::baked_preview::FILE),
            &serde_json::to_vec(&self.previews).map_err(|e| e.to_string())?,
        )
    }
    pub fn globals(&self) -> String {
        let baked_count = self
            .chunks
            .iter()
            .flat_map(|c| c.values())
            .filter(|l| l.baked)
            .count();
        let cache_slots = baked_count.clamp(1, 128).next_power_of_two();
        let baked_enabled = baked_count > 0;
        let ordered_capacity = self
            .ordered
            .iter()
            .flatten()
            .map(|o| o.max_packets)
            .max()
            .unwrap_or(0);
        let hashes = if self.bytes.is_empty() {
            "0".into()
        } else {
            self.bytes
                .chunks_exact(PAGE_BYTES)
                .map(|page| format!("{}u", page_hash(page)))
                .collect::<Vec<_>>()
                .join(",")
        };
        let ordered_globals = format!(
            "inline constexpr bool ordered_draw_enabled={};\ninline constexpr size_t ordered_draw_capacity={};\n",
            self.ordered.iter().any(Option::is_some),
            ordered_capacity.max(1)
        );
        let geometry_globals = format!(
            "inline constexpr uint32_t stream_geometry_page_count={};\n",
            self.geometry_pages.unwrap_or_else(|| self.page_count())
        );
        ordered_globals
            + &geometry_globals
            + &format!(
                "inline constexpr uint32_t baked_draw_abi={};\ninline constexpr bool baked_draw_enabled={baked_enabled};\ninline constexpr size_t baked_draw_cache_slots={cache_slots};\ninline constexpr uint32_t stream_page_count={};\ninline constexpr size_t stream_pool_pages={};\ninline constexpr const char* stream_archive_path=\"GEOMETRY.BIN;1\";\ninline constexpr uint32_t stream_page_hashes[]={{{hashes}}};\n",
                crate::baked_draw::ABI_VERSION,
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
        let pages = self.page_tracks.get(bank).and_then(Option::as_ref);
        let mut pages_replaced = false;
        let ordered = self.ordered.get(bank).and_then(Option::as_ref);
        for line in header.lines() {
            let mut line = line.to_string();
            if let Some(pages) = pages
                && line.starts_with("inline constexpr VisibilityTrackData track_data=")
            {
                out.push_str(&pages.header());
                if let Some(ordered) = ordered {
                    out.push_str(&ordered.header(ordered.actor_count));
                }
                let base = line
                    .strip_suffix("};")
                    .ok_or("Unexpected visibility descriptor syntax")?;
                line = format!(
                    "{base},track_page_sets,track_page_samples{}}};",
                    if ordered.is_some() {
                        ",&ordered_data"
                    } else {
                        ""
                    }
                );
                pages_replaced = true;
            }
            let mut omit = false;
            for (key, location) in chunks {
                if line.starts_with(&format!("inline constexpr int16_t {key}_vertices"))
                    || line.starts_with(&format!("inline constexpr int32_t {key}_vertices"))
                    || line.starts_with(&format!("inline constexpr MeshQuad {key}_faces"))
                {
                    omit = true;
                    break;
                }
                if line.starts_with(&format!("inline constexpr MeshGeometry {key}=")) {
                    line = line
                        .replace(
                            &format!("{key}_vertices"),
                            if location.vertex_stride == 12 {
                                "MeshVertices{nullptr,12}"
                            } else {
                                "nullptr"
                            },
                        )
                        .replace(&format!("{key}_faces"), "nullptr");
                    if let Some([origin, center, extent]) = location.world {
                        // Flags baked (2) | static world (4), and the world-space bounds.
                        let (head, tail) = line
                            .split_once(",true,")
                            .ok_or("Unexpected mesh descriptor syntax")?;
                        let mut rest = tail;
                        for _ in 0..3 {
                            let close =
                                rest.find('}').ok_or("Unexpected mesh descriptor syntax")?;
                            rest = rest[close + 1..]
                                .strip_prefix(',')
                                .unwrap_or(&rest[close + 1..]);
                        }
                        let triple = |v: [i32; 3]| format!("{{{},{},{}}}", v[0], v[1], v[2]);
                        line = format!(
                            "{head},7,{},{},{},{rest}",
                            triple(origin),
                            triple(center),
                            triple(extent)
                        );
                    } else if location.baked {
                        line = line.replacen(",true,", ",3,", 1);
                    }
                    let base = line
                        .strip_suffix("};")
                        .ok_or("Unexpected mesh descriptor syntax")?;
                    line = if let Some((prefix, mask)) = base.rsplit_once(",0xffffffffu,0,0,") {
                        format!(
                            "{prefix},{},{},{},{mask}}};",
                            location.page, location.vertices, location.quads
                        )
                    } else {
                        format!(
                            "{base},{},{},{}}};",
                            location.page, location.vertices, location.quads
                        )
                    };
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
        if pages.is_some() && !pages_replaced {
            return Err("Baked page sets require the generated visibility descriptor".into());
        }
        Ok(out)
    }
}
pub fn clear(build: &Path) -> Result<(), String> {
    for name in [ARCHIVE, crate::baked_preview::FILE] {
        match std::fs::remove_file(build.join(name)) {
            Ok(()) => (),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            Err(e) => return Err(e.to_string()),
        }
    }
    Ok(())
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
fn encode_quad_with_palette(
    q: &Quad,
    indices: [u16; 4],
    index: usize,
    texture: i32,
    page: Option<crate::texture_cook::Page>,
    palette: &crate::vertex_colors::Palette,
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
    // Former material alignment padding; Material stays 24 bytes.
    out[30] = u8::from(q.material.double_sided);
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
    if let Some(colors) = q.vertex_colors {
        out[70] = 1;
        for (i, index) in palette.indices(colors).into_iter().enumerate() {
            u16_at(&mut out, 72 + i * 2, index);
        }
    }
    out
}

#[cfg(test)]
fn encode_quad(
    q: &Quad,
    indices: [u16; 4],
    index: usize,
    texture: i32,
    page: Option<crate::texture_cook::Page>,
) -> [u8; QUAD_BYTES] {
    encode_quad_with_palette(
        q,
        indices,
        index,
        texture,
        page,
        &crate::vertex_colors::Palette::quads([q]).unwrap(),
    )
}

#[cfg(test)]
pub fn compile(scenes: &[Scene], pool_pages: usize) -> Result<Bundle, String> {
    compile_with_budget(scenes, pool_pages, 4096)
}
#[cfg(test)]
pub fn compile_with_budget(
    scenes: &[Scene],
    pool_pages: usize,
    triangle_budget: usize,
) -> Result<Bundle, String> {
    if scenes.is_empty() {
        return Err("Streaming compilation needs at least one scene".into());
    }
    let palette = crate::vertex_colors::Palette::scene(&crate::scene_bank::resources(scenes))?;
    compile_with_palette(scenes, pool_pages, triangle_budget, &palette)
}
pub fn compile_with_palette(
    scenes: &[Scene],
    pool_pages: usize,
    triangle_budget: usize,
    palette: &crate::vertex_colors::Palette,
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
        vertex_colors: palette.clone(),
        pool_pages,
        triangle_budget,
        ..Default::default()
    };
    let mut used = PAGE_BYTES;
    let mut exact_pool = 2;
    let mut all_residency_baked = true;
    for scene in scenes {
        let track = crate::visibility_track::cook(scene)?;
        if track.is_some() {
            used = PAGE_BYTES;
        }
        let first_page = bundle.page_count();
        let layout = crate::texture::layout(scene)?.textures;
        let mut locations = BTreeMap::new();
        let mut lengths = BTreeMap::new();
        let mut preview_chunks = Vec::new();
        let mut page_track = None;
        let mut ordered = None;
        for (entity, e) in scene.actors.iter().enumerate().filter(|(_, e)| {
            e.kind == "Mesh"
                && (e.editable_mesh.is_some() || e.terrain.is_some())
                && e.skeletal_mesh.is_none()
        }) {
            let qs = crate::lighting::quads(e);
            let baked = !scene.fog.enabled
                && crate::baked_draw::eligible(e, &qs)
                && track.as_ref().is_some_and(|t| {
                    t.chunks
                        .keys()
                        .any(|key| key.starts_with(&format!("editable_{entity}_")))
                });
            if !baked {
                all_residency_baked = false;
            }
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
            // Static world: tracked baked meshes keep their chunks, faces and
            // vertex order, with world-space vertices (see world_chunk).
            let world_doc = if baked
                && e.terrain.is_none()
                && scene
                    .visibility_track
                    .as_ref()
                    .is_some_and(|t| t.static_world)
            {
                Some(
                    e.editable_mesh
                        .as_ref()
                        .and_then(|m| m.document.as_deref())
                        .ok_or("Static world mesh asset is unresolved")?,
                )
            } else {
                None
            };
            let world_transform = scene.world_matrix(entity);
            for (chunk_index, chunk) in parts.into_iter().enumerate() {
                let world = world_doc
                    .map(|doc| crate::mesh_compile::world_chunk(doc, &world_transform, &chunk))
                    .transpose()?;
                let chunk_vertices = world.as_ref().map_or(&chunk.vertices, |w| &w.vertices);
                let key = format!("editable_{entity}_{chunk_index}");
                if let Some((word, _)) = track.as_ref().and_then(|t| t.chunks.get(&key)) {
                    preview_chunks.push((
                        key,
                        crate::baked_preview::Chunk {
                            actor: e.id,
                            page: 0,
                            word: *word,
                            quads: chunk.faces.iter().map(|(_, i)| *i).collect(),
                        },
                    ));
                }
                let vertex_stride = if chunk_vertices
                    .iter()
                    .flatten()
                    .all(|v| i16::try_from(*v).is_ok())
                {
                    6
                } else {
                    12
                };
                let vertex_bytes = chunk_vertices.len() * vertex_stride;
                let quad_start = (vertex_bytes + 3) & !3;
                let stride = if baked {
                    crate::baked_draw::FACE_BYTES
                } else {
                    QUAD_BYTES
                };
                let length = quad_start + chunk.faces.len() * stride;
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
                for (i, v) in chunk_vertices.iter().enumerate() {
                    for (axis, value) in v.iter().enumerate() {
                        if vertex_stride == 6 {
                            i16_at(
                                &mut bundle.bytes,
                                start + i * 6 + axis * 2,
                                i16::try_from(*value).map_err(|_| "Narrow mesh vertex overflow")?,
                            );
                        } else {
                            i32_at(&mut bundle.bytes, start + i * 12 + axis * 4, *value);
                        }
                    }
                }
                for (face, (indices, index)) in chunk.faces.iter().enumerate() {
                    let q = &qs[*index];
                    let (texture, placement, resolved) = if let Some(id) = q.material.texture {
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
                        (
                            texture,
                            Some(p.1.page(t)),
                            Some(crate::baked_draw::ResolvedTexture::new(p.1, t)),
                        )
                    } else {
                        (-1, None, None)
                    };
                    let offset = start + quad_start + face * stride;
                    if baked {
                        let encoded = crate::baked_draw::encode_with_palette(
                            q, *indices, e, resolved, palette,
                        );
                        bundle.bytes[offset..offset + stride].copy_from_slice(&encoded);
                    } else {
                        let encoded = encode_quad_with_palette(
                            q, *indices, *index, texture, placement, palette,
                        );
                        bundle.bytes[offset..offset + stride].copy_from_slice(&encoded);
                    }
                }
                locations.insert(
                    format!("editable_{entity}_{chunk_index}"),
                    Location {
                        page: u32::try_from(page).map_err(|_| "Too many streaming pages")?,
                        vertices: used as u16,
                        quads: (used + quad_start) as u16,
                        vertex_stride: vertex_stride as u8,
                        baked,
                        world: world.as_ref().map(|w| [w.origin, w.center, w.extent]),
                    },
                );
                lengths.insert(format!("editable_{entity}_{chunk_index}"), length);
                used += length;
            }
        }
        if let Some(track) = track {
            let packing_goal = if locations.values().all(|l| l.baked) {
                pool_pages.saturating_sub(1).max(2)
            } else {
                pool_pages
            };
            if scene.ordered_draw.is_none() {
                pack_visible_together(
                    &mut bundle.bytes,
                    &mut locations,
                    &lengths,
                    &track,
                    scene.visibility_track.as_ref().unwrap(),
                    first_page,
                    packing_goal,
                );
            }
            used = PAGE_BYTES;
            let mut sets: Vec<BTreeMap<u32, ()>> = track
                .bits
                .iter()
                .map(|bits| {
                    track
                        .chunks
                        .iter()
                        .filter_map(|(key, (word, ids))| {
                            bits.range(word * 32..word * 32 + ids.len())
                                .next()
                                .map(|_| (locations[key].page, ()))
                        })
                        .collect()
                })
                .collect();
            if let Some(program) = &scene.ordered_draw {
                let resolved = crate::ordered_draw::resolve(scene, program)?;
                let cooked = crate::ordered_draw_cook::cook(
                    scene,
                    program,
                    &resolved,
                    bundle.page_count(),
                    pool_pages,
                    &ids,
                    palette,
                )?;
                bundle.bytes.extend_from_slice(&cooked.bytes);
                sets = cooked.sets.clone();
                ordered = Some(cooked);
                // Query copies are not render residency. Keep the configured
                // budget until mixed actors' demand is accounted for as well.
                all_residency_baked = false;
            }
            let peak = sets.iter().map(|s| s.len()).max().unwrap_or(0);
            let (sample, transition) = scene
                .visibility_track
                .as_ref()
                .unwrap()
                .residency_pairs()
                .into_iter()
                .map(|(i, next)| {
                    let union: BTreeSet<_> = sets[i].keys().chain(sets[next].keys()).collect();
                    (i, union.len())
                })
                .chain(sets.iter().enumerate().map(|(i, s)| (i, s.len())))
                .max_by_key(|(_, n)| *n)
                .unwrap_or((0, 0));
            if transition > pool_pages {
                return Err(format!(
                    "Baked visibility requires {transition} streaming pages at transition {sample} (peak view {peak}); configured pool has {pool_pages}. Repack geometry or reserve the measured working set."
                ));
            }
            exact_pool = exact_pool.max(transition);
            // These sets describe the tracked subset, not the entire bank.
            // The runtime still collects untracked actors by their bounds and
            // merges their requests into the same working set. Disabling this
            // table for a mixed bank needlessly rescans every tracked chunk for
            // readiness/prefetch, even when the untracked actor is inactive.
            // Keep the full configured pool for mixed banks (see below).
            page_track = Some(PageTrack::cook(&sets)?);
            bundle.previews.push(crate::baked_preview::Bake {
                version: if ordered.is_some() { 3 } else { 2 },
                signature: crate::baked_preview::signature(scene),
                pool_pages,
                sample_pages: ordered
                    .as_ref()
                    .map(|_| sets.iter().map(|s| s.keys().copied().collect()).collect()),
                chunks: preview_chunks
                    .into_iter()
                    .map(|(key, mut c)| {
                        c.page = locations[&key].page;
                        c
                    })
                    .collect(),
                samples: track.bits.clone(),
                edges: scene.visibility_track.as_ref().unwrap().residency_pairs(),
            });
            eprintln!(
                "Baked visibility: {} samples, peak {peak} pages/view, {transition} pages/transition, {} mask bytes",
                sets.len(),
                track.words * 8
            );
        }
        bundle.chunks.push(locations);
        bundle.page_tracks.push(page_track);
        bundle.ordered.push(ordered);
    }
    // For an entirely baked working set the configured pool is an upper bound.
    // Reserve its verified peak, including every view and continuous edge. Cuts
    // replace the old set atomically. Untracked geometry keeps the full budget.
    if all_residency_baked {
        bundle.pool_pages = pool_pages.min(exact_pool);
    }
    for preview in &mut bundle.previews {
        preview.pool_pages = bundle.pool_pages;
        preview.validate()?;
    }
    Ok(bundle)
}

// Group co-visible chunks instead of following asset import order. Select the
// layout by its measured peak working set, retaining the baseline if better.
// The output is deterministic and the exact resulting budget
// is checked above; this is an Epok packer, not the original game's packer.
fn pack_visible_together(
    bytes: &mut Vec<u8>,
    locations: &mut BTreeMap<String, Location>,
    lengths: &BTreeMap<String, usize>,
    track: &crate::visibility_track::Cooked,
    source: &crate::visibility_track::Track,
    first_page: usize,
    pool_pages: usize,
) {
    let pairs = source.residency_pairs();
    let view_count = track.bits.len() + pairs.len();
    let mut uses: Vec<Vec<usize>> = (0..track.bits.len()).map(|i| vec![i]).collect();
    for (p, &(a, b)) in pairs.iter().enumerate() {
        uses[a].push(track.bits.len() + p);
        uses[b].push(track.bits.len() + p);
    }
    let jobs: Vec<_> = locations
        .keys()
        .map(|key| {
            let mut views = BTreeSet::new();
            if let Some((word, ids)) = track.chunks.get(key) {
                for (i, bits) in track.bits.iter().enumerate() {
                    if bits
                        .range(word * 32..word * 32 + ids.len())
                        .next()
                        .is_some()
                    {
                        views.extend(uses[i].iter().copied());
                    }
                }
            }
            (key.clone(), views)
        })
        .collect();
    let score = |plan: &BTreeMap<String, (usize, usize)>| {
        let mut pages = vec![BTreeSet::new(); view_count];
        for (key, views) in &jobs {
            for &v in views {
                pages[v].insert(plan[key].0);
            }
        }
        (
            pages.iter().map(BTreeSet::len).max().unwrap_or(0),
            pages.iter().map(|p| p.len() * p.len()).sum::<usize>(),
            plan.values().map(|p| p.0).max().unwrap_or(0) + 1,
        )
    };
    // Keep the original layout as a candidate: optimization may never worsen
    // the measured maximum working set merely to reduce archive size.
    let mut plan: BTreeMap<_, _> = locations
        .iter()
        .map(|(k, v)| {
            (
                k.clone(),
                (v.page as usize - first_page, v.vertices as usize),
            )
        })
        .collect();
    let mut best_score = score(&plan);
    let lower_bound = (0..view_count)
        .map(|v| {
            jobs.iter()
                .filter(|(_, views)| views.contains(&v))
                .map(|(k, _)| lengths[k])
                .sum::<usize>()
                .div_ceil(PAGE_BYTES)
        })
        .max()
        .unwrap_or(0);
    let masks: Vec<Vec<u64>> = jobs
        .iter()
        .map(|(_, views)| {
            let mut m = vec![0u64; view_count.div_ceil(64)];
            for v in views {
                m[v / 64] |= 1u64 << (v % 64);
            }
            m
        })
        .collect();
    // Opt-in diagnostics for investigating page layout without rebuilding the
    // entire game. Contains only chunk lengths, visibility and page assignments.
    if let Some(path) = std::env::var_os("EPOK_STREAM_PACK_AUDIT") {
        let audit = serde_json::json!({"page_bytes": PAGE_BYTES, "view_count":view_count,
            "sample_count":track.bits.len(), "pairs":pairs,
            "jobs":jobs.iter().map(|(key,views)|serde_json::json!({
                "key":key,"bytes":(lengths[key]+3)&!3,"views":views,"page":plan[key].0
            })).collect::<Vec<_>>()});
        if let Err(error) = std::fs::write(&path, serde_json::to_vec(&audit).unwrap()) {
            eprintln!("Cannot write streaming pack audit: {error}");
        }
    }
    // Distinct graph views are constraints, not a frequency distribution.
    // Refine with a bounded, deterministic solver and still score its complete
    // output against every original sample/edge before accepting it.
    let refined = crate::visibility_packing::refine(
        &jobs
            .iter()
            .map(|(k, _)| (lengths[k] + 3) & !3)
            .collect::<Vec<_>>(),
        &jobs
            .iter()
            .map(|(_, v)| v.iter().copied().collect())
            .collect::<Vec<_>>(),
        &jobs.iter().map(|(k, _)| plan[k].0).collect::<Vec<_>>(),
        PAGE_BYTES,
        pool_pages,
    );
    let mut offsets = BTreeMap::<usize, usize>::new();
    let mut bins = BTreeMap::new();
    let mut candidate = BTreeMap::new();
    for ((key, _), page) in jobs.iter().zip(refined) {
        let next = bins.len();
        let bin = *bins.entry(page).or_insert(next);
        let offset = offsets.entry(bin).or_default();
        candidate.insert(key.clone(), (bin, *offset));
        *offset += (lengths[key] + 3) & !3;
    }
    let candidate_score = score(&candidate);
    if candidate_score < best_score {
        best_score = candidate_score;
        plan = candidate;
    }
    // Grow one page at a time by temporal overlap. Filling arbitrary existing
    // bins spreads a long-lived chunk's views over almost the entire archive.
    for strategy in 0..if best_score.0 > pool_pages { 4 } else { 0 } {
        let mut order: Vec<usize> = (0..jobs.len()).collect();
        order.sort_by_key(|&i| match strategy {
            1 => (
                usize::MAX - jobs[i].1.len(),
                *jobs[i].1.first().unwrap_or(&usize::MAX),
                i,
            ),
            2 => (
                jobs[i].1.len(),
                *jobs[i].1.first().unwrap_or(&usize::MAX),
                i,
            ),
            3 => (
                usize::MAX - lengths[&jobs[i].0],
                *jobs[i].1.first().unwrap_or(&usize::MAX),
                i,
            ),
            _ => (
                *jobs[i].1.first().unwrap_or(&usize::MAX),
                usize::MAX - jobs[i].1.len(),
                i,
            ),
        });
        let mut remaining = vec![true; jobs.len()];
        let mut candidate = BTreeMap::new();
        let mut page = 0;
        for &seed in &order {
            if !remaining[seed] {
                continue;
            }
            let mut used = 0;
            let mut mask = vec![0u64; view_count.div_ceil(64)];
            let mut next = Some(seed);
            while let Some(i) = next {
                let offset = (used + 3) & !3;
                used = offset + lengths[&jobs[i].0];
                remaining[i] = false;
                candidate.insert(jobs[i].0.clone(), (page, offset));
                for (a, b) in mask.iter_mut().zip(&masks[i]) {
                    *a |= *b;
                }
                next = order
                    .iter()
                    .copied()
                    .filter(|&j| {
                        remaining[j] && ((used + 3) & !3) + lengths[&jobs[j].0] <= PAGE_BYTES
                    })
                    .min_by_key(|&j| {
                        let intersection = mask
                            .iter()
                            .zip(&masks[j])
                            .map(|(a, b)| (a & b).count_ones() as u64)
                            .sum::<u64>();
                        let union = mask
                            .iter()
                            .zip(&masks[j])
                            .map(|(a, b)| (a | b).count_ones() as u64)
                            .sum::<u64>();
                        (
                            (union - intersection) * 1_000_000 / (union + 1),
                            usize::MAX - lengths[&jobs[j].0],
                            j,
                        )
                    });
            }
            page += 1;
        }
        let candidate_score = score(&candidate);
        if candidate_score < best_score {
            best_score = candidate_score;
            plan = candidate;
        }
    }
    // Refine balanced pages with deterministic moves/swaps. Count page
    // membership per sample, so evaluating a move touches only its two view
    // sets, never geometry bytes. The convex objective favors worst views.
    if jobs.len() > 1 && best_score.0 > pool_pages {
        let page_count = best_score.2 + 1;
        let mut assigned: Vec<_> = jobs.iter().map(|(k, _)| plan[k].0).collect();
        let mut loads = vec![0usize; page_count];
        let mut members = vec![vec![0u16; view_count]; page_count];
        for (i, (key, views)) in jobs.iter().enumerate() {
            loads[assigned[i]] += (lengths[key] + 3) & !3;
            for &v in views {
                members[assigned[i]][v] += 1;
            }
        }
        let mut counts: Vec<usize> = (0..view_count)
            .map(|v| members.iter().filter(|m| m[v] > 0).count())
            .collect();
        let powers: Vec<i64> = (0..=page_count).map(|v| (v as i64).pow(8)).collect();
        let mut rng = 0x12345678u32;
        let mut random = || {
            rng ^= rng << 13;
            rng ^= rng >> 17;
            rng ^= rng << 5;
            rng as usize
        };
        let mut changes = Vec::new();
        for _ in 0..250_000 {
            let a = random() % jobs.len();
            let b = random() % jobs.len();
            let pa = assigned[a];
            let swap = random() % 4 != 0;
            let pb = if swap {
                assigned[b]
            } else {
                random() % page_count
            };
            if pa == pb {
                continue;
            }
            let la = (lengths[&jobs[a].0] + 3) & !3;
            let lb = if swap {
                (lengths[&jobs[b].0] + 3) & !3
            } else {
                0
            };
            if loads[pa] - la + lb > PAGE_BYTES || loads[pb] - lb + la > PAGE_BYTES {
                continue;
            }
            changes.clear();
            let mut delta = 0i64;
            let empty = BTreeSet::new();
            let bv = if swap { &jobs[b].1 } else { &empty };
            for &v in jobs[a].1.union(bv) {
                let da = i32::from(bv.contains(&v)) - i32::from(jobs[a].1.contains(&v));
                if da == 0 {
                    continue;
                }
                let ma = members[pa][v] as i32 + da;
                let mb = members[pb][v] as i32 - da;
                let n = (counts[v] as i32 + i32::from(ma > 0) + i32::from(mb > 0)
                    - i32::from(members[pa][v] > 0)
                    - i32::from(members[pb][v] > 0)) as usize;
                delta += powers[n] - powers[counts[v]];
                changes.push((v, ma as u16, mb as u16, n));
            }
            if delta > 0 {
                continue;
            }
            for &(v, ma, mb, n) in &changes {
                members[pa][v] = ma;
                members[pb][v] = mb;
                counts[v] = n;
            }
            loads[pa] = loads[pa] - la + lb;
            loads[pb] = loads[pb] - lb + la;
            assigned[a] = pb;
            if swap {
                assigned[b] = pa;
            }
        }
        let mut offsets = vec![0; page_count];
        let mut candidate = BTreeMap::new();
        for (i, (key, _)) in jobs.iter().enumerate() {
            let p = assigned[i];
            candidate.insert(key.clone(), (p, offsets[p]));
            offsets[p] += (lengths[key] + 3) & !3;
        }
        let candidate_score = score(&candidate);
        if candidate_score < best_score {
            best_score = candidate_score;
            plan = candidate;
        }
    }
    eprintln!(
        "Visibility packing: {} chunks, lower bound {lower_bound} pages/transition, selected {} pages/transition in {} archive pages",
        jobs.len(),
        best_score.0,
        best_score.2
    );
    let mut packed = vec![0; best_score.2 * PAGE_BYTES];
    for (key, (bin, offset)) in plan {
        let location = locations.get_mut(&key).unwrap();
        let length = lengths[&key];
        let from = location.page as usize * PAGE_BYTES + usize::from(location.vertices);
        packed[bin * PAGE_BYTES + offset..bin * PAGE_BYTES + offset + length]
            .copy_from_slice(&bytes[from..from + length]);
        let quad_delta = location.quads - location.vertices;
        *location = Location {
            page: (first_page + bin) as u32,
            vertices: offset as u16,
            quads: offset as u16 + quad_delta,
            vertex_stride: location.vertex_stride,
            baked: location.baked,
            world: location.world,
        };
    }
    bytes.truncate(first_page * PAGE_BYTES);
    bytes.extend(packed);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ordered_draw_cook_rewrites_actual_page_sets_without_losing_editor_selection() {
        let scene = crate::ordered_draw::tests::fixture();
        let bundle = compile(std::slice::from_ref(&scene), 3).unwrap();
        let ordered = bundle.ordered[0].as_ref().unwrap();
        let preview = &bundle.previews[0];
        preview.validate().unwrap();
        assert_eq!(preview.version, 3);
        assert_eq!(preview.selection(Some(0))[&scene.actors[1].id].len(), 2);
        assert_eq!(preview.selection(Some(1))[&scene.actors[1].id].len(), 1);
        for (i, set) in ordered.sets.iter().enumerate() {
            assert_eq!(preview.pages(i), set.keys().copied().collect());
        }
        let header = bundle
            .rewrite(0, &crate::project::scene_header(&scene, &[]).unwrap())
            .unwrap();
        assert!(header.contains("OrderedDrawData ordered_data="));
        assert!(header.contains(",track_page_sets,track_page_samples,&ordered_data};"));
        assert!(
            crate::scene_bank::header_with_streaming(
                std::slice::from_ref(&scene),
                &[],
                Some(&bundle)
            )
            .unwrap()
            .contains("ordered_draw_enabled=true")
        );
        assert!(
            crate::scene_bank::header_with_streaming(&[scene], &[], None)
                .unwrap_err()
                .contains("streaming pages")
        );
    }
    #[test]
    #[ignore = "Requires a separately audited external source fixture; creates a new output directory"]
    fn ordered_draw_source_fixture() {
        let root = std::path::PathBuf::from(
            std::env::var_os("EPOK_ORDERED_PROJECT").expect("EPOK_ORDERED_PROJECT"),
        );
        let scene_path = std::path::PathBuf::from(
            std::env::var_os("EPOK_ORDERED_SCENE").expect("EPOK_ORDERED_SCENE"),
        );
        let program_path = std::path::PathBuf::from(
            std::env::var_os("EPOK_ORDERED_PROGRAM").expect("EPOK_ORDERED_PROGRAM"),
        );
        let output = std::path::PathBuf::from(
            std::env::var_os("EPOK_ORDERED_OUTPUT").expect("EPOK_ORDERED_OUTPUT"),
        );
        assert!(!output.exists(), "Output must be new");
        let mut scene = Scene::load_unresolved(&scene_path).unwrap();
        let assets = crate::assets::scan(&root, &mut Default::default());
        crate::mesh::resolve(&mut scene, &assets).unwrap();
        crate::texture::resolve(&mut scene, &assets).unwrap();
        scene.ordered_draw =
            Some(serde_json::from_slice(&std::fs::read(program_path).unwrap()).unwrap());
        scene.validate().unwrap();
        let bundle = compile(std::slice::from_ref(&scene), 7).unwrap();
        let ordered = bundle.ordered[0].as_ref().unwrap();
        let header = bundle
            .rewrite(0, &crate::project::scene_header(&scene, &[]).unwrap())
            .unwrap();
        let lists: Vec<Vec<u32>> = scene
            .ordered_draw
            .as_ref()
            .unwrap()
            .samples
            .iter()
            .map(|list| {
                list.iter()
                    .map(|token| ordered.primitive_tokens[(token >> 2) as usize] | (token & 3))
                    .collect()
            })
            .collect();
        let report = serde_json::json!({"scope":"Offline native packet/order and projection-block preparation; not a runtime fidelity claim", "abi":crate::ordered_draw_cook::ABI,
            "archive_bytes":bundle.bytes.len(),"ordered_payload_bytes":ordered.bytes.len(),"pool_pages":bundle.pool_pages,
            "max_packets":ordered.max_packets,"sample_count":ordered.sample_blocks.len(),
            "blocks":ordered.blocks.iter().map(|b|serde_json::json!({"page":b.page,"offset":b.offset,"first":b.first})).collect::<Vec<_>>(),
            "sample_blocks":ordered.sample_blocks,"primitive_tokens":ordered.primitive_tokens,
            "projections":ordered.projections.iter().map(|b|serde_json::json!({"first":b.first,"count":b.count,"center":b.center,"compact":b.compact,"low":b.low,"high":b.high})).collect::<Vec<_>>(),
            "samples":lists,"page_sets":ordered.sets.iter().map(|s|s.keys().collect::<Vec<_>>()).collect::<Vec<_>>(),
            "primitive_count":ordered.primitive_tokens.len(),"archive_sha256":crate::assets::hash(&bundle.bytes),
            "header_sha256":crate::assets::hash(header.as_bytes())});
        std::fs::create_dir_all(&output).unwrap();
        bundle.write(&output).unwrap();
        crate::project::write_changed(
            &output.join("ordered-cook.json"),
            &serde_json::to_vec(&report).unwrap(),
        )
        .unwrap();
        crate::project::write_changed(&output.join("scene.hh"), header.as_bytes()).unwrap();
        println!(
            "Prepared {} primitives / {} samples / {} blocks; {} archive bytes / {} ordered bytes / {} pool pages",
            ordered.primitive_tokens.len(),
            ordered.sample_blocks.len(),
            ordered.blocks.len(),
            bundle.bytes.len(),
            ordered.bytes.len(),
            bundle.pool_pages
        );
        if let Some(build) = std::env::var_os("EPOK_ORDERED_STAGE") {
            let build = std::path::PathBuf::from(build);
            assert!(!build.exists(), "Runtime stage directory must be new");
            std::fs::create_dir_all(&build).unwrap();
            crate::project::stage_into(&root, &scene, &build).unwrap();
            println!("Staged full PSX runtime at {}", build.display());
        }
    }
    #[test]
    fn page_track_dictionary_preserves_empty_shared_and_large_page_ids() {
        let sets = [vec![3, 90000], vec![], vec![3, 90000], vec![4], vec![]]
            .map(|pages| pages.into_iter().map(|p| (p, ())).collect());
        let cooked = PageTrack::cook(&sets).unwrap();
        assert_eq!(cooked.sets.len(), 3);
        assert_eq!(cooked.samples, vec![0, 1, 0, 2, 1]);
        for (i, expected) in sets.iter().enumerate() {
            assert_eq!(
                cooked.sets[usize::from(cooked.samples[i])],
                expected.keys().copied().collect::<Vec<_>>()
            );
        }
        assert!(cooked.header().contains("{track_page_ids+2,0}"));
    }
    #[test]
    fn static_world_cooks_world_space_chunks_with_shared_corners() {
        // Two scaled actors whose cubes touch: with static_world, both cook
        // the shared corners to the same world Q12 points.
        let mut actors: Vec<_> = (0..2).map(|_| self::scene(1).actors.remove(0)).collect();
        for e in &mut actors {
            let doc = std::sync::Arc::make_mut(
                e.editable_mesh.as_mut().unwrap().document.as_mut().unwrap(),
            );
            doc.materials[0].material.uv_scroll = [0., 0.];
            e.scale = [10.; 3];
        }
        actors[0].position = [122.25, 18.25, 1023.];
        actors[1].position = [132.25, 18.25, 1023.];
        let faces: Vec<_> = actors
            .iter()
            .flat_map(|e| {
                e.editable_mesh
                    .as_ref()
                    .unwrap()
                    .document
                    .as_ref()
                    .unwrap()
                    .faces
                    .iter()
                    .map(|f| f.id)
                    .collect::<Vec<_>>()
            })
            .collect();
        let camera = Scene::default().actors[0].clone();
        let mut scene = Scene {
            actors: actors.clone(),
            ..Default::default()
        };
        scene.visibility_track = Some(crate::visibility_track::Track {
            path: Vec::new(),
            camera: camera.id,
            sample_ms: 100,
            cull_backfaces: false,
            static_world: true,
            navigation: None,
            samples: vec![crate::visibility_track::Sample {
                position: [0.; 3],
                rotation: [0.; 3],
                fov: 90.,
                visible: (0..faces.len() as u16).collect(),
                cut: true,
            }],
            faces,
        });
        scene.actors.push(camera);
        let bundle = compile(std::slice::from_ref(&scene), 7).unwrap();
        let header = bundle
            .rewrite(0, &crate::project::scene_header(&scene, &[]).unwrap())
            .unwrap();
        let mut corners = Vec::new();
        for (entity, e) in actors.iter().enumerate() {
            let doc = e.editable_mesh.as_ref().unwrap().document.as_ref().unwrap();
            let transform = scene.world_matrix(entity);
            let expected: BTreeSet<[i32; 3]> = doc
                .faces
                .iter()
                .flat_map(|f| doc.points(f))
                .map(|p| crate::mesh_compile::world_q12(&transform, p).unwrap())
                .collect();
            let qs = crate::lighting::quads(e);
            let mut cooked = BTreeSet::new();
            for (i, chunk) in crate::mesh_compile::chunks(&qs).unwrap().iter().enumerate() {
                let key = format!("editable_{entity}_{i}");
                let location = &bundle.chunks[0][&key];
                let [origin, _, _] = location.world.expect("static world chunk");
                assert!(origin.iter().all(|v| v % 16 == 0));
                let line = header
                    .lines()
                    .find(|l| l.starts_with(&format!("inline constexpr MeshGeometry {key}=")))
                    .unwrap();
                assert!(
                    line.contains(&format!(
                        ",7,{{{},{},{}}},",
                        origin[0], origin[1], origin[2]
                    )),
                    "{line}"
                );
                let base = location.page as usize * PAGE_BYTES + location.vertices as usize;
                for v in 0..chunk.vertices.len() {
                    let point: [i32; 3] = std::array::from_fn(|c| {
                        origin[c]
                            + if location.vertex_stride == 6 {
                                i32::from(i16::from_le_bytes(
                                    bundle.bytes[base + v * 6 + c * 2..][..2]
                                        .try_into()
                                        .unwrap(),
                                ))
                            } else {
                                i32::from_le_bytes(
                                    bundle.bytes[base + v * 12 + c * 4..][..4]
                                        .try_into()
                                        .unwrap(),
                                )
                            }
                    });
                    cooked.insert(point);
                }
            }
            assert_eq!(cooked, expected);
            corners.push(cooked);
        }
        assert_eq!(corners[0].intersection(&corners[1]).count(), 4);
        // Without the opt-in the same scene cooks local chunks, as before.
        scene.visibility_track.as_mut().unwrap().static_world = false;
        assert!(
            compile(std::slice::from_ref(&scene), 7).unwrap().chunks[0]
                .values()
                .all(|l| l.world.is_none())
        );
    }
    #[test]
    fn baked_draw_roundtrips_compact_faces_and_preview_from_the_same_cook() {
        let mut scene = scene(1);
        let mesh = scene.actors[0].editable_mesh.as_mut().unwrap();
        let doc = std::sync::Arc::make_mut(mesh.document.as_mut().unwrap());
        doc.materials[0].material.uv_scroll = [0., 0.];
        let faces = doc.faces.iter().map(|f| f.id).collect();
        let camera = Scene::default().actors[0].clone();
        scene.visibility_track = Some(crate::visibility_track::Track {
            path: Vec::new(),
            camera: camera.id,
            sample_ms: 100,
            cull_backfaces: false,
            static_world: false,
            navigation: None,
            faces,
            samples: vec![crate::visibility_track::Sample {
                position: [0.; 3],
                rotation: [0.; 3],
                fov: 90.,
                visible: vec![0, 1],
                cut: true,
            }],
        });
        scene.actors.push(camera);
        let bundle = compile(std::slice::from_ref(&scene), 7).unwrap();
        assert_eq!(bundle.pool_pages, 2);
        let preview = &bundle.previews[0];
        preview.validate().unwrap();
        let pages = bundle.page_tracks[0].as_ref().unwrap();
        for (sample, bits) in preview.samples.iter().enumerate() {
            let expected = preview
                .chunks
                .iter()
                .filter(|c| {
                    bits.range(c.word * 32..c.word * 32 + c.quads.len())
                        .next()
                        .is_some()
                })
                .map(|c| c.page)
                .collect::<BTreeSet<_>>();
            assert_eq!(
                pages.sets[usize::from(pages.samples[sample])]
                    .iter()
                    .copied()
                    .collect::<BTreeSet<_>>(),
                expected
            );
        }
        assert_eq!(preview.signature, crate::baked_preview::signature(&scene));
        let qs = crate::lighting::quads(&scene.actors[0]);
        let parts = crate::mesh_compile::chunks(&qs).unwrap();
        for (i, chunk) in parts.iter().enumerate() {
            let location = &bundle.chunks[0][&format!("editable_0_{i}")];
            assert!(location.baked);
            let base = location.page as usize * PAGE_BYTES + location.quads as usize;
            for (q, (indices, index)) in chunk.faces.iter().enumerate() {
                let offset = base + q * crate::baked_draw::FACE_BYTES;
                assert_eq!(
                    &bundle.bytes[offset..offset + 32],
                    &crate::baked_draw::encode(&qs[*index], *indices, &scene.actors[0], None)
                );
            }
        }
        let header = bundle
            .rewrite(0, &crate::project::scene_header(&scene, &[]).unwrap())
            .unwrap();
        assert!(
            header
                .lines()
                .any(|l| l.contains("MeshGeometry editable_0_") && l.contains(",3,"))
        );
        assert!(header.lines().any(|l| {
            l.starts_with("inline constexpr VisibilityTrackData track_data=")
                && l.ends_with(",track_page_sets,track_page_samples};")
        }));
        let root = std::env::temp_dir().join(format!("epok-bake-{}", uuid::Uuid::new_v4()));
        let build = root.join(".epok/build");
        std::fs::create_dir_all(&build).unwrap();
        bundle.write(&build).unwrap();
        let mut state = crate::baked_preview::State {
            enabled: true,
            ..Default::default()
        };
        state.load(&root, &scene);
        assert!(state.error.is_none());
        assert_eq!(state.selection[&scene.actors[0].id].len(), 2);
        scene.actors[0].position[0] += 1.;
        state.load(&root, &scene);
        assert!(state.error.is_some());
        assert!(state.filter().is_none());
        clear(&build).unwrap();
        assert!(!build.join(crate::baked_preview::FILE).exists());
        std::fs::remove_dir_all(root).unwrap();
        scene.fog.enabled = true;
        assert!(
            compile(std::slice::from_ref(&scene), 7).unwrap().chunks[0]
                .values()
                .all(|l| !l.baked)
        );
        let banks = compile(&[self::scene(1), scene.clone()], 7).unwrap();
        assert!(banks.page_tracks[0].is_none());
        assert!(
            banks.page_tracks[1]
                .as_ref()
                .unwrap()
                .sets
                .iter()
                .flatten()
                .all(|&page| page > 0)
        );
        scene.actors.push(self::scene(1).actors.remove(0));
        let mixed = compile(std::slice::from_ref(&scene), 7).unwrap();
        assert!(mixed.page_tracks[0].is_some());
        assert_eq!(mixed.pool_pages, 7);
        assert!(
            mixed
                .rewrite(0, &crate::project::scene_header(&scene, &[]).unwrap())
                .unwrap()
                .contains("track_page_sets")
        );
    }
    #[test]
    fn mixed_banks_keep_tracked_page_sets_and_untracked_geometry() {
        let mut scene = scene(1);
        let doc = std::sync::Arc::make_mut(
            scene.actors[0]
                .editable_mesh
                .as_mut()
                .unwrap()
                .document
                .as_mut()
                .unwrap(),
        );
        doc.materials[0].material.uv_scroll = [0.; 2];
        let faces = doc.faces.iter().map(|f| f.id).collect();
        let camera = Scene::default().actors[0].clone();
        scene.visibility_track = Some(crate::visibility_track::Track {
            path: Vec::new(),
            camera: camera.id,
            sample_ms: 100,
            cull_backfaces: false,
            static_world: false,
            navigation: None,
            faces,
            samples: vec![
                crate::visibility_track::Sample {
                    position: [0.; 3],
                    rotation: [0.; 3],
                    fov: 90.,
                    visible: vec![0, 1],
                    cut: true,
                },
                crate::visibility_track::Sample {
                    position: [0.; 3],
                    rotation: [0.; 3],
                    fov: 90.,
                    visible: vec![],
                    cut: false,
                },
            ],
        });
        scene.actors.push(camera);
        scene.actors.push(self::scene(2).actors.remove(0));
        let mut previous = None;
        for active in [true, false, true] {
            scene.actors[2].active = active;
            let bundle = compile(std::slice::from_ref(&scene), 7).unwrap();
            assert_eq!(
                bundle.pool_pages, 7,
                "Mixed banks must not shrink to the tracked subset's budget"
            );
            let track = bundle.page_tracks[0].as_ref().unwrap();
            let preview = &bundle.previews[0];
            for (sample, bits) in preview.samples.iter().enumerate() {
                let expected = preview
                    .chunks
                    .iter()
                    .filter(|c| {
                        bits.range(c.word * 32..c.word * 32 + c.quads.len())
                            .next()
                            .is_some()
                    })
                    .map(|c| c.page)
                    .collect::<BTreeSet<_>>();
                assert_eq!(
                    track.sets[usize::from(track.samples[sample])]
                        .iter()
                        .copied()
                        .collect::<BTreeSet<_>>(),
                    expected
                );
            }
            assert!(
                track.sets[usize::from(track.samples[1])].is_empty(),
                "An empty tracked view must not request untracked pages"
            );
            let untracked = bundle.chunks[0]
                .iter()
                .filter(|(key, _)| key.starts_with("editable_2_"))
                .collect::<Vec<_>>();
            assert!(!untracked.is_empty());
            assert!(untracked.iter().all(|(_, location)| !location.baked));
            let header = bundle
                .rewrite(0, &crate::project::scene_header(&scene, &[]).unwrap())
                .unwrap();
            assert!(header.contains("track_page_sets,track_page_samples};"));
            assert!(header.contains("MeshGeometry editable_2_"));
            let state = (bundle.bytes, track.header());
            if let Some(ref previous) = previous {
                assert_eq!(
                    &state, previous,
                    "Activation must not erase payloads or invalidate the tracked page bake"
                );
            }
            previous = Some(state);
        }
    }
    #[test]
    fn visibility_packer_groups_disjoint_views_without_corrupting_payloads() {
        let mut bytes = vec![0; 4 * PAGE_BYTES];
        let mut locations = BTreeMap::new();
        let mut lengths = BTreeMap::new();
        let mut chunks = BTreeMap::new();
        let mut bits = vec![BTreeSet::new(); 4];
        for i in 0..32 {
            let key = format!("chunk_{i:02}");
            let page = i / 8;
            let offset = i % 8 * 8192;
            bytes[page * PAGE_BYTES + offset..page * PAGE_BYTES + offset + 8192].fill(i as u8);
            locations.insert(
                key.clone(),
                Location {
                    page: page as u32,
                    vertices: offset as u16,
                    quads: (offset + 100) as u16,
                    vertex_stride: 6,
                    baked: false,
                    world: None,
                },
            );
            lengths.insert(key.clone(), 8192);
            chunks.insert(key, (i, vec![uuid::Uuid::from_u128(i as u128 + 1)]));
            bits[i % 4].insert(i * 32);
        }
        let cooked = crate::visibility_track::Cooked {
            chunks,
            bits,
            words: 32,
        };
        let source = crate::visibility_track::Track {
            path: Vec::new(),
            navigation: None,
            camera: uuid::Uuid::nil(),
            sample_ms: 100,
            cull_backfaces: true,
            static_world: false,
            faces: vec![],
            samples: (0..4)
                .map(|_| crate::visibility_track::Sample {
                    position: [0.; 3],
                    rotation: [0.; 3],
                    fov: 90.,
                    visible: vec![],
                    cut: true,
                })
                .collect(),
        };
        let mut other_bytes = bytes.clone();
        let mut other_locations = locations.clone();
        pack_visible_together(&mut bytes, &mut locations, &lengths, &cooked, &source, 0, 2);
        pack_visible_together(
            &mut other_bytes,
            &mut other_locations,
            &lengths,
            &cooked,
            &source,
            0,
            2,
        );
        assert_eq!(bytes, other_bytes);
        assert_eq!(bytes.len(), 4 * PAGE_BYTES);
        for view in 0..4 {
            let pages: BTreeSet<_> = (view..32)
                .step_by(4)
                .map(|i| locations[&format!("chunk_{i:02}")].page)
                .collect();
            assert_eq!(pages.len(), 1);
        }
        for i in 0..32 {
            let loc = &locations[&format!("chunk_{i:02}")];
            let start = loc.page as usize * PAGE_BYTES + loc.vertices as usize;
            assert_eq!(loc.quads - loc.vertices, 100);
            assert!(bytes[start..start + 8192].iter().all(|&v| v == i as u8));
        }
    }
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
    fn texture_upload_pages_preserve_geometry_and_remove_resident_pixels() {
        let mut scene = scene(1);
        for (i, (width, height, four)) in [(256u16, 256u16, false), (7, 3, true), (64, 32, false)]
            .into_iter()
            .enumerate()
        {
            let id = uuid::Uuid::from_u128(i as u128 + 1);
            let mut actor = crate::scene::Actor::cube(format!("Texture {i}"));
            actor.material.texture = Some(id);
            scene.actors.push(actor);
            let mut palette = vec![0u16; 256];
            palette[1] = 0x8123;
            if !four {
                palette[32] = 0x8543;
            }
            scene.textures.insert(
                id,
                std::sync::Arc::new(crate::texture::Data {
                    width,
                    height,
                    rgba: vec![],
                    palette,
                    words: vec![0x0101; usize::from(width.div_ceil(4) * 2) * usize::from(height)],
                }),
            );
        }
        let mut bundle = compile(std::slice::from_ref(&scene), 2).unwrap();
        let geometry = bundle.bytes.clone();
        let geometry_pages = geometry.len() / PAGE_BYTES;
        assert!(
            bundle
                .globals()
                .contains(&format!("stream_geometry_page_count={geometry_pages};"))
        );
        let mut duplicate = compile(std::slice::from_ref(&scene), 2).unwrap();
        bundle.append_texture_pixels(&scene).unwrap();
        duplicate.append_texture_pixels(&scene).unwrap();
        assert_eq!(bundle.bytes, duplicate.bytes);
        assert_eq!(&bundle.bytes[..geometry.len()], geometry.as_slice());
        assert_eq!(bundle.page_count(), geometry.len() / PAGE_BYTES + 2);
        for id in crate::texture::ids(&scene) {
            let (page, offset) = bundle.texture_pixels(id).unwrap();
            let cooked = crate::texture_cook::cook(&scene.textures[&id]);
            let start = page as usize * PAGE_BYTES + usize::from(offset);
            assert_eq!(offset % 4, 0);
            assert!(usize::from(offset) + cooked.words.len() * 2 <= PAGE_BYTES);
            for (i, word) in cooked.words.iter().enumerate() {
                assert_eq!(u16_at(&bundle.bytes, start + i * 2), *word);
            }
        }
        let header = crate::scene_bank::header_with_streaming(
            std::slice::from_ref(&scene),
            &[],
            Some(&bundle),
        )
        .unwrap();
        assert!(header.contains(&format!("stream_geometry_page_count={geometry_pages};")));
        assert!(!header.contains("uint16_t texture_0_pixels"));
        assert!(header.contains("nullptr,texture_0_palette,8,"));
        assert!(header.contains("nullptr,texture_1_palette,4,"));
        assert!(header.contains("resident_texture_bytes=1056;"));
        let resident = crate::scene_bank::header(std::slice::from_ref(&scene), &[]).unwrap();
        assert!(resident.contains("stream_geometry_page_count=0;"));
        assert!(resident.contains("uint16_t texture_0_pixels"));
        assert!(bundle.append_texture_pixels(&scene).is_err());
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
                        i32::from(u16_at(page, loc.vertices as usize + v * 6 + axis * 2) as i16),
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
    fn wide_vertices_roundtrip_without_subdivision_in_resident_and_streamed_headers() {
        let mut scene = scene(1);
        let mesh = scene.actors[0].editable_mesh.as_mut().unwrap();
        let doc = std::sync::Arc::make_mut(mesh.document.as_mut().unwrap());
        for vertex in &mut doc.vertices {
            vertex[0] *= 64.;
        }
        doc.validate().unwrap();
        let authored_faces = doc.faces.len();
        let qs = crate::lighting::quads(&scene.actors[0]);
        assert_eq!(qs.len(), authored_faces);
        let parts = crate::mesh_compile::chunks(&qs).unwrap();
        assert!(parts.iter().any(|c| c.vertex_stride() == 12));
        let bundle = compile(std::slice::from_ref(&scene), 2).unwrap();
        let resident = crate::mesh_compile::header(&scene.actors[0], 0).unwrap();
        let streamed = bundle.rewrite(0, &resident).unwrap();
        assert!(resident.contains("inline constexpr int32_t editable_0_"));
        assert!(!streamed.contains("_vertices[][3]"));
        assert!(streamed.contains("={MeshVertices{nullptr,12},"));
        for (i, chunk) in parts.iter().enumerate() {
            let loc = &bundle.chunks[0][&format!("editable_0_{i}")];
            assert_eq!(usize::from(loc.vertex_stride), chunk.vertex_stride());
            let base = loc.page as usize * PAGE_BYTES + usize::from(loc.vertices);
            for (v, xyz) in chunk.vertices.iter().enumerate() {
                for (axis, value) in xyz.iter().enumerate() {
                    let actual = if loc.vertex_stride == 12 {
                        i32_at(&bundle.bytes, base + v * 12 + axis * 4)
                    } else {
                        i32::from(u16_at(&bundle.bytes, base + v * 6 + axis * 2) as i16)
                    };
                    assert_eq!(actual, *value);
                }
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
    fn vertex_colors_use_one_palette_across_streamed_banks_and_resident_references() {
        let colors = [[185, 128, 0], [2, 255, 17], [128, 64, 32], [19, 12, 11]];
        let mut first = scene(1);
        {
            let mesh = first.actors[0].editable_mesh.as_mut().unwrap();
            let doc = std::sync::Arc::make_mut(mesh.document.as_mut().unwrap());
            for face in &mut doc.faces {
                face.vertex_colors = Some(colors);
            }
        }
        let mut second = first.clone();
        second.name = "Second palette bank".into();
        let mesh = second.actors[0].editable_mesh.as_mut().unwrap();
        let doc = std::sync::Arc::make_mut(mesh.document.as_mut().unwrap());
        for face in &mut doc.faces {
            face.vertex_colors = Some([[1, 0, 0]; 4]);
        }
        let banks = [first, second];
        let mut extra = scene(1);
        let mesh = extra.actors[0].editable_mesh.as_mut().unwrap();
        let doc = std::sync::Arc::make_mut(mesh.document.as_mut().unwrap());
        for face in &mut doc.faces {
            face.vertex_colors = Some([[0, 0, 0]; 4]);
        }
        let shared =
            crate::scene_bank::resources(&[banks[0].clone(), banks[1].clone(), extra.clone()]);
        let palette = crate::vertex_colors::Palette::scene(&shared).unwrap();
        let bundle = compile_with_palette(&banks, 2, 4096, &palette).unwrap();
        for (bank, scene) in banks.iter().enumerate() {
            let qs = crate::lighting::quads(&scene.actors[0]);
            for (i, chunk) in crate::mesh_compile::chunks(&qs).unwrap().iter().enumerate() {
                let loc = &bundle.chunks[bank][&format!("editable_0_{i}")];
                assert!(!loc.baked);
                for (face, (_, index)) in chunk.faces.iter().enumerate() {
                    let offset =
                        loc.page as usize * PAGE_BYTES + loc.quads as usize + face * QUAD_BYTES;
                    assert_eq!(bundle.bytes[offset + 70], 1);
                    for (c, color) in palette
                        .indices(qs[*index].vertex_colors.unwrap())
                        .iter()
                        .enumerate()
                    {
                        assert_eq!(u16_at(&bundle.bytes, offset + 72 + c * 2), *color);
                    }
                }
            }
        }
        let registry = crate::actor_document::tests::registry();
        let header = crate::scene_bank::header_with_templates(
            &banks,
            &[],
            Some(&bundle),
            &[],
            Some(&extra),
            &[],
            &[],
            &registry,
        )
        .unwrap();
        assert!(header.contains(&palette.header()));
        assert_eq!(
            header
                .matches("inline constexpr uint32_t vertex_color_palette[]")
                .count(),
            1
        );
        // A palette generated only from startup/banks would shift these indices.
        let wrong = compile(&banks, 2).unwrap();
        assert!(
            crate::scene_bank::header_with_templates(
                &banks,
                &[],
                Some(&wrong),
                &[],
                Some(&extra),
                &[],
                &[],
                &registry
            )
            .unwrap_err()
            .contains("different vertex colour palettes")
        );
    }

    #[test]
    fn packed_uv_and_global_texture_index_have_native_offsets() {
        let scene = scene(1);
        let q = &crate::lighting::quads(&scene.actors[0])[0];
        let page = crate::texture_cook::Page {
            width: 64,
            height: 32,
            u: 32,
            v: 44,
        };
        let encoded = encode_quad(q, [1, 2, 3, 4], 7, 12, Some(page));
        assert_eq!(encoded[30], 0);
        let mut double_sided = q.clone();
        double_sided.material.double_sided = true;
        let two_sided = encode_quad(&double_sided, [1, 2, 3, 4], 7, 12, Some(page));
        assert_eq!(two_sided[30], 1);
        assert_eq!(two_sided.len(), QUAD_BYTES);
        assert!(
            two_sided
                .iter()
                .zip(&encoded)
                .enumerate()
                .all(|(i, (a, b))| i == 30 || a == b)
        );
        assert_eq!(i32_at(&encoded, 20), 12);
        assert_eq!(encoded[60], 1);
        for i in 0..4 {
            assert_eq!(
                u16_at(&encoded, 62 + i * 2),
                crate::mesh_compile::packed_uv(q.uv[i], page)
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
