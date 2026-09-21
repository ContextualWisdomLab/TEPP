use membership_core::{
    GroupId, MemberId, MembershipAssignment, MembershipDesign, MembershipNetwork,
    MembershipObservation, MembershipObservationSupportProjection, MembershipObservationSupportWire,
    MembershipRole, MembershipWeight, MEMBERSHIP_OBSERVATION_SUPPORT_WIRE_VERSION,
    project_membership_observations_wire,
};
use temporal_core::EventTime;
use uuid::Uuid;

fn event_time(value: &str) -> EventTime {
    EventTime::parse_rfc3339(value).expect("event time must parse")
}

fn member(value: u128) -> MemberId {
    MemberId::from_uuid(Uuid::from_u128(value))
}

fn group(value: u128) -> GroupId {
    GroupId::from_uuid(Uuid::from_u128(value))
}

fn assignment(
    member_id: MemberId,
    group_id: GroupId,
    role: MembershipRole,
    weight: f64,
) -> MembershipAssignment {
    MembershipAssignment::new(
        member_id,
        group_id,
        role,
        MembershipWeight::new(weight).expect("valid weight"),
        event_time("2026-01-01T00:00:00Z"),
        event_time("2026-12-31T23:59:59Z"),
    )
    .expect("valid assignment")
}

fn require_owner_projection(_: &MembershipObservationSupportProjection) {}

#[test]
fn owner_projection_is_canonical_reconstructable_and_redacts_raw_opaque_ids() {
    let first_member = member(0x100);
    let second_member = member(0x200);
    let first_group = group(0x300);
    let second_group = group(0x400);
    let project_group = group(0x500);
    let early = event_time("2026-03-01T00:00:00Z");
    let late = event_time("2026-09-01T00:00:00Z");

    let memberships = [
        assignment(
            first_member,
            first_group,
            MembershipRole::Department,
            0.5,
        ),
        assignment(
            first_member,
            second_group,
            MembershipRole::Department,
            0.5,
        ),
        assignment(
            first_member,
            project_group,
            MembershipRole::Project,
            1.0,
        ),
        assignment(
            second_member,
            second_group,
            MembershipRole::Department,
            1.0,
        ),
    ];

    let mut forward = MembershipNetwork::new();
    for item in memberships {
        forward.insert(item).expect("forward membership");
    }
    let mut reverse = MembershipNetwork::new();
    for item in memberships.into_iter().rev() {
        reverse.insert(item).expect("reverse membership");
    }

    let observations = [
        MembershipObservation::new(second_member, late),
        MembershipObservation::new(first_member, early),
        MembershipObservation::new(first_member, late),
        MembershipObservation::new(first_member, late),
    ];
    let reordered = [
        MembershipObservation::new(first_member, late),
        MembershipObservation::new(first_member, late),
        MembershipObservation::new(second_member, late),
        MembershipObservation::new(first_member, early),
    ];

    let projection = project_membership_observations_wire(&forward, &observations)
        .expect("owner-issued projection");
    require_owner_projection(&projection);
    assert_eq!(
        projection.classification().design(),
        MembershipDesign::CrossClassifiedMultipleMembership
    );
    assert_eq!(
        projection.wire().schema_version(),
        MEMBERSHIP_OBSERVATION_SUPPORT_WIRE_VERSION
    );
    assert_eq!(
        projection.wire().design_version(),
        projection.classification().version()
    );
    assert_eq!(
        projection.wire().design_name(),
        projection.classification().name()
    );
    assert_eq!(
        projection.wire().support_digest_version(),
        projection.classification().support_digest_version()
    );
    assert_eq!(projection.wire().observations().len(), observations.len());

    let expected_support_digest = projection
        .classification()
        .support_sha256()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join("");
    assert_eq!(projection.wire().support_sha256(), expected_support_digest);

    let canonical = projection.wire().to_json().expect("canonical JSON");
    let same = project_membership_observations_wire(&reverse, &reordered)
        .expect("equivalent owner-issued projection")
        .wire()
        .to_json()
        .expect("canonical JSON");
    assert_eq!(canonical, same);

    for raw in [
        first_member.as_uuid().to_string(),
        second_member.as_uuid().to_string(),
        first_group.as_uuid().to_string(),
        second_group.as_uuid().to_string(),
        project_group.as_uuid().to_string(),
    ] {
        assert!(
            !canonical.contains(&raw),
            "projection leaked opaque source UUID {raw}"
        );
    }

    let parsed = MembershipObservationSupportWire::from_json(&canonical)
        .expect("canonical released projection must parse");
    assert_eq!(&parsed, projection.wire());
    assert_eq!(parsed.member_count(), 2);
    assert_eq!(parsed.group_count(), 3);
    assert_eq!(parsed.observations()[1], parsed.observations()[2]);

    let first_observation = &parsed.observations()[0];
    assert_eq!(first_observation.member_ordinal(), 0);
    assert_eq!(first_observation.event_time(), "2026-03-01T00:00:00Z");
    let first_assignment = &first_observation.assignments()[0];
    assert_eq!(first_assignment.role(), "department");
    assert!(first_assignment.group_ordinal() < parsed.group_count());
    assert_eq!(first_assignment.weight_f64_bits(), "3fe0000000000000");
}

#[test]
fn parsed_wire_refuses_noncanonical_or_malformed_support_payloads() {
    let member_id = member(0x111);
    let group_id = group(0x222);
    let as_of = event_time("2026-06-01T00:00:00Z");
    let mut network = MembershipNetwork::new();
    network
        .insert(assignment(
            member_id,
            group_id,
            MembershipRole::Department,
            1.0,
        ))
        .expect("membership");
    let projection = project_membership_observations_wire(
        &network,
        &[MembershipObservation::new(member_id, as_of)],
    )
    .expect("owner projection");
    let canonical = projection.wire().to_json().expect("canonical JSON");

    assert!(MembershipObservationSupportWire::from_json(&(canonical.clone() + "\n")).is_err());
    assert!(
        MembershipObservationSupportWire::from_json(&canonical.replacen(
            MEMBERSHIP_OBSERVATION_SUPPORT_WIRE_VERSION,
            "tepp.membership_observation_support_projection.v2",
            1,
        ))
        .is_err()
    );
    assert!(
        MembershipObservationSupportWire::from_json(&canonical.replacen(
            "\"design_name\":\"nested\"",
            "\"design_name\":\"multiple_membership\"",
            1,
        ))
        .is_err()
    );
    assert!(
        MembershipObservationSupportWire::from_json(&canonical.replacen(
            "2026-06-01T00:00:00Z",
            "2026-06-01T09:00:00+09:00",
            1,
        ))
        .is_err()
    );
    assert!(
        MembershipObservationSupportWire::from_json(&canonical.replacen(
            "\"member_ordinal\":0",
            "\"member_ordinal\":2",
            1,
        ))
        .is_err()
    );
    assert!(
        MembershipObservationSupportWire::from_json(&canonical.replacen(
            "\"weight_f64_bits\":\"3ff0000000000000\"",
            "\"weight_f64_bits\":\"not-binary64\"",
            1,
        ))
        .is_err()
    );
    assert!(
        MembershipObservationSupportWire::from_json(&canonical.replacen(
            "\"schema_version\"",
            "\"unexpected\":true,\"schema_version\"",
            1,
        ))
        .is_err()
    );
}
