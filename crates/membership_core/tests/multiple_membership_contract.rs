//! Realistic multiple-membership contracts that prevent atomistic fallacy.

use membership_core::{
    GroupId, MemberId, MembershipAssignment, MembershipError, MembershipNetwork, MembershipRole,
    MembershipWeight,
};
use temporal_core::EventTime;

fn event_time(value: &str) -> EventTime {
    EventTime::parse_rfc3339(value).expect("event time must parse")
}

#[test]
fn one_document_can_belong_to_author_project_and_customer_simultaneously() {
    let document = MemberId::new();
    let author = GroupId::new();
    let project = GroupId::new();
    let customer = GroupId::new();
    let start = event_time("2026-01-01T00:00:00Z");
    let end = event_time("2026-12-31T23:59:59Z");
    let as_of = event_time("2026-06-15T12:00:00Z");

    let mut network = MembershipNetwork::new();
    network
        .insert(
            MembershipAssignment::new(
                document,
                author,
                MembershipRole::Author,
                MembershipWeight::full().expect("full weight"),
                start,
                end,
            )
            .expect("author membership"),
        )
        .expect("insert author");
    network
        .insert(
            MembershipAssignment::new(
                document,
                project,
                MembershipRole::Project,
                MembershipWeight::new(0.6).expect("partial project weight"),
                start,
                end,
            )
            .expect("project membership"),
        )
        .expect("insert project");
    network
        .insert(
            MembershipAssignment::new(
                document,
                customer,
                MembershipRole::Customer,
                MembershipWeight::new(0.4).expect("partial customer weight"),
                start,
                end,
            )
            .expect("customer membership"),
        )
        .expect("insert customer");

    assert_eq!(network.assignment_count(), 3);
    assert_eq!(network.active_group_multiplicity(document, as_of), 3);
    let weights = network.active_weight_by_role(document, as_of);
    assert_eq!(weights.get(&MembershipRole::Author).copied(), Some(1.0));
    assert_eq!(weights.get(&MembershipRole::Project).copied(), Some(0.6));
    assert_eq!(weights.get(&MembershipRole::Customer).copied(), Some(0.4));
}

#[test]
fn organization_can_change_from_partner_to_competitor_without_identity_rewrite() {
    let organization_as_member = MemberId::new();
    let market = GroupId::new();
    let partner_start = event_time("2024-01-01T00:00:00Z");
    let partner_end = event_time("2025-06-30T23:59:59Z");
    let competitor_start = event_time("2025-07-01T00:00:00Z");
    let competitor_end = event_time("2026-12-31T23:59:59Z");

    let mut network = MembershipNetwork::new();
    network
        .insert(
            MembershipAssignment::new(
                organization_as_member,
                market,
                MembershipRole::Partner,
                MembershipWeight::full().expect("full"),
                partner_start,
                partner_end,
            )
            .expect("partner window"),
        )
        .expect("insert partner");
    // Distinct role keys allow successive commercial postures for the same entity.
    network
        .insert(
            MembershipAssignment::new(
                organization_as_member,
                market,
                MembershipRole::Competitor,
                MembershipWeight::full().expect("full"),
                competitor_start,
                competitor_end,
            )
            .expect("competitor window"),
        )
        .expect("insert competitor");

    let during_partner = event_time("2025-01-15T00:00:00Z");
    let during_competitor = event_time("2026-01-15T00:00:00Z");
    assert_eq!(
        network
            .active_memberships_for(organization_as_member, during_partner)
            .iter()
            .map(|assignment| assignment.role())
            .collect::<Vec<_>>(),
        vec![MembershipRole::Partner]
    );
    assert_eq!(
        network
            .active_memberships_for(organization_as_member, during_competitor)
            .iter()
            .map(|assignment| assignment.role())
            .collect::<Vec<_>>(),
        vec![MembershipRole::Competitor]
    );
}

#[test]
fn duplicate_member_group_role_keys_are_rejected() {
    let member = MemberId::new();
    let group = GroupId::new();
    let start = event_time("2026-01-01T00:00:00Z");
    let end = event_time("2026-02-01T00:00:00Z");
    let assignment = MembershipAssignment::new(
        member,
        group,
        MembershipRole::Language,
        MembershipWeight::full().expect("full"),
        start,
        end,
    )
    .expect("assignment");
    let mut network = MembershipNetwork::new();
    network.insert(assignment).expect("first insert");
    assert_eq!(
        network.insert(assignment),
        Err(MembershipError::DuplicateMembershipAssignment)
    );
}

#[test]
fn weights_and_intervals_fail_closed() {
    assert_eq!(
        MembershipWeight::new(-0.1),
        Err(MembershipError::InvalidMembershipWeight)
    );
    assert_eq!(
        MembershipWeight::new(f64::NAN),
        Err(MembershipError::InvalidMembershipWeight)
    );
    let member = MemberId::new();
    let group = GroupId::new();
    let later = event_time("2026-02-01T00:00:00Z");
    let earlier = event_time("2026-01-01T00:00:00Z");
    assert_eq!(
        MembershipAssignment::new(
            member,
            group,
            MembershipRole::Episode,
            MembershipWeight::full().expect("full"),
            later,
            earlier,
        ),
        Err(MembershipError::InvalidValidityInterval)
    );
}
