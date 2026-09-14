//! Serialized box colliders and the editor-side representation of PSX world AABBs.
use crate::{scene::Scene, transform::Matrix};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Collider {
    pub enabled: bool,
    pub trigger: bool,
    pub center: [f32; 3],
    pub half_extents: [f32; 3],
    pub layer: u32,
    pub mask: u32,
    /// A ramp: the top face rises by this much across the box along `slope_axis`,
    /// low edge at that axis's minimum. Zero leaves an ordinary box. The solver
    /// treats a ramp as a surface to stand on rather than an obstacle, so it is
    /// walkable from any side and never blocks sideways.
    #[serde(default, skip_serializing_if = "crate::collision::is_zero")]
    pub slope_rise: f32,
    #[serde(default, skip_serializing_if = "crate::collision::is_default_axis")]
    pub slope_axis: u8,
}
pub fn is_zero(value: &f32) -> bool {
    *value == 0.
}
pub fn is_default_axis(value: &u8) -> bool {
    *value == 0
}
impl Default for Collider {
    fn default() -> Self {
        Self {
            enabled: true,
            trigger: false,
            center: [0.; 3],
            half_extents: [0.5; 3],
            slope_rise: 0.,
            slope_axis: 0,
            layer: 1,
            mask: u32::MAX,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Aabb {
    pub min: [f32; 3],
    pub max: [f32; 3],
}
impl Aabb {
    #[cfg(test)]
    pub fn overlaps(self, other: Self) -> bool {
        (0..3).all(|k| self.max[k] > other.min[k] && self.min[k] < other.max[k])
    }
    pub fn edges(self) -> Vec<([f32; 3], [f32; 3])> {
        let point = |bits: usize| {
            std::array::from_fn(|axis| {
                if bits & (1 << axis) == 0 {
                    self.min[axis]
                } else {
                    self.max[axis]
                }
            })
        };
        (0..8)
            .flat_map(|bits| {
                (0..3)
                    .filter(move |&axis| bits & (1 << axis) == 0)
                    .map(move |axis| (point(bits), point(bits | (1 << axis))))
            })
            .collect()
    }
}

pub fn world_bounds(collider: &Collider, matrix: Matrix) -> Aabb {
    let center = matrix.point(collider.center);
    let extent: [f32; 3] = std::array::from_fn(|row| {
        (0..3)
            .map(|axis| matrix.0[row][axis].abs() * collider.half_extents[axis])
            .sum()
    });
    Aabb {
        min: std::array::from_fn(|k| center[k] - extent[k]),
        max: std::array::from_fn(|k| center[k] + extent[k]),
    }
}

pub fn validate_component(collider: &Collider) -> Result<(), String> {
    if collider
        .center
        .iter()
        .any(|v| !v.is_finite() || v.abs() > 128.)
    {
        return Err("Collider center must be finite and within ±128".into());
    }
    if collider
        .half_extents
        .iter()
        .any(|v| !v.is_finite() || !(1. / 4096. ..=128.).contains(v))
    {
        return Err("Collider half extents must be between 1/4096 and 128".into());
    }
    if collider.layer == 0 {
        return Err("Collider layer needs at least one bit".into());
    }
    Ok(())
}

pub fn validate(scene: &Scene) -> Result<(), String> {
    for (index, entity) in scene.actors.iter().enumerate() {
        let Some(collider) = &entity.collider else {
            continue;
        };
        validate_component(collider).map_err(|error| format!("{}: {error}", entity.name))?;
        let bounds = world_bounds(collider, scene.world_matrix(index));
        if bounds
            .min
            .iter()
            .chain(&bounds.max)
            .any(|v| !v.is_finite() || v.abs() > 512.)
        {
            return Err(format!(
                "{}: collider world bounds exceed ±512",
                entity.name
            ));
        }
        if (0..3).any(|axis| bounds.max[axis] - bounds.min[axis] < 2. / 4096.) {
            return Err(format!(
                "{}: collider becomes smaller than PSX fixed-point precision",
                entity.name
            ));
        }
    }
    Ok(())
}

/// Append inside initialize_components(), once Object.collider is available.
pub fn cpp_setup(scene: &Scene) -> String {
    use std::fmt::Write;
    let mut output = String::new();
    for (index, entity) in scene.actors.iter().enumerate() {
        if let Some(collider) = &entity.collider {
            writeln!(output, "objects[{index}].collider.enabled={};objects[{index}].collider.trigger={};objects[{index}].collider.layer={}u;objects[{index}].collider.mask={}u;", collider.enabled, collider.trigger, collider.layer, collider.mask).unwrap();
            for axis in 0..3 {
                writeln!(output, "objects[{index}].collider.center[{axis}]=Fixed({},Fixed::RAW);objects[{index}].collider.half_extents[{axis}]=Fixed({},Fixed::RAW);", (collider.center[axis] * 4096.).round() as i32, (collider.half_extents[axis] * 4096.).round() as i32).unwrap();
            }
            if collider.slope_rise != 0. {
                writeln!(output, "objects[{index}].collider.slope_rise=Fixed({},Fixed::RAW);objects[{index}].collider.slope_axis={}u;", (collider.slope_rise * 4096.).round() as i32, collider.slope_axis).unwrap();
            }
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn legacy_defaults_and_round_trip() {
        let collider: Collider = serde_json::from_str("{}").unwrap();
        assert_eq!(collider, Collider::default());
        let mut scene = Scene::default();
        scene.actors[1].collider = Some(Collider {
            trigger: true,
            mask: 0b0100,
            layer: 0b0010,
            ..collider
        });
        let decoded: Scene = serde_json::from_str(&serde_json::to_string(&scene).unwrap()).unwrap();
        assert_eq!(scene, decoded);
        validate(&decoded).unwrap();
        let cpp = cpp_setup(&decoded);
        assert!(cpp.contains("collider.trigger=true"));
        assert!(cpp.contains("collider.mask=4u"));
    }
    #[test]
    fn rotated_scaled_offset_and_shear_enclose_all_corners() {
        let collider = Collider {
            center: [1., 2., 3.],
            half_extents: [0.2, 0.7, 0.4],
            ..Default::default()
        };
        let matrix = Matrix::trs([4., 3., -1.], [0., 20., 70.], [2., 1., 3.]).compose(Matrix::trs(
            [0.; 3],
            [15., 0., 45.],
            [1.; 3],
        ));
        let bounds = world_bounds(&collider, matrix);
        for bits in 0..8 {
            let p = matrix.point(std::array::from_fn(|axis| {
                collider.center[axis]
                    + if bits & (1 << axis) == 0 {
                        -collider.half_extents[axis]
                    } else {
                        collider.half_extents[axis]
                    }
            }));
            assert!(
                (0..3).all(|axis| p[axis] >= bounds.min[axis] - 0.0001
                    && p[axis] <= bounds.max[axis] + 0.0001)
            );
        }
        assert_eq!(bounds.edges().len(), 12);
    }
    #[test]
    fn reject_invalid_shapes_and_unrepresentable_world_bounds() {
        for value in [0., -1., f32::NAN, f32::INFINITY, 0.00001, 129.] {
            let collider = Collider {
                half_extents: [value, 1., 1.],
                ..Default::default()
            };
            assert!(validate_component(&collider).is_err());
        }
        let mut scene = Scene::default();
        scene.actors[1].collider = Some(Collider::default());
        scene.actors[1].position = [1000., 0., 0.];
        assert!(validate(&scene).is_err());
        scene.actors[1].position = [0.; 3];
        scene.actors[1].scale = [0.00001; 3];
        assert!(validate(&scene).is_err());
    }
    #[test]
    fn touching_boxes_are_not_overlaps() {
        let a = Aabb {
            min: [0.; 3],
            max: [1.; 3],
        };
        assert!(!a.overlaps(Aabb {
            min: [1., 0., 0.],
            max: [2., 1., 1.]
        }));
        assert!(a.overlaps(Aabb {
            min: [0.5; 3],
            max: [1.5; 3]
        }));
    }
}
