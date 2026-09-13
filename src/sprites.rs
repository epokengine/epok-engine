//! World-space sprites and generic, duration-based sprite sheet animation.
use crate::{scene::Scene, texture::BlendMode, transform::Matrix};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum Orientation {
    Fixed,
    #[default]
    Upright,
    Spherical,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Sprite {
    pub enabled: bool,
    pub texture: Option<Uuid>,
    /// Pixel rectangle x, y, width, height. Zero width/height means the full texture.
    pub region: [u16; 4],
    pub size: [f32; 2],
    pub pivot: [f32; 2],
    pub flip_x: bool,
    pub flip_y: bool,
    pub orientation: Orientation,
    pub color: [f32; 3],
    pub unlit: bool,
    pub blend: BlendMode,
    /// Quarter-world-unit ordering table bias; positive values draw farther away.
    pub depth_bias: i16,
}
impl Default for Sprite {
    fn default() -> Self {
        Self {
            enabled: true,
            texture: None,
            region: [0; 4],
            size: [1., 1.],
            pivot: [0.5, 0.],
            flip_x: false,
            flip_y: false,
            orientation: Orientation::Upright,
            color: [1.; 3],
            unlit: false,
            blend: BlendMode::Cutout,
            depth_bias: 0,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Frame {
    pub region: [u16; 4],
    pub duration: f32,
    pub event: u16,
}
impl Default for Frame {
    fn default() -> Self {
        Self {
            region: [0, 0, 16, 16],
            duration: 0.1,
            event: 0,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Clip {
    pub name: String,
    pub frames: Vec<Frame>,
    pub looping: bool,
}
impl Default for Clip {
    fn default() -> Self {
        Self {
            name: "Idle".into(),
            frames: vec![Frame::default()],
            looping: true,
        }
    }
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct SpriteSheet {
    pub clips: Vec<Clip>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct SpriteAnimator {
    pub sheet: SpriteSheet,
    pub clip: usize,
    pub playing: bool,
}
impl Default for SpriteAnimator {
    fn default() -> Self {
        Self {
            sheet: SpriteSheet {
                clips: vec![Clip::default()],
            },
            clip: 0,
            playing: true,
        }
    }
}

pub fn validate_sprite(s: &Sprite) -> Result<(), String> {
    if s.texture.is_some_and(|id| id.is_nil())
        || s.size
            .iter()
            .any(|v| !v.is_finite() || !(0.001..=128.).contains(v))
        || s.pivot
            .iter()
            .any(|v| !v.is_finite() || !(0. ..=1.).contains(v))
        || s.color
            .iter()
            .any(|v| !v.is_finite() || !(0. ..=1.).contains(v))
        || !(-511..=511).contains(&s.depth_bias)
    {
        return Err("Invalid Sprite size, pivot, color, texture or depth bias".into());
    }
    validate_region(s.region)
}
fn validate_region(r: [u16; 4]) -> Result<(), String> {
    if r[0] as u32 + r[2] as u32 > 256
        || r[1] as u32 + r[3] as u32 > 256
        || (r[2] == 0) != (r[3] == 0)
    {
        Err("Sprite atlas rectangle must fit a 256×256 texture page".into())
    } else {
        Ok(())
    }
}
pub fn validate(scene: &Scene) -> Result<(), String> {
    for e in &scene.entities {
        if let Some(s) = &e.sprite {
            validate_sprite(s)?;
            crate::texture::validate_region(s.texture, s.region, scene)?;
        }
        if let Some(a) = &e.sprite_animator {
            if e.sprite.is_none()
                || a.sheet.clips.is_empty()
                || a.sheet.clips.len() > 32
                || a.clip >= a.sheet.clips.len()
            {
                return Err(
                    "SpriteAnimator requires a Sprite and 1–32 clips with a valid selected clip"
                        .into(),
                );
            }
            let mut names = std::collections::BTreeSet::new();
            for c in &a.sheet.clips {
                if c.name.is_empty()
                    || c.name.len() > 64
                    || !names.insert(&c.name)
                    || c.frames.is_empty()
                    || c.frames.len() > 256
                {
                    return Err("Sprite clips require unique names and 1–256 frames".into());
                }
                for f in &c.frames {
                    validate_region(f.region)?;
                    crate::texture::validate_region(
                        e.sprite.as_ref().and_then(|s| s.texture),
                        f.region,
                        scene,
                    )?;
                    if !f.duration.is_finite() || !(1. / 60. ..=60.).contains(&f.duration) {
                        return Err(
                            "Sprite frame duration must be between 1/60 and 60 seconds".into()
                        );
                    }
                }
            }
        }
    }
    Ok(())
}
impl SpriteAnimator {
    pub fn frame_at(&self, seconds: f32) -> Option<&Frame> {
        let c = self.sheet.clips.get(self.clip)?;
        let total: f64 = c.frames.iter().map(|f| f.duration as f64).sum();
        if total <= 0. {
            return None;
        }
        let mut t = if self.playing {
            seconds.max(0.) as f64
        } else {
            0.
        };
        if c.looping {
            t %= total;
        }
        for f in &c.frames {
            if t < (f.duration as f64) {
                return Some(f);
            }
            t -= f.duration as f64;
        }
        c.frames.last()
    }
}

pub fn corners(s: &Sprite, world: Matrix, camera: [[f32; 3]; 3]) -> [[f32; 3]; 4] {
    let length = |v: [f32; 3]| v.iter().map(|v| v * v).sum::<f32>().sqrt();
    let mut right = camera[0];
    let mut up = camera[1];
    if s.orientation == Orientation::Upright {
        right[1] = 0.;
        let n = length(right);
        right = if n > 1e-6 {
            right.map(|v| v / n)
        } else {
            [1., 0., 0.]
        };
        up = [0., 1., 0.];
    }
    let scale = [
        length(world.vector([1., 0., 0.])),
        length(world.vector([0., 1., 0.])),
    ];
    [[0., 1.], [1., 1.], [1., 0.], [0., 0.]].map(|p| {
        let x = (p[0] - s.pivot[0]) * s.size[0];
        let y = (p[1] - s.pivot[1]) * s.size[1];
        if s.orientation == Orientation::Fixed {
            world.point([x, y, 0.])
        } else {
            let center = world.point([0.; 3]);
            std::array::from_fn(|i| center[i] + right[i] * x * scale[0] + up[i] * y * scale[1])
        }
    })
}
pub fn uv(s: &Sprite, width: u16, height: u16) -> [[f32; 2]; 4] {
    let [x, y, w, h] = s.region;
    let w = if w == 0 { width.saturating_sub(x) } else { w };
    let h = if h == 0 { height.saturating_sub(y) } else { h };
    [[0., 0.], [1., 0.], [1., 1.], [0., 1.]].map(|p| {
        [
            (x as f32 + if s.flip_x { 1. - p[0] } else { p[0] } * w.saturating_sub(1) as f32)
                / width.saturating_sub(1).max(1) as f32,
            (y as f32 + if s.flip_y { 1. - p[1] } else { p[1] } * h.saturating_sub(1) as f32)
                / height.saturating_sub(1).max(1) as f32,
        ]
    })
}
pub struct PreviewQuad {
    pub owner: usize,
    pub points: [[f32; 3]; 4],
    pub sprite: Sprite,
}
pub fn preview(scene: &Scene, camera: [[f32; 3]; 3], seconds: f32) -> Vec<PreviewQuad> {
    scene
        .entities
        .iter()
        .enumerate()
        .filter_map(|(owner, e)| {
            if !scene.is_active(owner) {
                return None;
            }
            let mut sprite = e.sprite.as_ref().filter(|s| s.enabled)?.clone();
            if let Some(frame) = e.sprite_animator.as_ref().and_then(|a| a.frame_at(seconds)) {
                sprite.region = frame.region;
            }
            Some(PreviewQuad {
                owner,
                points: corners(&sprite, scene.world_matrix(owner), camera),
                sprite,
            })
        })
        .collect()
}
pub fn cpp_sprite(s: &Sprite, texture_index: i32) -> String {
    let fixed = |v: f32| format!("Fixed({},Fixed::RAW)", (v * 4096.).round() as i32);
    format!(
        "Sprite{{{},{},{{{}}},{{{},{}}},{{{},{}}},{},{},SpriteOrientation::{:?},{{{}}},{},BlendMode::{:?},{}}}",
        s.enabled,
        texture_index,
        s.region.map(|v| v.to_string()).join(","),
        fixed(s.size[0]),
        fixed(s.size[1]),
        fixed(s.pivot[0]),
        fixed(s.pivot[1]),
        s.flip_x,
        s.flip_y,
        s.orientation,
        s.color
            .map(|v| ((v * 255.).round() as u8).to_string())
            .join(","),
        s.unlit,
        s.blend,
        s.depth_bias
    )
}
/// Returns global tables and initialization statements separately.
pub fn generate(scene: &Scene, texture_ids: &[Uuid]) -> (String, String) {
    let mut tables = String::new();
    let mut init = String::new();
    for (i, e) in scene.entities.iter().enumerate() {
        if let Some(s) = &e.sprite {
            init += &format!(
                "objects[{i}].sprite={};\n",
                cpp_sprite(
                    s,
                    s.texture
                        .and_then(|id| texture_ids.iter().position(|v| *v == id))
                        .map_or(-1, |v| v as i32)
                )
            );
        }
        if let Some(a) = &e.sprite_animator {
            for (ci, c) in a.sheet.clips.iter().enumerate() {
                tables += &format!("inline const SpriteFrame sprite_frames_{i}_{ci}[]={{");
                for f in &c.frames {
                    tables += &format!(
                        "{{{{{}}},Fixed({},Fixed::RAW),{}}},",
                        f.region.map(|v| v.to_string()).join(","),
                        (f.duration * 4096.).round() as i32,
                        f.event
                    );
                }
                tables += "};\n";
            }
            tables += &format!("inline const SpriteClip sprite_clips_{i}[]={{");
            for (ci, c) in a.sheet.clips.iter().enumerate() {
                tables += &format!(
                    "{{sprite_frames_{i}_{ci},{},{},{}}},",
                    c.frames.len(),
                    c.looping,
                    serde_json::to_string(&c.name).unwrap()
                );
            }
            tables += "};\n";
            init += &format!(
                "objects[{i}].sprite_animator.enabled=true;objects[{i}].sprite_animator.clips=sprite_clips_{i};objects[{i}].sprite_animator.clip_count={};objects[{i}].sprite_animator.play({});objects[{i}].sprite_animator.playing={};objects[{i}].sprite_animator.apply(objects[{i}].sprite);\n",
                a.sheet.clips.len(),
                a.clip,
                a.playing
            );
        }
    }
    (tables, init)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn billboard_preserves_vertical_axis_and_pivot() {
        let s = Sprite::default();
        let p = corners(
            &s,
            Matrix::IDENTITY,
            [[0.6, 0.7, 0.8], [0., 0.5, 0.5], [0., 0., 1.]],
        );
        assert_eq!(p[2][1], 0.);
        assert_eq!(p[0][1], 1.);
        assert!((p[1][0] - p[0][0] - 0.6).abs() < 0.0001);
    }
    #[test]
    fn clips_respect_durations_loop_and_final_frame() {
        let mut a = SpriteAnimator::default();
        a.sheet.clips[0].frames.push(Frame {
            duration: 0.3,
            event: 2,
            ..Frame::default()
        });
        assert_eq!(a.frame_at(0.15).unwrap().event, 2);
        assert_eq!(a.frame_at(0.41).unwrap().event, 0);
        a.sheet.clips[0].looping = false;
        assert_eq!(a.frame_at(9.).unwrap().event, 2);
    }
    #[test]
    fn atlas_flip_and_validation() {
        let mut s = Sprite {
            region: [16, 32, 16, 16],
            ..Sprite::default()
        };
        let u = uv(&s, 64, 64);
        s.flip_x = true;
        assert_eq!(uv(&s, 64, 64)[0], u[1]);
        s.region = [250, 0, 16, 16];
        assert!(validate_sprite(&s).is_err());
    }
}
