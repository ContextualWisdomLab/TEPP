//! Membership role wire-name contracts.

use membership_core::{MembershipError, MembershipRole};

#[test]
fn every_role_round_trips_through_its_wire_name() {
    for role in [
        MembershipRole::Author,
        MembershipRole::Department,
        MembershipRole::Organization,
        MembershipRole::Customer,
        MembershipRole::Partner,
        MembershipRole::Competitor,
        MembershipRole::Project,
        MembershipRole::OpportunityPool,
        MembershipRole::Template,
        MembershipRole::Language,
        MembershipRole::Location,
        MembershipRole::Episode,
    ] {
        assert_eq!(
            MembershipRole::from_wire_name(role.wire_name()).expect("wire name must parse"),
            role
        );
    }
}

#[test]
fn unknown_roles_are_rejected() {
    assert_eq!(
        MembershipRole::from_wire_name("friend"),
        Err(MembershipError::UnknownMembershipRole)
    );
}
