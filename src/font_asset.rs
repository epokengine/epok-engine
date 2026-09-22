//! Authored TrueType/OpenType fonts cooked to a 4bpp glyph atlas, metrics and a 16-entry CLUT.
//! Rasterization and packing are split so the packer, the encoder and the palette are testable
//! without a font file. Placement in VRAM and the runtime descriptor are emitted elsewhere.
use crate::{assets, import_settings::FontSettings};
use ab_glyph::{Font as _, ScaleFont as _};
use serde::{Deserialize, Serialize};
use std::path::Path;
use uuid::Uuid;

pub const IMPORTER_VERSION: u32 = 1;
pub const MAX_EDGE: usize = 256;
pub const OVERFLOW: &str = "Font atlas exceeds 256 px; reduce pixel height or character set";
const WIDTHS: [usize; 3] = [64, 128, 256];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlyphMetric {
    pub codepoint: u32,
    pub u: u8,
    pub v: u8,
    pub width: u8,
    pub height: u8,
    pub advance: u8,
    pub x_offset: i8,
    pub y_offset: i8,
}
#[derive(Clone, Debug, PartialEq)]
pub struct FontData {
    pub width: u16,
    pub height: u16,
    pub words: Vec<u16>,
    pub palette: [u16; 16],
    pub metrics: Vec<GlyphMetric>,
    pub line_height: u8,
    pub baseline: u8,
    pub rgba: Vec<u8>,
}
impl FontData {
    /// Atlas plus CLUT, the figure the import panel reports before committing.
    pub fn vram_bytes(&self) -> usize {
        self.words.len() * 2 + 32
    }
}
/// One rasterized glyph before packing: row-major 8-bit coverage, `width` by `height`.
/// Offsets are relative to the pen position on the baseline, y growing downward.
#[derive(Clone, Debug, PartialEq)]
pub struct RawGlyph {
    pub codepoint: u32,
    pub width: u8,
    pub height: u8,
    pub advance: u8,
    pub x_offset: i8,
    pub y_offset: i8,
    pub coverage: Vec<u8>,
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Rasterized {
    pub glyphs: Vec<RawGlyph>,
    pub line_height: u8,
    pub baseline: u8,
}

pub fn decode(source: &[u8], settings: &FontSettings) -> Result<FontData, String> {
    pack(&rasterize(source, settings)?, settings)
}
pub fn rasterize(source: &[u8], settings: &FontSettings) -> Result<Rasterized, String> {
    settings.validate()?;
    let font = ab_glyph::FontRef::try_from_slice(source)
        .map_err(|_| "Unsupported font file: import a TrueType or OpenType outline font")?;
    let scale = ab_glyph::PxScale::from(settings.pixel_height as f32);
    let scaled = font.as_scaled(scale);
    let mut glyphs = Vec::new();
    for c in settings.charset() {
        let id = font.glyph_id(c);
        // Unmapped codepoints resolve to .notdef; drop them rather than baking a box.
        if id.0 == 0 {
            continue;
        }
        let advance = scaled.h_advance(id).round().clamp(0., 255.) as u8;
        let (mut width, mut height, mut x_offset, mut y_offset) = (0u8, 0u8, 0i8, 0i8);
        let mut coverage = Vec::new();
        if let Some(outline) = font.outline_glyph(id.with_scale(scale)) {
            let bounds = outline.px_bounds();
            let (w, h) = (
                bounds.width().round().clamp(0., 255.) as usize,
                bounds.height().round().clamp(0., 255.) as usize,
            );
            if w > 0 && h > 0 {
                coverage = vec![0u8; w * h];
                outline.draw(|x, y, c| {
                    if let Some(slot) = coverage.get_mut(y as usize * w + x as usize) {
                        *slot = (c.clamp(0., 1.) * 255.).round() as u8;
                    }
                });
                (width, height) = (w as u8, h as u8);
                x_offset = bounds.min.x.round().clamp(-128., 127.) as i8;
                y_offset = bounds.min.y.round().clamp(-128., 127.) as i8;
            }
        }
        glyphs.push(RawGlyph {
            codepoint: c as u32,
            width,
            height,
            advance,
            x_offset,
            y_offset,
            coverage,
        });
    }
    if glyphs.is_empty() {
        return Err("The font has no glyphs for the selected characters".into());
    }
    if settings.monospace {
        // One cell for every glyph: the widest advance, or half the pixel height when the
        // face reports none.
        let mut cell = glyphs.iter().map(|g| g.advance).max().unwrap_or(0);
        if cell == 0 {
            cell = (settings.pixel_height / 2).clamp(1, 255) as u8;
        }
        for glyph in &mut glyphs {
            glyph.advance = cell;
        }
    }
    Ok(Rasterized {
        glyphs,
        line_height: (scaled.height() + scaled.line_gap())
            .round()
            .clamp(1., 255.) as u8,
        baseline: scaled.ascent().round().clamp(0., 255.) as u8,
    })
}
pub fn pack(raw: &Rasterized, settings: &FontSettings) -> Result<FontData, String> {
    settings.validate()?;
    if raw.glyphs.is_empty() {
        return Err("The font has no glyphs for the selected characters".into());
    }
    let pad = usize::from(settings.padding);
    let mut sorted = raw.glyphs.iter().collect::<Vec<_>>();
    sorted.sort_by_key(|g| (std::cmp::Reverse(g.height), g.codepoint));
    let (width, places, height) = WIDTHS
        .iter()
        .find_map(|&w| shelf(&sorted, w, pad).map(|(places, h)| (w, places, h)))
        .ok_or(OVERFLOW)?;
    let stride = width / 4;
    let mut words = vec![0u16; stride * height];
    let mut indices = vec![0u8; width * height];
    for (glyph, &(ox, oy)) in sorted.iter().zip(&places) {
        for y in 0..usize::from(glyph.height) {
            for x in 0..usize::from(glyph.width) {
                let index = quantize(
                    glyph.coverage[y * usize::from(glyph.width) + x],
                    settings.antialias,
                );
                if index == 0 {
                    continue;
                }
                let (ax, ay) = (ox + x, oy + y);
                indices[ay * width + ax] = index;
                words[ay * stride + ax / 4] |= u16::from(index) << ((ax % 4) * 4);
            }
        }
    }
    let palette = palette(settings.antialias);
    let mut rgba = Vec::with_capacity(width * height * 4);
    for &index in &indices {
        let color = palette[usize::from(index)];
        for k in 0..3 {
            rgba.push((((color >> (k * 5)) & 31) * 255 / 31) as u8);
        }
        rgba.push(if index == 0 { 0 } else { 255 });
    }
    let mut metrics = sorted
        .iter()
        .zip(&places)
        .map(|(g, &(ox, oy))| GlyphMetric {
            codepoint: g.codepoint,
            u: ox as u8,
            v: oy as u8,
            width: g.width,
            height: g.height,
            advance: g.advance,
            x_offset: g.x_offset,
            y_offset: g.y_offset,
        })
        .collect::<Vec<_>>();
    metrics.sort_by_key(|m| m.codepoint);
    metrics.dedup_by_key(|m| m.codepoint);
    Ok(FontData {
        width: width as u16,
        height: height as u16,
        words,
        palette,
        metrics,
        line_height: raw.line_height,
        baseline: raw.baseline,
        rgba,
    })
}
/// Tallest-first shelf placement. Returns each glyph's atlas origin and the used height,
/// or nothing when the set cannot fit this width within the 256 px ceiling.
fn shelf(glyphs: &[&RawGlyph], width: usize, pad: usize) -> Option<(Vec<(usize, usize)>, usize)> {
    let (mut x, mut y, mut shelf) = (0usize, 0usize, 0usize);
    let mut places = Vec::with_capacity(glyphs.len());
    for glyph in glyphs {
        let (w, h) = (usize::from(glyph.width), usize::from(glyph.height));
        if w == 0 || h == 0 {
            places.push((0, 0));
            continue;
        }
        if w + pad > width {
            return None;
        }
        if x + w + pad > width {
            (x, y, shelf) = (0, y + shelf, 0);
        }
        if y + h + pad > MAX_EDGE {
            return None;
        }
        places.push((x, y));
        x += w + pad;
        shelf = shelf.max(h + pad);
    }
    Some((places, (y + shelf).max(1)))
}
/// Index 0 is transparent. Without antialiasing, ink is the single white entry the built-in
/// font uses; with it, coverage spreads over a fifteen-step grey ramp.
fn quantize(coverage: u8, antialias: bool) -> u8 {
    if antialias {
        ((u32::from(coverage) * 15 + 127) / 255) as u8
    } else {
        u8::from(coverage >= 128)
    }
}
pub fn palette(antialias: bool) -> [u16; 16] {
    let mut palette = [0u16; 16];
    if antialias {
        for (i, entry) in palette.iter_mut().enumerate().skip(1) {
            let v = (i as u16) * 31 / 15;
            *entry = v | (v << 5) | (v << 10);
        }
    } else {
        palette[1] = 0x7fff;
    }
    palette
}
/// Header fragment for the cooked atlas. The Font descriptor carrying VRAM placement is
/// emitted by the export pass that allocates it, not here.
#[allow(dead_code)] // The atlas is emitted here; the export pass that places it arrives next.
pub fn header_fragment(data: &FontData, symbol: &str) -> String {
    let metrics = data
        .metrics
        .iter()
        .map(|m| {
            format!(
                "{{{},{},{},{},{},{},{},{}}}",
                m.codepoint, m.u, m.v, m.width, m.height, m.advance, m.x_offset, m.y_offset
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let list = |values: &[u16]| {
        values
            .iter()
            .map(u16::to_string)
            .collect::<Vec<_>>()
            .join(",")
    };
    format!(
        "inline constexpr GlyphMetric {symbol}_metrics[]={{{metrics}}};\nalignas(4) inline constexpr uint16_t {symbol}_pixels[]={{{}}};\ninline constexpr uint16_t {symbol}_palette[16]={{{}}};\n",
        list(&data.words),
        list(&data.palette)
    )
}
pub fn prepare(
    root: &Path,
    source: &str,
    destination: &str,
    settings: FontSettings,
    existing: Option<&assets::Record>,
    snapshot: bool,
) -> Result<assets::Candidate, String> {
    if existing.is_some_and(|r| r.meta.kind != assets::Kind::Font) {
        return Err("Reimport requires a Font asset".into());
    }
    let dest = assets::inside(root, destination)?;
    if dest.extension().is_none_or(|x| x != "epokasset") {
        return Err("Font destination must end in .epokasset".into());
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
        old.as_ref().ok_or("Missing font snapshot")?.source.clone()
    };
    decode(&bytes, &settings)?;
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
                kind: assets::Kind::Font,
                importer_version: IMPORTER_VERSION,
                source: if snapshot {
                    old.unwrap().meta.source
                } else {
                    source.replace('\\', "/")
                },
                source_hash: hash,
                settings: crate::import_settings::Settings::Font(settings),
                extra: Default::default(),
            },
            source: bytes,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::import_settings::CharRange;

    /// A solid block glyph: every pixel fully covered except a half-covered last column,
    /// which pins the antialias threshold.
    fn block(codepoint: u32, width: u8, height: u8) -> RawGlyph {
        let mut coverage = vec![255u8; usize::from(width) * usize::from(height)];
        for y in 0..usize::from(height) {
            coverage[y * usize::from(width) + usize::from(width) - 1] = 100;
        }
        RawGlyph {
            codepoint,
            width,
            height,
            advance: width + 1,
            x_offset: 0,
            y_offset: -i8::try_from(height).unwrap(),
            coverage,
        }
    }
    fn raw(glyphs: Vec<RawGlyph>) -> Rasterized {
        Rasterized {
            glyphs,
            line_height: 18,
            baseline: 14,
        }
    }
    fn settings() -> FontSettings {
        FontSettings {
            padding: 0,
            ..Default::default()
        }
    }
    fn pixel(data: &FontData, x: usize, y: usize) -> u8 {
        let stride = usize::from(data.width) / 4;
        ((data.words[y * stride + x / 4] >> ((x % 4) * 4)) & 0xf) as u8
    }

    #[test]
    fn packs_tallest_first_and_encodes_four_pixels_per_word() {
        let data = pack(
            &raw(vec![block(66, 4, 4), block(65, 8, 12), block(67, 4, 4)]),
            &settings(),
        )
        .unwrap();
        assert_eq!((data.width, data.height), (64, 12));
        assert_eq!(data.words.len(), 16 * 12);
        // The 12 px glyph takes the first shelf origin; the short ones follow it.
        assert_eq!(data.metrics[0].codepoint, 65);
        assert_eq!((data.metrics[0].u, data.metrics[0].v), (0, 0));
        assert_eq!((data.metrics[1].u, data.metrics[1].v), (8, 0));
        assert_eq!((data.metrics[2].u, data.metrics[2].v), (12, 0));
        // Low nibble first: pixel 0 is the low nibble of word 0.
        assert_eq!(data.words[0] & 0xf, 1);
        assert_eq!(pixel(&data, 7, 0), 0, "a half-covered column is not ink");
        assert_eq!(pixel(&data, 6, 0), 1);
        assert_eq!(pixel(&data, 16, 0), 0, "nothing is drawn past the glyphs");
        assert_eq!(data.rgba.len(), 64 * 12 * 4);
        assert_eq!(&data.rgba[..4], &[255, 255, 255, 255]);
    }
    #[test]
    fn metrics_are_sorted_and_deduplicated() {
        let data = pack(
            &raw(vec![block(90, 4, 4), block(65, 4, 4), block(90, 6, 4)]),
            &settings(),
        )
        .unwrap();
        assert_eq!(
            data.metrics.iter().map(|m| m.codepoint).collect::<Vec<_>>(),
            vec![65, 90]
        );
        assert_eq!((data.line_height, data.baseline), (18, 14));
        assert_eq!(data.metrics[0].y_offset, -4);
    }
    #[test]
    fn padding_separates_cells_and_grows_the_atlas() {
        let tight = pack(&raw(vec![block(65, 8, 8), block(66, 8, 8)]), &settings()).unwrap();
        assert_eq!((tight.metrics[1].u, tight.metrics[1].v), (8, 0));
        let padded = pack(
            &raw(vec![block(65, 8, 8), block(66, 8, 8)]),
            &FontSettings {
                padding: 2,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!((padded.metrics[1].u, padded.metrics[1].v), (10, 0));
        assert_eq!(padded.height, 10);
    }
    #[test]
    fn a_full_shelf_wraps_to_the_next_row() {
        let glyphs = (0..10).map(|i| block(65 + i, 16, 6)).collect::<Vec<_>>();
        let data = pack(&raw(glyphs), &settings()).unwrap();
        assert_eq!((data.width, data.height), (64, 18), "four per 64 px shelf");
        assert_eq!((data.metrics[3].u, data.metrics[3].v), (48, 0));
        assert_eq!((data.metrics[4].u, data.metrics[4].v), (0, 6));
    }
    #[test]
    fn the_atlas_widens_only_when_the_narrow_ones_overflow() {
        let wide = pack(&raw(vec![block(65, 100, 8)]), &settings()).unwrap();
        assert_eq!(wide.width, 128);
        // Forty 32x32 cells need nine shelves at 128 px but only five at 256 px.
        let tall = (0..40).map(|i| block(65 + i, 32, 32)).collect::<Vec<_>>();
        let data = pack(&raw(tall), &settings()).unwrap();
        assert_eq!((data.width, data.height), (256, 160));
    }
    #[test]
    fn an_oversized_set_is_rejected() {
        let glyphs = (0..70).map(|i| block(65 + i, 32, 32)).collect::<Vec<_>>();
        assert_eq!(pack(&raw(glyphs), &settings()), Err(OVERFLOW.into()));
        assert!(!pack(&raw(vec![]), &settings()).unwrap_err().is_empty());
    }
    #[test]
    fn the_palette_is_one_white_entry_or_a_grey_ramp() {
        let flat = pack(&raw(vec![block(65, 8, 8)]), &settings()).unwrap();
        assert_eq!(flat.palette[0], 0);
        assert_eq!(flat.palette[1], 0x7fff);
        assert_eq!(flat.palette[2..], [0; 14]);
        let smooth = pack(
            &raw(vec![block(65, 8, 8)]),
            &FontSettings {
                padding: 0,
                antialias: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(smooth.palette[0], 0);
        assert_eq!(smooth.palette[1], 0x0842);
        assert_eq!(smooth.palette[15], 0x7fff);
        assert!(smooth.palette.windows(2).skip(1).all(|w| w[0] < w[1]));
        // Coverage 100/255 lands on the sixth ramp step instead of disappearing.
        assert_eq!(pixel(&smooth, 7, 0), 6);
        assert_eq!(pixel(&smooth, 6, 0), 15);
    }
    #[test]
    fn a_blank_glyph_still_carries_its_advance() {
        let space = RawGlyph {
            codepoint: 32,
            width: 0,
            height: 0,
            advance: 5,
            x_offset: 0,
            y_offset: 0,
            coverage: Vec::new(),
        };
        let data = pack(&raw(vec![space, block(65, 6, 6)]), &settings()).unwrap();
        assert_eq!(data.metrics[0].codepoint, 32);
        assert_eq!(
            (data.metrics[0].advance, data.metrics[0].width),
            (5, 0),
            "space keeps its advance without occupying the atlas"
        );
        assert_eq!(data.height, 6);
    }
    #[test]
    fn settings_outside_their_range_are_rejected() {
        let bad = |s: FontSettings| pack(&raw(vec![block(65, 4, 4)]), &s).unwrap_err();
        assert!(
            bad(FontSettings {
                pixel_height: 5,
                ..Default::default()
            })
            .contains("pixel height")
        );
        assert!(
            bad(FontSettings {
                pixel_height: 65,
                ..Default::default()
            })
            .contains("pixel height")
        );
        assert!(
            bad(FontSettings {
                padding: 5,
                ..Default::default()
            })
            .contains("padding")
        );
        assert!(
            bad(FontSettings {
                ranges: Vec::new(),
                ..Default::default()
            })
            .contains("character")
        );
        assert!(
            bad(FontSettings {
                version: 2,
                ..Default::default()
            })
            .contains("version")
        );
    }
    #[test]
    fn the_default_character_set_covers_ascii_and_the_spanish_glyphs() {
        let set = FontSettings::default().charset();
        assert_eq!(set.len(), 95 + 16);
        assert!(set.contains(&' ') && set.contains(&'~') && set.contains(&'ñ'));
        assert!(!set.contains(&'\n'));
        let extra = FontSettings {
            characters: "€\u{7}".into(),
            ranges: vec![CharRange::Digits],
            ..Default::default()
        }
        .charset();
        assert_eq!(extra.iter().copied().collect::<String>(), "0123456789€");
        let latin = FontSettings {
            ranges: vec![CharRange::Latin1Supplement],
            ..Default::default()
        }
        .charset();
        assert_eq!(latin.len(), 96);
        assert!(latin.contains(&'\u{a0}') && latin.contains(&'ÿ'));
    }
    #[test]
    fn the_header_fragment_lists_metrics_pixels_and_palette() {
        let data = pack(&raw(vec![block(65, 4, 4)]), &settings()).unwrap();
        let text = header_fragment(&data, "title");
        assert!(
            text.contains("inline constexpr GlyphMetric title_metrics[]={{65,0,0,4,4,5,0,-4}};")
        );
        assert!(text.contains("alignas(4) inline constexpr uint16_t title_pixels[]={"));
        assert!(text.contains("inline constexpr uint16_t title_palette[16]={0,32767,0,"));
        assert_eq!(text.matches(';').count(), 3);
        assert_eq!(data.vram_bytes(), data.words.len() * 2 + 32);
    }
    /// End-to-end rasterization needs a real outline font. Point EPOK_TEST_FONT at a TTF
    /// or OTF and run `cargo test font -- --ignored` to exercise it.
    #[test]
    #[ignore = "set EPOK_TEST_FONT to a TrueType or OpenType file"]
    fn a_real_font_rasterizes_into_an_atlas() {
        let path = std::env::var("EPOK_TEST_FONT").expect("set EPOK_TEST_FONT");
        let bytes = std::fs::read(&path).unwrap();
        let data = decode(&bytes, &FontSettings::default()).unwrap();
        assert!(data.metrics.len() > 60);
        assert!(data.width >= 64 && data.height > 0 && data.height <= 256);
        assert_eq!(
            data.words.len(),
            usize::from(data.width) / 4 * usize::from(data.height)
        );
        assert_eq!(
            data.rgba.len(),
            usize::from(data.width) * usize::from(data.height) * 4
        );
        assert!(data.baseline > 0 && data.line_height >= data.baseline);
        assert!(
            data.metrics
                .windows(2)
                .all(|w| w[0].codepoint < w[1].codepoint)
        );
        assert!(decode(b"not a font", &FontSettings::default()).is_err());
    }
}
