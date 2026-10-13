//! Authored PSX Gouraud RGB8. Textured channels use 128 as neutral and may
//! brighten up to 255; untextured channels are direct RGB8. Material/lighting
//! tint multiplies these values, never converts them through display RGB.
use crate::{lighting::Quad, scene::Scene};
use std::collections::BTreeSet;

pub type Corners = [[u8; 3]; 4];
pub fn packed(rgb: [u8; 3]) -> u32 {
    u32::from(rgb[0]) | u32::from(rgb[1]) << 8 | u32::from(rgb[2]) << 16
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Palette(Vec<u32>);
impl Palette {
    pub fn quads<'a>(quads: impl IntoIterator<Item = &'a Quad>) -> Result<Self, String> {
        let colors = quads
            .into_iter()
            .filter_map(|q| q.vertex_colors)
            .flatten()
            .map(packed)
            .collect::<BTreeSet<_>>();
        if colors.len() > 65536 {
            return Err("Authored vertex colours exceed the 65536-entry RGB8 palette".into());
        }
        Ok(Self(colors.into_iter().collect()))
    }
    pub fn scene(scene: &Scene) -> Result<Self, String> {
        let quads = scene
            .actors
            .iter()
            .flat_map(crate::lighting::quads)
            .collect::<Vec<_>>();
        Self::quads(&quads)
    }
    pub fn indices(&self, colors: Corners) -> [u16; 4] {
        // All consumers use the palette of the complete resource scene. Missing
        // entries are an internal cooker mismatch, never an approximate colour.
        colors.map(|rgb| {
            self.0
                .binary_search(&packed(rgb))
                .expect("vertex palette/resource mismatch") as u16
        })
    }
    pub fn header(&self) -> String {
        let values = if self.0.is_empty() {
            "0".into()
        } else {
            self.0
                .iter()
                .map(|c| format!("{c}u"))
                .collect::<Vec<_>>()
                .join(",")
        };
        format!(
            "inline constexpr bool vertex_colors_enabled={};\ninline constexpr uint32_t vertex_color_palette[]={{{values}}};\n",
            !self.0.is_empty()
        )
    }
}
/// Same floor(a*b/255) as runtime::scale_channel; white tint is an identity.
pub fn tint(rgb: [u8; 3], shade: [u8; 3]) -> [u8; 3] {
    std::array::from_fn(|c| (u32::from(rgb[c]) * u32::from(shade[c]) / 255) as u8)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rgb8_palette_is_lossless_deterministic_and_tint_preserves_overbright() {
        let actor = crate::scene::Actor::cube("Colors".into());
        let mut a = crate::lighting::quads(&actor)[0].clone();
        a.vertex_colors = Some([[0, 128, 185], [255, 1, 127], [64, 128, 255], [0, 128, 185]]);
        let mut b = a.clone();
        b.vertex_colors.as_mut().unwrap().reverse();
        let palette = Palette::quads([&a, &b]).unwrap();
        assert_eq!(palette, Palette::quads([&b, &a]).unwrap());
        assert_eq!(palette.0.len(), 3);
        for rgb in a.vertex_colors.unwrap() {
            assert_eq!(tint(rgb, [255; 3]), rgb);
        }
        let indices = palette.indices(a.vertex_colors.unwrap());
        assert_eq!(
            indices.map(|i| palette.0[i as usize]),
            a.vertex_colors.unwrap().map(packed)
        );
        assert!(palette.header().contains("vertex_colors_enabled=true"));
        assert!(
            Palette::default()
                .header()
                .contains("vertex_colors_enabled=false")
        );
        for a in 0..=255u32 {
            for b in 0..=255u32 {
                assert_eq!(tint([a as u8; 3], [b as u8; 3]), [(a * b / 255) as u8; 3]);
            }
        }
    }
}
