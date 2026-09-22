//! Integer host evaluator for the cooked timeline profile; shared by preview and cook tests.
#[allow(dead_code)] // Also used by the standalone host/MIPS conformance harness.
pub fn sample(keys: &[(i32, i32)], tick: i32) -> i32 {
    sample_mode(keys, tick, 0, false)
}
/// Interpolation wire codes match reflection_schema::Interpolation and timeline.hpp.
/// All easing uses bounded Q12 integer alpha; division truncates toward zero.
pub fn sample_mode(keys: &[(i32, i32)], tick: i32, mode: u8, unsigned: bool) -> i32 {
    if keys.is_empty() || keys.len() > 256 {
        return 0;
    }
    if tick <= keys[0].0 {
        return keys[0].1;
    }
    let next = keys.partition_point(|key| key.0 <= tick);
    if next < keys.len() {
        let [(a, x), (b, y)] = [keys[next - 1], keys[next]];
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
    fn full_profile_keys_support_random_access_without_changing_interpolation() {
        let keys = (0..256)
            .map(|i| (i * 17, i * 51 - 4096))
            .collect::<Vec<_>>();
        for tick in (0..4336).rev() {
            assert_eq!(super::sample(&keys, tick), tick * 3 - 4096);
        }
        assert_eq!(super::sample(&keys, i32::MAX), keys[255].1);
    }
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
    /// `runtime/timeline.hpp` now takes `EaseIn`/`EaseOut` from the shared
    /// `epok::fixed_math::powq` instead of writing the squares out, so this pins the
    /// substitution as bit-identical over the whole alpha domain: one truncating
    /// multiply is exactly what `powq(t, 2)` performs.
    ///
    /// `Smoothstep` deliberately kept its own single division. The utility library's
    /// `Ease::SmoothStep` truncates twice, which disagrees with the timeline form on
    /// 2867 of the 4096 alphas by up to three raw units, so sharing that one would
    /// have changed every cooked timeline. This test is also the proof of that.
    #[test]
    fn timeline_quad_easing_is_the_shared_power_form_and_smoothstep_is_not() {
        fn powq(t: i64, n: u32) -> i64 {
            let mut acc = t;
            for _ in 1..n {
                acc = acc * t / 4096;
            }
            acc
        }
        let mut smoothstep_disagreements = 0;
        let mut smoothstep_worst = 0;
        for t in 0..=4096i64 {
            assert_eq!(t * t / 4096, powq(t, 2), "EaseIn drifted at t={t}");
            assert_eq!(
                4096 - (4096 - t) * (4096 - t) / 4096,
                4096 - powq(4096 - t, 2),
                "EaseOut drifted at t={t}"
            );
            let shared = t * t / 4096 * (12288 - 2 * t) / 4096;
            let timeline = t * t * (12288 - 2 * t) / (4096 * 4096);
            if shared != timeline {
                smoothstep_disagreements += 1;
                smoothstep_worst = smoothstep_worst.max((shared - timeline).abs());
            }
        }
        assert_eq!((smoothstep_disagreements, smoothstep_worst), (2867, 3));
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
