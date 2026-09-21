use membership_core::{
    GroupId, MemberId, MembershipAssignment, MembershipDesign, MembershipDesignWire,
    MembershipError, MembershipNetwork, MembershipObservation, MembershipRole, MembershipWeight,
    MEMBERSHIP_DESIGN_WIRE_VERSION, classify_membership_observations_wire,
};
use temporal_core::EventTime;

fn event_time(value: &str) -> EventTime {
    EventTime::parse_rfc3339(value).expect("event time must parse")
}

#[test]
fn wire_coordinate_parses_supported_version_and_design_names() {
    for (name, design) in [
        ("nested", MembershipDesign::Nested),
        ("cross_classified", MembershipDesign::CrossClassified),
        ("multiple_membership", MembershipDesign::MultipleMembership),
        (
            "cross_classified_multiple_membership",
            MembershipDesign::CrossClassifiedMultipleMembership,
        ),
        (
            "heterogeneous_classification",
            MembershipDesign::HeterogeneousClassification,
        ),
        (
            "heterogeneous_classification_multiple_membership",
            MembershipDesign::HeterogeneousClassificationMultipleMembership,
        ),
    ] {
        let coordinate = MembershipDesignWire::parse(MEMBERSHIP_DESIGN_WIRE_VERSION, name)
            .expect("supported owner coordinate must parse");
        assert_eq!(coordinate.version(), MEMBERSHIP_DESIGN_WIRE_VERSION);
        assert_eq!(coordinate.name(), name);
        assert_eq!(coordinate.design(), design);
    }
}

#[test]
fn owner_wire_coordinate_is_derived_from_longitudinal_membership_state() {
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

    let coordinate = classify_membership_observations_wire(
        &network,
        &[MembershipObservation::new(member, as_of)],
    )
    .expect("owner-derived coordinate");
    assert_eq!(coordinate.version(), MEMBERSHIP_DESIGN_WIRE_VERSION);
    assert_eq!(coordinate.name(), "nested");
    assert_eq!(coordinate.design(), MembershipDesign::Nested);
}

#[test]
fn owner_wire_coordinate_refuses_missing_observation_membership() {
    let member = MemberId::new();
    let as_of = event_time("2026-06-01T00:00:00Z");
    assert_eq!(
        classify_membership_observations_wire(
            &MembershipNetwork::new(),
            &[MembershipObservation::new(member, as_of)],
        ),
        Err(MembershipError::MissingObservationMembership),
    );
}

#[test]
fn owner_wire_coordinate_refuses_mismatched_version_even_for_valid_name() {
    assert_eq!(
        MembershipDesignWire::parse("tepp.membership_design.v2", "nested"),
        Err(MembershipError::UnsupportedWireVersion),
    );
}

#[test]
fn owner_wire_coordinate_refuses_unknown_name_under_supported_version() {
    assert_eq!(
        MembershipDesignWire::parse(MEMBERSHIP_DESIGN_WIRE_VERSION, "nested_probably"),
        Err(MembershipError::UnknownMembershipDesign),
    );
}
