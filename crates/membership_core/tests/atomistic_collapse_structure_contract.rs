//! Atomistic-collapse guards must preserve distinct groups for one member.

use membership_core::{
    GroupId, MemberId, MembershipAssignment, MembershipError, MembershipNetwork, MembershipRole,
    MembershipWeight, refuse_atomistic_collapse,
};
use temporal_core::EventTime;

fn event_time(value: &str) -> EventTime {
    EventTime::parse_rfc3339(value).expect("event time")
}

fn assignment(member: MemberId, group: GroupId, role: MembershipRole) -> MembershipAssignment {
    MembershipAssignment::new(
        member,
        group,
        role,
        MembershipWeight::full().expect("full weight"),
        event_time("2026-01-01T00:00:00Z"),
        event_time("2026-12-31T23:59:59Z"),
    )
    .expect("assignment")
}

#[test]
fn duplicate_group_rows_cannot_stand_in_for_a_missing_group() {
    let member = MemberId::new();
    let duplicated_group = GroupId::new();
    let omitted_group = GroupId::new();
    let mut network = MembershipNetwork::new();
    for edge in [
        assignment(member, duplicated_group, MembershipRole::Author),
        assignment(member, duplicated_group, MembershipRole::Department),
        assignment(member, omitted_group, MembershipRole::Project),
    ] {
        network.insert(edge).expect("insert");
    }

    let rows = network
        .estimation_rows_at(member, event_time("2026-06-01T00:00:00Z"))
        .expect("rows");
    assert_eq!(
        refuse_atomistic_collapse(&rows[..2], 2),
        Err(MembershipError::AtomisticCollapseRefused)
    );
    refuse_atomistic_collapse(&rows, 2).expect("two distinct groups are preserved");
}

#[test]
fn rows_from_different_members_fail_closed() {
    let first_member = MemberId::new();
    let second_member = MemberId::new();
    let instant = event_time("2026-06-01T00:00:00Z");
    let mut network = MembershipNetwork::new();
    network
        .insert(assignment(
            first_member,
            GroupId::new(),
            MembershipRole::Author,
        ))
        .expect("first insert");
    network
        .insert(assignment(
            second_member,
            GroupId::new(),
            MembershipRole::Project,
        ))
        .expect("second insert");

    let mut mixed = network
        .estimation_rows_at(first_member, instant)
        .expect("first rows");
    mixed.extend(
        network
            .estimation_rows_at(second_member, instant)
            .expect("second rows"),
    );
    assert_eq!(
        refuse_atomistic_collapse(&mixed, 2),
        Err(MembershipError::InvalidWirePayload)
    );
}
