//! Accepted VRAM device-class profiles.

/// Binary-gigabyte VRAM profile used to select micro-batches.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum VramProfile {
    /// 4 GiB class.
    Gib4,
    /// 6 GiB class.
    Gib6,
    /// 8 GiB class.
    Gib8,
    /// 12 GiB class.
    Gib12,
    /// 24 GiB class.
    Gib24,
}

impl VramProfile {
    /// One binary gigabyte in bytes.
    pub const GIBIBYTE: u64 = 1 << 30;

    /// Return every accepted profile in increasing capacity order.
    #[must_use]
    pub const fn all() -> [Self; 5] {
        [Self::Gib4, Self::Gib6, Self::Gib8, Self::Gib12, Self::Gib24]
    }

    /// Return the profile capacity in binary gigabytes.
    #[must_use]
    pub const fn gibibytes(self) -> u64 {
        match self {
            Self::Gib4 => 4,
            Self::Gib6 => 6,
            Self::Gib8 => 8,
            Self::Gib12 => 12,
            Self::Gib24 => 24,
        }
    }

    /// Return the profile capacity in bytes.
    #[must_use]
    pub const fn bytes(self) -> u64 {
        self.gibibytes() * Self::GIBIBYTE
    }

    /// Return the reserved safety headroom (one eighth of capacity).
    #[must_use]
    pub const fn safety_bytes(self) -> u64 {
        self.bytes() / 8
    }
}

#[cfg(test)]
mod tests {
    use super::VramProfile;

    #[test]
    fn capacities_match_accepted_profiles() {
        assert_eq!(VramProfile::GIBIBYTE, 1_073_741_824);
        assert_eq!(VramProfile::Gib6.gibibytes(), 6);
        assert_eq!(VramProfile::Gib8.bytes(), 8 * VramProfile::GIBIBYTE);
        assert_eq!(
            VramProfile::Gib12.safety_bytes(),
            VramProfile::Gib12.bytes() / 8
        );
        assert_eq!(VramProfile::all().len(), 5);
    }
}
