//! Native HUD commands share their compiler with the console packet renderer.
use crate::{hud, scene::Scene};

/// Native preview protocol, mirrored in `native/hud_preview.h`. Both sides check
/// it, so a cached executable built from an older header is rejected instead of
/// being misread. Bump on any frame-header or command-record change.
pub const HUD_PREVIEW_MAGIC: u32 = 0x3144_5548; // "HUD1"
pub const HUD_PREVIEW_PROTOCOL_VERSION: u32 = 3;
/// The child runs gameplay: BeginPlay/start, update, frame_update. Clear in the
/// edit phase, which only calls `editor_preview` construction hooks.
pub const HUD_PREVIEW_CAP_SIMULATE: u32 = 1;
/// Blueprint classes execute. The native preview never sets it: it compiles C++
/// controllers only, so Blueprint logic is not simulated here. Read by
/// `Frame::blueprint_support`, which the UI consults once the preview surfaces
/// the capability list; the constant is part of the protocol regardless.
#[allow(dead_code)]
pub const HUD_PREVIEW_CAP_BLUEPRINT: u32 = 2;

#[repr(C)]
struct Node {
    parent: i32,
    flags: i32,
    rect: [i32; 10],
    texture: i32,
    image_color: [i32; 3],
    region: [i32; 4],
    borders: [i32; 4],
    text_color: [i32; 3],
    progress: [i32; 7],
    /// Horizontal flags, vertical flags, minimum x/y and stretch in Q12.
    layout_element: [i32; 5],
    /// Kind, spacing x/y, padding left/top/right/bottom, columns.
    layout_container: [i32; 8],
    text: [u8; 512],
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Command(pub [i32; 16]);
unsafe extern "C" {
    fn epok_hud_compile(
        nodes: *const Node,
        count: u32,
        dimensions: *const i32,
        textures: u32,
        width: i32,
        height: i32,
        budget: *const u32,
        commands: *mut Command,
        capacity: u32,
        stats: *mut u32,
    ) -> u32;
    #[allow(dead_code)]
    fn epok_hud_resolve(parent: *const i32, rect: *const i32, result: *mut i32);
    fn epok_hud_layout(
        nodes: *const Node,
        count: u32,
        width: i32,
        height: i32,
        rects: *mut i32,
    ) -> u32;
}
fn fixed(v: f32) -> i32 {
    (f64::from(v) * 4096.).round() as i32
}
fn color(v: [f32; 3]) -> [i32; 3] {
    v.map(|v| (v.clamp(0., 1.) * 255.).round() as i32)
}
fn rect(r: &hud::RectTransform) -> [i32; 10] {
    let mut values = [0; 10];
    for (out, input) in
        values
            .chunks_exact_mut(2)
            .zip([r.anchor_min, r.anchor_max, r.pivot, r.position, r.size])
    {
        out.copy_from_slice(&input.map(fixed));
    }
    values
}
/// One rect against one parent, the anchor math on its own. `layouts` replaced it
/// as the viewport's query, but it stays as the pin on the Q12 boundary the whole
/// protocol is quantized to, and as the single-rect entry any host can call.
#[allow(dead_code)]
pub fn resolve(parent: hud::Rect, r: &hud::RectTransform) -> hud::Rect {
    let mut result = [0; 4];
    unsafe {
        epok_hud_resolve(
            parent.map(fixed).as_ptr(),
            rect(r).as_ptr(),
            result.as_mut_ptr(),
        );
    }
    result.map(|v| v as f32 / 4096.)
}
/// One wire node per actor, in actor order. `ids` is the texture bank the image
/// indices address. Shared by the command compiler and the layout query so the
/// editor never marshals the scene two different ways.
fn nodes(scene: &Scene, ids: &[uuid::Uuid]) -> Vec<Node> {
    scene
        .actors
        .iter()
        .map(|e| {
            let mut n = Node {
                parent: e.parent.map_or(-1, |p| p as i32),
                flags: 1 | if e.active { 2 } else { 0 },
                rect: rect(&e.rect.clone().unwrap_or_default()),
                texture: -1,
                image_color: [0; 3],
                region: [0; 4],
                borders: [0; 4],
                text_color: [0; 3],
                progress: [0; 7],
                layout_element: [1, 1, 0, 0, 0],
                layout_container: [0, 0, 0, 0, 0, 0, 0, 1],
                text: [0; 512],
            };
            if e.canvas.as_ref().is_some_and(|c| c.enabled) {
                n.flags |= 4;
            }
            if e.rect.is_some() {
                n.flags |= 8;
            }
            if let Some(c) = &e.image {
                if c.enabled {
                    n.flags |= 16;
                }
                n.texture = c
                    .texture
                    .and_then(|id| ids.iter().position(|v| *v == id))
                    .map_or(-1, |i| i as i32);
                n.image_color = color(c.color);
                n.region = c.region.map(i32::from);
                n.borders = c.borders.map(i32::from);
            }
            if let Some(c) = &e.text {
                if c.enabled {
                    n.flags |= 32;
                }
                if c.wrap {
                    n.flags |= 128;
                }
                n.text_color = color(c.color);
                let bytes = c.text.as_bytes();
                let end = bytes.len().min(511);
                n.text[..end].copy_from_slice(&bytes[..end]);
            }
            if let Some(c) = &e.progress {
                if c.enabled {
                    n.flags |= 64;
                }
                n.progress[0] = fixed(c.value);
                n.progress[1..4].copy_from_slice(&color(c.color));
                n.progress[4..7].copy_from_slice(&color(c.background));
            }
            if let Some(c) = &e.layout_element {
                if c.enabled {
                    n.flags |= 256;
                }
                n.layout_element = [
                    i32::from(c.horizontal),
                    i32::from(c.vertical),
                    fixed(c.minimum[0]),
                    fixed(c.minimum[1]),
                    fixed(c.stretch),
                ];
            }
            if let Some(c) = &e.layout_container {
                if c.enabled {
                    n.flags |= 512;
                }
                n.layout_container[0] = i32::from(c.kind as u8);
                n.layout_container[1..3].copy_from_slice(&c.spacing.map(fixed));
                n.layout_container[3..7].copy_from_slice(&c.padding.map(fixed));
                n.layout_container[7] = i32::from(c.columns);
            }
            n
        })
        .collect()
}
/// Every actor's resolved rect, or `None` where the actor carries no Canvas and
/// no RectTransform.
///
/// The gizmos answer "where does this element sit", so what only hides a subtree
/// is ignored here: a disabled Canvas still reports rects, and so does an
/// inactive element that places itself from its own anchors, exactly as the
/// anchor-chain walk this replaced did. Under a container `active` stops being
/// visibility and becomes geometry — it decides whether the child takes a cell —
/// so there the authored flag is passed through and the outlines keep agreeing
/// with the pixels the same core produces.
pub fn layouts(scene: &Scene) -> Vec<Option<hud::Rect>> {
    let ids = crate::texture::ids(scene);
    let mut nodes = nodes(scene, &ids);
    for (i, n) in nodes.iter_mut().enumerate() {
        let parent = scene.spatial_parent(i);
        n.parent = parent.map_or(-1, |p| p as i32);
        if !parent
            .and_then(|p| scene.actors[p].layout_container.as_ref())
            .is_some_and(|c| c.enabled && c.kind != hud::LayoutKind::None)
        {
            n.flags |= 2;
        }
        if scene.actors[i].canvas.is_some() {
            n.flags |= 4;
        }
    }
    let mut out = vec![0; nodes.len() * 4];
    unsafe {
        epok_hud_layout(
            nodes.as_ptr(),
            nodes.len() as u32,
            i32::from(scene.display_size[0]),
            i32::from(scene.display_size[1]),
            out.as_mut_ptr(),
        );
    }
    scene
        .actors
        .iter()
        .enumerate()
        .map(|(i, e)| {
            let q12 = [out[i * 4], out[i * 4 + 1], out[i * 4 + 2], out[i * 4 + 3]];
            // The core zeroes a node it did not lay out. An element that really
            // resolved to an empty rect at the origin has no rectangle on screen
            // either, so both answer the caller the same way.
            ((e.canvas.is_some() || e.rect.is_some()) && q12 != [0; 4])
                .then(|| q12.map(|v| v as f32 / 4096.))
        })
        .collect()
}
pub fn compile(scene: &Scene) -> (Vec<Command>, [u32; 5]) {
    let ids = crate::texture::ids(scene);
    let dimensions: Vec<i32> = ids
        .iter()
        .flat_map(|id| {
            scene
                .textures
                .get(id)
                .map_or([0, 0], |t| [i32::from(t.width), i32::from(t.height)])
        })
        .collect();
    let nodes = nodes(scene, &ids);
    let b = &scene.hud_budget;
    let budget = [
        b.layouts as u32,
        b.rectangles as u32,
        b.texts as u32,
        b.glyphs as u32,
    ];
    let mut commands = vec![Command::default(); b.rectangles + b.glyphs];
    let mut stats = [0; 5];
    let count = unsafe {
        epok_hud_compile(
            nodes.as_ptr(),
            nodes.len() as u32,
            dimensions.as_ptr(),
            ids.len() as u32,
            i32::from(scene.display_size[0]),
            i32::from(scene.display_size[1]),
            budget.as_ptr(),
            commands.as_mut_ptr(),
            commands.len() as u32,
            stats.as_mut_ptr(),
        )
    };
    commands.truncate(count as usize);
    (commands, stats)
}

pub fn render(scene: &Scene, commands: &[Command], seconds: f32, fade: u8) -> Vec<u8> {
    render_with_background(scene, commands, seconds, fade, true)
}

/// Transparent HUD layer for Native PC Play. Geometry is rendered by the host
/// scene backend; this layer contains only console-quantized HUD pixels and fade.
pub fn render_overlay(scene: &Scene, commands: &[Command], seconds: f32, fade: u8) -> Vec<u8> {
    render_with_background(scene, commands, seconds, fade, false)
}

fn render_with_background(
    scene: &Scene,
    commands: &[Command],
    seconds: f32,
    fade: u8,
    opaque_background: bool,
) -> Vec<u8> {
    let [width, height] = scene.display_size.map(usize::from);
    let mut pixels = if opaque_background {
        [33, 40, 52, 255].repeat(width * height)
    } else {
        [0, 0, 0, fade].repeat(width * height)
    };
    let ids = crate::texture::ids(scene);
    let textures: Vec<_> = ids
        .iter()
        .map(|id| {
            scene
                .textures
                .get(id)
                .map(|t| (t, crate::palette::preview_rgba(scene, *id, seconds, t)))
        })
        .collect();
    for Command(c) in commands {
        let [x0, y0, x1, y1] = [c[3], c[4], c[5], c[6]];
        if x1 <= x0 || y1 <= y0 {
            continue;
        }
        let rgb = [c[11], c[12], c[13]].map(|v| v.clamp(0, 255) as u8);
        let glyph = if c[0] == 2 {
            let ch = if c[2] <= 126 {
                char::from_u32(c[2] as u32)
            } else {
                crate::bitmap_font::EXTRA.chars().nth((c[2] - 127) as usize)
            };
            ch.and_then(crate::bitmap_font::glyph)
        } else {
            None
        };
        for y in y0.max(0)..y1.min(height as i32) {
            for x in x0.max(0)..x1.min(width as i32) {
                let result = match c[0] {
                    0 => Some(rgb),
                    1 => textures
                        .get(c[2] as usize)
                        .and_then(|t| t.as_ref())
                        .and_then(|(t, rgba)| {
                            // Source bounds are inclusive, as in PSX quad packets.
                            // Nearest sampling; GPU subpixel edge rules are validated
                            // separately from the shared command/layout contract.
                            let u = c[7]
                                + ((i64::from(x - x0) * 2 + 1) * i64::from(c[9] - c[7] + 1)
                                    / (i64::from(x1 - x0) * 2))
                                    as i32;
                            let v = c[8]
                                + ((i64::from(y - y0) * 2 + 1) * i64::from(c[10] - c[8] + 1)
                                    / (i64::from(y1 - y0) * 2))
                                    as i32;
                            if u < 0 || v < 0 || u >= i32::from(t.width) || v >= i32::from(t.height)
                            {
                                return None;
                            }
                            let offset = (v as usize * usize::from(t.width) + u as usize) * 4;
                            if rgba[offset + 3] == 0 {
                                return None;
                            }
                            Some(std::array::from_fn(|i| {
                                // Texture modulation is an integer 7-bit gain on PSX.
                                let gain = u32::from(rgb[i]).div_ceil(2);
                                let five =
                                    ((u32::from(rgba[offset + i]) >> 3) * gain / 128).min(31) as u8;
                                (five << 3) | (five >> 2)
                            }))
                        }),
                    2 => glyph.and_then(|g| {
                        let u = c[7] + x - x0;
                        let v = c[8] + y - y0;
                        if !(0..8).contains(&u)
                            || !(0..16).contains(&v)
                            || g[v as usize] & (1 << u) == 0
                        {
                            None
                        } else {
                            Some(rgb)
                        }
                    }),
                    _ => None,
                };
                if let Some(rgb) = result {
                    let offset = (y as usize * width + x as usize) * 4;
                    pixels[offset..offset + 3].copy_from_slice(&rgb);
                    pixels[offset + 3] = 255;
                }
            }
        }
    }
    if fade > 0 {
        for p in pixels.chunks_exact_mut(4) {
            for c in &mut p[..3] {
                *c = (u16::from(*c) * u16::from(255 - fade) / 255) as u8;
            }
        }
    }
    pixels
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_layout_quantizes_like_console() {
        let r = hud::RectTransform {
            position: [0.0002, -0.0002],
            ..Default::default()
        };
        let out = resolve([0., 0., 640., 480.], &r);
        assert_eq!(out, [270. + 1. / 4096., 224. - 1. / 4096., 100., 32.]);
    }
    /// `Node` mirrors `EpokHudNode` byte for byte. The C side is not visible from
    /// Rust, so the field count and the total size are pinned here: a record added
    /// on one side without the other would otherwise be read as garbage.
    #[test]
    fn the_wire_node_matches_the_protocol_it_declares() {
        const FIELDS: usize = 1 + 1 + 10 + 1 + 3 + 4 + 4 + 3 + 7 + 5 + 8;
        assert_eq!(size_of::<Node>(), FIELDS * 4 + 512);
        assert_eq!(align_of::<Node>(), 4);
        assert_eq!(HUD_PREVIEW_PROTOCOL_VERSION, 3);
    }
    #[test]
    fn dynamic_budget_and_disabled_parent_match_console() {
        let mut s = Scene::default();
        s.actors.clear();
        let mut root = crate::scene::Actor::cube("Canvas".into());
        root.kind = "Empty".into();
        root.canvas = Some(Default::default());
        s.actors.push(root);
        for _ in 0..3 {
            let mut e = crate::scene::Actor::cube("Image".into());
            e.kind = "Empty".into();
            e.parent = Some(0);
            e.rect = Some(Default::default());
            e.image = Some(Default::default());
            s.actors.push(e);
        }
        s.hud_budget.rectangles = 2;
        let (commands, stats) = compile(&s);
        assert_eq!(commands.len(), 2);
        assert_eq!(stats, [2, 0, 0, 0, 1]);
        s.actors[0].active = false;
        assert!(compile(&s).0.is_empty());
    }
}
