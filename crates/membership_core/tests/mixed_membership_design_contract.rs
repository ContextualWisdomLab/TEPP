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
fn mixed_cross_classified_and_multiple_membership_remains_lossless() {
    let start = event_time("2026-01-01T00:00:00Z");
    let end = event_time("2026-12-31T00:00:00Z");
    let as_of = event_time("2026-06-01T00:00:00Z");

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

    assert_eq!(
        classify_membership_design(&network, as_of).expect("mixed design"),
        MembershipDesign::CrossClassifiedMultipleMembership,
        "the owner must preserve simultaneous cross-classification and multiple membership",
    );

    let outcomes = [
        NestedOutcome::new(cross_member, 1.0).expect("cross outcome"),
        NestedOutcome::new(multiple_member, 2.0).expect("multiple outcome"),
    ];
    assert_eq!(
        nested_intraclass_correlation(&network, as_of, &outcomes),
        Err(MembershipError::NestedIccInapplicable),
    );
}
