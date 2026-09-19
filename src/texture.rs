//! Portable PNG textures, deterministic PSX palettes and a framebuffer-safe VRAM layout.
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
#[derive(Clone, Debug, PartialEq)]
pub struct Data {
    pub width: u16,
    pub height: u16,
    pub rgba: Vec<u8>,
    pub words: Vec<u16>,
    pub palette: Vec<u16>,
}

pub fn decode(bytes: &[u8]) -> Result<Data, String> {
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
            decode(&assets::Package::load(&r.path)?.source)
        };
        match load() {
            Ok(d) => {
                scene.textures.insert(id, Arc::new(d));
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
#[derive(Debug, Clone, PartialEq)]
pub struct Placement {
    pub x: u16,
    pub y: u16,
    pub clut_y: u16,
}
pub fn layout(scene: &Scene) -> Result<Vec<(Uuid, Placement)>, String> {
    let mut cursor = [[0u16; 2]; 3];
    let mut result = vec![];
    for (i, id) in ids(scene).into_iter().enumerate() {
        if i >= 32 {
            return Err("VRAM palette budget exceeded (32 textures)".into());
        }
        let t = scene
            .textures
            .get(&id)
            .ok_or_else(|| format!("Unresolved texture {id}"))?;
        let mut place = None;
        for row in 0..2 {
            for (col, rows) in cursor.iter_mut().enumerate() {
                let limit = if row == 0 {
                    256
                } else if col == 2 {
                    128 // Resident loading image at (960,384), font at (960,448).
                } else {
                    224
                };
                if rows[row] + t.height <= limit {
                    place = Some(Placement {
                        x: 640 + col as u16 * 128,
                        y: row as u16 * 256 + rows[row],
                        clut_y: 480 + i as u16,
                    });
                    rows[row] += t.height;
                    break;
                }
            }
            if place.is_some() {
                break;
            }
        }
        result.push((id,place.ok_or("Texture VRAM exhausted; reduce atlas height/count. Framebuffers and font are reserved.")?));
    }
    Ok(result)
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
        "{{{{{},{},{}}}, {},{},BlendMode::{:?},{},{{{},{}}}}}",
        (m.color[0] * 255.).round() as u8,
        (m.color[1] * 255.).round() as u8,
        (m.color[2] * 255.).round() as u8,
        m.unlit,
        m.texture.map(symbol).unwrap_or("-1".into()),
        m.blend,
        m.depth_bias,
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
    let layout = layout(scene)?;
    let resident_bytes = layout
        .iter()
        .map(|(id, _)| {
            let t = &scene.textures[id];
            (t.words.len() + t.palette.len()) * 2
        })
        .sum::<usize>();
    let mut out = String::new();
    out += &format!("inline constexpr size_t resident_texture_bytes={resident_bytes};\n");
    let mut assets = vec![];
    for (i, (id, p)) in layout.iter().enumerate() {
        let t = &scene.textures[id];
        out += &format!("inline constexpr int {}={i};\n", symbol(*id));
        for (name, values) in [("pixels", &t.words), ("palette", &t.palette)] {
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
            "{{{},{},{},{},{},640,{},texture_{i}_pixels,texture_{i}_palette}}",
            t.width,
            t.height,
            p.x,
            p.y,
            t.width.div_ceil(4) * 2,
            p.clut_y
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
    Ok(out)
}

pub fn picker(ui: &imgui::Ui, index: &assets::Index, material: &mut Material) -> bool {
    let mut changed = false;
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
mod tests {
    use super::*;
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
    fn palette_preserves_opaque_black_cutout_and_odd_rows() {
        let d = decode(&png()).unwrap();
        assert_eq!(d.words.len(), 2);
        assert_eq!(d.rgba, [0, 0, 0, 255, 0, 0, 0, 0, 255, 0, 0, 255]);
        assert_eq!(d.palette[1] & 0x8000, 0x8000);
        assert_eq!(decode(&png()).unwrap(), d);
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
        assert!(h.contains("640,0,2,640,480"));
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
        let mut d = decode(&png()).unwrap();
        d.height = 256;
        for _ in 0..3 {
            let id = Uuid::new_v4();
            scene
                .actors
                .push(crate::scene::Actor::cube("Texture".into()));
            scene.actors.last_mut().unwrap().material.texture = Some(id);
            scene.textures.insert(id, Arc::new(d.clone()));
        }
        for (_, p) in layout(&scene).unwrap() {
            assert!(p.x >= 640);
            assert_eq!(p.y, 0);
            assert!((480..512).contains(&p.clut_y));
        }
        let id = Uuid::new_v4();
        scene.actors[0].material.texture = Some(id);
        scene.textures.insert(id, Arc::new(d));
        assert!(layout(&scene).is_err());
    }
}
