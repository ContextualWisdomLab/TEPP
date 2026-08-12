//! Time-varying weighted membership assignments.

use crate::{GroupId, MemberId, MembershipError, MembershipRole, MembershipWeight};
use temporal_core::{EventTime, TemporalBoundary, TemporalInterval, TemporalPrecision};

/// One weighted, role-typed, time-varying membership assignment.
///
/// Assignments are the atomic unit of multilevel and multiple-membership
/// structure. A single member may hold many simultaneous assignments across
/// non-nested groups (cross-classification) and roles.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MembershipAssignment {
    member_id: MemberId,
    group_id: GroupId,
    role: MembershipRole,
    weight: MembershipWeight,
    validity: TemporalInterval<EventTime>,
}

impl MembershipAssignment {
    /// Construct a validated membership assignment on event time.
    ///
    /// # Errors
    ///
    /// Returns a weight or validity error when inputs fail domain validation.
    pub fn new(
        member_id: MemberId,
        group_id: GroupId,
        role: MembershipRole,
        weight: MembershipWeight,
        valid_from: EventTime,
        valid_to: EventTime,
    ) -> Result<Self, MembershipError> {
        let validity = TemporalInterval::bounded(
            TemporalBoundary::Included(valid_from),
            TemporalBoundary::Included(valid_to),
            TemporalPrecision::Second,
        )
        .map_err(|_| MembershipError::InvalidValidityInterval)?;
        Ok(Self {
            member_id,
            group_id,
            role,
            weight,
            validity,
        })
    }

    /// Return the member identifier.
    #[must_use]
    pub const fn member_id(self) -> MemberId {
        self.member_id
    }

    /// Return the group identifier.
    #[must_use]
    pub const fn group_id(self) -> GroupId {
        self.group_id
    }

    /// Return the contextual role.
    #[must_use]
    pub const fn role(self) -> MembershipRole {
        self.role
    }

    /// Return the membership weight.
    #[must_use]
    pub const fn weight(self) -> MembershipWeight {
        self.weight
    }

    /// Return the event-time validity interval.
    #[must_use]
    pub const fn validity(self) -> TemporalInterval<EventTime> {
        self.validity
    }

    /// Return whether this assignment is active at an event time.
    #[must_use]
    pub fn is_active_at(self, instant: EventTime) -> bool {
        self.validity.contains(instant)
    }
}
