//! Untrusted payloads fail closed without identity, provenance, size, and depth.

use payload_bound::{
    PayloadBound, PayloadBoundError, PayloadKind, identity_recovery_rate, refuse_untrusted_payload,
};

fn bound() -> PayloadBound {
    PayloadBound::new(8, 2).expect("bound")
}

#[test]
fn untrusted_payloads_fail_closed_until_identity_provenance_size_and_depth_validate() {
    assert_eq!(
        PayloadBound::new(0, 1),
        Err(PayloadBoundError::InvalidBound)
    );
    assert_eq!(
        PayloadBound::new(1, 0),
        Err(PayloadBoundError::InvalidBound)
    );
    assert_eq!(
        refuse_untrusted_payload(PayloadKind::Document, Some(""), Some("prov"), 1, 1, bound()),
        Err(PayloadBoundError::MissingIdentity)
    );
    assert_eq!(
        refuse_untrusted_payload(PayloadKind::Document, Some("id"), Some(""), 1, 1, bound()),
        Err(PayloadBoundError::MissingProvenance)
    );
    assert_eq!(
        refuse_untrusted_payload(PayloadKind::LlmOutput, None, Some("prov"), 1, 1, bound()),
        Err(PayloadBoundError::MissingIdentity)
    );
    assert_eq!(
        refuse_untrusted_payload(
            PayloadKind::SerializedRecord,
            Some("id"),
            None,
            1,
            1,
            bound()
        ),
        Err(PayloadBoundError::MissingProvenance)
    );
    assert_eq!(
        refuse_untrusted_payload(
            PayloadKind::ModelCheckpoint,
            Some("id"),
            Some("prov"),
            9,
            1,
            bound()
        ),
        Err(PayloadBoundError::PayloadTooLarge)
    );
    assert_eq!(
        refuse_untrusted_payload(
            PayloadKind::Document,
            Some("id"),
            Some("prov"),
            1,
            3,
            bound()
        ),
        Err(PayloadBoundError::PayloadTooDeep)
    );
    refuse_untrusted_payload(
        PayloadKind::Document,
        Some("id"),
        Some("prov"),
        8,
        2,
        bound(),
    )
    .expect("bounded trusted-enough payload");
}

#[test]
fn recovered_accept_flags_match_known_truth_better_than_accepting_every_payload() {
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
fn empty_or_mismatched_payload_flags_fail_closed() {
    assert_eq!(
        identity_recovery_rate(&[], &[]),
        Err(PayloadBoundError::InvalidPayloadDecision)
    );
    assert_eq!(
        identity_recovery_rate(&[true], &[]),
        Err(PayloadBoundError::InvalidPayloadDecision)
    );
    assert_eq!(
        identity_recovery_rate(&[true, false], &[true]),
        Err(PayloadBoundError::InvalidPayloadDecision)
    );
}
