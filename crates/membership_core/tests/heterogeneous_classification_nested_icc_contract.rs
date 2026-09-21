use membership_core::{
    GroupId, MemberId, MembershipAssignment, MembershipDesign, MembershipError, MembershipNetwork,
    MembershipRole, MembershipWeight, NestedOutcome, classify_membership_design,
    nested_intraclass_correlation,
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
fn heterogeneous_population_roles_are_not_one_nested_classification() {
    let start = event_time("2026-01-01T00:00:00Z");
    let end = event_time("2026-12-31T00:00:00Z");
    let as_of = event_time("2026-06-01T00:00:00Z");
    let department = GroupId::new();
    let project = GroupId::new();
    let members = [MemberId::new(), MemberId::new(), MemberId::new(), MemberId::new()];
    let mut network = MembershipNetwork::new();

    insert(
        &mut network,
        members[0],
        department,
        MembershipRole::Department,
        1.0,
        start,
        end,
    );
    insert(
        &mut network,
        members[1],
        department,
        MembershipRole::Department,
        1.0,
        start,
        end,
    );
    insert(
        &mut network,
        members[2],
        project,
        MembershipRole::Project,
        1.0,
        start,
        end,
    );
    insert(
        &mut network,
        members[3],
        project,
        MembershipRole::Project,
        1.0,
        start,
        end,
    );

    assert_eq!(
        classify_membership_design(&network, as_of).expect("active design"),
        MembershipDesign::HeterogeneousClassification,
        "different active role/classification dimensions must not collapse into one nested level",
    );

    let outcomes = [
        NestedOutcome::new(members[0], 1.0).expect("finite outcome"),
        NestedOutcome::new(members[1], 1.5).expect("finite outcome"),
        NestedOutcome::new(members[2], 3.0).expect("finite outcome"),
        NestedOutcome::new(members[3], 3.5).expect("finite outcome"),
    ];
    assert_eq!(
        nested_intraclass_correlation(&network, as_of, &outcomes),
        Err(MembershipError::NestedIccInapplicable),
        "one-way nested ICC must fail closed across heterogeneous classification roles",
    );
}

#[test]
fn heterogeneous_roles_do_not_hide_multiple_membership_signal() {
    let start = event_time("2026-01-01T00:00:00Z");
    let end = event_time("2026-12-31T00:00:00Z");
    let as_of = event_time("2026-06-01T00:00:00Z");
    let mut network = MembershipNetwork::new();

    insert(
        &mut network,
        MemberId::new(),
        GroupId::new(),
        MembershipRole::Department,
        0.6,
        start,
        end,
    );
    insert(
        &mut network,
        MemberId::new(),
        GroupId::new(),
        MembershipRole::Project,
        1.0,
        start,
        end,
    );

    assert_eq!(
        classify_membership_design(&network, as_of).expect("active design"),
        MembershipDesign::HeterogeneousClassificationMultipleMembership,
        "population-level role heterogeneity must not erase a partial/multiple-membership signal",
    );
}
