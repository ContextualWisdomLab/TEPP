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
    fn weight_constructors_accept_finite_non_negative_values() {
        let zero = MembershipWeight::new(0.0).expect("zero").value();
        let full = MembershipWeight::full().expect("full").value();
        assert!((zero - 0.0).abs() < f64::EPSILON);
        assert!((full - 1.0).abs() < f64::EPSILON);
        assert_eq!(
            MembershipWeight::new(f64::INFINITY),
            Err(MembershipError::InvalidMembershipWeight)
        );
    }
}
