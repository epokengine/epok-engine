//! Portable PNG / indexed TIM textures and a framebuffer-safe VRAM layout.
use crate::{
    assets,
    scene::{Material, Scene},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    sync::Arc,
};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlendMode {
    #[default]
    Cutout,
    Average,
    Add,
    Subtract,
    AddQuarter,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Data {
    pub width: u16,
    pub height: u16,
    pub rgba: Vec<u8>,
    pub words: Vec<u16>,
    pub palette: Vec<u16>,
}

pub fn decode(bytes: &[u8]) -> Result<Data, String> {
    if bytes.starts_with(&0x10u32.to_le_bytes()) {
        return decode_tim(bytes);
    }
    let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().map_err(|e| format!("PNG: {e}"))?;
    let (w, h) = (reader.info().width, reader.info().height);
    if w == 0 || h == 0 || w > 256 || h > 256 {
        return Err(
            "Texture dimensions must be 1..256 pixels per axis; split larger images into atlases"
                .into(),
        );
    }
    let mut buffer = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buffer).map_err(|e| e.to_string())?;
    let channels = info.color_type.samples();
    let mut colors = Vec::with_capacity((w * h) as usize);
    for p in buffer[..info.buffer_size()].chunks_exact(channels) {
        let (r, g, b, a) = match info.color_type {
            png::ColorType::Rgba => (p[0], p[1], p[2], p[3]),
            png::ColorType::Rgb => (p[0], p[1], p[2], 255),
            png::ColorType::Grayscale => (p[0], p[0], p[0], 255),
            png::ColorType::GrayscaleAlpha => (p[0], p[0], p[0], p[1]),
            _ => return Err("Unsupported PNG output".into()),
        };
        // STP is set for every visible texel: cutout ignores it, blend primitives use it.
        colors.push(if a < 128 {
            0
        } else {
            0x8000 | u16::from(r >> 3) | (u16::from(g >> 3) << 5) | (u16::from(b >> 3) << 10)
        });
    }
    let mut counts = BTreeMap::new();
    for &c in &colors {
        if c != 0 {
            *counts.entry(c).or_insert(0usize) += 1;
        }
    }
    let mut ranked = counts.into_iter().collect::<Vec<_>>();
    ranked.sort_by_key(|&(c, n)| (std::cmp::Reverse(n), c));
    let mut palette = vec![0];
    palette.extend(ranked.iter().take(255).map(|&(c, _)| c));
    if palette.len() == 1 {
        palette.push(0x8000);
    }
    let mut indices = Vec::with_capacity(colors.len());
    let mut rgba = Vec::with_capacity(colors.len() * 4);
    let mut nearest = BTreeMap::new();
    for c in colors {
        let i = if c == 0 {
            0
        } else {
            *nearest.entry(c).or_insert_with(|| {
                (1..palette.len())
                    .min_by_key(|&i| {
                        (0..3)
                            .map(|k| {
                                let d = ((c >> (k * 5)) & 31) as i32
                                    - ((palette[i] >> (k * 5)) & 31) as i32;
                                d * d
                            })
                            .sum::<i32>()
                    })
                    .unwrap()
            })
        };
        indices.push(i as u8);
        let c = palette[i];
        for k in 0..3 {
            rgba.push((((c >> (k * 5)) & 31) * 255 / 31) as u8);
        }
        rgba.push(if i == 0 { 0 } else { 255 });
    }
    palette.resize(256, 0);
    // PsyQo DMA requires an even number of 16-bit VRAM words; pad each row to four texels.
    let row = (w as usize).div_ceil(4) * 2;
    let mut words = vec![0; row * h as usize];
    for y in 0..h as usize {
        for x in 0..w as usize {
            words[y * row + x / 2] |= u16::from(indices[y * w as usize + x]) << ((x % 2) * 8);
        }
    }
    Ok(Data {
        width: w as u16,
        height: h as u16,
        rgba,
        words,
        palette,
    })
}

/// Source discovery only; the decoder validates the actual file contents.
pub fn source_candidate(source: &str) -> bool {
    Path::new(source)
        .extension()
        .and_then(|s| s.to_str())
        .is_some_and(|s| s.eq_ignore_ascii_case("png") || s.eq_ignore_ascii_case("tim"))
}

impl Data {
    /// PNG keeps its legacy STP-on policy. Native CLUTs can also contain opaque
    /// texels inside a blending primitive; the host preview needs a second pass.
    pub fn has_opaque_texels(&self) -> bool {
        self.palette
            .iter()
            .any(|&word| word != 0 && word & 0x8000 == 0)
    }
}

/// A single 4/8-bit TIM image with one CLUT, relocated by the normal VRAM cook.
/// Retain CLUT words and indices exactly (including visible index zero and
/// transparent entries at other indices). No color quantization or STP rewrite.
fn decode_tim(bytes: &[u8]) -> Result<Data, String> {
    fn block<'a>(bytes: &mut &'a [u8]) -> Result<(u16, u16, &'a [u8]), String> {
        if bytes.len() < 12 {
            return Err("TIM: truncated image/CLUT header".into());
        }
        let size = u32::from_le_bytes(bytes[0..4].try_into().unwrap()) as usize;
        let width = u16::from_le_bytes(bytes[8..10].try_into().unwrap());
        let height = u16::from_le_bytes(bytes[10..12].try_into().unwrap());
        let payload = usize::from(width)
            .checked_mul(usize::from(height))
            .and_then(|n| n.checked_mul(2))
            .ok_or("TIM: block size overflow")?;
        if size != 12 + payload || size > bytes.len() {
            return Err("TIM: invalid image/CLUT block length".into());
        }
        let data = &bytes[12..size];
        *bytes = &bytes[size..];
        Ok((width, height, data))
    }
    if bytes.len() < 8 {
        return Err("TIM: truncated file header".into());
    }
    let flags = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
    if !matches!(flags, 8 | 9) {
        return Err("TIM: only indexed 4/8-bit images with one CLUT are supported".into());
    }
    let bits = if flags == 8 { 4 } else { 8 };
    let mut remaining = &bytes[8..];
    let (cw, ch, clut) = block(&mut remaining)?;
    if cw != 1 << bits || ch != 1 {
        return Err("TIM: export exactly one 16/256-color CLUT per image".into());
    }
    let (word_width, height, pixels) = block(&mut remaining)?;
    let width = usize::from(word_width) * (16 / bits);
    if width == 0 || width > 256 || height == 0 || height > 256 {
        return Err("TIM: texture dimensions must be 1..256 pixels per axis".into());
    }
    if !remaining.is_empty() {
        return Err("TIM: unexpected trailing data".into());
    }
    let mut palette: Vec<u16> = clut
        .chunks_exact(2)
        .map(|p| u16::from_le_bytes([p[0], p[1]]))
        .collect();
    palette.resize(256, 0);
    let row = width.div_ceil(4) * 2;
    let mut words = vec![0; row * usize::from(height)];
    let mut rgba = Vec::with_capacity(width * usize::from(height) * 4);
    for y in 0..usize::from(height) {
        for x in 0..width {
            let byte = pixels[y * usize::from(word_width) * 2 + x * bits / 8];
            let index = if bits == 4 {
                (byte >> ((x % 2) * 4)) & 15
            } else {
                byte
            };
            words[y * row + x / 2] |= u16::from(index) << ((x % 2) * 8);
            let color = palette[usize::from(index)];
            for channel in 0..3 {
                rgba.push((((color >> (channel * 5)) & 31) * 255 / 31) as u8);
            }
            rgba.push(if color == 0 { 0 } else { 255 });
        }
    }
    Ok(Data {
        width: width as u16,
        height,
        rgba,
        words,
        palette,
    })
}
pub fn prepare(
    root: &Path,
    source: &str,
    destination: &str,
    existing: Option<&assets::Record>,
    snapshot: bool,
) -> Result<assets::Candidate, String> {
    if existing.is_some_and(|r| r.meta.kind != assets::Kind::Texture) {
        return Err("Reimport requires a Texture asset".into());
    }
    let dest = assets::inside(root, destination)?;
    if dest.extension().is_none_or(|x| x != "epokasset") {
        return Err("Texture destination must end in .epokasset".into());
    }
    let old = existing
        .map(|r| assets::Package::load(&r.path))
        .transpose()?;
    let path = if snapshot {
        None
    } else {
        Some(assets::inside(root, source)?)
    };
    let bytes = if let Some(p) = &path {
        assets::read_bounded(p)?
    } else {
        old.as_ref()
            .ok_or("Missing texture snapshot")?
            .source
            .clone()
    };
    decode(&bytes)?;
    let hash = assets::hash(&bytes);
    Ok(assets::Candidate {
        destination: dest,
        expected: existing.map(|r| r.revision.clone()),
        source_path: path,
        source_hash: hash.clone(),
        package: assets::Package {
            meta: assets::Metadata {
                version: 2,
                id: existing.map_or_else(Uuid::new_v4, |r| r.meta.id),
                kind: assets::Kind::Texture,
                importer_version: 1,
                source: if snapshot {
                    old.unwrap().meta.source
                } else {
                    source.replace('\\', "/")
                },
                source_hash: hash,
                settings: crate::import_settings::Settings::Texture,
                extra: Default::default(),
            },
            source: bytes,
        },
    })
}
pub fn ids(scene: &Scene) -> Vec<Uuid> {
    let mut ids = BTreeSet::new();
    for e in &scene.actors {
        ids.extend(e.material.texture);
        ids.extend(e.image.as_ref().and_then(|s| s.texture));
        ids.extend(e.sprite.as_ref().and_then(|s| s.texture));
        ids.extend(e.particle_emitter.as_ref().and_then(|s| s.sprite.texture));
        ids.extend(e.palette_animator.as_ref().and_then(|a| a.texture));
        if let Some(m) = &e.editable_mesh {
            ids.extend(m.materials.values().filter_map(|m| m.texture));
            if let Some(d) = &m.document {
                ids.extend(d.materials.iter().filter_map(|m| m.material.texture));
            }
        }
        if let Some(t) = &e.terrain {
            ids.extend(t.material.texture);
        }
        if let Some(model) = e.skeletal_mesh.as_ref().and_then(|m| m.model.as_ref()) {
            ids.extend(model.materials.iter().filter_map(|m| m.texture));
        }
    }
    ids.into_iter().collect()
}
pub fn resolve(scene: &mut Scene, index: &assets::Index) -> Result<(), String> {
    scene.textures.clear();
    let mut errors = vec![];
    for id in ids(scene) {
        let load = || {
            let r = index.resolve(id)?;
            if r.meta.kind != assets::Kind::Texture {
                return Err(format!("{id} is not a Texture"));
            }
            crate::source_cache::load(&r.path, &format!("texture-psx-v1-{id}"), |bytes| {
                let package = assets::Package::parse(bytes)?;
                if package.meta.kind != assets::Kind::Texture || package.meta.id != id {
                    return Err("The texture asset was replaced; refresh the asset index".into());
                }
                decode(&package.source)
            })
        };
        match load() {
            Ok(d) => {
                scene.textures.insert(id, d);
            }
            Err(e) => errors.push(e),
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}
pub fn validate_region(id: Option<Uuid>, r: [u16; 4], scene: &Scene) -> Result<(), String> {
    let (w, h) = id
        .and_then(|id| scene.textures.get(&id))
        .map_or((256, 256), |t| (t.width, t.height));
    if id.is_some_and(|id| id.is_nil())
        || r[0] >= w
        || r[1] >= h
        || (r[2] == 0) != (r[3] == 0)
        || u32::from(r[0]) + u32::from(r[2]) > u32::from(w)
        || u32::from(r[1]) + u32::from(r[3]) > u32::from(h)
    {
        Err("Atlas region exceeds texture dimensions".into())
    } else {
        Ok(())
    }
}
pub use crate::texture_cook::Layout;
/// The scene's resident VRAM allocation: textures, then authored fonts.
pub fn layout(scene: &Scene) -> Result<Layout, String> {
    crate::texture_cook::layout(scene)
}
pub fn symbol(id: Uuid) -> String {
    format!("texture_id_{}", id.simple())
}
pub fn validate_material(m: &Material) -> Result<(), String> {
    if m.texture.is_some_and(|id| id.is_nil())
        || m.uv_scroll.iter().any(|v| !v.is_finite() || v.abs() > 4.)
        || m.depth_bias.unsigned_abs() > 511
        || m.color
            .iter()
            .any(|v| !v.is_finite() || !(0. ..=1.).contains(v))
    {
        Err("Material requires a valid Texture UUID, color 0..1, depth bias ±511 and UV scroll within ±4 cycles/second".into())
    } else {
        Ok(())
    }
}
pub fn material_cpp(m: &Material) -> String {
    format!(
        "{{{{{},{},{}}}, {},{},BlendMode::{:?},{},{},{{{},{}}}}}",
        (m.color[0] * 255.).round() as u8,
        (m.color[1] * 255.).round() as u8,
        (m.color[2] * 255.).round() as u8,
        m.unlit,
        m.texture.map(symbol).unwrap_or("-1".into()),
        m.blend,
        m.depth_bias,
        m.double_sided,
        (m.uv_scroll[0] * 4096.).round() as i32,
        (m.uv_scroll[1] * 4096.).round() as i32
    )
}
pub fn uv_cpp(uv: [[f32; 2]; 4]) -> String {
    format!(
        "{{{}}}",
        uv.iter()
            .map(|p| format!(
                "{{{},{}}}",
                (p[0].clamp(0., 1.) * 4096.).round() as i16,
                (p[1].clamp(0., 1.) * 4096.).round() as i16
            ))
            .collect::<Vec<_>>()
            .join(",")
    )
}
pub fn header(scene: &Scene) -> Result<String, String> {
    for e in &scene.actors {
        for q in crate::lighting::quads(e) {
            if q.material.texture.is_some()
                && q.uv
                    .iter()
                    .flatten()
                    .any(|v| !v.is_finite() || !(0. ..=1.).contains(v))
            {
                return Err(format!(
                    "{}: textured mesh UVs must be normalized to 0..1 within an atlas",
                    e.name
                ));
            }
            if q.material.depth_bias.unsigned_abs() > 511
                || q.material.texture.is_some_and(|id| id.is_nil())
            {
                return Err("Invalid texture reference or material depth bias (±511)".into());
            }
        }
    }
    let layout = layout(scene)?.textures;
    let resident_bytes = layout
        .iter()
        .map(|(id, _)| {
            let t = &scene.textures[id];
            let cooked = crate::texture_cook::cook(t);
            (cooked.words.len() + cooked.palette.len()) * 2
        })
        .sum::<usize>();
    let mut out = String::new();
    out += &format!("inline constexpr size_t resident_texture_bytes={resident_bytes};\n");
    let mut assets = vec![];
    for (i, (id, p)) in layout.iter().enumerate() {
        let t = &scene.textures[id];
        let cooked = crate::texture_cook::cook(t);
        out += &format!("inline constexpr int {}={i};\n", symbol(*id));
        for (name, values) in [("pixels", &cooked.words), ("palette", &cooked.palette)] {
            out += &format!(
                "alignas(4) inline constexpr uint16_t texture_{i}_{name}[]={{{}}};\n",
                values
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            );
        }
        assets.push(format!(
            "{{{},{},{},{},{},{},{},texture_{i}_pixels,texture_{i}_palette,{}}}",
            t.width,
            t.height,
            p.x,
            p.y,
            cooked.format.word_width(t.width),
            p.clut_x,
            p.clut_y,
            cooked.format.bits()
        ));
    }
    out += &format!(
        "inline constexpr size_t texture_count={};\ninline constexpr Texture texture_assets[]={{{}}};\n",
        assets.len(),
        if assets.is_empty() {
            "{}".into()
        } else {
            assets.join(",")
        }
    );
    out += "inline constexpr auto texture_packet_words=bake_texture_packets(texture_assets);\ninline const uint32_t* baked_texture_bank(const Texture* bank){return bank==texture_assets?texture_packet_words.words:nullptr;}\n";
    Ok(out)
}

pub fn picker(ui: &imgui::Ui, index: &assets::Index, material: &mut Material) -> bool {
    let mut changed = false;
    changed |= ui.checkbox(
        crate::gui::field(ui, "Double sided"),
        &mut material.double_sided,
    );
    if let Some(id) = material.texture
        && let Err(error) = index.resolve(id)
    {
        ui.text_wrapped(error);
    }
    let label = material
        .texture
        .and_then(|id| index.resolve(id).ok())
        .map(|r| r.meta.source.as_str())
        .unwrap_or("None");
    if let Some(_c) = ui.begin_combo(crate::gui::field(ui, "Texture"), label) {
        if ui.selectable("None") {
            material.texture = None;
            changed = true;
        }
        for r in index
            .usable()
            .filter(|r| r.meta.kind == assets::Kind::Texture)
        {
            if ui.selectable(&r.meta.source) {
                material.texture = Some(r.meta.id);
                changed = true;
            }
        }
    }
    if let Some(_c) = ui.begin_combo(
        crate::gui::field(ui, "Blend"),
        format!("{:?}", material.blend),
    ) {
        for v in [
            BlendMode::Cutout,
            BlendMode::Average,
            BlendMode::Add,
            BlendMode::Subtract,
            BlendMode::AddQuarter,
        ] {
            if ui.selectable(format!("{v:?}")) {
                material.blend = v;
                changed = true;
            }
        }
    }
    changed |= crate::gui::Drag::new(crate::gui::field(ui, "Depth bias"))
        .range(-64, 64)
        .build(ui, &mut material.depth_bias);
    changed |= crate::gui::Drag::new(crate::gui::field(ui, "UV scroll / second"))
        .range(-4., 4.)
        .speed(0.01)
        .build_array(ui, &mut material.uv_scroll);
    changed
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    pub fn tim_fixture(
        bits: usize,
        palette: &[u16],
        width_words: u16,
        height: u16,
        pixels: &[u8],
    ) -> Vec<u8> {
        let mut out = 0x10u32.to_le_bytes().to_vec();
        out.extend_from_slice(&(if bits == 4 { 8u32 } else { 9u32 }).to_le_bytes());
        for (width, height, data) in [
            (
                palette.len() as u16,
                1,
                palette
                    .iter()
                    .flat_map(|v| v.to_le_bytes())
                    .collect::<Vec<_>>(),
            ),
            (width_words, height, pixels.to_vec()),
        ] {
            out.extend_from_slice(&(12u32 + data.len() as u32).to_le_bytes());
            out.extend_from_slice(&[0; 4]); // Original VRAM position is intentionally not authoritative.
            out.extend_from_slice(&width.to_le_bytes());
            out.extend_from_slice(&height.to_le_bytes());
            out.extend_from_slice(&data);
        }
        out
    }
    #[test]
    fn tim_retains_all_clut_bits_indices_and_nonzero_index_zero() {
        for bits in [4, 8] {
            let mut palette = vec![0; 1 << bits];
            palette[..4].copy_from_slice(&[0x001f, 0x801f, 0, 0x8000]);
            let pixels = if bits == 4 {
                vec![0x10, 0x32, 0x01, 0x23]
            } else {
                vec![0, 1, 2, 3, 1, 0, 3, 2]
            };
            let encoded = tim_fixture(bits, &palette, (bits / 4) as u16, 2, &pixels);
            let data = decode(&encoded).unwrap();
            assert_eq!((data.width, data.height), (4, 2));
            assert_eq!(data.words, [0x0100, 0x0302, 0x0001, 0x0203]);
            assert_eq!(&data.palette[..1 << bits], &palette);
            assert_eq!(
                &data.rgba[..16],
                &[255, 0, 0, 255, 255, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 255]
            );
            assert!(data.has_opaque_texels());
            let cooked = crate::texture_cook::cook(&data);
            assert_eq!(&cooked.palette[..4], &[0x001f, 0x801f, 0, 0x8000]);
            assert_eq!(cooked.words, [0x3210, 0, 0x2301, 0]);
            for n in 0..encoded.len() {
                assert!(decode(&encoded[..n]).is_err(), "prefix {n}");
            }
            let mut trailing = encoded.clone();
            trailing.push(0);
            assert!(decode(&trailing).is_err());
            let mut bad = encoded.clone();
            bad[8..12].copy_from_slice(&u32::MAX.to_le_bytes());
            assert!(decode(&bad).is_err());
        }
    }
    #[test]
    fn tim_preserves_full_256_colors_without_reserved_transparency_or_quantization() {
        let palette: Vec<u16> = (0..256).map(|i| 0x8000 | i).collect();
        let pixels: Vec<u8> = (0..=255).collect();
        let data = decode(&tim_fixture(8, &palette, 128, 1, &pixels)).unwrap();
        assert_eq!(data.palette, palette);
        assert!(data.rgba.chunks_exact(4).all(|p| p[3] == 255));
        assert_eq!(
            crate::texture_cook::cook(&data).format,
            crate::texture_cook::Format::Indexed8
        );
        for (i, word) in data.words.iter().enumerate() {
            assert_eq!(*word, (i * 2) as u16 | (((i * 2 + 1) as u16) << 8));
        }
        assert!(!data.has_opaque_texels());
    }
    #[test]
    fn tim_rejects_ambiguous_palettes_dimensions_and_unsupported_modes() {
        let valid = tim_fixture(4, &[0; 16], 1, 1, &[0, 0]);
        for flags in [0u32, 1, 2, 3, 10, 11, 0x108] {
            let mut bytes = valid.clone();
            bytes[4..8].copy_from_slice(&flags.to_le_bytes());
            assert!(decode(&bytes).unwrap_err().contains("only indexed"));
        }
        assert!(
            decode(&tim_fixture(4, &[0; 32], 1, 1, &[0, 0]))
                .unwrap_err()
                .contains("one 16/256")
        );
        assert!(
            decode(&tim_fixture(4, &[0; 16], 65, 1, &[0; 130]))
                .unwrap_err()
                .contains("dimensions")
        );
        assert!(decode(&tim_fixture(4, &[0; 16], 1, 0, &[])).is_err());
        for path in ["assets/a.tim", "assets/a.TIM", "assets/a.png"] {
            assert!(source_candidate(path));
        }
        assert!(!source_candidate("assets/a.tim.bak"));
    }
    #[test]
    fn tim_import_reimport_snapshot_and_discovery_keep_asset_identity() {
        let root = crate::workspace::tests::temp("native-tim");
        std::fs::create_dir_all(root.join("assets")).unwrap();
        let mut palette = [0; 16];
        palette[0] = 0x001f;
        palette[1] = 0x8000;
        let bytes = tim_fixture(4, &palette, 1, 1, &[0x10, 0x01]);
        std::fs::write(root.join("assets/t.tim"), &bytes).unwrap();
        let id = assets::commit(
            prepare(&root, "assets/t.tim", "assets/t.epokasset", None, false).unwrap(),
        )
        .unwrap();
        let index = assets::scan(&root, &mut Default::default());
        assert!(
            index
                .linked_source(index.resolve(id).unwrap())
                .unwrap()
                .is_some()
        );
        let mut scene = Scene::default();
        scene.actors[1].material.texture = Some(id);
        resolve(&mut scene, &index).unwrap();
        assert_eq!(scene.textures[&id].palette[..2], [0x001f, 0x8000]);
        std::fs::remove_file(root.join("assets/t.tim")).unwrap();
        let candidate = prepare(
            &root,
            "assets/t.tim",
            "assets/t.epokasset",
            Some(index.resolve(id).unwrap()),
            true,
        )
        .unwrap();
        assert_eq!(assets::commit(candidate).unwrap(), id);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn prepared_textures_share_storage_and_do_not_hide_package_changes() {
        let root = crate::workspace::tests::temp("prepared-texture");
        std::fs::create_dir_all(root.join("assets")).unwrap();
        std::fs::write(root.join("assets/t.png"), png()).unwrap();
        let id = assets::commit(
            prepare(&root, "assets/t.png", "assets/t.epokasset", None, false).unwrap(),
        )
        .unwrap();
        let index = assets::scan(&root, &mut Default::default());
        let mut scene = Scene::default();
        scene.actors[1].material.texture = Some(id);
        resolve(&mut scene, &index).unwrap();
        let original = scene.textures[&id].clone();
        resolve(&mut scene, &index).unwrap();
        assert!(Arc::ptr_eq(&original, &scene.textures[&id]));
        let path = root.join("assets/t.epokasset");
        let mut package = assets::Package::load(&path).unwrap();
        let mut png = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut png, 1, 1);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder
                .write_header()
                .unwrap()
                .write_image_data(&[0, 0, 255, 255])
                .unwrap();
        }
        package.source = png;
        package.meta.source_hash = assets::hash(&package.source);
        std::fs::write(&path, package.bytes().unwrap()).unwrap();
        resolve(&mut scene, &index).unwrap(); // deliberately stale index
        assert_eq!(scene.textures[&id].rgba, [0, 0, 255, 255]);
        assert!(!Arc::ptr_eq(&original, &scene.textures[&id]));
        assert_eq!(original.width, 3);
        package.meta.id = Uuid::new_v4();
        std::fs::write(&path, package.bytes().unwrap()).unwrap();
        assert!(resolve(&mut scene, &index).is_err());
        assert!(!scene.textures.contains_key(&id));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn double_sided_is_opt_in_and_survives_material_serialization() {
        let mut material: Material = serde_json::from_str("{\"unlit\":true}").unwrap();
        assert!(!material.double_sided);
        material.double_sided = true;
        let copy: Material =
            serde_json::from_slice(&serde_json::to_vec(&material).unwrap()).unwrap();
        assert_eq!(copy, material);
        assert_eq!(
            material_cpp(&material),
            "{{255,255,255}, true,-1,BlendMode::Cutout,0,true,{0,0}}"
        );
    }
    pub fn png() -> Vec<u8> {
        let mut bytes = vec![];
        {
            let mut e = png::Encoder::new(&mut bytes, 3, 1);
            e.set_color(png::ColorType::Rgba);
            e.set_depth(png::BitDepth::Eight);
            e.write_header()
                .unwrap()
                .write_image_data(&[0, 0, 0, 255, 255, 0, 0, 0, 255, 0, 0, 255])
                .unwrap();
        }
        bytes
    }
    #[test]
    fn small_effect_atlas_does_not_fragment_full_height_pages() {
        let mut scene = Scene::default();
        scene.actors.clear();
        for (i, height) in [64, 256, 256, 256].into_iter().enumerate() {
            let id = Uuid::from_u128(i as u128 + 1);
            let mut a = crate::scene::Actor::cube(format!("texture {i}"));
            a.material.texture = Some(id);
            scene.actors.push(a);
            let mut texture = decode(&png()).unwrap();
            texture.width = 256;
            texture.height = height;
            texture.words = vec![0x2020; 128 * height as usize];
            texture.palette[32] = 0x8123; // Full-width 8-bit atlas, not a three-pixel mock.
            scene.textures.insert(id, std::sync::Arc::new(texture));
        }
        let packed = layout(&scene).unwrap().textures;
        assert_eq!(
            packed.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
            ids(&scene)
        );
        for (i, (_, p)) in packed.iter().enumerate() {
            assert_eq!(p.x % 64, 0);
            assert_eq!(p.clut_x % 16, 0);
            if i != 0 {
                assert_eq!(p.y % 256, 0);
            }
        }
    }
    #[test]
    fn palette_preserves_opaque_black_cutout_and_odd_rows() {
        let d = decode(&png()).unwrap();
        assert_eq!(d.words.len(), 2);
        assert_eq!(d.rgba, [0, 0, 0, 255, 0, 0, 0, 0, 255, 0, 0, 255]);
        assert_eq!(d.palette[1] & 0x8000, 0x8000);
        assert_eq!(decode(&png()).unwrap(), d);
    }
    /// Fonts are packed by the texture allocator as 4bpp atlases with a
    /// sixteen-entry CLUT, in the same VRAM as the textures and never over them.
    #[test]
    fn fonts_share_the_texture_allocator_without_overlap() {
        let font = |width: u16| crate::font_asset::FontData {
            width,
            height: 32,
            words: vec![0; usize::from(width) / 4 * 32],
            palette: crate::font_asset::palette(false),
            metrics: vec![crate::font_asset::GlyphMetric {
                codepoint: 65,
                u: 0,
                v: 0,
                width: 6,
                height: 8,
                advance: 7,
                x_offset: 0,
                y_offset: -8,
            }],
            line_height: 12,
            baseline: 9,
            rgba: vec![],
        };
        let mut scene = Scene::default();
        scene.actors.clear();
        let mut canvas = crate::scene::Actor::cube("Canvas".into());
        canvas.kind = "Empty".into();
        canvas.canvas = Some(Default::default());
        scene.actors.push(canvas);
        let mut texture = decode(&png()).unwrap();
        texture.width = 256;
        texture.height = 128;
        let texture_id = Uuid::new_v4();
        scene.actors[0].material.texture = Some(texture_id);
        scene.textures.insert(texture_id, Arc::new(texture));
        let id = Uuid::new_v4();
        let mut label = crate::scene::Actor::cube("Label".into());
        label.kind = "Empty".into();
        label.parent = Some(0);
        label.rect = Some(Default::default());
        label.text = Some(crate::hud::Text {
            font: Some(id),
            ..Default::default()
        });
        scene.actors.push(label);
        scene.fonts.insert(id, Arc::new(font(64)));
        let rects = |packed: &crate::texture::Layout, scene: &Scene| {
            let mut out = vec![];
            for (id, p) in &packed.textures {
                let t = &scene.textures[id];
                out.push([p.x, p.y, p.format.word_width(t.width), t.height]);
                out.push([p.clut_x, p.clut_y, p.format.colors(), 1]);
            }
            for (id, p) in &packed.fonts {
                let f = &scene.fonts[id];
                assert_eq!(p.format, crate::texture_cook::Format::Indexed4);
                assert_eq!(p.clut_x % 16, 0);
                // A font row fits its 256-texel texture page.
                assert!(
                    (p.x % 64 + p.format.word_width(f.width)) * 4 <= 256
                        && p.y % 256 + f.height <= 256
                );
                out.push([p.x, p.y, p.format.word_width(f.width), f.height]);
                out.push([p.clut_x, p.clut_y, 16, 1]);
            }
            for (i, a) in out.iter().enumerate() {
                assert!(
                    a[0] >= 640 && a[0] + a[2] <= 1024 && a[1] + a[3] <= 512,
                    "{a:?}"
                );
                for b in &out[i + 1..] {
                    let apart = a[0] + a[2] <= b[0]
                        || b[0] + b[2] <= a[0]
                        || a[1] + a[3] <= b[1]
                        || b[1] + b[3] <= a[1];
                    assert!(apart, "VRAM overlap {a:?} {b:?}");
                }
            }
        };
        let packed = layout(&scene).unwrap();
        assert_eq!(packed.textures.len(), 1);
        assert_eq!(packed.fonts.len(), 1);
        assert_eq!(packed.fonts[0].0, id);
        rects(&packed, &scene);
        // Fonts are global across scene banks: a bank with other textures
        // keeps the same font placement and packs its textures around it.
        let mut other = scene.clone();
        let other_texture = Uuid::new_v4();
        let mut tall = decode(&png()).unwrap();
        tall.width = 256;
        tall.height = 256;
        other.textures.clear();
        other.textures.insert(other_texture, Arc::new(tall));
        other.actors[0].material.texture = Some(other_texture);
        let other_packed = layout(&other).unwrap();
        assert_eq!(other_packed.fonts, packed.fonts);
        rects(&other_packed, &other);
        // Unlike the old 32-row CLUT band, many fonts pack beside the texture.
        let label = |scene: &mut Scene, font: Uuid| {
            let mut e = crate::scene::Actor::cube("Label".into());
            e.kind = "Empty".into();
            e.parent = Some(0);
            e.rect = Some(Default::default());
            e.text = Some(crate::hud::Text {
                font: Some(font),
                ..Default::default()
            });
            scene.actors.push(e);
        };
        for _ in 0..39 {
            let extra = Uuid::new_v4();
            label(&mut scene, extra);
            scene.fonts.insert(extra, Arc::new(font(64)));
        }
        let packed = layout(&scene).unwrap();
        assert_eq!(packed.fonts.len(), 40);
        rects(&packed, &scene);
        // More atlases than VRAM holds are refused, not overlapped.
        for _ in 0..400 {
            let extra = Uuid::new_v4();
            label(&mut scene, extra);
            scene.fonts.insert(extra, Arc::new(font(256)));
        }
        assert!(layout(&scene).unwrap_err().contains("VRAM exhausted"));
    }
    #[test]
    fn png_import_portable_reimport_mesh_uv_and_hud_roundtrip() {
        let root = crate::workspace::editor_home()
            .join(".epok")
            .join(format!("texture-test-{}", Uuid::new_v4()));
        std::fs::create_dir_all(root.join("assets")).unwrap();
        std::fs::write(root.join("assets/t.png"), png()).unwrap();
        let id = assets::commit(
            prepare(&root, "assets/t.png", "assets/t.epokasset", None, false).unwrap(),
        )
        .unwrap();
        let index = assets::scan(&root, &mut Default::default());
        let r = index.resolve(id).unwrap();
        let mut scene = Scene::default();
        scene.actors[1].material.texture = Some(id);
        scene.actors[1].material.blend = BlendMode::Add;
        scene.actors[1].lighting.subdivisions = 2;
        resolve(&mut scene, &index).unwrap();
        let h = crate::project::scene_header(&scene, &[]).unwrap();
        assert!(h.contains("texture_id_"));
        assert!(h.contains("BlendMode::Add"));
        assert!(h.contains("{2048,2048}"));
        let p = layout(&scene).unwrap().textures[0].1;
        assert!(h.contains(&format!(
            "3,1,{},{},2,{},{},texture_0_pixels,texture_0_palette,4",
            p.x, p.y, p.clut_x, p.clut_y
        )));
        let saved = serde_json::to_vec(&scene).unwrap();
        let mut decoded: Scene = serde_json::from_slice(&saved).unwrap();
        assert!(decoded.textures.is_empty());
        resolve(&mut decoded, &index).unwrap();
        assert_eq!(scene, decoded);
        std::fs::remove_file(root.join("assets/t.png")).unwrap();
        let candidate =
            prepare(&root, "assets/t.png", "assets/t.epokasset", Some(r), true).unwrap();
        assert_eq!(assets::commit(candidate).unwrap(), id);
        let mut canvas = crate::scene::Actor::cube("Canvas".into());
        canvas.kind = "Empty".into();
        canvas.canvas = Some(Default::default());
        let canvas_i = scene.actors.len();
        scene.actors.push(canvas);
        let mut image = crate::scene::Actor::cube("Image".into());
        image.kind = "Empty".into();
        image.parent = Some(canvas_i);
        image.rect = Some(crate::hud::RectTransform {
            anchor_min: [0., 1.],
            anchor_max: [0., 1.],
            pivot: [0., 1.],
            size: [3., 1.],
            ..Default::default()
        });
        image.image = Some(crate::hud::Image {
            texture: Some(id),
            color: [1.; 3],
            ..Default::default()
        });
        scene.actors.push(image);
        let pixels = crate::hud::render(&scene);
        assert_eq!(
            &pixels[..12],
            &[0, 0, 0, 255, 33, 40, 52, 255, 255, 0, 0, 255]
        );
        scene
            .actors
            .last_mut()
            .unwrap()
            .image
            .as_mut()
            .unwrap()
            .region = [2, 0, 2, 1];
        assert!(crate::hud::validate(&scene).is_err());
    }
    #[test]
    fn vram_rejects_overflow_without_touching_framebuffer_or_font() {
        let mut scene = Scene::default();
        scene.display_size = [640, 480];
        let mut d = decode(&png()).unwrap();
        d.width = 256;
        d.height = 256;
        d.words = vec![0x2020; 32768];
        d.palette[32] = 0x8123;
        for _ in 0..3 {
            let id = Uuid::new_v4();
            scene
                .actors
                .push(crate::scene::Actor::cube("Texture".into()));
            scene.actors.last_mut().unwrap().material.texture = Some(id);
            scene.textures.insert(id, Arc::new(d.clone()));
        }
        for (_, p) in layout(&scene).unwrap().textures {
            assert!(p.x >= 640);
            assert_eq!(p.y % 256, 0);
            assert!(p.x + 128 <= 1024 && p.clut_x + 256 <= 1024);
            assert!(p.clut_y < 512);
        }
        for _ in 0..5 {
            let id = Uuid::new_v4();
            let mut actor = crate::scene::Actor::cube("Overflow".into());
            actor.material.texture = Some(id);
            scene.actors.push(actor);
            scene.textures.insert(id, Arc::new(d.clone()));
        }
        assert!(layout(&scene).is_err());
        // Narrow framebuffers free their unused columns, never the used ones.
        scene.display_size = [320, 240];
        for (_, p) in layout(&scene).unwrap().textures {
            assert!(p.x >= 320 && p.clut_x >= 320);
        }
    }
}
