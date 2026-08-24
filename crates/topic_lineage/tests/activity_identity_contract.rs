//! Topic identity survives dormancy and reactivation.

use topic_lineage::{
    TopicActivity, TopicIdentity, TopicLineageError, TopicLineageRecord, identity_recovery_rate,
    refuse_new_identity_on_reactivation,
};
use uuid::Uuid;

#[test]
fn reactivation_cannot_mint_a_new_topic_identity() {
    let identity = TopicIdentity::from_uuid(Uuid::from_u128(11));
    let active = TopicLineageRecord::active(identity);
    let dormant = active.make_dormant().expect("dormant");
    assert_eq!(dormant.activity(), TopicActivity::Dormant);
    assert_eq!(dormant.identity(), identity);

    let reactivated = dormant.reactivate().expect("reactivated");
    assert_eq!(reactivated.activity(), TopicActivity::Reactivated);
    assert_eq!(reactivated.identity(), identity);

    let other = TopicIdentity::from_uuid(Uuid::from_u128(99));
    assert_eq!(
        refuse_new_identity_on_reactivation(identity, other),
        Err(TopicLineageError::ReactivationIsNotNewTopic)
    );
}

#[test]
fn recovered_identities_match_known_truth_better_than_minted_replacements() {
    let stable = TopicIdentity::from_uuid(Uuid::from_u128(3));
    let truth = [stable, stable, stable];
    let recovered = [stable, stable, stable];
    let minted = [stable, stable, TopicIdentity::from_uuid(Uuid::from_u128(4))];

    let recovered_rate = identity_recovery_rate(&truth, &recovered).expect("recovered");
    let minted_rate = identity_recovery_rate(&truth, &minted).expect("minted");
    assert!((recovered_rate - 1.0).abs() < f64::EPSILON);
    assert!((minted_rate - (2.0 / 3.0)).abs() < 1e-12);
    assert!(recovered_rate > minted_rate);
}

#[test]
fn empty_identity_payloads_fail_closed() {
    assert_eq!(
        identity_recovery_rate(&[], &[]),
        Err(TopicLineageError::InvalidIdentityPayload)
    );
}
