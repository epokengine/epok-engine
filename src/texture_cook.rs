//! Lossless indexed texture preparation and offline PSX VRAM rectangle packing.
//! Source/host textures stay unchanged; only generated PSX resources use this.
use crate::{scene::Scene, texture::Data};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Indexed4,
    Indexed8,
}
impl Format {
    pub fn for_data(data: &Data) -> Self {
        if data.palette.iter().skip(16).all(|&c| c == 0)
            && data.words.iter().all(|&w| w & 0xf0f0 == 0)
        {
            Self::Indexed4
        } else {
            Self::Indexed8
        }
    }
    pub fn bits(self) -> u16 {
        if self == Self::Indexed4 { 4 } else { 8 }
    }
    pub fn colors(self) -> u16 {
        if self == Self::Indexed4 { 16 } else { 256 }
    }
    pub fn texels_per_word(self) -> u16 {
        16 / self.bits()
    }
    pub fn word_width(self, width: u16) -> u16 {
        // PsyQo DMA uploads an even number of halfwords per row.
        width.div_ceil(self.texels_per_word() * 2) * 2
    }
}

pub struct Cooked {
    pub words: Vec<u16>,
    pub palette: Vec<u16>,
    pub format: Format,
}
pub fn cook(data: &Data) -> Cooked {
    let format = Format::for_data(data);
    let mut palette = data
        .palette
        .iter()
        .copied()
        .take(format.colors() as usize)
        .collect::<Vec<_>>();
    palette.resize(format.colors() as usize, 0);
    let words = if format == Format::Indexed8 {
        data.words.clone()
    } else {
        let source_row = data.width.div_ceil(4) as usize * 2;
        let target_row = format.word_width(data.width) as usize;
        let mut words = vec![0; target_row * data.height as usize];
        for y in 0..data.height as usize {
            for x in 0..data.width as usize {
                let index = (data.words[y * source_row + x / 2] >> ((x % 2) * 8)) & 255;
                words[y * target_row + x / 4] |= index << ((x % 4) * 4);
            }
        }
        words
    };
    Cooked {
        words,
        palette,
        format,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Placement {
    pub x: u16,
    pub y: u16,
    pub clut_x: u16,
    pub clut_y: u16,
    pub format: Format,
}
/// Page-relative texel origin is shared by generated geometry and runtime UVs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Page {
    pub width: u16,
    pub height: u16,
    pub u: u16,
    pub v: u16,
}
impl Placement {
    pub fn page(self, data: &Data) -> Page {
        Page {
            width: data.width,
            height: data.height,
            u: self.x % 64 * self.format.texels_per_word(),
            v: self.y % 256,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Rect {
    x: u16,
    y: u16,
    w: u16,
    h: u16,
}
impl Rect {
    fn right(self) -> u16 {
        self.x + self.w
    }
    fn bottom(self) -> u16 {
        self.y + self.h
    }
    fn overlaps(self, b: Self) -> bool {
        self.x < b.right() && b.x < self.right() && self.y < b.bottom() && b.y < self.bottom()
    }
    fn contains(self, b: Self) -> bool {
        self.x <= b.x && self.y <= b.y && self.right() >= b.right() && self.bottom() >= b.bottom()
    }
}

fn occupy(free: &mut Vec<Rect>, used: Rect) {
    let mut next = Vec::new();
    for r in free.drain(..) {
        if !r.overlaps(used) {
            next.push(r);
            continue;
        }
        if used.x > r.x {
            next.push(Rect {
                w: used.x - r.x,
                ..r
            });
        }
        if used.right() < r.right() {
            next.push(Rect {
                x: used.right(),
                w: r.right() - used.right(),
                ..r
            });
        }
        if used.y > r.y {
            next.push(Rect {
                h: used.y - r.y,
                ..r
            });
        }
        if used.bottom() < r.bottom() {
            next.push(Rect {
                y: used.bottom(),
                h: r.bottom() - used.bottom(),
                ..r
            });
        }
    }
    // MaxRects free rectangles may overlap each other, never occupied space.
    for i in 0..next.len() {
        if !next
            .iter()
            .enumerate()
            .any(|(j, r)| j != i && r.contains(next[i]) && (*r != next[i] || j < i))
        {
            free.push(next[i]);
        }
    }
}

#[derive(Clone, Copy)]
struct Request {
    index: usize,
    palette: bool,
    w: u16,
    h: u16,
    format: Format,
}
fn place(free: &[Rect], request: Request, top_left: bool) -> Option<Rect> {
    let align = if request.palette { 16 } else { 2 };
    let mut best = None;
    for &space in free {
        let start_x = space.x.div_ceil(align) * align;
        // Moving to the next page can make a wide texture legal in this space.
        for x in
            std::iter::once(start_x).chain(((start_x.div_ceil(64) * 64)..space.right()).step_by(64))
        {
            for y in [space.y, space.y.div_ceil(256) * 256] {
                let r = Rect {
                    x,
                    y,
                    w: request.w,
                    h: request.h,
                };
                if !space.contains(r) {
                    continue;
                }
                if !request.palette
                    && (y % 256 + r.h > 256
                        || (x % 64 + r.w) * request.format.texels_per_word() > 256)
                {
                    continue;
                }
                let dx = space.right() - r.right();
                let dy = space.bottom() - r.bottom();
                let score = if top_left {
                    (y, x, dx.min(dy), dx.max(dy))
                } else {
                    (dx.min(dy), dx.max(dy), y, x)
                };
                if best.as_ref().is_none_or(|(old, _)| score < *old) {
                    best = Some((score, r));
                }
            }
        }
    }
    best.map(|(_, r)| r)
}

// The widest display reservation: every column a 640-wide framebuffer can use,
// at every VRAM row. Authored fonts are global and are always placed outside
// it, so their VRAM never depends on the project's resolution.
pub const DISPLAY_RESERVE: [u16; 2] = [640, 512];

/// Columns the framebuffers of a `size` display occupy, at every VRAM row
/// (including the unused lines between and after PsyQo's buffers). Textures may
/// use everything to the right: a 320-wide project gains 320 columns, about
/// 320 KiB, over the fixed 640-column reservation. Memory reports share it.
pub fn display_reserve(size: [u16; 2]) -> [u16; 2] {
    [size[0].clamp(256, DISPLAY_RESERVE[0]), DISPLAY_RESERVE[1]]
}

/// The resident VRAM allocation of one scene: its textures and the authored
/// HUD fonts. Fonts are 4bpp atlases with a sixteen-entry CLUT, packed by the
/// same allocator as 4bpp textures and in the same space.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Layout {
    pub textures: Vec<(Uuid, Placement)>,
    pub fonts: Vec<(Uuid, Placement)>,
}

fn initial_free(reserved_width: u16) -> Vec<Rect> {
    let mut free = vec![Rect {
        x: reserved_width,
        y: 0,
        w: 1024 - reserved_width,
        h: DISPLAY_RESERVE[1],
    }];
    // Framebuffers occupy x<width. Loading image and font own this corner.
    occupy(
        &mut free,
        Rect {
            x: 960,
            y: 384,
            w: 64,
            h: 128,
        },
    );
    free
}

/// Fonts are global and resident for the whole run: one generated header
/// serves every scene bank. They are therefore placed first, from the font set
/// alone (every resolved font in `scene.fonts`, by id), so their VRAM is the
/// same whichever bank's textures are packed around them. Callers packing a
/// bank give it the build's full font set.
fn place_fonts(scene: &Scene, free: &mut Vec<Rect>) -> Result<Vec<(Uuid, Placement)>, String> {
    for id in crate::hud::font_ids(scene) {
        if !scene.fonts.contains_key(&id) {
            return Err(format!("Unresolved font {id}"));
        }
    }
    let mut ids: Vec<Uuid> = scene.fonts.keys().copied().collect();
    ids.sort_unstable();
    let format = Format::Indexed4;
    let mut requests = Vec::new();
    for (index, id) in ids.iter().enumerate() {
        let f = &scene.fonts[id];
        requests.push(Request {
            index,
            palette: false,
            w: format.word_width(f.width),
            h: f.height,
            format,
        });
        requests.push(Request {
            index,
            palette: true,
            w: format.colors(),
            h: 1,
            format,
        });
    }
    requests.sort_by_key(|r| {
        (
            std::cmp::Reverse(u32::from(r.w) * u32::from(r.h)),
            r.index,
            r.palette,
        )
    });
    let mut placed = vec![
        Placement {
            x: 0,
            y: 0,
            clut_x: 0,
            clut_y: 0,
            format
        };
        ids.len()
    ];
    for request in requests {
        // Best fit packs fonts towards the edges of free space, away from the
        // open area tall texture atlases need.
        let rect = place(free, request, false).ok_or_else(|| format!(
            "Font VRAM exhausted: {} fonts cannot be placed beside the framebuffers, loading image and built-in font. Reduce the pixel height or the character set.",
            ids.len()))?;
        occupy(free, rect);
        let p = &mut placed[request.index];
        if request.palette {
            p.clut_x = rect.x;
            p.clut_y = rect.y;
        } else {
            p.x = rect.x;
            p.y = rect.y;
        }
    }
    Ok(ids.into_iter().zip(placed).collect())
}

pub fn layout(scene: &Scene) -> Result<Layout, String> {
    let ids = crate::texture::ids(scene);
    let data: Vec<_> = ids
        .iter()
        .map(|id| {
            scene
                .textures
                .get(id)
                .ok_or_else(|| format!("Unresolved texture {id}"))
        })
        .collect::<Result<_, _>>()?;
    // Fonts are placed in the resolution-independent region first, then the
    // same rectangles are excluded from the texture space of this display.
    let mut legacy = initial_free(DISPLAY_RESERVE[0]);
    let fonts = place_fonts(scene, &mut legacy)?;
    let mut base = initial_free(display_reserve(scene.display_size)[0]);
    for (id, p) in &fonts {
        let f = &scene.fonts[id];
        let format = Format::Indexed4;
        occupy(
            &mut base,
            Rect {
                x: p.x,
                y: p.y,
                w: format.word_width(f.width),
                h: f.height,
            },
        );
        occupy(
            &mut base,
            Rect {
                x: p.clut_x,
                y: p.clut_y,
                w: format.colors(),
                h: 1,
            },
        );
    }
    let mut requests = Vec::new();
    for (index, t) in data.iter().enumerate() {
        let format = Format::for_data(t);
        requests.push(Request {
            index,
            palette: false,
            w: format.word_width(t.width),
            h: t.height,
            format,
        });
        requests.push(Request {
            index,
            palette: true,
            w: format.colors(),
            h: 1,
            format,
        });
    }
    // Deterministic offline alternatives: palettes-first keeps long CLUT rows
    // available; area-first can fit tall atlases around those reservations.
    // Top-left pixel packing also preserves full-width CLUT rows when several
    // 256-high atlases would otherwise interleave across the two page bands.
    for strategy in 0..4 {
        requests.sort_by_key(|r| {
            let group = match strategy {
                0 => u32::from(!r.palette),
                1 | 3 => u32::from(r.palette),
                _ => 0,
            };
            (
                group,
                std::cmp::Reverse(u32::from(r.w) * u32::from(r.h)),
                std::cmp::Reverse(r.h),
                r.index,
                r.palette,
            )
        });
        let mut free = base.clone();
        let mut placed = vec![
            Placement {
                x: 0,
                y: 0,
                clut_x: 0,
                clut_y: 0,
                format: Format::Indexed8
            };
            ids.len()
        ];
        let mut complete = true;
        for &request in &requests {
            let Some(rect) = place(&free, request, strategy == 3) else {
                complete = false;
                break;
            };
            occupy(&mut free, rect);
            let p = &mut placed[request.index];
            p.format = request.format;
            if request.palette {
                p.clut_x = rect.x;
                p.clut_y = rect.y;
            } else {
                p.x = rect.x;
                p.y = rect.y;
            }
        }
        if complete {
            return Ok(Layout {
                textures: ids.into_iter().zip(placed).collect(),
                fonts,
            });
        }
    }
    let bytes: usize = data
        .iter()
        .map(|t| {
            let f = Format::for_data(t);
            (usize::from(f.word_width(t.width)) * usize::from(t.height) + usize::from(f.colors()))
                * 2
        })
        .sum();
    Err(format!(
        "Texture VRAM exhausted: {} textures need {bytes} packed bytes; cannot place texture pages and CLUTs without overlapping framebuffers, loading image, font or {} authored fonts. Use texture residency or smaller source resources.",
        ids.len(),
        fonts.len()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    fn fixture(count: usize) -> Scene {
        let mut scene = Scene::default();
        scene.actors.clear();
        for i in 0..count {
            let id = Uuid::from_u128(i as u128 + 1);
            let mut actor = crate::scene::Actor::cube(format!("Tex {i}"));
            actor.material.texture = Some(id);
            scene.actors.push(actor);
            let width = if i % 5 == 0 { 64 } else { 32 };
            let height = if i % 7 == 0 { 64 } else { 32 };
            let mut palette = vec![0; 256];
            let high = i % 3 != 0;
            for (i, p) in palette
                .iter_mut()
                .enumerate()
                .take(if high { 64 } else { 16 })
                .skip(1)
            {
                *p = 0x8000 | i as u16;
            }
            scene.textures.insert(
                id,
                Arc::new(Data {
                    width,
                    height,
                    rgba: vec![],
                    words: vec![
                        if high { 0x2030 } else { 0x0203 };
                        width as usize * height as usize / 2
                    ],
                    palette,
                }),
            );
        }
        scene
    }
    pub(super) fn validate(scene: &Scene, layout: &[(Uuid, Placement)]) {
        let mut rects = Vec::new();
        for (id, p) in layout {
            let data = &scene.textures[id];
            let page = p.page(data);
            assert!(page.u + data.width <= 256 && page.v + data.height <= 256);
            assert_eq!(p.clut_x % 16, 0);
            for r in [
                Rect {
                    x: p.x,
                    y: p.y,
                    w: p.format.word_width(data.width),
                    h: data.height,
                },
                Rect {
                    x: p.clut_x,
                    y: p.clut_y,
                    w: p.format.colors(),
                    h: 1,
                },
            ] {
                assert!(
                    r.x >= display_reserve(scene.display_size)[0]
                        && r.right() <= 1024
                        && r.bottom() <= 512
                );
                assert!(!r.overlaps(Rect {
                    x: 960,
                    y: 384,
                    w: 64,
                    h: 128
                }));
                assert!(
                    rects.iter().all(|&other| !r.overlaps(other)),
                    "VRAM overlap: {r:?}"
                );
                rects.push(r);
            }
        }
    }
    #[test]
    fn hundreds_of_crops_pack_deterministically_without_page_wrap_or_overlap() {
        let scene = fixture(197);
        let packed = layout(&scene).unwrap().textures;
        assert_eq!(packed, layout(&scene).unwrap().textures);
        validate(&scene, &packed);
        assert!(packed.iter().any(|(_, p)| p.x % 64 != 0));
    }
    #[test]
    fn nibble_conversion_preserves_every_index_and_palette_word() {
        for width in [1, 2, 3, 4, 7, 8, 15, 16, 255, 256] {
            let mut data = Data {
                width,
                height: 3,
                rgba: vec![],
                words: vec![0; width.div_ceil(4) as usize * 2 * 3],
                palette: vec![0; 256],
            };
            for i in 1..16 {
                data.palette[i] = 0x8000 | i as u16;
            }
            let row = width.div_ceil(4) as usize * 2;
            for y in 0..3 {
                for x in 0..width as usize {
                    data.words[y * row + x / 2] |= (((x + y) % 16) as u16) << ((x % 2) * 8);
                }
            }
            let cooked = cook(&data);
            assert_eq!(cooked.format, Format::Indexed4);
            for y in 0..3 {
                for x in 0..width as usize {
                    let index = (cooked.words
                        [y * cooked.format.word_width(width) as usize + x / 4]
                        >> ((x % 4) * 4))
                        & 15;
                    assert_eq!(index as usize, (x + y) % 16);
                    assert_eq!(cooked.palette[index as usize], data.palette[index as usize]);
                }
            }
            data.palette[17] = 0x8123;
            let cooked = cook(&data);
            assert_eq!(cooked.format, Format::Indexed8);
            assert_eq!(cooked.words, data.words);
            assert_eq!(cooked.palette, data.palette);
        }
    }
    #[test]
    fn impossible_layout_fails_without_reducing_or_dropping_textures() {
        let mut scene = fixture(8);
        for data in scene.textures.values_mut() {
            let d = Arc::make_mut(data);
            d.width = 256;
            d.height = 256;
            d.words = vec![0x2030; 32768];
            d.palette[32] = 0x8123;
        }
        scene.display_size = [640, 480];
        assert!(layout(&scene).unwrap_err().contains("VRAM exhausted"));
        // The same eight 256x256 8bpp atlases fit beside 320-wide framebuffers,
        // which leave 320 more columns at every row to the textures.
        scene.display_size = [320, 240];
        let packed = layout(&scene).unwrap().textures;
        validate(&scene, &packed);
        assert!(packed.iter().any(|(_, p)| p.x < 640));
    }
    #[test]
    fn fonts_keep_their_placement_whatever_the_display_width() {
        let mut scene = fixture(4);
        let id = Uuid::from_u128(0xf0);
        scene.fonts.insert(
            id,
            Arc::new(crate::font_asset::FontData {
                width: 96,
                height: 40,
                words: vec![0; 24 * 40],
                palette: [0; 16],
                metrics: vec![],
                line_height: 16,
                baseline: 12,
                rgba: vec![],
            }),
        );
        scene.display_size = [640, 480];
        let wide = layout(&scene).unwrap();
        scene.display_size = [256, 240];
        let narrow = layout(&scene).unwrap();
        assert_eq!(wide.fonts, narrow.fonts);
        assert!(
            narrow
                .fonts
                .iter()
                .all(|(_, p)| p.x >= 640 && p.clut_x >= 640)
        );
    }
    #[test]
    #[ignore = "requires EPOK_TEXTURE_PROJECT and EPOK_TEXTURE_SCENE"]
    fn real_scene_cook_roundtrips_every_texel_and_fits_vram() {
        let root = std::path::PathBuf::from(std::env::var_os("EPOK_TEXTURE_PROJECT").unwrap());
        let path = root.join(std::env::var_os("EPOK_TEXTURE_SCENE").unwrap());
        let before = std::fs::read(&path).unwrap();
        let mut scene = Scene::load_unresolved(&path).unwrap();
        let index = crate::assets::scan(&root, &mut Default::default());
        crate::mesh::resolve(&mut scene, &index).unwrap();
        crate::terrain::resolve(&mut scene, &index).unwrap();
        crate::skeletal::resolve(&mut scene, &index).unwrap();
        crate::texture::resolve(&mut scene, &index).unwrap();
        let packed = layout(&scene).unwrap().textures;
        validate(&scene, &packed);
        let mut bytes = 0;
        let mut texels = 0;
        let mut four_bit = 0;
        // Simulate actual GPU uploads, then sample via page base and packed UVs.
        let mut vram = vec![0u16; 1024 * 512];
        for (id, p) in &packed {
            let data = &scene.textures[id];
            let cooked = cook(data);
            let row = cooked.format.word_width(data.width) as usize;
            for y in 0..data.height as usize {
                let dst = (p.y as usize + y) * 1024 + p.x as usize;
                vram[dst..dst + row].copy_from_slice(&cooked.words[y * row..(y + 1) * row]);
            }
            let clut = p.clut_y as usize * 1024 + p.clut_x as usize;
            vram[clut..clut + cooked.palette.len()].copy_from_slice(&cooked.palette);
            bytes += (cooked.words.len() + cooked.palette.len()) * 2;
            four_bit += usize::from(cooked.format == Format::Indexed4);
        }
        for (id, p) in &packed {
            let data = &scene.textures[id];
            let page = p.page(data);
            let source_row = data.width.div_ceil(4) as usize * 2;
            let ppw = p.format.texels_per_word() as usize;
            for y in 0..data.height as usize {
                for x in 0..data.width as usize {
                    let source = (data.words[y * source_row + x / 2] >> ((x % 2) * 8)) & 255;
                    let u = page.u as usize + x;
                    let v = page.v as usize + y;
                    let word = vram
                        [(p.y as usize / 256 * 256 + v) * 1024 + p.x as usize / 64 * 64 + u / ppw];
                    let cooked = (word >> ((u % ppw) * p.format.bits() as usize))
                        & (if ppw == 4 { 15 } else { 255 });
                    assert_eq!(source, cooked, "{id} at {x},{y}");
                    assert_eq!(
                        data.palette[source as usize],
                        vram[p.clut_y as usize * 1024 + p.clut_x as usize + cooked as usize]
                    );
                    texels += 1;
                }
            }
        }
        assert_eq!(std::fs::read(path).unwrap(), before);
        eprintln!(
            "textures={} four_bit={four_bit} bytes={bytes} checked_texels={texels}",
            packed.len()
        );
    }
}
