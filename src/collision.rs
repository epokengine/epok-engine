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
    /// A compound static collider: solid boxes in the owner's local space, each
    /// `[min x, min y, min z, max x, max y, max z]`. The collider's own box must
    /// enclose them; queries test the parts, so one actor can carry a whole floor
    /// plan. The owner must be translated only.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub boxes: Vec<[f32; 6]>,
}
/// Most parts one compound collider may carry.
pub const MAX_COMPOUND_BOXES: usize = 2048;
/// Bound immutable cell lists per collider; larger cells reduce duplication.
const COMPOUND_GRID_BYTES: usize = 8 * 1024;

struct CompoundGrid {
    origin: [i16; 2],
    shift: u32,
    columns: usize,
    rows: usize,
    offsets: Vec<u16>,
    indices: Vec<u16>,
}

/// Index the exact quantized parts the target tests. XZ cells are conservative
/// over every height and include touching boundaries; the solver still tests Y.
/// Source-ordered lists let queries preserve contact and depenetration ties.
fn compound_grid(boxes: &[[i16; 6]]) -> Option<CompoundGrid> {
    if boxes.len() < 16 || boxes.len() > MAX_COMPOUND_BOXES {
        return None;
    }
    let origin = [
        boxes.iter().map(|b| b[0]).min()?,
        boxes.iter().map(|b| b[2]).min()?,
    ];
    let maximum = [
        boxes.iter().map(|b| b[3]).max()?,
        boxes.iter().map(|b| b[5]).max()?,
    ];
    // Start at eight world units. Coarsen pathological long/overlapping parts
    // until the lists fit, or keep the linear path if only one cell remains.
    for shift in 11..=16 {
        let cell = |value: i16, axis: usize| {
            ((i32::from(value) - i32::from(origin[axis])) >> shift) as usize
        };
        let columns = cell(maximum[0], 0) + 1;
        let rows = cell(maximum[1], 1) + 1;
        let count = columns * rows;
        if count == 1 {
            return None;
        }
        let references: usize = boxes
            .iter()
            .map(|b| (cell(b[3], 0) - cell(b[0], 0) + 1) * (cell(b[5], 1) - cell(b[2], 1) + 1))
            .sum();
        if (count + 1 + references) * 2 > COMPOUND_GRID_BYTES {
            continue;
        }
        let mut lists = vec![Vec::new(); count];
        for (index, b) in boxes.iter().enumerate() {
            for z in cell(b[2], 1)..=cell(b[5], 1) {
                for x in cell(b[0], 0)..=cell(b[3], 0) {
                    lists[z * columns + x].push(index as u16);
                }
            }
        }
        let mut offsets = Vec::with_capacity(count + 1);
        let mut indices = Vec::with_capacity(references);
        offsets.push(0);
        for list in lists {
            indices.extend(list);
            offsets.push(indices.len() as u16);
        }
        return Some(CompoundGrid {
            origin,
            shift,
            columns,
            rows,
            offsets,
            indices,
        });
    }
    None
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
            boxes: Vec::new(),
        }
    }
}
impl Collider {
    /// A compound collider enclosing `boxes`, given in the owner's local space.
    pub fn compound(boxes: Vec<[f32; 6]>) -> Self {
        let mut min = [f32::MAX; 3];
        let mut max = [f32::MIN; 3];
        for b in &boxes {
            for k in 0..3 {
                min[k] = min[k].min(b[k]);
                max[k] = max[k].max(b[k + 3]);
            }
        }
        if boxes.is_empty() {
            min = [-0.5; 3];
            max = [0.5; 3];
        }
        Self {
            center: std::array::from_fn(|k| (min[k] + max[k]) * 0.5),
            half_extents: std::array::from_fn(|k| ((max[k] - min[k]) * 0.5).max(1. / 256.)),
            boxes,
            ..Self::default()
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
    if !collider.boxes.is_empty() {
        if collider.boxes.len() > MAX_COMPOUND_BOXES {
            return Err(format!(
                "A compound collider holds at most {MAX_COMPOUND_BOXES} boxes"
            ));
        }
        if collider.trigger || collider.slope_rise != 0. {
            return Err("A compound collider cannot be a trigger or a ramp".into());
        }
        for b in &collider.boxes {
            if b.iter().any(|v| !v.is_finite() || v.abs() > 127.99) {
                return Err("Compound collider boxes must stay within local ±128 units".into());
            }
            if (0..3).any(|k| b[k + 3] - b[k] < 1. / 128.) {
                return Err("Compound collider boxes need a positive size on every axis".into());
            }
            if (0..3).any(|k| {
                b[k] < collider.center[k] - collider.half_extents[k] - 1. / 256.
                    || b[k + 3] > collider.center[k] + collider.half_extents[k] + 1. / 256.
            }) {
                return Err("The collider box must enclose every compound part".into());
            }
        }
    }
    Ok(())
}
/// The solid world boxes a collider presents to queries: its parts when compound,
/// otherwise its own box.
pub fn world_parts(collider: &Collider, matrix: Matrix) -> Vec<Aabb> {
    if collider.boxes.is_empty() {
        return vec![world_bounds(collider, matrix)];
    }
    let origin = matrix.point([0.; 3]);
    collider
        .boxes
        .iter()
        .map(|b| Aabb {
            min: std::array::from_fn(|k| origin[k] + b[k]),
            max: std::array::from_fn(|k| origin[k] + b[k + 3]),
        })
        .collect()
}
/// Whether a world matrix only translates, as compound colliders and height grids need.
pub fn translation_only(matrix: Matrix) -> bool {
    (0..3).all(|r| (0..3).all(|c| (matrix.0[r][c] - if r == c { 1. } else { 0. }).abs() < 1e-5))
}

pub fn validate(scene: &Scene) -> Result<(), String> {
    for (index, entity) in scene.actors.iter().enumerate() {
        let Some(collider) = &entity.collider else {
            continue;
        };
        validate_component(collider).map_err(|error| format!("{}: {error}", entity.name))?;
        if !collider.boxes.is_empty() && !translation_only(scene.world_matrix(index)) {
            return Err(format!(
                "{}: a compound collider's actor and its parents must not rotate or scale",
                entity.name
            ));
        }
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
            if !collider.boxes.is_empty() {
                // Function-scope storage, as for terrain heights: the parts are Q8
                // offsets from the owner's origin.
                let boxes: Vec<[i16; 6]> = collider
                    .boxes
                    .iter()
                    .map(|b| b.map(|v| ((v * 256.).round() as i32).clamp(-32767, 32767) as i16))
                    .collect();
                let values = boxes
                    .iter()
                    .flatten()
                    .map(i16::to_string)
                    .collect::<Vec<_>>()
                    .join(",");
                writeln!(output, "static constexpr int16_t collider_boxes_{index}[]={{{values}}};objects[{index}].collider.boxes=collider_boxes_{index};objects[{index}].collider.box_count={}u;", collider.boxes.len()).unwrap();
                if let Some(grid) = compound_grid(&boxes) {
                    let offsets = grid
                        .offsets
                        .iter()
                        .map(u16::to_string)
                        .collect::<Vec<_>>()
                        .join(",");
                    let indices = grid
                        .indices
                        .iter()
                        .map(u16::to_string)
                        .collect::<Vec<_>>()
                        .join(",");
                    writeln!(
                        output,
                        "static constexpr uint16_t collider_grid_offsets_{index}[]={{{offsets}}};"
                    )
                    .unwrap();
                    writeln!(
                        output,
                        "static constexpr uint16_t collider_grid_indices_{index}[]={{{indices}}};"
                    )
                    .unwrap();
                    writeln!(output, "static constexpr CompoundGrid collider_grid_{index}={{collider_boxes_{index},collider_grid_offsets_{index},collider_grid_indices_{index},{},{},{},{},{},{}}};objects[{index}].collider.box_grid=&collider_grid_{index};", boxes.len(), grid.columns, grid.rows, grid.origin[0], grid.origin[1], grid.shift + 4).unwrap();
                }
            }
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn compound_grid_covers_quantized_boundaries_in_source_order() {
        let boxes: Vec<_> = (0..64)
            .map(|i| {
                let x = (i % 8) as i16 * 2048 - 8192;
                let z = (i / 8) as i16 * 2048 - 8192;
                [x, -128, z, x + 2048, 0, z + 2048]
            })
            .collect();
        let grid = compound_grid(&boxes).unwrap();
        assert_eq!(grid.origin, [-8192, -8192]);
        assert_eq!(grid.shift, 11);
        assert_eq!((grid.columns, grid.rows), (9, 9));
        assert!((grid.offsets.len() + grid.indices.len()) * 2 <= COMPOUND_GRID_BYTES);
        for z in 0..grid.rows {
            for x in 0..grid.columns {
                let cell = z * grid.columns + x;
                let indices =
                    &grid.indices[grid.offsets[cell] as usize..grid.offsets[cell + 1] as usize];
                assert!(indices.windows(2).all(|pair| pair[0] < pair[1]));
                let lo = [x as i32 * 2048 - 8192, z as i32 * 2048 - 8192];
                for (index, b) in boxes.iter().enumerate() {
                    // A part ending on a cell boundary must be present in the
                    // following cell too: ray endpoints count as contacts.
                    if i32::from(b[0]) < lo[0] + 2048
                        && i32::from(b[3]) >= lo[0]
                        && i32::from(b[2]) < lo[1] + 2048
                        && i32::from(b[5]) >= lo[1]
                    {
                        assert!(indices.contains(&(index as u16)));
                    }
                }
            }
        }
        let mut scene = Scene::default();
        scene.actors[1].collider = Some(Collider::compound(
            boxes
                .iter()
                .map(|b| b.map(|v| f32::from(v) / 256.))
                .collect(),
        ));
        let header = cpp_setup(&scene);
        assert!(header.contains("static constexpr CompoundGrid collider_grid_1="));
        assert!(header.contains("collider.box_grid=&collider_grid_1"));
    }

    #[test]
    fn compound_grid_is_bounded_for_large_overlapping_parts() {
        assert!(compound_grid(&[[0, 0, 0, 1, 1, 1]; 15]).is_none());
        let boxes = vec![[-32767, -128, -32767, 32767, 0, 32767]; MAX_COMPOUND_BOXES];
        // Every box covers the whole footprint: finer cells duplicate all
        // references. A one-cell index has no benefit, so use the exact scan.
        assert!(compound_grid(&boxes).is_none());
        let mut staggered = boxes;
        for (i, b) in staggered.iter_mut().enumerate() {
            b[0] = ((i % 16) as i16 - 8) * 2048;
            b[2] = ((i / 16 % 16) as i16 - 8) * 2048;
            b[3] = b[0] + 2048;
            b[5] = b[2] + 2048;
        }
        let grid = compound_grid(&staggered).unwrap();
        assert!(grid.shift > 11);
        assert!((grid.offsets.len() + grid.indices.len()) * 2 <= COMPOUND_GRID_BYTES);
        assert_eq!(
            grid.offsets.last().copied(),
            Some(grid.indices.len() as u16)
        );
    }

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
