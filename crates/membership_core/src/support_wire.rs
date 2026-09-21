//! Privacy-reduced reconstructable wire projection for longitudinal Membership support.

use crate::icc::MembershipDesignSignals;
use crate::network::exact_nonnegative_binary64_sum_exceeds_one;
use crate::{
    MembershipDesignClassification, MembershipDesignWire, MembershipError, MembershipNetwork,
    MembershipObservation, MembershipRole, MembershipWeight,
    MEMBERSHIP_OBSERVATION_SUPPORT_DIGEST_VERSION, classify_membership_observations_wire,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use temporal_core::EventTime;

/// Version identifier for the reconstructable longitudinal Membership support wire projection.
pub const MEMBERSHIP_OBSERVATION_SUPPORT_WIRE_VERSION: &str =
    "tepp.membership_observation_support_projection.v1";

const MAX_SUPPORT_JSON_BYTES: usize = 4 * 1024 * 1024;

/// One active Membership assignment in a released support observation.
///
/// Group identity is projection-local. Exact membership weight is retained as the lowercase
/// hexadecimal binary64 bit pattern so JSON number formatting cannot alter the scientific input.
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MembershipObservationSupportAssignmentWire {
    role: String,
    group_ordinal: u32,
    weight_f64_bits: String,
}

impl MembershipObservationSupportAssignmentWire {
    /// Return the stable Membership role wire name.
    #[must_use]
    pub fn role(&self) -> &str {
        &self.role
    }

    /// Return the projection-local group ordinal.
    #[must_use]
    pub const fn group_ordinal(&self) -> u32 {
        self.group_ordinal
    }

    /// Return the exact lowercase hexadecimal binary64 membership-weight bits.
    #[must_use]
    pub fn weight_f64_bits(&self) -> &str {
        &self.weight_f64_bits
    }
}

/// One member/event-time row in a released longitudinal Membership support projection.
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MembershipObservationSupportObservationWire {
    member_ordinal: u32,
    event_time: String,
    assignments: Vec<MembershipObservationSupportAssignmentWire>,
}

impl MembershipObservationSupportObservationWire {
    /// Return the projection-local member ordinal.
    #[must_use]
    pub const fn member_ordinal(&self) -> u32 {
        self.member_ordinal
    }

    /// Return the canonical UTC RFC 3339 event-time coordinate.
    #[must_use]
    pub fn event_time(&self) -> &str {
        &self.event_time
    }

    /// Return the canonical active Membership topology at this observation.
    #[must_use]
    pub fn assignments(&self) -> &[MembershipObservationSupportAssignmentWire] {
        &self.assignments
    }
}

/// Versioned, reconstructable, privacy-reduced serialization value for longitudinal Membership support.
///
/// The wire intentionally omits raw [`crate::MemberId`] and [`crate::GroupId`] UUIDs. Local ordinals
/// preserve repeated-member and repeated-group linkage only within this projection. The bound source
/// support digest remains an owner-side provenance fingerprint and should not be interpreted as
/// anonymity or as a natural-person identifier.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MembershipObservationSupportWire {
    schema_version: String,
    design_version: String,
    design_name: String,
    support_digest_version: String,
    support_sha256: String,
    member_count: u32,
    group_count: u32,
    observations: Vec<MembershipObservationSupportObservationWire>,
}

impl MembershipObservationSupportWire {
    /// Parse and validate canonical support JSON.
    ///
    /// Parsed wire values are serialization data only. Parsing does not prove that the support was
    /// derived from canonical Membership state; new owner authority is issued through
    /// [`project_membership_observations_wire`].
    ///
    /// # Errors
    ///
    /// Returns [`MembershipError::InvalidWirePayload`] for malformed or byte-noncanonical payloads,
    /// and [`MembershipError::UnsupportedWireVersion`] for an unsupported projection version.
    pub fn from_json(payload: &str) -> Result<Self, MembershipError> {
        if payload.len() > MAX_SUPPORT_JSON_BYTES {
            return Err(MembershipError::InvalidWirePayload);
        }
        let wire: Self =
            serde_json::from_str(payload).map_err(|_| MembershipError::InvalidWirePayload)?;
        wire.validate()?;
        if wire.to_json()? != payload {
            return Err(MembershipError::InvalidWirePayload);
        }
        Ok(wire)
    }

    /// Serialize validated canonical support JSON.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed Membership wire error when any coordinate is invalid or the bounded
    /// payload size is exceeded.
    pub fn to_json(&self) -> Result<String, MembershipError> {
        self.validate()?;
        let payload = serde_json::to_string(self).map_err(|_| MembershipError::InvalidWirePayload)?;
        if payload.len() > MAX_SUPPORT_JSON_BYTES {
            return Err(MembershipError::InvalidWirePayload);
        }
        Ok(payload)
    }

    /// Return the exact support projection schema version.
    #[must_use]
    pub fn schema_version(&self) -> &str {
        &self.schema_version
    }

    /// Return the Membership design vocabulary version bound into this projection.
    #[must_use]
    pub fn design_version(&self) -> &str {
        &self.design_version
    }

    /// Return the owner design name encoded by this projection.
    #[must_use]
    pub fn design_name(&self) -> &str {
        &self.design_name
    }

    /// Return the exact source-support digest contract version.
    #[must_use]
    pub fn support_digest_version(&self) -> &str {
        &self.support_digest_version
    }

    /// Return the lowercase source-support SHA-256 owned by Membership classification.
    #[must_use]
    pub fn support_sha256(&self) -> &str {
        &self.support_sha256
    }

    /// Return the number of projection-local members.
    #[must_use]
    pub const fn member_count(&self) -> u32 {
        self.member_count
    }

    /// Return the number of projection-local groups.
    #[must_use]
    pub const fn group_count(&self) -> u32 {
        self.group_count
    }

    /// Return the canonical observation multiset in projection order.
    #[must_use]
    pub fn observations(&self) -> &[MembershipObservationSupportObservationWire] {
        &self.observations
    }

    fn validate(&self) -> Result<(), MembershipError> {
        if self.schema_version != MEMBERSHIP_OBSERVATION_SUPPORT_WIRE_VERSION {
            return Err(MembershipError::UnsupportedWireVersion);
        }
        let design = MembershipDesignWire::parse(&self.design_version, &self.design_name)?;
        if self.support_digest_version != MEMBERSHIP_OBSERVATION_SUPPORT_DIGEST_VERSION
            || !valid_lower_hex(&self.support_sha256, 64)
            || self.member_count == 0
            || self.group_count == 0
            || self.observations.is_empty()
            || !nondecreasing(&self.observations)
        {
            return Err(MembershipError::InvalidWirePayload);
        }

        let mut members = BTreeSet::new();
        let mut groups = BTreeSet::new();
        let mut signals = MembershipDesignSignals::default();
        for observation in &self.observations {
            let event_time = EventTime::parse_rfc3339(&observation.event_time)
                .map_err(|_| MembershipError::InvalidWirePayload)?;
            if event_time.to_rfc3339() != observation.event_time
                || observation.assignments.is_empty()
                || !strictly_increasing(&observation.assignments)
            {
                return Err(MembershipError::InvalidWirePayload);
            }
            members.insert(observation.member_ordinal);
            let mut topology = Vec::with_capacity(observation.assignments.len());
            let mut active_edges = BTreeSet::new();
            let mut weights_by_role: BTreeMap<MembershipRole, Vec<f64>> = BTreeMap::new();
            for assignment in &observation.assignments {
                let role = MembershipRole::from_wire_name(&assignment.role)?;
                let bits = parse_weight_bits(&assignment.weight_f64_bits)?;
                let weight = f64::from_bits(bits);
                MembershipWeight::new(weight)?;
                if !active_edges.insert((role, assignment.group_ordinal)) {
                    return Err(MembershipError::InvalidWirePayload);
                }
                weights_by_role.entry(role).or_default().push(weight);
                groups.insert(assignment.group_ordinal);
                topology.push((
                    role,
                    assignment.group_ordinal,
                    bits != 1.0_f64.to_bits(),
                ));
            }
            if weights_by_role
                .values()
                .any(|weights| exact_nonnegative_binary64_sum_exceeds_one(weights.iter().copied()))
            {
                return Err(MembershipError::InvalidWirePayload);
            }
            signals.observe_tokens(topology);
        }
        if !is_contiguous_zero_based(&members, self.member_count)
            || !is_contiguous_zero_based(&groups, self.group_count)
            || signals.finish()? != design.design()
        {
            return Err(MembershipError::InvalidWirePayload);
        }
        Ok(())
    }
}

/// Owner-issued local coordinate for one caller observation admitted into a support projection.
///
/// The coordinate is parallel to the caller's input slice and intentionally contains no raw member
/// or group identity. It lets a downstream bounded context bind its own observation/document identity
/// to the same projection-local member ordinal without reproducing Membership's opaque-ID ordering.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MembershipObservationSupportCoordinate {
    member_ordinal: u32,
    event_time: EventTime,
}

impl MembershipObservationSupportCoordinate {
    /// Return the Membership-issued projection-local member ordinal.
    #[must_use]
    pub const fn member_ordinal(self) -> u32 {
        self.member_ordinal
    }

    /// Return the exact event time supplied for this admitted observation.
    #[must_use]
    pub const fn event_time(self) -> EventTime {
        self.event_time
    }
}

/// Membership-owned authority joining canonical design classification and reconstructable support wire.
///
/// Unlike [`MembershipObservationSupportWire`], this value cannot be created by parsing released
/// bytes. It is issued only after canonical Membership state is resolved for every requested event
/// time. Callers may serialize [`Self::wire`] for a released projection and zip
/// [`Self::input_coordinates`] with their own observation identities without reconstructing raw
/// Membership ordinal logic.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MembershipObservationSupportProjection {
    classification: MembershipDesignClassification,
    wire: MembershipObservationSupportWire,
    input_coordinates: Vec<MembershipObservationSupportCoordinate>,
}

impl MembershipObservationSupportProjection {
    /// Return the canonical owner-derived design classification bound to this support.
    #[must_use]
    pub const fn classification(&self) -> MembershipDesignClassification {
        self.classification
    }

    /// Return the reconstructable, privacy-reduced serialization value.
    #[must_use]
    pub const fn wire(&self) -> &MembershipObservationSupportWire {
        &self.wire
    }

    /// Return Membership-issued local coordinates parallel to the admitted caller observation slice.
    #[must_use]
    pub fn input_coordinates(&self) -> &[MembershipObservationSupportCoordinate] {
        &self.input_coordinates
    }
}

/// Classify longitudinal Membership support and issue its reconstructable owner projection.
///
/// Local member/group ordinals are derived from sorted opaque owner identifiers but the identifiers
/// themselves are not serialized. Observation and active-assignment ordering are canonicalized, while
/// duplicate declared observations remain multiplicity-sensitive. The wire binds the exact source
/// support digest already issued by [`classify_membership_observations_wire`]. The authority also
/// returns a non-wire input-coordinate slice so downstream owners can bind their own observation
/// identities to Membership-issued local member ordinals without reimplementing that mapping.
///
/// # Errors
///
/// Propagates Membership admission/classification errors and returns a fail-closed wire error if local
/// ordinal space or canonical serialization cannot represent the admitted support.
pub fn project_membership_observations_wire(
    network: &MembershipNetwork,
    observations: &[MembershipObservation],
) -> Result<MembershipObservationSupportProjection, MembershipError> {
    let classification = classify_membership_observations_wire(network, observations)?;

    let members = observations
        .iter()
        .map(|observation| observation.member_id())
        .collect::<BTreeSet<_>>();
    let member_count = u32::try_from(members.len()).map_err(|_| MembershipError::InvalidWirePayload)?;
    let member_ordinals = members
        .into_iter()
        .enumerate()
        .map(|(ordinal, member_id)| {
            u32::try_from(ordinal)
                .map(|ordinal| (member_id, ordinal))
                .map_err(|_| MembershipError::InvalidWirePayload)
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    let input_coordinates = observations
        .iter()
        .map(|observation| {
            member_ordinals
                .get(&observation.member_id())
                .copied()
                .map(|member_ordinal| MembershipObservationSupportCoordinate {
                    member_ordinal,
                    event_time: observation.event_time(),
                })
                .ok_or(MembershipError::InvalidWirePayload)
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut active_support = Vec::with_capacity(observations.len());
    let mut group_ids = BTreeSet::new();
    for observation in observations {
        let active = network.active_memberships_for(observation.member_id(), observation.event_time());
        for assignment in &active {
            group_ids.insert(assignment.group_id());
        }
        active_support.push((*observation, active));
    }
    let group_count = u32::try_from(group_ids.len()).map_err(|_| MembershipError::InvalidWirePayload)?;
    let group_ordinals = group_ids
        .into_iter()
        .enumerate()
        .map(|(ordinal, group_id)| {
            u32::try_from(ordinal)
                .map(|ordinal| (group_id, ordinal))
                .map_err(|_| MembershipError::InvalidWirePayload)
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?;

    let mut wire_observations = Vec::with_capacity(active_support.len());
    for (observation, active) in active_support {
        let member_ordinal = *member_ordinals
            .get(&observation.member_id())
            .ok_or(MembershipError::InvalidWirePayload)?;
        let mut assignments = active
            .into_iter()
            .map(|assignment| {
                let group_ordinal = *group_ordinals
                    .get(&assignment.group_id())
                    .ok_or(MembershipError::InvalidWirePayload)?;
                Ok(MembershipObservationSupportAssignmentWire {
                    role: assignment.role().wire_name().to_owned(),
                    group_ordinal,
                    weight_f64_bits: format!("{:016x}", assignment.weight().value().to_bits()),
                })
            })
            .collect::<Result<Vec<_>, MembershipError>>()?;
        assignments.sort();
        wire_observations.push(MembershipObservationSupportObservationWire {
            member_ordinal,
            event_time: observation.event_time().to_rfc3339(),
            assignments,
        });
    }
    wire_observations.sort();

    let support_sha256 = classification
        .support_sha256()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join("");
    let wire = MembershipObservationSupportWire {
        schema_version: MEMBERSHIP_OBSERVATION_SUPPORT_WIRE_VERSION.to_owned(),
        design_version: classification.version().to_owned(),
        design_name: classification.name().to_owned(),
        support_digest_version: classification.support_digest_version().to_owned(),
        support_sha256,
        member_count,
        group_count,
        observations: wire_observations,
    };
    let _canonical_json = wire.to_json()?;
    Ok(MembershipObservationSupportProjection {
        classification,
        wire,
        input_coordinates,
    })
}

fn parse_weight_bits(value: &str) -> Result<u64, MembershipError> {
    if !valid_lower_hex(value, 16) {
        return Err(MembershipError::InvalidWirePayload);
    }
    u64::from_str_radix(value, 16).map_err(|_| MembershipError::InvalidWirePayload)
}

fn valid_lower_hex(value: &str, width: usize) -> bool {
    value.len() == width
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn strictly_increasing<T: Ord>(values: &[T]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

fn nondecreasing<T: Ord>(values: &[T]) -> bool {
    values.windows(2).all(|pair| pair[0] <= pair[1])
}

fn is_contiguous_zero_based(values: &BTreeSet<u32>, count: u32) -> bool {
    usize::try_from(count).is_ok_and(|expected| {
        values.len() == expected && values.iter().copied().eq(0..count)
    })
}
