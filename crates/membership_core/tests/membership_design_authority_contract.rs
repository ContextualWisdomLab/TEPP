use membership_core::{
    GroupId, MemberId, MembershipAssignment, MembershipDesign, MembershipDesignClassification,
    MembershipDesignWire, MembershipNetwork, MembershipObservation, MembershipRole,
    MembershipWeight, MEMBERSHIP_DESIGN_WIRE_VERSION,
    MEMBERSHIP_OBSERVATION_SUPPORT_DIGEST_VERSION, classify_membership_observations_wire,
};
use temporal_core::EventTime;

fn event_time(value: &str) -> EventTime {
    EventTime::parse_rfc3339(value).expect("event time must parse")
}

fn require_owner_classification(_: MembershipDesignClassification) {}

#[test]
fn canonical_classification_is_distinct_from_a_parsed_wire_coordinate() {
    let start = event_time("2026-01-01T00:00:00Z");
    let end = event_time("2026-12-31T23:59:59Z");
    let early = event_time("2026-03-01T00:00:00Z");
    let as_of = event_time("2026-06-01T00:00:00Z");
    let late = event_time("2026-09-01T00:00:00Z");
    let member = MemberId::new();
    let other_member = MemberId::new();
    let group = GroupId::new();
    let mut network = MembershipNetwork::new();
    for member_id in [member, other_member] {
        network
            .insert(
                MembershipAssignment::new(
                    member_id,
                    group,
                    MembershipRole::Department,
                    MembershipWeight::full().expect("full weight"),
                    start,
                    end,
                )
                .expect("valid assignment"),
            )
            .expect("insert assignment");
    }

    let observations = [
        MembershipObservation::new(member, late),
        MembershipObservation::new(member, early),
        MembershipObservation::new(member, as_of),
    ];
    let classification = classify_membership_observations_wire(&network, &observations)
        .expect("canonical owner classification");
    require_owner_classification(classification);

    assert_eq!(classification.observation_count(), 3);
    assert_eq!(classification.earliest_event_time(), early);
    assert_eq!(classification.latest_event_time(), late);
    assert_eq!(
        classification.support_digest_version(),
        MEMBERSHIP_OBSERVATION_SUPPORT_DIGEST_VERSION
    );

    let reordered = classify_membership_observations_wire(
        &network,
        &[
            MembershipObservation::new(member, as_of),
            MembershipObservation::new(member, late),
            MembershipObservation::new(member, early),
        ],
    )
    .expect("reordered support");
    assert_eq!(classification.support_sha256(), reordered.support_sha256());

    let same_window_different_support = classify_membership_observations_wire(
        &network,
        &[
            MembershipObservation::new(member, late),
            MembershipObservation::new(member, early),
            MembershipObservation::new(other_member, as_of),
        ],
    )
    .expect("different support with same window");
    assert_eq!(same_window_different_support.observation_count(), 3);
    assert_eq!(same_window_different_support.earliest_event_time(), early);
    assert_eq!(same_window_different_support.latest_event_time(), late);
    assert_eq!(same_window_different_support.design(), classification.design());
    assert_ne!(
        same_window_different_support.support_sha256(),
        classification.support_sha256()
    );

    let duplicated = classify_membership_observations_wire(
        &network,
        &[
            MembershipObservation::new(member, late),
            MembershipObservation::new(member, early),
            MembershipObservation::new(member, as_of),
            MembershipObservation::new(member, as_of),
        ],
    )
    .expect("duplicate coordinates remain part of declared support");
    assert_ne!(duplicated.support_sha256(), classification.support_sha256());

    let wire = classification.wire();
    assert_eq!(wire.version(), MEMBERSHIP_DESIGN_WIRE_VERSION);
    assert_eq!(wire.name(), "nested");
    assert_eq!(wire.design(), MembershipDesign::Nested);

    let parsed = MembershipDesignWire::parse(MEMBERSHIP_DESIGN_WIRE_VERSION, "nested")
        .expect("released coordinate must deserialize");
    assert_eq!(parsed, wire);
}
