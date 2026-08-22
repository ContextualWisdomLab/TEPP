//! In-memory multiple-membership networks for multilevel estimation inputs.

use crate::{GroupId, MemberId, MembershipAssignment, MembershipError, MembershipRole};
use std::collections::{BTreeMap, BTreeSet};
use temporal_core::EventTime;

/// An in-memory network of weighted multiple memberships.
///
/// The network preserves every accepted assignment so estimators can model
/// cross-classification and multiple membership instead of collapsing documents
/// into independent rows (atomistic fallacy).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MembershipNetwork {
    assignments: Vec<MembershipAssignment>,
    keys: BTreeSet<(MemberId, GroupId, MembershipRole)>,
}

impl MembershipNetwork {
    /// Create an empty membership network.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert one validated assignment.
    ///
    /// # Errors
    ///
    /// Returns [`MembershipError::DuplicateMembershipAssignment`] when the same
    /// `(member, group, role)` key already exists. Validity revisions must be
    /// modeled as new temporal edges rather than silent overwrites.
    pub fn insert(&mut self, assignment: MembershipAssignment) -> Result<(), MembershipError> {
        let key = (
            assignment.member_id(),
            assignment.group_id(),
            assignment.role(),
        );
        if !self.keys.insert(key) {
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

    /// Emit one estimation row per active assignment at `instant`.
    ///
    /// Cross-classified and multiple-membership structure is preserved. Callers
    /// must not collapse these rows into a single independent observation.
    ///
    /// # Errors
    ///
    /// Returns [`MembershipError::InvalidWirePayload`] when the member has no
    /// active assignment at `instant`.
    pub fn estimation_rows_at(
        &self,
        member_id: MemberId,
        instant: EventTime,
    ) -> Result<Vec<EstimationMembershipRow>, MembershipError> {
        let rows: Vec<EstimationMembershipRow> = self
            .active_memberships_for(member_id, instant)
            .into_iter()
            .map(EstimationMembershipRow::from_assignment)
            .collect();
        if rows.is_empty() {
            return Err(MembershipError::InvalidWirePayload);
        }
        Ok(rows)
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

/// One multilevel estimation row; never a collapsed independent observation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EstimationMembershipRow {
    member_id: MemberId,
    group_id: GroupId,
    role: MembershipRole,
    weight: f64,
}

impl EstimationMembershipRow {
    /// Copy the scientifically relevant fields from one assignment.
    #[must_use]
    pub fn from_assignment(assignment: MembershipAssignment) -> Self {
        Self {
            member_id: assignment.member_id(),
            group_id: assignment.group_id(),
            role: assignment.role(),
            weight: assignment.weight().value(),
        }
    }

    /// Member identity on this row.
    #[must_use]
    pub const fn member_id(self) -> MemberId {
        self.member_id
    }

    /// Group identity on this row.
    #[must_use]
    pub fn group_id(self) -> GroupId {
        self.group_id
    }

    /// Contextual role on this row.
    #[must_use]
    pub const fn role(self) -> MembershipRole {
        self.role
    }

    /// Membership weight used by the multilevel estimator.
    #[must_use]
    pub const fn weight(self) -> f64 {
        self.weight
    }
}

/// Refuse an estimator input that dropped known multiple memberships.
///
/// Rows must represent one member and must preserve at least the required
/// number of distinct group identities. Multiple roles within one group cannot
/// substitute for a missing group context.
///
/// # Errors
///
/// Returns [`MembershipError::InvalidWirePayload`] for an empty row set, a zero
/// required multiplicity, or mixed member identities. Returns
/// [`MembershipError::AtomisticCollapseRefused`] when fewer distinct groups than
/// `required_group_multiplicity` remain.
pub fn refuse_atomistic_collapse(
    rows: &[EstimationMembershipRow],
    required_group_multiplicity: usize,
) -> Result<(), MembershipError> {
    if rows.is_empty() || required_group_multiplicity == 0 {
        return Err(MembershipError::InvalidWirePayload);
    }
    let expected_member = rows[0].member_id();
    let mut distinct_groups = BTreeSet::new();
    for row in rows {
        if row.member_id() != expected_member {
            return Err(MembershipError::InvalidWirePayload);
        }
        distinct_groups.insert(row.group_id());
    }
    if distinct_groups.len() < required_group_multiplicity {
        return Err(MembershipError::AtomisticCollapseRefused);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::MembershipNetwork;
    use crate::{
        GroupId, MemberId, MembershipAssignment, MembershipError, MembershipRole, MembershipWeight,
    };
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
        assert_eq!(network.active_group_multiplicity(member, during), 1);
        let active_weights = network.active_weight_by_role(member, during);
        assert_eq!(active_weights.get(&MembershipRole::Template), Some(&1.0));
        assert_eq!(
            active[0].validity().certainty(),
            temporal_core::TemporalCertainty::Bounded
        );
        let assignment_total = network.assignments().count();
        assert_eq!(assignment_total, 1);
        assert_eq!(network.active_group_multiplicity(member, before), 0);
        assert!(network.active_weight_by_role(other, during).is_empty());
        let rows = network.estimation_rows_at(member, during).expect("one row");
        assert_eq!(rows.len(), 1);
        assert_eq!(
            network.estimation_rows_at(other, during),
            Err(MembershipError::InvalidWirePayload)
        );
        assert_eq!(
            network.insert(active[0]),
            Err(MembershipError::DuplicateMembershipAssignment)
        );
        network
            .insert(
                MembershipAssignment::new(
                    other,
                    GroupId::new(),
                    MembershipRole::Project,
                    MembershipWeight::full().expect("full"),
                    start,
                    end,
                )
                .expect("other assignment"),
            )
            .expect("other insert");
        super::refuse_atomistic_collapse(&rows, 1).expect("single membership");
        assert_eq!(
            super::refuse_atomistic_collapse(&[], 1),
            Err(MembershipError::InvalidWirePayload)
        );
        assert_eq!(
            super::refuse_atomistic_collapse(&rows, 2),
            Err(MembershipError::AtomisticCollapseRefused)
        );
        let mut mixed_rows = rows.clone();
        mixed_rows.extend(
            network
                .estimation_rows_at(other, during)
                .expect("other row"),
        );
        assert_eq!(
            super::refuse_atomistic_collapse(&mixed_rows, 2),
            Err(MembershipError::InvalidWirePayload)
        );
        assert_eq!(rows[0].member_id(), member);
        assert_eq!(rows[0].group_id(), group);
        assert_eq!(rows[0].role(), MembershipRole::Template);
        assert!((rows[0].weight() - 1.0).abs() < 1e-15);
        assert_eq!(
            super::refuse_atomistic_collapse(&rows, 0),
            Err(MembershipError::InvalidWirePayload)
        );
    }
}
