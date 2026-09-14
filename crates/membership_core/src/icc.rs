//! Nested ICC with fail-closed cross-classified and multiple-membership gates.

use crate::{MemberId, MembershipError, MembershipNetwork, MembershipRole};
use std::collections::{BTreeMap, BTreeSet};
use temporal_core::EventTime;

/// Membership design implied by active assignments at one event time.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum MembershipDesign {
    /// Each active member belongs to exactly one group in one role.
    Nested,
    /// At least one member is active in two or more roles.
    CrossClassified,
    /// At least one member is active in two or more groups of the same role.
    MultipleMembership,
}

impl MembershipDesign {
    /// Return the stable wire name for this membership design.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Nested => "nested",
            Self::CrossClassified => "cross_classified",
            Self::MultipleMembership => "multiple_membership",
        }
    }

    /// Return whether a one-way nested ICC is identified for this design.
    #[must_use]
    pub const fn allows_nested_icc(self) -> bool {
        match self {
            Self::Nested => true,
            Self::CrossClassified | Self::MultipleMembership => false,
        }
    }
}

/// One finite outcome attached to an opaque member for nested ICC recovery.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NestedOutcome {
    member_id: MemberId,
    value: f64,
}

impl NestedOutcome {
    /// Construct a finite nested ICC outcome.
    ///
    /// # Errors
    ///
    /// Returns [`MembershipError::InvalidOutcome`] when `value` is not finite.
    pub fn new(member_id: MemberId, value: f64) -> Result<Self, MembershipError> {
        if value.is_finite() {
            Ok(Self { member_id, value })
        } else {
            Err(MembershipError::InvalidOutcome)
        }
    }

    /// Return the outcome member.
    #[must_use]
    pub const fn member_id(self) -> MemberId {
        self.member_id
    }

    /// Return the finite outcome value.
    #[must_use]
    pub const fn value(self) -> f64 {
        self.value
    }
}

/// Classify active memberships at `instant` without collapsing structure.
///
/// Multiple membership is reported before cross-classification so a member
/// who occupies two groups in one role is not misread as a nested hierarchy.
///
/// # Errors
///
/// Returns [`MembershipError::InsufficientClusterStructure`] when no assignment
/// is active at `instant`.
pub fn classify_membership_design(
    network: &MembershipNetwork,
    instant: EventTime,
) -> Result<MembershipDesign, MembershipError> {
    let mut members = BTreeSet::new();
    for assignment in network.assignments() {
        members.insert(assignment.member_id());
    }
    classify_members(network, instant, members.iter().copied())
}

/// Recover the one-way random-intercept ICC for a nested membership design.
///
/// The CPU `f64` estimator is the unbalanced ANOVA (Snijders & Bosker)
/// estimator
/// `σ²_u / (σ²_u + σ²_e)` with
/// `σ²_e = MSW` and `σ²_u = max(0, (MSB − MSW) / n₀)`.
/// Cross-classified and multiple-membership designs fail closed: a nested
/// ICC is not a substitute for an MMMC model.
///
/// # Errors
///
/// Returns a membership error when outcomes are empty, duplicated, unknown,
/// inactive, non-nested, or when cluster residual structure is insufficient.
pub fn nested_intraclass_correlation(
    network: &MembershipNetwork,
    instant: EventTime,
    outcomes: &[NestedOutcome],
) -> Result<f64, MembershipError> {
    if outcomes.is_empty() {
        return Err(MembershipError::InsufficientClusterStructure);
    }
    let mut seen = BTreeSet::new();
    let mut groups: BTreeMap<crate::GroupId, Vec<f64>> = BTreeMap::new();
    let mut outcome_members = BTreeSet::new();
    for outcome in outcomes {
        if !seen.insert(outcome.member_id()) {
            return Err(MembershipError::DuplicateOutcomeMember);
        }
        let active = network.active_memberships_for(outcome.member_id(), instant);
        if active.is_empty() {
            return Err(MembershipError::UnknownOutcomeMember);
        }
        outcome_members.insert(outcome.member_id());
        let group = active[0].group_id();
        groups.entry(group).or_default().push(outcome.value());
    }
    match classify_members(network, instant, outcome_members.iter().copied())? {
        MembershipDesign::Nested => {}
        MembershipDesign::CrossClassified | MembershipDesign::MultipleMembership => {
            return Err(MembershipError::NestedIccInapplicable);
        }
    }
    anova_nested_icc(&groups)
}

fn classify_members<I>(
    network: &MembershipNetwork,
    instant: EventTime,
    members: I,
) -> Result<MembershipDesign, MembershipError>
where
    I: IntoIterator<Item = MemberId>,
{
    let mut saw_active = false;
    let mut saw_cross = false;
    for member_id in members {
        let active = network.active_memberships_for(member_id, instant);
        if active.is_empty() {
            continue;
        }
        saw_active = true;
        let mut groups_by_role: BTreeMap<MembershipRole, BTreeSet<crate::GroupId>> =
            BTreeMap::new();
        for assignment in active {
            groups_by_role
                .entry(assignment.role())
                .or_default()
                .insert(assignment.group_id());
        }
        for groups in groups_by_role.values() {
            if groups.len() >= 2 {
                return Ok(MembershipDesign::MultipleMembership);
            }
        }
        if groups_by_role.len() >= 2 {
            saw_cross = true;
        }
    }
    if !saw_active {
        return Err(MembershipError::InsufficientClusterStructure);
    }
    if saw_cross {
        Ok(MembershipDesign::CrossClassified)
    } else {
        Ok(MembershipDesign::Nested)
    }
}

fn anova_nested_icc(groups: &BTreeMap<crate::GroupId, Vec<f64>>) -> Result<f64, MembershipError> {
    let cluster_count = groups.len();
    if cluster_count < 2 {
        return Err(MembershipError::InsufficientClusterStructure);
    }
    let mut sample_size = 0_usize;
    let mut total = 0.0;
    for values in groups.values() {
        sample_size += values.len();
        for &value in values {
            total += value;
        }
    }
    if sample_size <= cluster_count {
        return Err(MembershipError::InsufficientClusterStructure);
    }
    let n = sample_size as f64;
    let j = cluster_count as f64;
    let grand_mean = total / n;
    let mut sum_of_squares_between = 0.0;
    let mut sum_of_squares_within = 0.0;
    let mut sum_cluster_size_squared = 0.0;
    for values in groups.values() {
        let cluster_size = values.len() as f64;
        sum_cluster_size_squared += cluster_size * cluster_size;
        let mut cluster_total = 0.0;
        for &value in values {
            cluster_total += value;
        }
        let cluster_mean = cluster_total / cluster_size;
        let between = cluster_mean - grand_mean;
        sum_of_squares_between += cluster_size * between * between;
        for &value in values {
            let within = value - cluster_mean;
            sum_of_squares_within += within * within;
        }
    }
    if sum_of_squares_between + sum_of_squares_within == 0.0 {
        return Err(MembershipError::InsufficientClusterStructure);
    }
    let mean_square_between = sum_of_squares_between / (j - 1.0);
    let mean_square_within = sum_of_squares_within / (n - j);
    let harmonic_cluster_size = (n - sum_cluster_size_squared / n) / (j - 1.0);
    let cluster_variance =
        ((mean_square_between - mean_square_within) / harmonic_cluster_size).max(0.0);
    Ok(cluster_variance / (cluster_variance + mean_square_within))
}

#[cfg(test)]
mod tests {
    use super::{MembershipDesign, NestedOutcome, anova_nested_icc};
    use crate::{GroupId, MemberId, MembershipError};
    use std::collections::BTreeMap;

    #[test]
    fn design_gate_and_outcome_accessors_cover_local_branches() {
        assert!(MembershipDesign::Nested.allows_nested_icc());
        assert!(!MembershipDesign::CrossClassified.allows_nested_icc());
        assert!(!MembershipDesign::MultipleMembership.allows_nested_icc());
        assert_eq!(MembershipDesign::Nested.wire_name(), "nested");
        assert_eq!(
            MembershipDesign::CrossClassified.wire_name(),
            "cross_classified"
        );
        assert_eq!(
            MembershipDesign::MultipleMembership.wire_name(),
            "multiple_membership"
        );
        let member = MemberId::new();
        let outcome = NestedOutcome::new(member, 1.5).expect("finite");
        assert_eq!(outcome.member_id(), member);
        assert!((outcome.value() - 1.5).abs() < 1e-12);
        assert_eq!(
            NestedOutcome::new(member, f64::NEG_INFINITY),
            Err(MembershipError::InvalidOutcome)
        );
        let mut one = BTreeMap::new();
        one.insert(GroupId::new(), vec![1.0, 2.0]);
        assert_eq!(
            anova_nested_icc(&one),
            Err(MembershipError::InsufficientClusterStructure)
        );
    }
}
