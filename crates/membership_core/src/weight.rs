//! Validated membership weights for partial or full affiliation.

use crate::MembershipError;
use serde::{Deserialize, Serialize};

/// A finite, non-negative membership weight.
///
/// Weights of `1.0` represent full affiliation. Values in `(0, 1)` represent
/// partial multiple membership and must be preserved rather than rounded away
/// before multilevel estimation.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct MembershipWeight(f64);

impl MembershipWeight {
    /// Construct a validated membership weight.
    ///
    /// # Errors
    ///
    /// Returns [`MembershipError::InvalidMembershipWeight`] when `value` is
    /// negative, infinite, or not a number.
    pub fn new(value: f64) -> Result<Self, MembershipError> {
        if value.is_finite() && value >= 0.0 {
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

#[cfg(test)]
mod tests {
    use super::MembershipWeight;
    use crate::MembershipError;

    #[test]
    fn weight_constructors_accept_only_bounded_affiliation_shares() {
        let zero = MembershipWeight::new(0.0).expect("zero").value();
        let partial = MembershipWeight::new(0.5).expect("partial").value();
        let full = MembershipWeight::full().expect("full").value();
        assert_eq!(zero.to_bits(), 0.0_f64.to_bits());
        assert_eq!(partial.to_bits(), 0.5_f64.to_bits());
        assert_eq!(full.to_bits(), 1.0_f64.to_bits());

        for invalid in [
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

        assert!(serde_json::from_str::<MembershipWeight>("1.25").is_err());
        assert!(serde_json::from_str::<MembershipWeight>("-0.25").is_err());
    }
}
