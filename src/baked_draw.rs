//! Offline packet attributes for static, unlit tracked scenery. This version
//! preserves the existing cooked face IDs and clipping semantics exactly.
use crate::{lighting::Quad, scene::Actor, texture::BlendMode};

pub const FACE_BYTES: usize = 32;
pub const ABI_VERSION: u32 = 3;
/// Face flags, in the high nibble of the TPAGE half of the second texture
/// word (bits the GPU ignores in a polygon's texpage attribute). The runtime
/// clears them before writing the word to a packet.
pub const FLAG_DOUBLE_SIDED: u16 = 0x1000;
pub const FLAG_SEMI_TRANSPARENT: u16 = 0x2000;
pub const FLAG_TEXTURED: u16 = 0x4000;
pub const FLAG_INDEXED_COLOR: u16 = 0x8000;

/// A face's texture resolved to VRAM at cook time: the page-relative UV
/// origin, and the CLUT and base texture page words the GPU expects. Blend
/// is added per face; display mode bits (dithering, field) at run time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResolvedTexture {
    pub page: crate::texture_cook::Page,
    pub clut: u16,
    pub tpage: u16,
}
impl ResolvedTexture {
    pub fn new(placement: crate::texture_cook::Placement, data: &crate::texture::Data) -> Self {
        let tpage = ((placement.x / 64) & 15)
            | (((placement.y / 256) & 1) << 4)
            | if placement.format == crate::texture_cook::Format::Indexed4 {
                0
            } else {
                0x80
            };
        Self {
            page: placement.page(data),
            clut: (placement.clut_y << 6) | (placement.clut_x / 16),
            tpage,
        }
    }
}

pub fn eligible(actor: &Actor, quads: &[Quad]) -> bool {
    // Lighting and scrolling still use the general path. Baking attributes for
    // them would silently freeze a dynamic value.
    actor.skeletal_mesh.is_none()
        // The face's immutable unlit flag must suffice even if a script
        // changes the instance material or lighting enabled flag later.
        && quads.iter().all(|q| q.material.unlit && q.material.uv_scroll == [0., 0.]
            // Coloured packet words are raw palette values. Nonwhite face tint
            // stays on the general path to preserve integer tint order exactly.
            && (q.vertex_colors.is_none() || q.material.color == [1.; 3]))
}

/// Layout (ABI 3): indices[4], color, shaded, then the three texture words
/// exactly as GT3/GT4 packets take them (UV0|CLUT, UV1|TPAGE with the face
/// flags in its high nibble, UV2|UV3), depth bias and topology.
pub fn encode_with_palette(
    q: &Quad,
    indices: [u16; 4],
    _actor: &Actor,
    texture: Option<ResolvedTexture>,
    palette: &crate::vertex_colors::Palette,
) -> [u8; FACE_BYTES] {
    let mut out = [0; FACE_BYTES];
    for (i, v) in indices.into_iter().enumerate() {
        out[i * 2..i * 2 + 2].copy_from_slice(&v.to_le_bytes());
    }
    let textured = texture.is_some();
    let channel = |v: f32| u32::from((v * 255.).round() as u8);
    let mut color = 0;
    let mut shaded = 0;
    for c in 0..3 {
        let mut value = channel(q.material.color[c]);
        shaded |= value << (c * 8);
        if textured {
            let m = value * 128 + 127;
            value = (m + 1 + (m >> 8)) >> 8;
        }
        color |= value << (c * 8);
    }
    let blend = match q.material.blend {
        BlendMode::Cutout => 0,
        BlendMode::Average => 1,
        BlendMode::Add => 2,
        BlendMode::Subtract => 3,
        BlendMode::AddQuarter => 4,
    };
    let command = if textured { 0x34000000 } else { 0x30000000 }
        | if blend != 0 && textured {
            0x02000000
        } else {
            0
        };
    out[8..12].copy_from_slice(&(color | command).to_le_bytes());
    out[12..16].copy_from_slice(&shaded.to_le_bytes());
    let mut flags = if q.material.double_sided {
        FLAG_DOUBLE_SIDED
    } else {
        0
    };
    if let Some(texture) = texture {
        for c in 0..4 {
            let at = [16, 20, 24, 26][c];
            out[at..at + 2].copy_from_slice(
                &crate::mesh_compile::packed_uv(q.uv[c], texture.page).to_le_bytes(),
            );
        }
        out[18..20].copy_from_slice(&texture.clut.to_le_bytes());
        let abr = if blend == 0 { 0 } else { blend as u16 - 1 };
        flags |= texture.tpage
            | (abr << 5)
            | FLAG_TEXTURED
            | if blend != 0 { FLAG_SEMI_TRANSPARENT } else { 0 };
    }
    if let Some(colors) = q.vertex_colors {
        for (i, index) in palette.indices(colors).into_iter().enumerate() {
            out[8 + i * 2..10 + i * 2].copy_from_slice(&index.to_le_bytes());
        }
        flags |= FLAG_INDEXED_COLOR;
    }
    out[22..24].copy_from_slice(&flags.to_le_bytes());
    out[28..30].copy_from_slice(&q.material.depth_bias.to_le_bytes());
    // A compile-time topology fact, not a screen-space facing/area decision.
    let topology: u16 = if indices[2] != indices[3] {
        0
    } else if indices[0] == indices[1] || indices[1] == indices[2] || indices[0] == indices[2] {
        2
    } else {
        1
    };
    out[30..32].copy_from_slice(&topology.to_le_bytes());
    out
}

#[cfg(test)]
pub fn encode(
    q: &Quad,
    indices: [u16; 4],
    actor: &Actor,
    texture: Option<ResolvedTexture>,
) -> [u8; FACE_BYTES] {
    encode_with_palette(
        q,
        indices,
        actor,
        texture,
        &crate::vertex_colors::Palette::quads([q]).unwrap(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn indexed_gouraud_reuses_uniform_words_without_expanding_faces() {
        let mut actor = Actor::cube("Gouraud".into());
        actor.material.unlit = true;
        let mut q = crate::lighting::quads(&actor)[0].clone();
        let colors = [[0, 128, 185], [255, 1, 127], [64, 128, 255], [10, 20, 30]];
        q.vertex_colors = Some(colors);
        let palette = crate::vertex_colors::Palette::quads([&q]).unwrap();
        let encoded = encode_with_palette(&q, [1, 2, 3, 4], &actor, None, &palette);
        assert_eq!(encoded.len(), 32);
        assert_ne!(
            u16::from_le_bytes(encoded[22..24].try_into().unwrap()) & FLAG_INDEXED_COLOR,
            0
        );
        for (i, index) in palette.indices(colors).into_iter().enumerate() {
            assert_eq!(
                u16::from_le_bytes(encoded[8 + i * 2..10 + i * 2].try_into().unwrap()),
                index
            );
        }
        assert!(eligible(&actor, &[q.clone()]));
        q.material.color = [0.5; 3];
        assert!(!eligible(&actor, &[q]));
    }
    #[test]
    fn compact_packet_attributes_match_generic_rounding() {
        let mut actor = Actor::cube("Bake".into());
        actor.material.unlit = true;
        actor.material.color = [0.25, 0.5, 1.];
        let mut q = crate::lighting::quads(&actor)[0].clone();
        q.material.color = [1., 0.5, 0.25];
        q.material.blend = BlendMode::AddQuarter;
        q.material.double_sided = true;
        q.material.unlit = true;
        q.material.depth_bias = -7;
        let texture = ResolvedTexture {
            page: crate::texture_cook::Page {
                width: 256,
                height: 128,
                u: 0,
                v: 0,
            },
            clut: 0x1234,
            tpage: 0x9d,
        };
        let data = encode(&q, [1, 2, 3, 3], &actor, Some(texture));
        assert_eq!(&data[..8], &[1, 0, 2, 0, 3, 0, 3, 0]);
        assert_eq!(
            u32::from_le_bytes(data[8..12].try_into().unwrap()),
            0x36204080
        );
        assert_eq!(
            u32::from_le_bytes(data[12..16].try_into().unwrap()),
            0x004080ff
        );
        let half = |at: usize| u16::from_le_bytes(data[at..at + 2].try_into().unwrap());
        // UV0|CLUT, UV1|TPAGE (+ AddQuarter's ABR 3 and the flags), UV2|UV3.
        for (c, at) in [16, 20, 24, 26].into_iter().enumerate() {
            assert_eq!(
                half(at),
                crate::mesh_compile::packed_uv(q.uv[c], texture.page)
            );
        }
        assert_eq!(half(18), 0x1234);
        assert_eq!(
            half(22),
            0x9d | 3 << 5 | FLAG_TEXTURED | FLAG_SEMI_TRANSPARENT | FLAG_DOUBLE_SIDED
        );
        assert_eq!(i16::from_le_bytes(data[28..30].try_into().unwrap()), -7);
        assert_eq!(u16::from_le_bytes(data[30..32].try_into().unwrap()), 1);
        for (indices, topology) in [([0, 1, 2, 3], 0u16), ([0, 0, 1, 1], 2), ([0, 1, 0, 0], 2)] {
            let bytes = encode(&q, indices, &actor, None);
            assert_eq!(
                u16::from_le_bytes(bytes[30..32].try_into().unwrap()),
                topology
            );
        }
        assert!(eligible(&actor, &[q.clone()]));
        q.material.uv_scroll[0] = 1.;
        assert!(!eligible(&actor, &[q]));
    }

    #[test]
    fn baked_command_byte_matches_all_material_variants_without_runtime_decoding() {
        let actor = Actor::cube("Commands".into());
        let mut q = crate::lighting::quads(&actor)[0].clone();
        for colors in [None, Some([[185, 128, 0]; 4])] {
            q.vertex_colors = colors;
            for texture in [
                None,
                Some(ResolvedTexture {
                    page: crate::texture_cook::Page {
                        width: 64,
                        height: 64,
                        u: 0,
                        v: 0,
                    },
                    clut: 0x7fc0,
                    tpage: 0x1f,
                }),
            ] {
                for blend in [
                    BlendMode::Cutout,
                    BlendMode::Average,
                    BlendMode::Add,
                    BlendMode::Subtract,
                    BlendMode::AddQuarter,
                ] {
                    q.material.blend = blend;
                    let data = encode(&q, [0, 1, 2, 2], &actor, texture);
                    let flags = u16::from_le_bytes(data[22..24].try_into().unwrap());
                    // The runtime's opcode: 0x30, +0x04 textured, +0x02 semi-transparent.
                    let opcode = 0x30
                        | if flags & FLAG_TEXTURED != 0 { 0x04 } else { 0 }
                        | if flags & FLAG_SEMI_TRANSPARENT != 0 {
                            0x02
                        } else {
                            0
                        };
                    let expected = if texture.is_none() {
                        0x30
                    } else if blend == BlendMode::Cutout {
                        0x34
                    } else {
                        0x36
                    };
                    assert_eq!(opcode, expected);
                    if colors.is_none() {
                        assert_eq!(data[11], expected);
                    }
                    assert_eq!(flags & FLAG_INDEXED_COLOR != 0, colors.is_some());
                    let abr = if texture.is_none() || blend == BlendMode::Cutout {
                        0
                    } else {
                        blend as u16 - 1
                    };
                    assert_eq!(flags >> 5 & 3, abr);
                    assert_eq!(data.len(), 32);
                }
            }
        }
    }
}
