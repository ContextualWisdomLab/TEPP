//! Stable Membership-owned wire vocabulary for analytical design classification.

use crate::icc::{MembershipDesign, MembershipObservation, classify_membership_observations};
use crate::{MembershipAssignment, MembershipError, MembershipNetwork};
use sha2::{Digest, Sha256};
use temporal_core::EventTime;

/// Version identifier for the stable Membership design wire vocabulary.
///
/// Released projections should bind this owner-issued version beside [`MembershipDesign::wire_name`]
/// instead of inventing a consumer-local vocabulary version.
pub const MEMBERSHIP_DESIGN_WIRE_VERSION: &str = "tepp.membership_design.v1";

/// Version identifier for canonical longitudinal observation-support SHA-256 evidence.
///
/// The digest is identity binding for the classified opaque support and active Membership topology.
/// It is not a substitute for reconstructable cohort/window evidence in a released analysis result.
pub const MEMBERSHIP_OBSERVATION_SUPPORT_DIGEST_VERSION: &str =
    "tepp.membership_observation_support.v1";

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
/// Membership state at every observation's own event time. The retained support count, event-time
/// bounds, and canonical SHA-256 are all derived from the same observation slice and canonical active
/// Membership topology.
///
/// The digest binds exact opaque support identity but does not reconstruct the cohort by itself.
/// Released analytical projections still need privacy-appropriate observation-support coordinates
/// that a consumer can interpret and compare against this owner-derived identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MembershipDesignClassification {
    wire: MembershipDesignWire,
    observation_count: usize,
    earliest_event_time: EventTime,
    latest_event_time: EventTime,
    support_sha256: [u8; 32],
}

impl MembershipDesignClassification {
    fn from_design_and_observations(
        network: &MembershipNetwork,
        design: MembershipDesign,
        observations: &[MembershipObservation],
    ) -> Self {
        // `classify_membership_observations` has already rejected empty support before this owner
        // authority can be issued, so index 0 is an invariant rather than a second admission branch.
        let first = observations[0];
        let mut earliest_event_time = first.event_time();
        let mut latest_event_time = first.event_time();
        for observation in &observations[1..] {
            let event_time = observation.event_time();
            earliest_event_time = earliest_event_time.min(event_time);
            latest_event_time = latest_event_time.max(event_time);
        }
        Self {
            wire: MembershipDesignWire::from_design(design),
            observation_count: observations.len(),
            earliest_event_time,
            latest_event_time,
            support_sha256: canonical_observation_support_sha256(network, observations),
        }
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

    /// Return the version of the canonical observation-support digest contract.
    #[must_use]
    pub const fn support_digest_version(self) -> &'static str {
        MEMBERSHIP_OBSERVATION_SUPPORT_DIGEST_VERSION
    }

    /// Return the canonical SHA-256 of the classified opaque support and active Membership topology.
    #[must_use]
    pub const fn support_sha256(self) -> [u8; 32] {
        self.support_sha256
    }
}

/// Classify longitudinal Membership support and issue an owner-derived classification result.
///
/// Each observation is resolved at its own event time by the canonical Membership classifier. The
/// returned [`MembershipDesignClassification`] is distinct from a parsed wire coordinate, so callers
/// cannot turn an arbitrary supported `{version, name}` pair into owner-derived classification
/// authority. Support cardinality, event-time bounds, and exact support identity are derived from the
/// observation slice and canonical active Membership state rather than accepted as caller metadata.
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
    Ok(MembershipDesignClassification::from_design_and_observations(
        network,
        design,
        observations,
    ))
}

fn canonical_observation_support_sha256(
    network: &MembershipNetwork,
    observations: &[MembershipObservation],
) -> [u8; 32] {
    let mut support = observations
        .iter()
        .copied()
        .map(|observation| {
            let mut active =
                network.active_memberships_for(observation.member_id(), observation.event_time());
            active.sort_by(|left, right| {
                left.role()
                    .cmp(&right.role())
                    .then_with(|| left.group_id().cmp(&right.group_id()))
                    .then_with(|| {
                        left.weight()
                            .value()
                            .to_bits()
                            .cmp(&right.weight().value().to_bits())
                    })
            });
            (observation, active)
        })
        .collect::<Vec<_>>();
    support.sort_by(|(left, _), (right, _)| {
        left.member_id()
            .cmp(&right.member_id())
            .then_with(|| left.event_time().cmp(&right.event_time()))
    });

    let mut hasher = Sha256::new();
    hash_frame(
        &mut hasher,
        MEMBERSHIP_OBSERVATION_SUPPORT_DIGEST_VERSION.as_bytes(),
    );
    hash_count(&mut hasher, support.len());
    for (observation, active) in support {
        hasher.update(observation.member_id().as_uuid().as_bytes());
        let event_time = observation.event_time().to_rfc3339();
        hash_frame(&mut hasher, event_time.as_bytes());
        hash_count(&mut hasher, active.len());
        for assignment in active {
            hash_assignment(&mut hasher, assignment);
        }
    }
    hasher.finalize().into()
}

fn hash_assignment(hasher: &mut Sha256, assignment: MembershipAssignment) {
    hash_frame(hasher, assignment.role().wire_name().as_bytes());
    hasher.update(assignment.group_id().as_uuid().as_bytes());
    hasher.update(assignment.weight().value().to_bits().to_be_bytes());
}

fn hash_count(hasher: &mut Sha256, value: usize) {
    let value = u128::try_from(value).expect("usize must fit in u128");
    hasher.update(value.to_be_bytes());
}

fn hash_frame(hasher: &mut Sha256, value: &[u8]) {
    hash_count(hasher, value.len());
    hasher.update(value);
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
