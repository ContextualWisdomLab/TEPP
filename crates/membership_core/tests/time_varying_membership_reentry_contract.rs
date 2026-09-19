use membership_core::{
    GroupId, MemberId, MembershipAssignment, MembershipError, MembershipNetwork, MembershipRole,
    MembershipWeight, admit_single_membership,
};
use temporal_core::EventTime;

fn event_time(value: &str) -> EventTime {
    EventTime::parse_rfc3339(value).expect("event time must parse")
}

fn assignment(
    member: MemberId,
    group: GroupId,
    valid_from: &str,
    valid_to: &str,
) -> MembershipAssignment {
    MembershipAssignment::new(
        member,
        group,
        MembershipRole::Department,
        MembershipWeight::full().expect("full membership weight"),
        event_time(valid_from),
        event_time(valid_to),
    )
    .expect("bounded membership assignment")
}

#[test]
fn disjoint_same_identity_reentry_preserves_longitudinal_membership_spells() {
    let member = MemberId::new();
    let group = GroupId::new();
    let mut network = MembershipNetwork::new();

    network
        .insert(assignment(
            member,
            group,
            "2026-01-01T00:00:00Z",
            "2026-01-31T23:59:59Z",
        ))
        .expect("first membership spell");
    network
        .insert(assignment(
            member,
            group,
            "2026-03-01T00:00:00Z",
            "2026-03-31T23:59:59Z",
        ))
        .expect("strictly disjoint re-entry must remain representable");

    let first = admit_single_membership(
        &network,
        member,
        event_time("2026-01-15T00:00:00Z"),
    )
    .expect("first spell admission");
    let second = admit_single_membership(
        &network,
        member,
        event_time("2026-03-15T00:00:00Z"),
    )
    .expect("second spell admission");

    assert_eq!(first.group_id(), group);
    assert_eq!(second.group_id(), group);
    assert_eq!(
        admit_single_membership(
            &network,
            member,
            event_time("2026-02-15T00:00:00Z"),
        ),
        Err(MembershipError::SingleMembershipProfileInapplicable)
    );
}

#[test]
fn disjoint_same_identity_spells_are_order_independent_at_insertion() {
    let member = MemberId::new();
    let group = GroupId::new();
    let mut network = MembershipNetwork::new();

    network
        .insert(assignment(
            member,
            group,
            "2026-03-01T00:00:00Z",
            "2026-03-31T23:59:59Z",
        ))
        .expect("later spell inserted first");
    network
        .insert(assignment(
            member,
            group,
            "2026-01-01T00:00:00Z",
            "2026-01-31T23:59:59Z",
        ))
        .expect("earlier disjoint spell inserted second");

    assert_eq!(network.assignment_count(), 2);
    assert_eq!(
        admit_single_membership(
            &network,
            member,
            event_time("2026-01-15T00:00:00Z"),
        )
        .expect("earlier spell admission")
        .group_id(),
        group
    );
}

#[test]
fn same_identity_closed_intervals_cannot_overlap_or_share_an_endpoint() {
    let member = MemberId::new();
    let group = GroupId::new();
    let mut network = MembershipNetwork::new();

    network
        .insert(assignment(
            member,
            group,
            "2026-01-01T00:00:00Z",
            "2026-01-31T00:00:00Z",
        ))
        .expect("first membership spell");

    let touching = assignment(
        member,
        group,
        "2026-01-31T00:00:00Z",
        "2026-02-28T00:00:00Z",
    );
    assert_eq!(
        network.insert(touching),
        Err(MembershipError::DuplicateMembershipAssignment)
    );

    let overlapping = assignment(
        member,
        group,
        "2026-01-15T00:00:00Z",
        "2026-02-15T00:00:00Z",
    );
    assert_eq!(
        network.insert(overlapping),
        Err(MembershipError::DuplicateMembershipAssignment)
    );
}
