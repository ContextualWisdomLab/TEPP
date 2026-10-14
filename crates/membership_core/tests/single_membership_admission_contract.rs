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
    role: MembershipRole,
    weight: f64,
) -> MembershipAssignment {
    MembershipAssignment::new(
        member,
        group,
        role,
        MembershipWeight::new(weight).expect("finite membership weight"),
        event_time("2026-09-01T00:00:00Z"),
        event_time("2026-09-30T23:59:59Z"),
    )
    .expect("bounded assignment")
}

#[test]
fn full_single_membership_yields_owner_admission() {
    let member = MemberId::new();
    let group = GroupId::new();
    let instant = event_time("2026-09-19T00:00:00Z");
    let mut network = MembershipNetwork::new();
    network
        .insert(assignment(
            member,
            group,
            MembershipRole::Department,
            1.0,
        ))
        .expect("insert nested membership");

    let admitted = admit_single_membership(&network, member, instant).expect("single admission");
    assert_eq!(admitted.member_id(), member);
    assert_eq!(admitted.group_id(), group);
    assert_eq!(admitted.role(), MembershipRole::Department);
    assert_eq!(admitted.weight().value().to_bits(), 1.0_f64.to_bits());
    assert_eq!(admitted.event_time(), instant);
}

#[test]
fn simultaneous_group_or_role_membership_fails_closed() {
    let instant = event_time("2026-09-19T00:00:00Z");

    let member = MemberId::new();
    let mut multiple = MembershipNetwork::new();
    multiple
        .insert(assignment(
            member,
            GroupId::new(),
            MembershipRole::Department,
            0.5,
        ))
        .expect("insert first weighted membership");
    multiple
        .insert(assignment(
            member,
            GroupId::new(),
            MembershipRole::Department,
            0.5,
        ))
        .expect("insert second weighted membership");
    assert_eq!(
        admit_single_membership(&multiple, member, instant),
        Err(MembershipError::SingleMembershipProfileInapplicable)
    );

    let member = MemberId::new();
    let shared_group = GroupId::new();
    let mut cross_classified = MembershipNetwork::new();
    cross_classified
        .insert(assignment(
            member,
            shared_group,
            MembershipRole::Department,
            1.0,
        ))
        .expect("insert department membership");
    cross_classified
        .insert(assignment(
            member,
            shared_group,
            MembershipRole::Project,
            1.0,
        ))
        .expect("insert project membership");
    assert_eq!(
        admit_single_membership(&cross_classified, member, instant),
        Err(MembershipError::SingleMembershipProfileInapplicable)
    );
}

#[test]
fn partial_or_inactive_membership_fails_closed() {
    let member = MemberId::new();
    let mut network = MembershipNetwork::new();
    network
        .insert(assignment(
            member,
            GroupId::new(),
            MembershipRole::Department,
            0.75,
        ))
        .expect("insert partial membership");

    assert_eq!(
        admit_single_membership(&network, member, event_time("2026-09-19T00:00:00Z")),
        Err(MembershipError::SingleMembershipProfileInapplicable)
    );
    assert_eq!(
        admit_single_membership(&network, member, event_time("2026-10-01T00:00:00Z")),
        Err(MembershipError::SingleMembershipProfileInapplicable)
    );
}
