//! Location is not entity identity and not a language channel.

use location_membership::{
    LocationKind, LocationMembershipError, identity_recovery_rate,
    refuse_location_as_entity_identity, refuse_location_as_language_channel,
};

#[test]
fn location_cannot_become_entity_identity_or_language() {
    assert_eq!(
        refuse_location_as_entity_identity(LocationKind::Location),
        Err(LocationMembershipError::LocationIsNotEntityIdentity)
    );
    assert_eq!(
        refuse_location_as_language_channel(LocationKind::Location),
        Err(LocationMembershipError::LocationIsNotLanguageChannel)
    );
    refuse_location_as_entity_identity(LocationKind::EntityIdentity).expect("entity");
    refuse_location_as_language_channel(LocationKind::LanguageChannel).expect("language");
}

#[test]
fn recovered_kinds_match_known_truth_better_than_an_entity_collapse() {
    let truth = [
        LocationKind::Location,
        LocationKind::EntityIdentity,
        LocationKind::LanguageChannel,
    ];
    let recovered = truth;
    let collapsed = [
        LocationKind::EntityIdentity,
        LocationKind::EntityIdentity,
        LocationKind::EntityIdentity,
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
fn empty_or_mismatched_kind_payloads_fail_closed() {
    assert_eq!(
        identity_recovery_rate(&[], &[]),
        Err(LocationMembershipError::InvalidLocationPayload)
    );
    assert_eq!(
        identity_recovery_rate(&[LocationKind::Location], &[]),
        Err(LocationMembershipError::InvalidLocationPayload)
    );
    assert_eq!(
        identity_recovery_rate(
            &[LocationKind::Location, LocationKind::EntityIdentity],
            &[LocationKind::Location]
        ),
        Err(LocationMembershipError::InvalidLocationPayload)
    );
}
