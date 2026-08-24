//! Nested ICC recovery and fail-closed design-refusal contracts.
//!
//! These tests exist before the public ICC API so the increment is RED first.

use membership_core::{
    GroupId, MemberId, MembershipAssignment, MembershipDesign, MembershipError, MembershipNetwork,
    MembershipRole, MembershipWeight, NestedOutcome, classify_membership_design,
    nested_intraclass_correlation,
};
use temporal_core::EventTime;

fn event_time(value: &str) -> EventTime {
    EventTime::parse_rfc3339(value).expect("event time must parse")
}

fn insert_nested(
    network: &mut MembershipNetwork,
    member: MemberId,
    group: GroupId,
    start: EventTime,
    end: EventTime,
) {
    network
        .insert(
            MembershipAssignment::new(
                member,
                group,
                MembershipRole::Author,
                MembershipWeight::full().expect("full weight"),
                start,
                end,
            )
            .expect("assignment"),
        )
        .expect("insert");
}

#[test]
fn balanced_anova_recovers_known_icc_and_reports_computed_rmse() {
    let start = event_time("2026-01-01T00:00:00Z");
    let end = event_time("2026-12-31T00:00:00Z");
    let as_of = event_time("2026-06-01T00:00:00Z");
    let groups = [
        GroupId::new(),
        GroupId::new(),
        GroupId::new(),
        GroupId::new(),
    ];
    // Four groups of two: means 2,3,4,5 with within deviation ±1.
    let rows = [
        (groups[0], [1.0, 3.0]),
        (groups[1], [2.0, 4.0]),
        (groups[2], [3.0, 5.0]),
        (groups[3], [4.0, 6.0]),
    ];
    let mut network = MembershipNetwork::new();
    let mut outcomes = Vec::new();
    for (group, values) in rows {
        for value in values {
            let member = MemberId::new();
            insert_nested(&mut network, member, group, start, end);
            outcomes.push(NestedOutcome::new(member, value).expect("finite outcome"));
        }
    }

    assert_eq!(
        classify_membership_design(&network, as_of).expect("design"),
        MembershipDesign::Nested
    );
    assert!(MembershipDesign::Nested.allows_nested_icc());
    assert!(!MembershipDesign::CrossClassified.allows_nested_icc());
    assert!(!MembershipDesign::MultipleMembership.allows_nested_icc());

    let recovered = nested_intraclass_correlation(&network, as_of, &outcomes).expect("nested ICC");
    let true_icc = 0.25;
    let rmse = (recovered - true_icc).abs();
    assert!(
        rmse < 1e-12,
        "balanced ANOVA must recover ICC=1/4; recovered={recovered}, rmse={rmse}"
    );
}

#[test]
fn zero_within_variance_recovers_unit_icc_and_equal_groups_recover_zero() {
    let start = event_time("2026-01-01T00:00:00Z");
    let end = event_time("2026-12-31T00:00:00Z");
    let as_of = event_time("2026-06-01T00:00:00Z");

    let mut perfect = MembershipNetwork::new();
    let g_lo = GroupId::new();
    let g_hi = GroupId::new();
    let mut perfect_outcomes = Vec::new();
    for value in [0.0, 0.0] {
        let member = MemberId::new();
        insert_nested(&mut perfect, member, g_lo, start, end);
        perfect_outcomes.push(NestedOutcome::new(member, value).expect("lo"));
    }
    for value in [4.0, 4.0] {
        let member = MemberId::new();
        insert_nested(&mut perfect, member, g_hi, start, end);
        perfect_outcomes.push(NestedOutcome::new(member, value).expect("hi"));
    }
    let unit = nested_intraclass_correlation(&perfect, as_of, &perfect_outcomes).expect("icc=1");
    assert!((unit - 1.0).abs() < 1e-12);

    let mut flat = MembershipNetwork::new();
    let g_a = GroupId::new();
    let g_b = GroupId::new();
    let mut flat_outcomes = Vec::new();
    for value in [1.0, 3.0] {
        let member = MemberId::new();
        insert_nested(&mut flat, member, g_a, start, end);
        flat_outcomes.push(NestedOutcome::new(member, value).expect("a"));
    }
    for value in [0.0, 4.0] {
        let member = MemberId::new();
        insert_nested(&mut flat, member, g_b, start, end);
        flat_outcomes.push(NestedOutcome::new(member, value).expect("b"));
    }
    let zero = nested_intraclass_correlation(&flat, as_of, &flat_outcomes).expect("icc=0");
    assert!(zero.abs() < 1e-12);
}

#[test]
fn cross_classified_and_multiple_membership_refuse_nested_icc() {
    let start = event_time("2026-01-01T00:00:00Z");
    let end = event_time("2026-12-31T00:00:00Z");
    let as_of = event_time("2026-06-01T00:00:00Z");
    let member = MemberId::new();
    let author = GroupId::new();
    let project = GroupId::new();

    let mut cross = MembershipNetwork::new();
    insert_nested(&mut cross, member, author, start, end);
    cross
        .insert(
            MembershipAssignment::new(
                member,
                project,
                MembershipRole::Project,
                MembershipWeight::new(0.5).expect("partial"),
                start,
                end,
            )
            .expect("project"),
        )
        .expect("insert project");
    assert_eq!(
        classify_membership_design(&cross, as_of).expect("cc"),
        MembershipDesign::CrossClassified
    );
    let outcome = NestedOutcome::new(member, 1.0).expect("outcome");
    assert_eq!(
        nested_intraclass_correlation(&cross, as_of, &[outcome]),
        Err(MembershipError::NestedIccInapplicable)
    );

    let mut multiple = MembershipNetwork::new();
    let dept_a = GroupId::new();
    let dept_b = GroupId::new();
    multiple
        .insert(
            MembershipAssignment::new(
                member,
                dept_a,
                MembershipRole::Department,
                MembershipWeight::new(0.6).expect("a"),
                start,
                end,
            )
            .expect("dept a"),
        )
        .expect("insert a");
    multiple
        .insert(
            MembershipAssignment::new(
                member,
                dept_b,
                MembershipRole::Department,
                MembershipWeight::new(0.4).expect("b"),
                start,
                end,
            )
            .expect("dept b"),
        )
        .expect("insert b");
    assert_eq!(
        classify_membership_design(&multiple, as_of).expect("mm"),
        MembershipDesign::MultipleMembership
    );
    assert_eq!(
        nested_intraclass_correlation(&multiple, as_of, &[outcome]),
        Err(MembershipError::NestedIccInapplicable)
    );
}

#[test]
fn nested_icc_fails_closed_on_degenerate_inputs() {
    let start = event_time("2026-01-01T00:00:00Z");
    let end = event_time("2026-12-31T00:00:00Z");
    let as_of = event_time("2026-06-01T00:00:00Z");
    let before = event_time("2025-01-01T00:00:00Z");

    assert_eq!(
        NestedOutcome::new(MemberId::new(), f64::NAN),
        Err(MembershipError::InvalidOutcome)
    );
    assert_eq!(
        NestedOutcome::new(MemberId::new(), f64::INFINITY),
        Err(MembershipError::InvalidOutcome)
    );

    let empty = MembershipNetwork::new();
    assert_eq!(
        classify_membership_design(&empty, as_of),
        Err(MembershipError::InsufficientClusterStructure)
    );
    assert_eq!(
        nested_intraclass_correlation(&empty, as_of, &[]),
        Err(MembershipError::InsufficientClusterStructure)
    );

    let member = MemberId::new();
    let group = GroupId::new();
    let mut inactive = MembershipNetwork::new();
    insert_nested(&mut inactive, member, group, start, end);
    assert_eq!(
        classify_membership_design(&inactive, before),
        Err(MembershipError::InsufficientClusterStructure)
    );
    let outcome = NestedOutcome::new(member, 1.0).expect("finite");
    assert_eq!(
        nested_intraclass_correlation(&inactive, before, &[outcome]),
        Err(MembershipError::UnknownOutcomeMember)
    );
    assert_eq!(
        nested_intraclass_correlation(&inactive, as_of, &[outcome, outcome]),
        Err(MembershipError::DuplicateOutcomeMember)
    );
    assert_eq!(
        nested_intraclass_correlation(
            &inactive,
            as_of,
            &[NestedOutcome::new(MemberId::new(), 1.0).expect("stranger")]
        ),
        Err(MembershipError::UnknownOutcomeMember)
    );

    let only = NestedOutcome::new(member, 2.0).expect("one");
    assert_eq!(
        nested_intraclass_correlation(&inactive, as_of, &[only]),
        Err(MembershipError::InsufficientClusterStructure)
    );

    let mut singletons = MembershipNetwork::new();
    let g1 = GroupId::new();
    let g2 = GroupId::new();
    let m1 = MemberId::new();
    let m2 = MemberId::new();
    insert_nested(&mut singletons, m1, g1, start, end);
    insert_nested(&mut singletons, m2, g2, start, end);
    assert_eq!(
        nested_intraclass_correlation(
            &singletons,
            as_of,
            &[
                NestedOutcome::new(m1, 1.0).expect("s1"),
                NestedOutcome::new(m2, 2.0).expect("s2"),
            ]
        ),
        Err(MembershipError::InsufficientClusterStructure)
    );

    let mut constants = MembershipNetwork::new();
    let c1 = GroupId::new();
    let c2 = GroupId::new();
    let mut constant_outcomes = Vec::new();
    for group in [c1, c2] {
        for _ in 0..2 {
            let member = MemberId::new();
            insert_nested(&mut constants, member, group, start, end);
            constant_outcomes.push(NestedOutcome::new(member, 5.0).expect("const"));
        }
    }
    assert_eq!(
        nested_intraclass_correlation(&constants, as_of, &constant_outcomes),
        Err(MembershipError::InsufficientClusterStructure)
    );

    let inspected = NestedOutcome::new(member, -2.5).expect("signed");
    assert_eq!(inspected.member_id(), member);
    assert!((inspected.value() + 2.5).abs() < 1e-12);
}
