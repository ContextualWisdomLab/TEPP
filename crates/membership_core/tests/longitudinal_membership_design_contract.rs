use membership_core::{
    GroupId, MemberId, MembershipAssignment, MembershipDesign, MembershipError,
    MembershipNetwork, MembershipObservation, MembershipRole, MembershipWeight,
    classify_membership_observations,
};
use temporal_core::EventTime;

fn event_time(value: &str) -> EventTime {
    EventTime::parse_rfc3339(value).expect("event time must parse")
}

fn insert(
    network: &mut MembershipNetwork,
    member_id: MemberId,
    group_id: GroupId,
    role: MembershipRole,
    weight: f64,
    start: EventTime,
    end: EventTime,
) {
    network
        .insert(
            MembershipAssignment::new(
                member_id,
                group_id,
                role,
                MembershipWeight::new(weight).expect("bounded membership weight"),
                start,
                end,
            )
            .expect("valid assignment"),
        )
        .expect("non-conflicting assignment");
}

#[test]
fn longitudinal_support_preserves_reentry_without_inventing_multiple_membership() {
    let jan_start = event_time("2026-01-01T00:00:00Z");
    let mar_end = event_time("2026-03-31T23:59:59Z");
    let sep_start = event_time("2026-09-01T00:00:00Z");
    let dec_end = event_time("2026-12-31T23:59:59Z");
    let feb = event_time("2026-02-15T00:00:00Z");
    let oct = event_time("2026-10-15T00:00:00Z");
    let member = MemberId::new();
    let mut network = MembershipNetwork::new();

    insert(
        &mut network,
        member,
        GroupId::new(),
        MembershipRole::Department,
        1.0,
        jan_start,
        mar_end,
    );
    insert(
        &mut network,
        member,
        GroupId::new(),
        MembershipRole::Department,
        1.0,
        sep_start,
        dec_end,
    );

    let observations = [
        MembershipObservation::new(member, feb),
        MembershipObservation::new(member, oct),
    ];
    assert_eq!(observations[0].member_id(), member);
    assert_eq!(observations[0].event_time(), feb);
    assert_eq!(
        classify_membership_observations(&network, &observations).expect("longitudinal design"),
        MembershipDesign::Nested,
    );
}

#[test]
fn longitudinal_support_detects_classification_heterogeneity_across_event_times() {
    let jan_start = event_time("2026-01-01T00:00:00Z");
    let jun_end = event_time("2026-06-30T23:59:59Z");
    let jul_start = event_time("2026-07-01T00:00:00Z");
    let dec_end = event_time("2026-12-31T23:59:59Z");
    let mar = event_time("2026-03-01T00:00:00Z");
    let oct = event_time("2026-10-01T00:00:00Z");
    let member = MemberId::new();
    let mut network = MembershipNetwork::new();

    insert(
        &mut network,
        member,
        GroupId::new(),
        MembershipRole::Author,
        1.0,
        jan_start,
        jun_end,
    );
    insert(
        &mut network,
        member,
        GroupId::new(),
        MembershipRole::Project,
        1.0,
        jul_start,
        dec_end,
    );

    let observations = [
        MembershipObservation::new(member, mar),
        MembershipObservation::new(member, oct),
    ];
    assert_eq!(
        classify_membership_observations(&network, &observations).expect("heterogeneous design"),
        MembershipDesign::HeterogeneousClassification,
    );
}

#[test]
fn longitudinal_support_preserves_cross_classified_and_multiple_membership_signals() {
    let start = event_time("2026-01-01T00:00:00Z");
    let end = event_time("2026-12-31T23:59:59Z");
    let spring = event_time("2026-04-01T00:00:00Z");
    let autumn = event_time("2026-10-01T00:00:00Z");
    let cross_member = MemberId::new();
    let multiple_member = MemberId::new();
    let mut network = MembershipNetwork::new();

    insert(
        &mut network,
        cross_member,
        GroupId::new(),
        MembershipRole::Author,
        1.0,
        start,
        end,
    );
    insert(
        &mut network,
        cross_member,
        GroupId::new(),
        MembershipRole::Project,
        1.0,
        start,
        end,
    );
    insert(
        &mut network,
        multiple_member,
        GroupId::new(),
        MembershipRole::Department,
        0.6,
        start,
        end,
    );
    insert(
        &mut network,
        multiple_member,
        GroupId::new(),
        MembershipRole::Department,
        0.4,
        start,
        end,
    );

    let observations = [
        MembershipObservation::new(cross_member, spring),
        MembershipObservation::new(multiple_member, autumn),
    ];
    assert_eq!(
        classify_membership_observations(&network, &observations).expect("mixed design"),
        MembershipDesign::CrossClassifiedMultipleMembership,
    );
}

#[test]
fn longitudinal_support_keeps_partial_weight_as_multiple_membership() {
    let start = event_time("2026-01-01T00:00:00Z");
    let end = event_time("2026-12-31T23:59:59Z");
    let as_of = event_time("2026-06-01T00:00:00Z");
    let nested_member = MemberId::new();
    let partial_member = MemberId::new();
    let mut network = MembershipNetwork::new();

    insert(
        &mut network,
        nested_member,
        GroupId::new(),
        MembershipRole::Department,
        1.0,
        start,
        end,
    );
    insert(
        &mut network,
        partial_member,
        GroupId::new(),
        MembershipRole::Department,
        0.5,
        start,
        end,
    );

    let observations = [
        MembershipObservation::new(nested_member, as_of),
        MembershipObservation::new(partial_member, as_of),
    ];
    assert_eq!(
        classify_membership_observations(&network, &observations).expect("multiple design"),
        MembershipDesign::MultipleMembership,
    );
}

#[test]
fn longitudinal_support_refuses_missing_membership_and_empty_support() {
    let member = MemberId::new();
    let instant = event_time("2026-06-01T00:00:00Z");
    let network = MembershipNetwork::new();
    let missing = [MembershipObservation::new(member, instant)];

    assert_eq!(
        classify_membership_observations(&network, &missing),
        Err(MembershipError::MissingObservationMembership),
    );
    assert_eq!(
        classify_membership_observations(&network, &[]),
        Err(MembershipError::InsufficientClusterStructure),
    );
}
