//! Reachable-state contract for duplicate longitudinal support coordinates.

use membership_core::MembershipObservationSupportWire;

const CONTRADICTORY_DUPLICATE_SUPPORT: &str = r#"{"schema_version":"tepp.membership_observation_support_projection.v1","design_version":"tepp.membership_design.v1","design_name":"cross_classified","support_digest_version":"tepp.membership_observation_support.v1","support_sha256":"0000000000000000000000000000000000000000000000000000000000000000","wire_digest_version":"tepp.membership_observation_support_projection_digest.v1","wire_sha256":"1dce5cf6f4eb87625bf2684aa4bf4eb840675ab7439379fb34253946fc75d4bc","member_count":1,"group_count":2,"observations":[{"member_ordinal":0,"event_time":"2026-06-01T00:00:00Z","assignments":[{"role":"department","group_ordinal":0,"weight_f64_bits":"3ff0000000000000"}]},{"member_ordinal":0,"event_time":"2026-06-01T00:00:00Z","assignments":[{"role":"department","group_ordinal":0,"weight_f64_bits":"3ff0000000000000"},{"role":"project","group_ordinal":1,"weight_f64_bits":"3ff0000000000000"}]}]}"#;

#[test]
fn parsed_wire_refuses_contradictory_topology_for_one_duplicate_observation_coordinate() {
    assert!(
        MembershipObservationSupportWire::from_json(CONTRADICTORY_DUPLICATE_SUPPORT).is_err(),
        "one canonical member/event-time coordinate cannot resolve to two different active topologies"
    );
}
