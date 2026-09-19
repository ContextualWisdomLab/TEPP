//! Fail-closed admission for estimators that only support full single membership.

use crate::{
    GroupId, MemberId, MembershipError, MembershipNetwork, MembershipRole, MembershipWeight,
};
use temporal_core::EventTime;

/// Owner-issued proof that one member has exactly one active, full-weight membership.
///
/// This value is intentionally constructed only from [`MembershipNetwork`] state. Consumers that
/// implement a nested or single-membership estimator can require this admission instead of trusting
/// a caller-selected cluster key or silently flattening cross-classified / multiple-membership
/// structure.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SingleMembershipAdmission {
    member_id: MemberId,
    group_id: GroupId,
    role: MembershipRole,
    weight: MembershipWeight,
    event_time: EventTime,
}

impl SingleMembershipAdmission {
    /// Return the admitted member identity.
    #[must_use]
    pub const fn member_id(self) -> MemberId {
        self.member_id
    }

    /// Return the sole active group identity.
    #[must_use]
    pub const fn group_id(self) -> GroupId {
        self.group_id
    }

    /// Return the sole active contextual role.
    #[must_use]
    pub const fn role(self) -> MembershipRole {
        self.role
    }

    /// Return the full membership weight.
    #[must_use]
    pub const fn weight(self) -> MembershipWeight {
        self.weight
    }

    /// Return the event time at which membership was admitted.
    #[must_use]
    pub const fn event_time(self) -> EventTime {
        self.event_time
    }
}

/// Admit one member for an estimator whose declared estimand supports only full single membership.
///
/// The admission is derived from canonical [`MembershipNetwork`] state. It fails closed when the
/// member is inactive, has more than one simultaneous active assignment (including same-group
/// cross-role assignments), or has a partial membership weight. A future cross-classified or
/// weighted estimator must use a separate versioned admission path rather than reusing this proof.
///
/// # Errors
///
/// Returns [`MembershipError::SingleMembershipProfileInapplicable`] whenever the active membership
/// structure cannot be represented without loss by a full single-membership estimator.
pub fn admit_single_membership(
    network: &MembershipNetwork,
    member_id: MemberId,
    event_time: EventTime,
) -> Result<SingleMembershipAdmission, MembershipError> {
    let active = network.active_memberships_for(member_id, event_time);
    let [assignment] = active.as_slice() else {
        return Err(MembershipError::SingleMembershipProfileInapplicable);
    };
    if assignment.weight().value().to_bits() != 1.0_f64.to_bits() {
        return Err(MembershipError::SingleMembershipProfileInapplicable);
    }
    Ok(SingleMembershipAdmission {
        member_id,
        group_id: assignment.group_id(),
        role: assignment.role(),
        weight: assignment.weight(),
        event_time,
    })
}
