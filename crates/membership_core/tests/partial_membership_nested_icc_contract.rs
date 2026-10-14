use membership_core::{
    GroupId, MemberId, MembershipAssignment, MembershipDesign, MembershipError, MembershipNetwork,
    MembershipRole, MembershipWeight, NestedOutcome, classify_membership_design,
    nested_intraclass_correlation,
};
use temporal_core::EventTime;

fn event_time(value: &str) -> EventTime {
    EventTime::parse_rfc3339(value).expect("event time must parse")
}

fn two_cluster_population(partial_first_member: bool) -> (MembershipNetwork, Vec<NestedOutcome>) {
    let start = event_time("2026-01-01T00:00:00Z");
    let end = event_time("2026-12-31T00:00:00Z");
    let groups = [GroupId::new(), GroupId::new()];
    let rows = [(groups[0], [1.0, 3.0]), (groups[1], [2.0, 4.0])];
    let mut network = MembershipNetwork::new();
    let mut outcomes = Vec::new();
    let mut row_index = 0_usize;

    for (group, values) in rows {
        for value in values {
            let member = MemberId::new();
            let weight = if partial_first_member && row_index == 0 {
                0.5
            } else {
                1.0
            };
            network
                .insert(
                    MembershipAssignment::new(
                        member,
                        group,
                        MembershipRole::Department,
                        MembershipWeight::new(weight).expect("bounded membership weight"),
                        start,
                        end,
                    )
                    .expect("membership assignment"),
                )
                .expect("insert membership");
            outcomes.push(NestedOutcome::new(member, value).expect("finite outcome"));
            row_index += 1;
        }
    }

    (network, outcomes)
}

#[test]
fn lone_partial_weight_cannot_be_reinterpreted_as_full_nested_structure() {
    let as_of = event_time("2026-06-01T00:00:00Z");
    let (partial, partial_outcomes) = two_cluster_population(true);

    assert_eq!(
        classify_membership_design(&partial, as_of).expect("classify weighted design"),
        MembershipDesign::MultipleMembership
    );
    assert_eq!(
        nested_intraclass_correlation(&partial, as_of, &partial_outcomes),
        Err(MembershipError::NestedIccInapplicable)
    );

    let (full, full_outcomes) = two_cluster_population(false);
    assert_eq!(
        classify_membership_design(&full, as_of).expect("classify full nested design"),
        MembershipDesign::Nested
    );
    assert!(nested_intraclass_correlation(&full, as_of, &full_outcomes).is_ok());
}

#[test]
fn full_weight_cross_classification_remains_distinct_from_multiple_membership() {
    let start = event_time("2026-01-01T00:00:00Z");
    let end = event_time("2026-12-31T00:00:00Z");
    let as_of = event_time("2026-06-01T00:00:00Z");
    let member = MemberId::new();
    let mut network = MembershipNetwork::new();

    for (group, role) in [
        (GroupId::new(), MembershipRole::Author),
        (GroupId::new(), MembershipRole::Project),
    ] {
        network
            .insert(
                MembershipAssignment::new(
                    member,
                    group,
                    role,
                    MembershipWeight::full().expect("full membership weight"),
                    start,
                    end,
                )
                .expect("membership assignment"),
            )
            .expect("insert membership");
    }

    assert_eq!(
        classify_membership_design(&network, as_of).expect("classify cross-classified design"),
        MembershipDesign::CrossClassified
    );
    let outcome = NestedOutcome::new(member, 1.0).expect("finite outcome");
    assert_eq!(
        nested_intraclass_correlation(&network, as_of, &[outcome]),
        Err(MembershipError::NestedIccInapplicable)
    );
}
