//! Deterministic seedable generator for reproducible truth corpora.

/// SplitMix64-style generator with explicit seed control.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SeededRng {
    state: u64,
}

impl SeededRng {
    /// Construct from an explicit seed.
    #[must_use]
    pub const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Advance and return the next `u64`.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Return a value in `0..upper` when `upper > 0`; otherwise `0`.
    pub fn gen_range(&mut self, upper: u64) -> u64 {
        if upper == 0 {
            return 0;
        }
        self.next_u64() % upper
    }

    /// Return whether a Bernoulli trial with rate in basis points succeeds.
    ///
    /// `rate_bps` is interpreted on `0..=10_000` (0%..=100%). Values above
    /// `10_000` are treated as always-true for fail-closed over-subscription.
    pub fn bernoulli_bps(&mut self, rate_bps: u32) -> bool {
        if rate_bps == 0 {
            return false;
        }
        if rate_bps >= 10_000 {
            return true;
        }
        self.gen_range(10_000) < u64::from(rate_bps)
    }
}

#[cfg(test)]
mod tests {
    use super::SeededRng;

    #[test]
    fn same_seed_is_deterministic() {
        let mut a = SeededRng::new(7);
        let mut b = SeededRng::new(7);
        assert_eq!(a.next_u64(), b.next_u64());
        assert_eq!(a.gen_range(10), b.gen_range(10));
        assert_eq!(SeededRng::new(1).gen_range(0), 0);
        assert!(!SeededRng::new(2).bernoulli_bps(0));
        assert!(SeededRng::new(2).bernoulli_bps(10_000));
        assert!(SeededRng::new(2).bernoulli_bps(10_001));
        let mut mid = SeededRng::new(3);
        let sample = mid.bernoulli_bps(5_000);
        let mut mid2 = SeededRng::new(3);
        assert_eq!(sample, mid2.bernoulli_bps(5_000));
    }
}
