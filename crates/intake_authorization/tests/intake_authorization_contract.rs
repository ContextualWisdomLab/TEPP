//! Untrusted intake fails closed without a grant; bounds are not authorization.

use intake_authorization::{
    identity_recovery_rate, refuse_bounds_as_authorization, refuse_intake_without_grant,
    GrantPresence, IntakeAuthorizationError, IntakeKind,
};

#[test]
fn untrusted_intake_fails_closed_without_a_grant() {
    for kind in [
        IntakeKind::Document,
        IntakeKind::SerializedRecord,
        IntakeKind::ModelCheckpoint,
        IntakeKind::LlmOutput,
    ] {
        assert_eq!(
            refuse_intake_without_grant(kind, GrantPresence::Absent),
            Err(IntakeAuthorizationError::MissingGrant)
        );
        refuse_intake_without_grant(kind, GrantPresence::Present).expect("grant present");
    }
    assert_eq!(
        refuse_bounds_as_authorization(),
        Err(IntakeAuthorizationError::BoundsAreNotAuthorization)
    );
}

#[test]
fn recovered_grant_flags_match_known_truth_better_than_accepting_every_intake() {
    let truth = [true, false, false];
    let recovered = [true, false, false];
    let collapsed = [true, true, true];
    let recovered_rate = identity_recovery_rate(&truth, &recovered).expect("recovered");
    let collapsed_rate = identity_recovery_rate(&truth, &collapsed).expect("collapsed");
    let expected = {
        let mut matches = 0_u32;
        for (truth_flag, decided_flag) in truth.iter().zip(recovered.iter()) {
            if truth_flag == decided_flag {
                matches += 1;
            }
        }
        f64::from(matches) / f64::from(u32::try_from(truth.len()).expect("len"))
    };
    assert!((recovered_rate - expected).abs() < f64::EPSILON);
    assert!(recovered_rate > collapsed_rate);
}

#[test]
fn empty_or_mismatched_grant_flags_fail_closed() {
    assert_eq!(
        identity_recovery_rate(&[], &[]),
        Err(IntakeAuthorizationError::InvalidIntakePayload)
    );
    assert_eq!(
        identity_recovery_rate(&[true], &[]),
        Err(IntakeAuthorizationError::InvalidIntakePayload)
    );
    assert_eq!(
        identity_recovery_rate(&[true, false], &[true]),
        Err(IntakeAuthorizationError::InvalidIntakePayload)
    );
}
