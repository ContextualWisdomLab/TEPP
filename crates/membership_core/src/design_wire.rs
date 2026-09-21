//! Stable Membership-owned wire vocabulary for analytical design classification.

use crate::icc::{MembershipDesign, MembershipObservation, classify_membership_observations};
use crate::{MembershipError, MembershipNetwork};

/// Version identifier for the stable Membership design wire vocabulary.
///
/// Released projections should bind this owner-issued version beside [`MembershipDesign::wire_name`]
/// instead of inventing a consumer-local vocabulary version.
pub const MEMBERSHIP_DESIGN_WIRE_VERSION: &str = "tepp.membership_design.v1";

/// Membership-owned wire coordinate for one analytical design classification.
///
/// The coordinate binds the supported vocabulary version and owner-issued design name as one
/// immutable value object. Authoritative coordinates are issued by Membership-owned classification
/// over canonical network state; parsing an existing wire pair does not establish that authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MembershipDesignWire {
    design: MembershipDesign,
}

impl MembershipDesignWire {
    /// Bind an already owner-derived design to the current Membership wire vocabulary.
    #[must_use]
    pub(crate) const fn from_design(design: MembershipDesign) -> Self {
        Self { design }
    }

    /// Parse one complete Membership design wire coordinate.
    ///
    /// This is a deserialization boundary only. Parsing does not prove that the design describes any
    /// particular network or observation support; authoritative classification must still originate
    /// from Membership-owned domain state.
    ///
    /// # Errors
    ///
    /// Returns [`MembershipError::UnsupportedWireVersion`] unless `version` exactly matches
    /// [`MEMBERSHIP_DESIGN_WIRE_VERSION`]. Returns [`MembershipError::UnknownMembershipDesign`]
    /// when the supported version is paired with an unrecognized design name.
    pub fn parse(version: &str, name: &str) -> Result<Self, MembershipError> {
        if version != MEMBERSHIP_DESIGN_WIRE_VERSION {
            return Err(MembershipError::UnsupportedWireVersion);
        }
        Ok(Self::from_design(MembershipDesign::from_wire_name(name)?))
    }

    /// Return the bound vocabulary version.
    #[must_use]
    pub const fn version(self) -> &'static str {
        MEMBERSHIP_DESIGN_WIRE_VERSION
    }

    /// Return the stable Membership-owned design name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        self.design.wire_name()
    }

    /// Return the recovered Membership-domain design.
    #[must_use]
    pub const fn design(self) -> MembershipDesign {
        self.design
    }
}

/// Classify longitudinal Membership support and issue its stable wire coordinate.
///
/// Each observation is resolved at its own event time by the canonical Membership classifier. This
/// function is the owner-issued path for new analytical evidence; callers cannot mint a coordinate
/// from a freely chosen [`MembershipDesign`] enum value.
///
/// # Errors
///
/// Propagates the fail-closed longitudinal classification errors from
/// [`classify_membership_observations`], including empty support and missing observation membership.
pub fn classify_membership_observations_wire(
    network: &MembershipNetwork,
    observations: &[MembershipObservation],
) -> Result<MembershipDesignWire, MembershipError> {
    classify_membership_observations(network, observations).map(MembershipDesignWire::from_design)
}

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
