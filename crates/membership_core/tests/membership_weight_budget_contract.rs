//! Owner-level contracts for time-varying multiple-membership share budgets.

use membership_core::{
    GroupId, MemberId, MembershipAssignment, MembershipError, MembershipNetwork, MembershipRole,
    MembershipWeight,
};
use temporal_core::EventTime;

fn event_time(value: &str) -> EventTime {
    EventTime::parse_rfc3339(value).expect("event time must parse")
}

fn assignment(
    member_id: MemberId,
    group_id: GroupId,
    role: MembershipRole,
    weight: f64,
    valid_from: &str,
    valid_to: &str,
) -> MembershipAssignment {
    MembershipAssignment::new(
        member_id,
        group_id,
        role,
        MembershipWeight::new(weight).expect("bounded membership share"),
        event_time(valid_from),
        event_time(valid_to),
    )
    .expect("valid assignment")
}

#[test]
fn overlapping_same_role_shares_cannot_exceed_unity() {
    let member = MemberId::new();
    let first_group = GroupId::new();
    let second_group = GroupId::new();
    let as_of = event_time("2026-06-15T00:00:00Z");
    let mut network = MembershipNetwork::new();

    network
        .insert(assignment(
            member,
            first_group,
            MembershipRole::Project,
            0.75,
            "2026-01-01T00:00:00Z",
            "2026-12-31T23:59:59Z",
        ))
        .expect("first partial share remains valid");

    let before_count = network.assignment_count();
    let result = network.insert(assignment(
        member,
        second_group,
        MembershipRole::Project,
        0.5,
        "2026-03-01T00:00:00Z",
        "2026-09-30T23:59:59Z",
    ));

    assert_eq!(result, Err(MembershipError::InvalidMembershipWeight));
    assert_eq!(network.assignment_count(), before_count);
    assert_eq!(
        network
            .active_weight_by_role(member, as_of)
            .get(&MembershipRole::Project)
            .copied(),
        Some(0.75)
    );
}

#[test]
fn aggregate_budget_is_checked_across_more_than_one_existing_membership() {
    let member = MemberId::new();
    let mut network = MembershipNetwork::new();
    for group_id in [GroupId::new(), GroupId::new()] {
        network
            .insert(assignment(
                member,
                group_id,
                MembershipRole::Project,
                0.375,
                "2026-01-01T00:00:00Z",
                "2026-12-31T23:59:59Z",
            ))
            .expect("pair remains below unity");
    }

    assert_eq!(
        network.insert(assignment(
            member,
            GroupId::new(),
            MembershipRole::Project,
            0.375,
            "2026-01-01T00:00:00Z",
            "2026-12-31T23:59:59Z",
        )),
        Err(MembershipError::InvalidMembershipWeight)
    );
    assert_eq!(network.assignment_count(), 2);
}

#[test]
fn exact_unity_same_role_overlap_remains_valid_multiple_membership() {
    let member = MemberId::new();
    let mut network = MembershipNetwork::new();
    for (group_id, weight) in [(GroupId::new(), 0.75), (GroupId::new(), 0.25)] {
        network
            .insert(assignment(
                member,
                group_id,
                MembershipRole::Project,
                weight,
                "2026-01-01T00:00:00Z",
                "2026-12-31T23:59:59Z",
            ))
            .expect("shares summing to unity remain valid");
    }

    assert_eq!(
        network
            .active_weight_by_role(member, event_time("2026-06-15T00:00:00Z"))
            .get(&MembershipRole::Project)
            .copied(),
        Some(1.0)
    );
}

#[test]
fn role_budgets_are_independent_and_disjoint_spells_do_not_accumulate() {
    let member = MemberId::new();
    let mut network = MembershipNetwork::new();

    network
        .insert(assignment(
            member,
            GroupId::new(),
            MembershipRole::Project,
            1.0,
            "2026-01-01T00:00:00Z",
            "2026-03-31T23:59:59Z",
        ))
        .expect("first project spell");
    network
        .insert(assignment(
            member,
            GroupId::new(),
            MembershipRole::Project,
            1.0,
            "2026-04-01T00:00:00Z",
            "2026-06-30T23:59:59Z",
        ))
        .expect("disjoint project spell");
    network
        .insert(assignment(
            member,
            GroupId::new(),
            MembershipRole::Customer,
            1.0,
            "2026-01-01T00:00:00Z",
            "2026-12-31T23:59:59Z",
        ))
        .expect("independent role budget");

    assert_eq!(network.assignment_count(), 3);
}
