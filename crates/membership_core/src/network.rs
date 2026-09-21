//! In-memory multiple-membership networks for multilevel estimation inputs.

use crate::{GroupId, MemberId, MembershipAssignment, MembershipError, MembershipRole};
use std::collections::{BTreeMap, BTreeSet};
use temporal_core::{AllenRelation, EventTime, TemporalBoundary, classify_interval_relation};

const BINARY64_MIN_EXPONENT: i32 = -1074;
const BINARY64_UNIT_EXPONENT_INDEX: usize = 1074;

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
    /// Concurrent assignments for the same `(member, role)` may represent
    /// multiple membership across groups, but their known active shares must not
    /// exceed `1.0`. The admission check compares the exact sum of the represented
    /// non-negative binary64 values with unity; ordinary `f64` accumulation is not
    /// used because a true overrun can round back to exactly `1.0`. Partial local
    /// views may sum below unity; insertion never normalizes, clamps, or infers a
    /// missing complementary membership.
    ///
    /// # Errors
    ///
    /// Returns [`MembershipError::DuplicateMembershipAssignment`] when an
    /// existing assignment has the same `(member, group, role)` identity and its
    /// validity interval is not strictly before or after the candidate interval.
    /// Returns [`MembershipError::InvalidMembershipWeight`] when inserting the
    /// candidate would make known concurrent shares for the same `(member, role)`
    /// exceed unity at any event time.
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
        if self.exceeds_same_role_share_budget(assignment) {
            return Err(MembershipError::InvalidMembershipWeight);
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

    fn exceeds_same_role_share_budget(&self, candidate: MembershipAssignment) -> bool {
        let candidate_validity = candidate.validity();
        let mut checkpoints = BTreeSet::new();
        collect_known_boundary(&mut checkpoints, candidate_validity.lower());
        collect_known_boundary(&mut checkpoints, candidate_validity.upper());

        for existing in self.assignments.iter().copied().filter(|existing| {
            existing.member_id() == candidate.member_id() && existing.role() == candidate.role()
        }) {
            let validity = existing.validity();
            collect_known_boundary(&mut checkpoints, validity.lower());
            collect_known_boundary(&mut checkpoints, validity.upper());
        }

        checkpoints.into_iter().any(|instant| {
            if !candidate.is_active_at(instant) {
                return false;
            }
            let existing_weights = self
                .assignments
                .iter()
                .copied()
                .filter(|existing| {
                    existing.member_id() == candidate.member_id()
                        && existing.role() == candidate.role()
                        && existing.is_active_at(instant)
                })
                .map(|existing| existing.weight().value());
            exact_nonnegative_binary64_sum_exceeds_one(
                existing_weights.chain(std::iter::once(candidate.weight().value())),
            )
        })
    }
}

fn collect_known_boundary(
    checkpoints: &mut BTreeSet<EventTime>,
    boundary: TemporalBoundary<EventTime>,
) {
    match boundary {
        TemporalBoundary::Included(instant) | TemporalBoundary::Excluded(instant) => {
            checkpoints.insert(instant);
        }
        TemporalBoundary::Unbounded => {}
    }
}

/// Compare a sum of canonical non-negative binary64 weights with unity exactly.
///
/// Each finite binary64 value is already an exact dyadic rational. Expanding its
/// significand into powers of two and carrying those bins upward preserves that
/// represented value without introducing another rounding step. This keeps the
/// membership admission boundary exact while leaving estimator arithmetic to its
/// numerical owner.
pub(crate) fn exact_nonnegative_binary64_sum_exceeds_one<I>(weights: I) -> bool
where
    I: IntoIterator<Item = f64>,
{
    let mut exponent_bins = [0_u128; BINARY64_UNIT_EXPONENT_INDEX + 1];
    for value in weights {
        debug_assert!(value.is_finite() && (0.0..=1.0).contains(&value));
        if value == 0.0 {
            continue;
        }

        let bits = value.to_bits();
        let raw_exponent = i32::try_from((bits >> 52) & 0x7ff)
            .expect("binary64 exponent field always fits in i32");
        let fraction = bits & ((1_u64 << 52) - 1);
        let (mut significand, base_exponent) = if raw_exponent == 0 {
            (fraction, BINARY64_MIN_EXPONENT)
        } else {
            ((1_u64 << 52) | fraction, raw_exponent - 1075)
        };
        let mut significand_bit = 0_i32;
        while significand != 0 {
            if significand & 1 == 1 {
                let exponent = base_exponent + significand_bit;
                let index = usize::try_from(exponent - BINARY64_MIN_EXPONENT)
                    .expect("validated membership weights never fall below binary64 minimum");
                exponent_bins[index] += 1;
            }
            significand >>= 1;
            significand_bit += 1;
        }
    }

    for index in 0..BINARY64_UNIT_EXPONENT_INDEX {
        let carry = exponent_bins[index] / 2;
        exponent_bins[index] %= 2;
        exponent_bins[index + 1] += carry;
    }

    let units = exponent_bins[BINARY64_UNIT_EXPONENT_INDEX];
    units > 1
        || (units == 1
            && exponent_bins[..BINARY64_UNIT_EXPONENT_INDEX]
                .iter()
                .any(|&bit| bit != 0))
}

#[cfg(test)]
mod tests {
    use super::{MembershipNetwork, exact_nonnegative_binary64_sum_exceeds_one};
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

    #[test]
    fn exact_share_budget_comparison_covers_zero_subnormal_unity_and_overrun() {
        let minimum_subnormal = f64::from_bits(1);
        assert!(!exact_nonnegative_binary64_sum_exceeds_one([
            0.0,
            -0.0,
            minimum_subnormal
        ]));
        assert!(!exact_nonnegative_binary64_sum_exceeds_one([0.75, 0.25]));
        assert!(exact_nonnegative_binary64_sum_exceeds_one([
            1.0,
            minimum_subnormal
        ]));
        assert!(exact_nonnegative_binary64_sum_exceeds_one([0.75, 0.5]));
    }
}
