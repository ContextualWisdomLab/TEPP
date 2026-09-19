//! In-memory multiple-membership networks for multilevel estimation inputs.

use crate::{GroupId, MemberId, MembershipAssignment, MembershipError, MembershipRole};
use std::collections::{BTreeMap, BTreeSet};
use temporal_core::{AllenRelation, EventTime, classify_interval_relation};

/// An in-memory network of weighted multiple memberships.
///
/// The network preserves every accepted assignment so estimators can model
/// cross-classification and multiple membership instead of collapsing documents
/// into independent rows (atomistic fallacy).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MembershipNetwork {
    assignments: Vec<MembershipAssignment>,
}

impl MembershipNetwork {
    /// Create an empty membership network.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert one validated assignment.
    ///
    /// Repeated `(member, group, role)` identities are valid when their closed
    /// event-time intervals are strictly disjoint, preserving longitudinal leave
    /// and re-entry spells. Overlapping or endpoint-touching intervals for the
    /// same identity fail closed because they would create two simultaneously
    /// active copies of one membership edge.
    ///
    /// # Errors
    ///
    /// Returns [`MembershipError::DuplicateMembershipAssignment`] when an
    /// existing assignment has the same `(member, group, role)` identity and its
    /// validity interval is not strictly before or after the candidate interval.
    pub fn insert(&mut self, assignment: MembershipAssignment) -> Result<(), MembershipError> {
        let conflicts = self.assignments.iter().copied().any(|existing| {
            existing.member_id() == assignment.member_id()
                && existing.group_id() == assignment.group_id()
                && existing.role() == assignment.role()
                && !matches!(
                    classify_interval_relation(&existing.validity(), &assignment.validity()),
                    Ok(AllenRelation::Before | AllenRelation::After)
                )
        });
        if conflicts {
            return Err(MembershipError::DuplicateMembershipAssignment);
        }
        self.assignments.push(assignment);
        Ok(())
    }

    /// Return the number of stored assignments.
    #[must_use]
    pub fn assignment_count(&self) -> usize {
        self.assignments.len()
    }

    /// Iterate all assignments.
    pub fn assignments(&self) -> impl Iterator<Item = MembershipAssignment> + '_ {
        self.assignments.iter().copied()
    }

    /// Return active assignments for one member at an event time.
    #[must_use]
    pub fn active_memberships_for(
        &self,
        member_id: MemberId,
        instant: EventTime,
    ) -> Vec<MembershipAssignment> {
        self.assignments
            .iter()
            .copied()
            .filter(|assignment| {
                assignment.member_id() == member_id && assignment.is_active_at(instant)
            })
            .collect()
    }

    /// Count distinct group contexts active for one member at an event time.
    ///
    /// Counts above one are the defining signal of multiple membership and must
    /// not be discarded before multilevel estimation.
    #[must_use]
    pub fn active_group_multiplicity(&self, member_id: MemberId, instant: EventTime) -> usize {
        let mut groups = BTreeSet::new();
        for assignment in self.active_memberships_for(member_id, instant) {
            groups.insert(assignment.group_id());
        }
        groups.len()
    }

    /// Aggregate total active weight by role for one member.
    #[must_use]
    pub fn active_weight_by_role(
        &self,
        member_id: MemberId,
        instant: EventTime,
    ) -> BTreeMap<MembershipRole, f64> {
        let mut totals = BTreeMap::new();
        for assignment in self.active_memberships_for(member_id, instant) {
            *totals.entry(assignment.role()).or_insert(0.0) += assignment.weight().value();
        }
        totals
    }
}

#[cfg(test)]
mod tests {
    use super::MembershipNetwork;
    use crate::{GroupId, MemberId, MembershipAssignment, MembershipRole, MembershipWeight};
    use temporal_core::EventTime;

    fn event_time(value: &str) -> EventTime {
        EventTime::parse_rfc3339(value).expect("event time must parse")
    }

    #[test]
    fn inactive_and_foreign_members_are_excluded_from_active_queries() {
        let member = MemberId::new();
        let other = MemberId::new();
        let group = GroupId::new();
        let start = event_time("2026-01-01T00:00:00Z");
        let end = event_time("2026-01-31T00:00:00Z");
        let before = event_time("2025-12-01T00:00:00Z");
        let during = event_time("2026-01-15T00:00:00Z");

        let mut network = MembershipNetwork::new();
        network
            .insert(
                MembershipAssignment::new(
                    member,
                    group,
                    MembershipRole::Template,
                    MembershipWeight::full().expect("full"),
                    start,
                    end,
                )
                .expect("assignment"),
            )
            .expect("insert");

        assert!(network.active_memberships_for(member, before).is_empty());
        assert!(network.active_memberships_for(other, during).is_empty());
        let active = network.active_memberships_for(member, during);
        assert_eq!(active.len(), 1);
        assert_eq!(
            active[0].validity().certainty(),
            temporal_core::TemporalCertainty::Bounded
        );
        let assignment_total = network.assignments().count();
        assert_eq!(assignment_total, 1);
        assert_eq!(network.active_group_multiplicity(member, before), 0);
        assert!(network.active_weight_by_role(other, during).is_empty());
    }
}
