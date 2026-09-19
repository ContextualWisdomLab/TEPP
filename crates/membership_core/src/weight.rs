//! Validated membership weights for partial or full affiliation.

use crate::MembershipError;
use serde::{Deserialize, Serialize};

/// A finite membership share in the closed unit interval `[0, 1]`.
///
/// Weights of `1.0` represent full affiliation. Values in `(0, 1)` represent
/// partial multiple membership and must be preserved rather than rounded away
/// before multilevel estimation. Values above `1.0` are not affiliation shares
/// and fail closed rather than being normalized or clamped.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct MembershipWeight(f64);

impl MembershipWeight {
    /// Construct a validated membership weight.
    ///
    /// # Errors
    ///
    /// Returns [`MembershipError::InvalidMembershipWeight`] when `value` is
    /// outside `[0, 1]`, infinite, or not a number.
    pub fn new(value: f64) -> Result<Self, MembershipError> {
        if value.is_finite() && (0.0..=1.0).contains(&value) {
            Ok(Self(value))
        } else {
            Err(MembershipError::InvalidMembershipWeight)
        }
    }

    /// Full single-membership weight of one.
    ///
    /// # Errors
    ///
    /// Never fails; present for API uniformity with [`Self::new`].
    pub fn full() -> Result<Self, MembershipError> {
        Self::new(1.0)
    }

    /// Return the numeric weight.
    #[must_use]
    pub const fn value(self) -> f64 {
        self.0
    }
}

impl<'de> Deserialize<'de> for MembershipWeight {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = f64::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::MembershipWeight;
    use crate::MembershipError;

    #[test]
    fn weight_constructors_accept_only_positive_bounded_affiliation_shares() {
        let minimum_positive = MembershipWeight::new(f64::from_bits(1))
            .expect("minimum positive binary64 share")
            .value();
        let partial = MembershipWeight::new(0.5).expect("partial").value();
        let full = MembershipWeight::full().expect("full").value();
        assert_eq!(minimum_positive.to_bits(), 1);
        assert_eq!(partial.to_bits(), 0.5_f64.to_bits());
        assert_eq!(full.to_bits(), 1.0_f64.to_bits());

        for invalid in [
            0.0,
            -0.0,
            1.0 + f64::EPSILON,
            -f64::EPSILON,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NAN,
        ] {
            assert_eq!(
                MembershipWeight::new(invalid),
                Err(MembershipError::InvalidMembershipWeight)
            );
        }
    }

    #[test]
    fn serde_deserialization_cannot_bypass_weight_validation() {
        let partial: MembershipWeight =
            serde_json::from_str("0.5").expect("valid partial wire weight");
        assert_eq!(partial.value().to_bits(), 0.5_f64.to_bits());

        assert!(serde_json::from_str::<MembershipWeight>("0.0").is_err());
        assert!(serde_json::from_str::<MembershipWeight>("-0.0").is_err());
        assert!(serde_json::from_str::<MembershipWeight>("1.25").is_err());
        assert!(serde_json::from_str::<MembershipWeight>("-0.25").is_err());
    }
}
