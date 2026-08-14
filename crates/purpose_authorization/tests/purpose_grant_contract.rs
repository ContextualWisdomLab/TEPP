//! Purpose grants cannot be reused across purposes or replaced by blanket masking.

use purpose_authorization::{
    AuthorizationGrant, PrincipalId, PurposeAuthorizationError, PurposeCode, purpose_recovery_rate,
    refuse_blanket_mask_as_authorization, refuse_cross_purpose_use,
};
use uuid::Uuid;

#[test]
fn a_grant_cannot_authorize_a_different_purpose_or_a_blanket_mask() {
    let grant = AuthorizationGrant::new(
        PurposeCode::PsychometricAnalysis,
        PrincipalId::from_uuid(Uuid::from_u128(3)),
    );
    assert_eq!(
        grant.authorize(PurposeCode::ExportFulfillment),
        Err(PurposeAuthorizationError::CrossPurposeUse)
    );
    assert_eq!(
        refuse_cross_purpose_use(
            PurposeCode::PsychometricAnalysis,
            PurposeCode::LegalPreservation
        ),
        Err(PurposeAuthorizationError::CrossPurposeUse)
    );
    assert_eq!(
        refuse_blanket_mask_as_authorization(),
        Err(PurposeAuthorizationError::BlanketMaskIsNotAuthorization)
    );
    grant
        .authorize(PurposeCode::PsychometricAnalysis)
        .expect("same purpose");
}

#[test]
fn recovered_purposes_match_known_truth_better_than_a_single_purpose() {
    let truth = [
        PurposeCode::PsychometricAnalysis,
        PurposeCode::LegalPreservation,
        PurposeCode::OperationsAudit,
    ];
    let recovered = truth;
    let collapsed = [
        PurposeCode::PsychometricAnalysis,
        PurposeCode::PsychometricAnalysis,
        PurposeCode::PsychometricAnalysis,
    ];
    let recovered_rate = purpose_recovery_rate(&truth, &recovered).expect("recovered");
    let collapsed_rate = purpose_recovery_rate(&truth, &collapsed).expect("collapsed");
    let expected = {
        let mut matches = 0_u32;
        for (truth_purpose, decided_purpose) in truth.iter().zip(recovered.iter()) {
            if truth_purpose == decided_purpose {
                matches += 1;
            }
        }
        f64::from(matches) / f64::from(u32::try_from(truth.len()).expect("len"))
    };
    assert!((recovered_rate - expected).abs() < f64::EPSILON);
    assert!(recovered_rate > collapsed_rate);
}

#[test]
fn empty_purpose_payloads_fail_closed() {
    assert_eq!(
        purpose_recovery_rate(&[], &[]),
        Err(PurposeAuthorizationError::InvalidPurposePayload)
    );
}
