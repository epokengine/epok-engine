//! Integer host evaluator for the cooked timeline profile; shared by preview and cook tests.
#[allow(dead_code)] // Also used by the standalone host/MIPS conformance harness.
pub fn sample(keys: &[(i32, i32)], tick: i32) -> i32 {
    sample_mode(keys, tick, 0, false)
}
/// Interpolation wire codes match reflection_schema::Interpolation and timeline.hpp.
/// All easing uses bounded Q12 integer alpha; division truncates toward zero.
pub fn sample_mode(keys: &[(i32, i32)], tick: i32, mode: u8, unsigned: bool) -> i32 {
    if keys.is_empty() || keys.len() > 4 {
        return 0;
    }
    if tick <= keys[0].0 {
        return keys[0].1;
    }
    for pair in keys.windows(2) {
        let [(a, x), (b, y)] = [pair[0], pair[1]];
        if tick < b {
            let span = i64::from(b) - i64::from(a);
            if span <= 0 {
                return x;
            }
            if mode == 1 {
                return x;
            }
            let x = if unsigned {
                i64::from(x as u32)
            } else {
                i64::from(x)
            };
            let y = if unsigned {
                i64::from(y as u32)
            } else {
                i64::from(y)
            };
            let part = i64::from(tick) - i64::from(a);
            let value = if mode == 0 {
                x + (y - x) * part / span
            } else {
                let t = part * 4096 / span;
                let eased = match mode {
                    2 => t * t * (12288 - 2 * t) / (4096 * 4096),
                    3 => t * t / 4096,
                    4 => 4096 - (4096 - t) * (4096 - t) / 4096,
                    _ => t,
                };
                x + (y - x) * eased / 4096
            };
            return if unsigned {
                value.clamp(0, i64::from(u32::MAX)) as u32 as i32
            } else {
                value.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
            };
        }
    }
    keys[keys.len() - 1].1
}

#[cfg(test)]
mod tests {
    #[test]
    fn easing_discrete_and_unsigned_values_keep_their_declared_domain() {
        let keys = [(0, 0), (4096, 4096)];
        assert_eq!(super::sample_mode(&keys, 1024, 0, false), 1024);
        assert_eq!(super::sample_mode(&keys, 1024, 1, false), 0);
        assert_eq!(super::sample_mode(&keys, 1024, 2, false), 640);
        assert_eq!(super::sample_mode(&keys, 1024, 3, false), 256);
        assert_eq!(super::sample_mode(&keys, 1024, 4, false), 1792);
        for mode in 0..5 {
            assert_eq!(super::sample_mode(&keys, 4096, mode, false), 4096);
            let mut previous = 0;
            for tick in 0..=4096 {
                let value = super::sample_mode(&[(0, 0), (4096, -1)], tick, mode, true) as u32;
                assert!(
                    value >= previous,
                    "Unsigned curve decreased at mode={mode} tick={tick}"
                );
                previous = value;
            }
            assert_eq!(previous, u32::MAX);
        }
        assert_eq!(
            super::sample_mode(&[(0, -1), (4096, 0)], 2048, 0, true) as u32,
            2147483648
        );
    }
    #[test]
    fn native_conformance_vectors_use_signed_division_and_wide_intermediates() {
        assert_eq!(super::sample(&[(0, 4096), (3, -4096)], 1), 1366);
        assert_eq!(
            super::sample(&[(0, i32::MIN), (i32::MAX, i32::MAX)], i32::MAX / 2),
            -2
        );
        assert_eq!(super::sample(&[(0, -4), (4096, 8)], -1), -4);
        assert_eq!(super::sample(&[(0, -4), (4096, 8)], i32::MAX), 8);
    }
}
