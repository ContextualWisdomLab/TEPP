//! Language, episode, template, and opportunity-pool targets are not entities.

use membership_target::{
    identity_recovery_rate, refuse_collapsed_target, MembershipTargetError, MembershipTargetKind,
};

#[test]
fn non_entity_targets_cannot_collapse_into_entity_or_project() {
    assert_eq!(
        refuse_collapsed_target(MembershipTargetKind::Language, MembershipTargetKind::Entity),
        Err(MembershipTargetError::TargetKindCollapsed)
    );
    assert_eq!(
        refuse_collapsed_target(MembershipTargetKind::Episode, MembershipTargetKind::Entity),
        Err(MembershipTargetError::TargetKindCollapsed)
    );
    assert_eq!(
        refuse_collapsed_target(
            MembershipTargetKind::Department,
            MembershipTargetKind::Entity
        ),
        Err(MembershipTargetError::TargetKindCollapsed)
    );
    assert_eq!(
        refuse_collapsed_target(
            MembershipTargetKind::Template,
            MembershipTargetKind::Project
        ),
        Err(MembershipTargetError::TargetKindCollapsed)
    );
    assert_eq!(
        refuse_collapsed_target(
            MembershipTargetKind::OpportunityPool,
            MembershipTargetKind::Project
        ),
        Err(MembershipTargetError::TargetKindCollapsed)
    );
    refuse_collapsed_target(MembershipTargetKind::Entity, MembershipTargetKind::Entity)
        .expect("entity stays entity");
    refuse_collapsed_target(MembershipTargetKind::Project, MembershipTargetKind::Project)
        .expect("project stays project");
}

#[test]
fn recovered_targets_match_known_truth_better_than_an_entity_collapse() {
    let truth = [
        MembershipTargetKind::Language,
        MembershipTargetKind::Episode,
        MembershipTargetKind::Template,
    ];
    let recovered = truth;
    let collapsed = [
        MembershipTargetKind::Entity,
        MembershipTargetKind::Entity,
        MembershipTargetKind::Entity,
    ];
    let recovered_rate = identity_recovery_rate(&truth, &recovered).expect("recovered");
    let collapsed_rate = identity_recovery_rate(&truth, &collapsed).expect("collapsed");
    let expected = {
        let mut matches = 0_u32;
        for (truth_kind, decided_kind) in truth.iter().zip(recovered.iter()) {
            if truth_kind == decided_kind {
                matches += 1;
            }
        }
        f64::from(matches) / f64::from(u32::try_from(truth.len()).expect("len"))
    };
    assert!((recovered_rate - expected).abs() < f64::EPSILON);
    assert!(recovered_rate > collapsed_rate);
}

#[test]
fn empty_or_mismatched_target_payloads_fail_closed() {
    assert_eq!(
        identity_recovery_rate(&[], &[]),
        Err(MembershipTargetError::InvalidTargetPayload)
    );
    assert_eq!(
        identity_recovery_rate(&[MembershipTargetKind::Language], &[]),
        Err(MembershipTargetError::InvalidTargetPayload)
    );
    assert_eq!(
        identity_recovery_rate(
            &[
                MembershipTargetKind::Language,
                MembershipTargetKind::Episode
            ],
            &[MembershipTargetKind::Language]
        ),
        Err(MembershipTargetError::InvalidTargetPayload)
    );
}
