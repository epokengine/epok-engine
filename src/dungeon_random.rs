//! The dungeon generator's random stream. `runtime/dungeon_random.hpp` is its C++ twin:
//! both produce the same numbers for the same seed, so a layout chosen in the editor can
//! be reproduced on the console. Only 32-bit integer operations take part.

/// xoshiro128** seeded through a SplitMix32 sequence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Random {
    state: [u32; 4],
}

impl Random {
    pub fn new(seed: u32) -> Self {
        let mut weyl = seed;
        let state = std::array::from_fn(|_| {
            weyl = weyl.wrapping_add(0x9e37_79b9);
            let mut z = weyl;
            z = (z ^ (z >> 16)).wrapping_mul(0x85eb_ca6b);
            z = (z ^ (z >> 13)).wrapping_mul(0xc2b2_ae35);
            z ^ (z >> 16)
        });
        let mut random = Self { state };
        if random.state == [0; 4] {
            random.state[0] = 1;
        }
        random
    }

    /// An independent stream for one stage of generation, so adding draws to one stage
    /// never changes the numbers another stage sees.
    pub fn stream(seed: u32, stage: u32) -> Self {
        Self::new(seed ^ stage.wrapping_mul(0x9e37_79b9).rotate_left(16))
    }

    pub fn next_u32(&mut self) -> u32 {
        let s = &mut self.state;
        let result = s[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let t = s[1] << 9;
        s[2] ^= s[0];
        s[3] ^= s[1];
        s[1] ^= s[2];
        s[0] ^= s[3];
        s[2] ^= t;
        s[3] = s[3].rotate_left(11);
        result
    }

    /// A uniform value in `0..n` without modulo bias; 0 when `n` is 0.
    pub fn below(&mut self, n: u32) -> u32 {
        if n == 0 {
            return 0;
        }
        let mut product = u64::from(self.next_u32()) * u64::from(n);
        if (product as u32) < n {
            let threshold = n.wrapping_neg() % n;
            while (product as u32) < threshold {
                product = u64::from(self.next_u32()) * u64::from(n);
            }
        }
        (product >> 32) as u32
    }

    /// A uniform value in `low..=high`; `low` when the range is empty.
    pub fn range(&mut self, low: i32, high: i32) -> i32 {
        if high <= low {
            return low;
        }
        let span = (i64::from(high) - i64::from(low) + 1) as u32;
        low.wrapping_add(self.below(span) as i32)
    }

    /// True with `percent` in 100.
    pub fn chance(&mut self, percent: u32) -> bool {
        self.below(100) < percent.min(100)
    }
}

#[cfg(test)]
mod tests {
    use super::Random;

    // The same values are asserted by tests/runtime/dungeon_random.cpp.
    #[test]
    fn matches_the_runtime_golden_values() {
        let mut zero = Random::new(0);
        assert_eq!(
            [0; 4].map(|_| zero.next_u32()),
            [0xe308_dc58, 0x4392_d0e4, 0x0331_8f97, 0xac59_3a63]
        );
        let mut seeded = Random::new(12345);
        assert_eq!(
            [0; 4].map(|_| seeded.next_u32()),
            [0x1eea_3cc1, 0x1a40_a62e, 0xfc4c_d240, 0xdffc_5b56]
        );
        let mut digits = Random::new(12345);
        assert_eq!(
            [0; 12].map(|_| digits.below(10)),
            [1, 1, 9, 8, 9, 3, 8, 4, 6, 1, 2, 7]
        );
        let mut large = Random::new(0xdead_beef);
        assert_eq!(
            [0; 6].map(|_| large.below(1000)),
            [537, 599, 291, 628, 540, 741]
        );
    }

    #[test]
    fn ranges_and_chances_stay_in_bounds() {
        let mut random = Random::new(7);
        for _ in 0..2000 {
            let value = random.range(-3, 4);
            assert!((-3..=4).contains(&value));
        }
        assert_eq!(random.range(5, 5), 5);
        assert_eq!(random.range(9, 2), 9);
        assert_eq!(random.below(0), 0);
        assert!(!(0..100).any(|_| random.chance(0)));
        assert!((0..100).all(|_| random.chance(100)));
    }

    #[test]
    fn stages_are_independent_streams() {
        let mut a = Random::stream(42, 1);
        let mut b = Random::stream(42, 2);
        assert_ne!(a.next_u32(), b.next_u32());
        assert_eq!(Random::stream(42, 1), Random::stream(42, 1));
    }
}
