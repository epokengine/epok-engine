//! Lossless dictionaries and bounded deltas for already-quantized Q12 cameras.
use std::collections::BTreeMap;
pub struct Packed {
    pub origins: Vec<[i32; 3]>,
    pub scalars: Vec<i32>,
    pub samples: Vec<([i16; 3], u16, [u16; 4])>,
}
pub fn pack(values: &[[i32; 7]]) -> Option<Packed> {
    let mut out = Packed {
        origins: Vec::new(),
        scalars: Vec::new(),
        samples: Vec::new(),
    };
    let mut lookup = BTreeMap::new();
    for v in values {
        let p = [v[0], v[1], v[2]];
        if out.origins.last().is_none_or(|origin| {
            (0..3).any(|i| i16::try_from(i64::from(p[i]) - i64::from(origin[i])).is_err())
        }) {
            out.origins.push(p);
        }
        let origin = u16::try_from(out.origins.len() - 1).ok()?;
        let delta = std::array::from_fn(|i| (p[i] - out.origins[usize::from(origin)][i]) as i16);
        let mut indices = [0; 4];
        for i in 0..4 {
            indices[i] = if let Some(&index) = lookup.get(&v[i + 3]) {
                index
            } else {
                let index = u16::try_from(out.scalars.len()).ok()?;
                out.scalars.push(v[i + 3]);
                lookup.insert(v[i + 3], index);
                index
            };
        }
        out.samples.push((delta, origin, indices));
    }
    Some(out)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn camera_dictionary_is_lossless_across_delta_limits_and_cuts() {
        let values = [
            [0, 1, 2, 100, 200, 300, 90 * 4096],
            [32767, -32767, 2, 100, 400, 300, 90 * 4096],
            [32768, -32768, 2, 700, 400, 300, 90 * 4096],
            [-100_000, 200_000, 7, 100, 200, 300, 90 * 4096],
        ];
        let packed = pack(&values).unwrap();
        assert_eq!(packed.origins.len(), 3);
        for (expected, (delta, origin, indices)) in values.iter().zip(&packed.samples) {
            let actual: [i32; 7] = std::array::from_fn(|i| {
                if i < 3 {
                    packed.origins[usize::from(*origin)][i] + i32::from(delta[i])
                } else {
                    packed.scalars[usize::from(indices[i - 3])]
                }
            });
            assert_eq!(&actual, expected);
        }
        assert_eq!(packed.scalars.len(), 6);
    }
}
