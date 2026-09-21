//! Stable Membership-owned wire vocabulary for analytical design classification.

use crate::{MembershipDesign, MembershipError};

impl MembershipDesign {
    /// Parse one stable Membership-owned design name.
    ///
    /// This parser is a serialization boundary only. It does not derive or authorize a design;
    /// estimators and projections must still obtain the authoritative enum from Membership-owned
    /// classification over canonical network state.
    ///
    /// # Errors
    ///
    /// Returns [`MembershipError::UnknownMembershipDesign`] for unrecognized names.
    pub fn from_wire_name(name: &str) -> Result<Self, MembershipError> {
        match name {
            "nested" => Ok(Self::Nested),
            "cross_classified" => Ok(Self::CrossClassified),
            "multiple_membership" => Ok(Self::MultipleMembership),
            "cross_classified_multiple_membership" => Ok(Self::CrossClassifiedMultipleMembership),
            "heterogeneous_classification" => Ok(Self::HeterogeneousClassification),
            "heterogeneous_classification_multiple_membership" => {
                Ok(Self::HeterogeneousClassificationMultipleMembership)
            }
            _ => Err(MembershipError::UnknownMembershipDesign),
        }
    }

    /// Return the stable Membership-owned design name for released projections.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Nested => "nested",
            Self::CrossClassified => "cross_classified",
            Self::MultipleMembership => "multiple_membership",
            Self::CrossClassifiedMultipleMembership => "cross_classified_multiple_membership",
            Self::HeterogeneousClassification => "heterogeneous_classification",
            Self::HeterogeneousClassificationMultipleMembership => {
                "heterogeneous_classification_multiple_membership"
            }
        }
    }
}
