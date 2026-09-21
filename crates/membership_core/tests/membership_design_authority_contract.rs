use membership_core::{
    GroupId, MemberId, MembershipAssignment, MembershipDesign, MembershipDesignClassification,
    MembershipDesignWire, MembershipNetwork, MembershipObservation, MembershipRole,
    MembershipWeight, MEMBERSHIP_DESIGN_WIRE_VERSION, classify_membership_observations_wire,
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
    let as_of = event_time("2026-06-01T00:00:00Z");
    let member = MemberId::new();
    let mut network = MembershipNetwork::new();
    network
        .insert(
            MembershipAssignment::new(
                member,
                GroupId::new(),
                MembershipRole::Department,
                MembershipWeight::full().expect("full weight"),
                start,
                end,
            )
            .expect("valid assignment"),
        )
        .expect("insert assignment");

    let classification = classify_membership_observations_wire(
        &network,
        &[MembershipObservation::new(member, as_of)],
    )
    .expect("canonical owner classification");
    require_owner_classification(classification);

    let wire = classification.wire();
    assert_eq!(wire.version(), MEMBERSHIP_DESIGN_WIRE_VERSION);
    assert_eq!(wire.name(), "nested");
    assert_eq!(wire.design(), MembershipDesign::Nested);

    let parsed = MembershipDesignWire::parse(MEMBERSHIP_DESIGN_WIRE_VERSION, "nested")
        .expect("released coordinate must deserialize");
    assert_eq!(parsed, wire);
}
