//! Stable Membership-owned wire vocabulary for analytical design classification.

use crate::icc::{MembershipDesign, MembershipObservation, classify_membership_observations};
use crate::{MembershipError, MembershipNetwork};
use temporal_core::EventTime;

/// Version identifier for the stable Membership design wire vocabulary.
///
/// Released projections should bind this owner-issued version beside [`MembershipDesign::wire_name`]
/// instead of inventing a consumer-local vocabulary version.
pub const MEMBERSHIP_DESIGN_WIRE_VERSION: &str = "tepp.membership_design.v1";

/// Stable Membership design coordinate used at serialization boundaries.
///
/// This value object binds the supported vocabulary version and design name. It can be reconstructed
/// from released payloads, so possession of this type alone does not establish that the design was
/// classified from canonical Membership state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MembershipDesignWire {
    design: MembershipDesign,
}

impl MembershipDesignWire {
    /// Bind an already validated design to the current Membership wire vocabulary.
    #[must_use]
    const fn from_design(design: MembershipDesign) -> Self {
        Self { design }
    }

    /// Parse one complete Membership design wire coordinate.
    ///
    /// This is a deserialization boundary only. Parsing does not prove that the design describes any
    /// particular network or observation support. New authoritative classifications are represented
    /// by [`MembershipDesignClassification`] and are issued only through Membership-owned domain
    /// classification.
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

    /// Return the design encoded by this wire coordinate.
    ///
    /// The returned enum reflects only the parsed/serialized coordinate. It is not proof that the
    /// design was derived from canonical Membership state.
    #[must_use]
    pub const fn design(self) -> MembershipDesign {
        self.design
    }
}

/// Membership-owned result of classifying canonical longitudinal observation support.
///
/// Unlike [`MembershipDesignWire`], this type cannot be reconstructed from a `{version, name}` pair.
/// Its private state is issued only after [`classify_membership_observations`] evaluates canonical
/// Membership state at every observation's own event time. The retained support count and event-time
/// bounds are derived from the same observation slice and prevent the classification from becoming a
/// completely detached design label.
///
/// These coordinates are not a complete reconstruction of the classified cohort. Distinct supports
/// can share the same count and bounds, so released analytical projections still need their own
/// privacy-appropriate observation-support evidence and exact identity binding where required.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MembershipDesignClassification {
    wire: MembershipDesignWire,
    observation_count: usize,
    earliest_event_time: EventTime,
    latest_event_time: EventTime,
}

impl MembershipDesignClassification {
    fn from_design_and_observations(
        design: MembershipDesign,
        observations: &[MembershipObservation],
    ) -> Result<Self, MembershipError> {
        let Some(first) = observations.first().copied() else {
            return Err(MembershipError::InsufficientClusterStructure);
        };
        let mut earliest_event_time = first.event_time();
        let mut latest_event_time = first.event_time();
        for observation in &observations[1..] {
            let event_time = observation.event_time();
            earliest_event_time = earliest_event_time.min(event_time);
            latest_event_time = latest_event_time.max(event_time);
        }
        Ok(Self {
            wire: MembershipDesignWire::from_design(design),
            observation_count: observations.len(),
            earliest_event_time,
            latest_event_time,
        })
    }

    /// Return the stable wire coordinate for released projections.
    #[must_use]
    pub const fn wire(self) -> MembershipDesignWire {
        self.wire
    }

    /// Return the Membership-domain design derived from canonical state.
    #[must_use]
    pub const fn design(self) -> MembershipDesign {
        self.wire.design()
    }

    /// Return the bound Membership vocabulary version.
    #[must_use]
    pub const fn version(self) -> &'static str {
        self.wire.version()
    }

    /// Return the stable design name for released projections.
    #[must_use]
    pub const fn name(self) -> &'static str {
        self.wire.name()
    }

    /// Return the number of longitudinal observation coordinates classified.
    #[must_use]
    pub const fn observation_count(self) -> usize {
        self.observation_count
    }

    /// Return the earliest event time in the classified observation support.
    #[must_use]
    pub const fn earliest_event_time(self) -> EventTime {
        self.earliest_event_time
    }

    /// Return the latest event time in the classified observation support.
    #[must_use]
    pub const fn latest_event_time(self) -> EventTime {
        self.latest_event_time
    }
}

/// Classify longitudinal Membership support and issue an owner-derived classification result.
///
/// Each observation is resolved at its own event time by the canonical Membership classifier. The
/// returned [`MembershipDesignClassification`] is distinct from a parsed wire coordinate, so callers
/// cannot turn an arbitrary supported `{version, name}` pair into owner-derived classification
/// authority. Support cardinality and event-time bounds are derived from the exact observation slice
/// passed to classification rather than accepted as caller-supplied metadata.
///
/// # Errors
///
/// Propagates the fail-closed longitudinal classification errors from
/// [`classify_membership_observations`], including empty support and missing observation membership.
pub fn classify_membership_observations_wire(
    network: &MembershipNetwork,
    observations: &[MembershipObservation],
) -> Result<MembershipDesignClassification, MembershipError> {
    let design = classify_membership_observations(network, observations)?;
    MembershipDesignClassification::from_design_and_observations(design, observations)
}

impl MembershipDesign {
    /// Parse one stable Membership-owned design name.
    ///
    /// This parser is a serialization boundary only. It does not derive or authorize a design;
    /// estimators and projections must still obtain authoritative classification from Membership-owned
    /// domain state.
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
