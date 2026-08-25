//! Customer and competitor cannot occupy the same group at once.

use role_contradiction::{
    identity_recovery_rate, refuse_contradictory_roles, refuse_role_as_entity_class,
    roles_contradict, ContextualRole, RoleContradictionError,
};

#[test]
fn customer_and_competitor_cannot_share_a_group() {
    assert!(roles_contradict(
        ContextualRole::Customer,
        ContextualRole::Competitor
    ));
    assert!(roles_contradict(
        ContextualRole::Competitor,
        ContextualRole::Customer
    ));
    assert!(!roles_contradict(
        ContextualRole::Customer,
        ContextualRole::Partner
    ));
    assert!(!roles_contradict(
        ContextualRole::Partner,
        ContextualRole::Competitor
    ));
    assert!(!roles_contradict(
        ContextualRole::Customer,
        ContextualRole::Customer
    ));
    assert_eq!(
        refuse_contradictory_roles(ContextualRole::Customer, ContextualRole::Competitor),
        Err(RoleContradictionError::CustomerCompetitorOverlap)
    );
    assert_eq!(
        refuse_contradictory_roles(ContextualRole::Competitor, ContextualRole::Customer),
        Err(RoleContradictionError::CustomerCompetitorOverlap)
    );
    refuse_contradictory_roles(ContextualRole::Customer, ContextualRole::Partner)
        .expect("customer-partner is not a contradiction");
    refuse_contradictory_roles(ContextualRole::Partner, ContextualRole::Competitor)
        .expect("partner-competitor is coopetition, not a contradiction");
}

#[test]
fn commercial_roles_are_not_entity_classes() {
    for role in [
        ContextualRole::Customer,
        ContextualRole::Partner,
        ContextualRole::Competitor,
    ] {
        assert_eq!(
            refuse_role_as_entity_class(role),
            Err(RoleContradictionError::RoleIsNotEntityClass)
        );
    }
}

#[test]
fn recovered_roles_match_known_truth_better_than_a_commercial_collapse() {
    let truth = [
        ContextualRole::Customer,
        ContextualRole::Partner,
        ContextualRole::Competitor,
    ];
    let recovered = truth;
    let collapsed = [
        ContextualRole::Customer,
        ContextualRole::Customer,
        ContextualRole::Customer,
    ];
    let recovered_rate = identity_recovery_rate(&truth, &recovered).expect("recovered");
    let collapsed_rate = identity_recovery_rate(&truth, &collapsed).expect("collapsed");
    let expected = {
        let mut matches = 0_u32;
        for (truth_role, decided_role) in truth.iter().zip(recovered.iter()) {
            if truth_role == decided_role {
                matches += 1;
            }
        }
        f64::from(matches) / f64::from(u32::try_from(truth.len()).expect("len"))
    };
    assert!((recovered_rate - expected).abs() < f64::EPSILON);
    assert!(recovered_rate > collapsed_rate);
}

#[test]
fn empty_or_mismatched_role_payloads_fail_closed() {
    assert_eq!(
        identity_recovery_rate(&[], &[]),
        Err(RoleContradictionError::InvalidRolePayload)
    );
    assert_eq!(
        identity_recovery_rate(&[ContextualRole::Customer], &[]),
        Err(RoleContradictionError::InvalidRolePayload)
    );
    assert_eq!(
        identity_recovery_rate(
            &[ContextualRole::Customer, ContextualRole::Competitor],
            &[ContextualRole::Customer]
        ),
        Err(RoleContradictionError::InvalidRolePayload)
    );
}
