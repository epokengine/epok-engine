//! SubViewport2D and SplitScreen2D records (World2D initiative, phase 4). Mirrors
//! `runtime/world2d_viewports.hpp`. A SubViewport2D's picture is 15-bit colour in VRAM:
//! the texture packer reserves it (see `texture_cook::layout`).
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const MAX_PICTURE: [u16; 2] = [256, 256];

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct SubViewport2D {
    /// An actor with a Camera2D; none draws through the active camera.
    pub camera: Option<Uuid>,
    /// Width (a multiple of 16) and height of the picture in pixels.
    pub size: [u16; 2],
    pub background: [u8; 3],
    /// Redraw every this many frames.
    pub interval: u8,
    /// Draw only this canvas layer (none: every world canvas).
    pub layer: Option<i8>,
    /// Hide `layer` from every other view, so only the picture shows it.
    pub hide_layer: bool,
    pub enabled: bool,
}
impl Default for SubViewport2D {
    fn default() -> Self {
        Self {
            camera: None,
            size: [128, 96],
            background: [0, 0, 0],
            interval: 1,
            layer: None,
            hide_layer: false,
            enabled: true,
        }
    }
}
impl SubViewport2D {
    /// The picture size the console uses: width rounded up to 16, both within 256.
    pub fn picture(&self) -> [u16; 2] {
        [
            self.size[0].clamp(16, MAX_PICTURE[0]).div_ceil(16) * 16,
            self.size[1].clamp(8, MAX_PICTURE[1]),
        ]
    }
    /// VRAM bytes the picture takes (15-bit colour).
    pub fn vram_bytes(&self) -> usize {
        let [w, h] = self.picture();
        usize::from(w) * usize::from(h) * 2
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct SplitScreen2D {
    /// Actors with a Camera2D: the left (or top) and the right (or bottom) view.
    pub cameras: [Option<Uuid>; 2],
    pub side_by_side: bool,
    pub divider: u8,
    pub divider_color: [u8; 3],
    pub enabled: bool,
}
impl Default for SplitScreen2D {
    fn default() -> Self {
        Self {
            cameras: [None, None],
            side_by_side: true,
            divider: 2,
            divider_color: [0, 0, 0],
            enabled: true,
        }
    }
}
/// Screen rectangle (x, y, w, h) of half `index`, as the runtime computes it.
pub fn half(split: &SplitScreen2D, display: [u16; 2], index: usize) -> [i32; 4] {
    let (w, h, gap) = (
        i32::from(display[0]),
        i32::from(display[1]),
        i32::from(split.divider),
    );
    if split.side_by_side {
        let left = (w - gap) / 2;
        if index == 0 {
            [0, 0, left, h]
        } else {
            [left + gap, 0, w - left - gap, h]
        }
    } else {
        let top = (h - gap) / 2;
        if index == 0 {
            [0, 0, w, top]
        } else {
            [0, top + gap, w, h - top - gap]
        }
    }
}

/// The pictures of a scene's SubViewport2D components: (component id, size).
pub fn pictures(scene: &crate::scene::Scene) -> Vec<(Uuid, [u16; 2])> {
    scene
        .actors
        .iter()
        .flat_map(|a| a.components.iter())
        .filter(|c| c.class.class_id.as_deref() == Some(crate::world2d::SUBVIEWPORT2D))
        .map(|c| (c.id, crate::world2d::read::<SubViewport2D>(c).picture()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pictures_round_their_width_to_sixteen() {
        let v = SubViewport2D {
            size: [100, 300],
            ..Default::default()
        };
        assert_eq!(v.picture(), [112, 256]);
        assert_eq!(v.vram_bytes(), 112 * 256 * 2);
    }

    #[test]
    fn halves_match_the_runtime() {
        let split = SplitScreen2D::default();
        assert_eq!(half(&split, [320, 240], 0), [0, 0, 159, 240]);
        assert_eq!(half(&split, [320, 240], 1), [161, 0, 159, 240]);
        let stacked = SplitScreen2D {
            side_by_side: false,
            ..split
        };
        assert_eq!(half(&stacked, [320, 240], 1), [0, 121, 320, 119]);
    }
}
