//! Sculpt and paint brush math shared by the blockout vertex brush and the
//! terrain heightmap brush. One falloff profile keeps a stroke feeling the same
//! in both tools, and keeps the arithmetic deterministic so tests can assert on
//! exact displacements rather than on screenshots.
use serde::{Deserialize, Serialize};

/// Radial weight profile across the brush disc.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Falloff {
    /// Smoothstep: flat centre, soft rim. The default for terrain sculpting.
    #[default]
    Smooth,
    Linear,
    /// Concentrates the stroke near the centre; good for ridges and gullies.
    Sharp,
    /// No falloff. The whole disc moves together, which is how plateaus and
    /// hard-edged platforms are cut.
    Constant,
}
impl Falloff {
    /// `t` is the normalized distance from the brush centre: 0 at the centre,
    /// 1 at the rim. Out-of-range input is clamped so callers may pass raw
    /// ratios without guarding first.
    pub fn weight(self, t: f32) -> f32 {
        let t = if t.is_finite() { t.clamp(0., 1.) } else { 1. };
        match self {
            Self::Smooth => {
                let u = 1. - t;
                u * u * (3. - 2. * u)
            }
            Self::Linear => 1. - t,
            Self::Sharp => {
                let u = 1. - t;
                u * u * u
            }
            Self::Constant => 1.,
        }
    }
    pub const ALL: [Self; 4] = [Self::Smooth, Self::Linear, Self::Sharp, Self::Constant];
    pub fn label(self) -> &'static str {
        match self {
            Self::Smooth => "Smooth",
            Self::Linear => "Linear",
            Self::Sharp => "Sharp",
            Self::Constant => "Constant",
        }
    }
}

/// What a stroke does to the samples under it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    #[default]
    Raise,
    Lower,
    /// Averages each sample with its neighbours; strength is the blend amount.
    Smooth,
    /// Pulls samples towards `reference`, the height picked when the stroke began.
    Flatten,
    /// Drives samples towards `target` regardless of where they started.
    Set,
    /// Adds deterministic value noise scaled by strength.
    Noise,
    /// Assigns the selected atlas tile. Terrain only; the mesh brush ignores it.
    Paint,
}
impl Mode {
    pub const SCULPT: [Self; 6] = [
        Self::Raise,
        Self::Lower,
        Self::Smooth,
        Self::Flatten,
        Self::Set,
        Self::Noise,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Raise => "Raise",
            Self::Lower => "Lower",
            Self::Smooth => "Smooth",
            Self::Flatten => "Flatten",
            Self::Set => "Set",
            Self::Noise => "Noise",
            Self::Paint => "Paint",
        }
    }
    /// Whether the stroke needs a height sampled at the press, which is what
    /// makes Flatten level to the surface actually clicked on.
    pub fn samples_reference(self) -> bool {
        matches!(self, Self::Flatten)
    }
}

pub const MIN_RADIUS: f32 = 0.25;
pub const MAX_RADIUS: f32 = 64.;
pub const MAX_STRENGTH: f32 = 8.;
/// Upper bound on interpolated samples for one mouse move. A fast drag across
/// the whole map must not turn into an unbounded loop.
pub const MAX_STEPS: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Brush {
    pub mode: Mode,
    pub falloff: Falloff,
    pub radius: f32,
    /// World units per applied sample for the height modes, blend fraction for
    /// Smooth. Always positive; Lower negates it internally.
    pub strength: f32,
    /// Absolute height used by Set.
    pub target: f32,
    /// Height picked under the cursor when the stroke began, used by Flatten.
    pub reference: f32,
    /// Atlas tile assigned by Paint.
    pub tile: u8,
    /// Quarter-turn applied to painted tiles. Four means "scatter": a
    /// deterministic per-cell rotation, which is what stops a repeating ground
    /// texture from reading as a grid.
    pub tile_rotation: u8,
    pub seed: u32,
}
impl Default for Brush {
    fn default() -> Self {
        Self {
            mode: Mode::Raise,
            falloff: Falloff::Smooth,
            // Small and gentle by default: detail passes are far more common
            // than blocking out, and a wide strong brush is one drag from a
            // mess. Widen with `]` when roughing in.
            radius: 1.,
            strength: 0.3,
            target: 0.,
            reference: 0.,
            tile: 0,
            tile_rotation: 0,
            seed: 1,
        }
    }
}
impl Brush {
    /// Weight at a horizontal offset from the brush centre. Returns zero
    /// outside the disc so callers can iterate a bounding box and skip.
    pub fn weight(&self, dx: f32, dz: f32) -> f32 {
        if self.radius <= 0. {
            return 0.;
        }
        let distance = (dx * dx + dz * dz).sqrt();
        if distance > self.radius {
            return 0.;
        }
        self.falloff.weight(distance / self.radius)
    }
    /// Signed height delta this brush applies to a sample at `weight`, given
    /// the sample's current height. Smooth and Paint are handled by the caller
    /// because they need neighbours or a separate grid.
    pub fn delta(&self, weight: f32, current: f32) -> f32 {
        if weight <= 0. {
            return 0.;
        }
        match self.mode {
            Mode::Raise => self.strength * weight,
            Mode::Lower => -self.strength * weight,
            Mode::Flatten => (self.reference - current) * weight * self.strength.min(1.),
            Mode::Set => (self.target - current) * weight * self.strength.min(1.),
            Mode::Smooth | Mode::Noise | Mode::Paint => 0.,
        }
    }
    /// Rotation for a painted cell. Fixed below four, deterministic scatter at
    /// four, so a repainted cell always lands the same way.
    pub fn rotation_for(&self, i: u16, j: u16) -> u8 {
        if self.tile_rotation < 4 {
            return self.tile_rotation & 3;
        }
        let value = noise(i32::from(i), i32::from(j), self.seed ^ 0x5F35_6495);
        (((value + 1.) * 2.) as u8) & 3
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.tile >= 64 || self.tile_rotation > 4 {
            return Err("Brush tile must be below 64 and rotation between 0 and 4".into());
        }
        if !self.radius.is_finite() || !(MIN_RADIUS..=MAX_RADIUS).contains(&self.radius) {
            return Err(format!(
                "Brush radius must be between {MIN_RADIUS} and {MAX_RADIUS}"
            ));
        }
        if !self.strength.is_finite() || !(0. ..=MAX_STRENGTH).contains(&self.strength) {
            return Err(format!(
                "Brush strength must be between 0 and {MAX_STRENGTH}"
            ));
        }
        if !self.target.is_finite() || !self.reference.is_finite() {
            return Err("Brush heights must be finite".into());
        }
        Ok(())
    }
}

/// Deterministic value noise in -1..1. Seeded integer hashing rather than a
/// random generator, so a Noise stroke replays identically in a test and in a
/// rebuilt project.
pub fn noise(x: i32, z: i32, seed: u32) -> f32 {
    let mut h = (x as u32).wrapping_mul(0x9E37_79B1)
        ^ (z as u32).wrapping_mul(0x85EB_CA77)
        ^ seed.wrapping_mul(0xC2B2_AE3D);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2545_F491);
    h ^= h >> 13;
    h = h.wrapping_mul(0x27D4_EB2F);
    h ^= h >> 16;
    f32::from(((h >> 8) & 0xFFFF) as u16) / 32768. - 1.
}

/// Press-drag-release state for a viewport stroke. The editor only has a
/// click gesture; a brush needs the intermediate positions too, otherwise a
/// fast drag leaves a dotted line instead of a continuous stroke.
#[derive(Clone, Debug, Default)]
pub struct Stroke {
    active: bool,
    last: Option<[f32; 3]>,
    /// Samples applied since the stroke began, for diagnostics and tests.
    pub applied: u32,
}
impl Stroke {
    pub fn active(&self) -> bool {
        self.active
    }
    /// Begin a stroke at a world point. Returns the single sample to apply.
    pub fn begin(&mut self, point: [f32; 3]) -> Vec<[f32; 3]> {
        self.active = true;
        self.last = Some(point);
        self.applied = 1;
        vec![point]
    }
    /// Continue a stroke. Emits evenly spaced samples between the previous
    /// point and this one so the stroke stays continuous under a fast drag.
    pub fn extend(&mut self, point: [f32; 3], spacing: f32) -> Vec<[f32; 3]> {
        if !self.active {
            return self.begin(point);
        }
        let Some(previous) = self.last else {
            return self.begin(point);
        };
        let step = if spacing.is_finite() && spacing > 1e-3 {
            spacing
        } else {
            1e-3
        };
        let delta: [f32; 3] = std::array::from_fn(|c| point[c] - previous[c]);
        let distance = (delta[0] * delta[0] + delta[2] * delta[2]).sqrt();
        if !distance.is_finite() {
            return vec![];
        }
        let count = ((distance / step).ceil() as usize).clamp(1, MAX_STEPS);
        let mut out = Vec::with_capacity(count);
        for i in 1..=count {
            let t = i as f32 / count as f32;
            out.push(std::array::from_fn(|c| previous[c] + delta[c] * t));
        }
        self.last = Some(point);
        self.applied = self.applied.saturating_add(count as u32);
        out
    }
    /// End a stroke. Returns whether anything was applied, which is the signal
    /// to push an undo entry and publish the asset.
    pub fn end(&mut self) -> bool {
        let applied = self.active && self.applied > 0;
        self.active = false;
        self.last = None;
        self.applied = 0;
        applied
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn falloff_is_one_at_the_centre_and_zero_at_the_rim() {
        for falloff in Falloff::ALL {
            assert!((falloff.weight(0.) - 1.).abs() < 1e-6, "{falloff:?}");
            if falloff != Falloff::Constant {
                assert!(falloff.weight(1.).abs() < 1e-6, "{falloff:?}");
            }
            // Clamping, not extrapolation, outside the disc.
            assert_eq!(falloff.weight(4.), falloff.weight(1.));
            assert_eq!(falloff.weight(-2.), falloff.weight(0.));
        }
    }
    #[test]
    fn falloff_is_monotonic() {
        for falloff in Falloff::ALL {
            let mut previous = f32::INFINITY;
            for i in 0..=16 {
                let w = falloff.weight(i as f32 / 16.);
                assert!(w <= previous + 1e-6, "{falloff:?} rose at {i}");
                previous = w;
            }
        }
    }
    #[test]
    fn weight_is_zero_outside_the_radius() {
        let brush = Brush {
            radius: 4.,
            ..Default::default()
        };
        assert!(brush.weight(0., 0.) > 0.9);
        assert_eq!(brush.weight(5., 0.), 0.);
        assert_eq!(brush.weight(3., 3.), 0.);
        assert!(brush.weight(2., 2.) > 0.);
    }
    #[test]
    fn lower_mirrors_raise_and_set_converges() {
        let raise = Brush {
            mode: Mode::Raise,
            strength: 2.,
            ..Default::default()
        };
        let lower = Brush {
            mode: Mode::Lower,
            ..raise
        };
        assert_eq!(raise.delta(0.5, 0.), -lower.delta(0.5, 0.));
        let set = Brush {
            mode: Mode::Set,
            target: 10.,
            strength: 1.,
            ..Default::default()
        };
        let mut height = 0.;
        for _ in 0..8 {
            height += set.delta(1., height);
        }
        assert!((height - 10.).abs() < 1e-3);
    }
    #[test]
    fn noise_is_deterministic_and_bounded() {
        for x in -4..4 {
            for z in -4..4 {
                let value = noise(x, z, 7);
                assert_eq!(value, noise(x, z, 7));
                assert!((-1. ..=1.).contains(&value), "{value}");
            }
        }
        assert_ne!(noise(1, 2, 7), noise(1, 2, 8));
        assert_ne!(noise(1, 2, 7), noise(2, 1, 7));
    }
    #[test]
    fn stroke_interpolates_and_is_bounded() {
        let mut stroke = Stroke::default();
        assert_eq!(stroke.begin([0., 0., 0.]).len(), 1);
        assert!(stroke.active());
        let samples = stroke.extend([4., 0., 0.], 1.);
        assert_eq!(samples.len(), 4);
        assert_eq!(samples[3], [4., 0., 0.]);
        // A jump far larger than the spacing stays bounded.
        assert_eq!(stroke.extend([4000., 0., 0.], 1.).len(), MAX_STEPS);
        assert!(stroke.end());
        assert!(!stroke.active());
        assert!(!stroke.end());
    }
    #[test]
    fn tile_rotation_is_fixed_below_four_and_scatters_at_four() {
        let fixed = Brush {
            tile_rotation: 2,
            ..Default::default()
        };
        assert_eq!(fixed.rotation_for(0, 0), 2);
        assert_eq!(fixed.rotation_for(9, 4), 2);
        let scatter = Brush {
            tile_rotation: 4,
            ..Default::default()
        };
        let mut seen = std::collections::BTreeSet::new();
        for j in 0..16 {
            for i in 0..16 {
                let r = scatter.rotation_for(i, j);
                assert!(r < 4);
                assert_eq!(r, scatter.rotation_for(i, j));
                seen.insert(r);
            }
        }
        assert!(seen.len() > 1, "scatter produced one rotation only");
    }
    #[test]
    fn brush_validation_rejects_out_of_range() {
        assert!(Brush::default().validate().is_ok());
        assert!(
            Brush {
                radius: 0.,
                ..Default::default()
            }
            .validate()
            .is_err()
        );
        assert!(
            Brush {
                strength: f32::NAN,
                ..Default::default()
            }
            .validate()
            .is_err()
        );
    }
}
