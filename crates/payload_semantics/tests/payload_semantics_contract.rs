//! Untrusted payloads fail closed until scientific semantics validate.

use payload_semantics::{
    PayloadKind, PayloadSemanticsError, ScientificRole, refuse_bounds_as_semantics,
    refuse_untrusted_scientific_claim, semantics_recovery_rate,
};

#[test]
fn untrusted_payloads_fail_closed_until_scientific_semantics_validate() {
    for kind in [
        PayloadKind::Document,
        PayloadKind::ExternalMetadata,
        PayloadKind::SerializedRecord,
    ] {
        refuse_untrusted_scientific_claim(kind, ScientificRole::EvidenceContext)
            .expect("evidence context");
        assert_eq!(
            refuse_untrusted_scientific_claim(kind, ScientificRole::EstimatorResult),
            Err(PayloadSemanticsError::UntrustedPayloadIsNotEstimator)
        );
        assert_eq!(
            refuse_untrusted_scientific_claim(kind, ScientificRole::PosteriorSummary),
            Err(PayloadSemanticsError::UntrustedPayloadIsNotEstimator)
        );
        assert_eq!(
            refuse_untrusted_scientific_claim(kind, ScientificRole::InterpretationNarrative),
            Err(PayloadSemanticsError::EvidenceIsNotInterpretation)
        );
    }

    refuse_untrusted_scientific_claim(
        PayloadKind::LlmOutput,
        ScientificRole::InterpretationNarrative,
    )
    .expect("interpretation");
    assert_eq!(
        refuse_untrusted_scientific_claim(PayloadKind::LlmOutput, ScientificRole::EvidenceContext),
        Err(PayloadSemanticsError::LlmOutputIsNotEvidence)
    );
    assert_eq!(
        refuse_untrusted_scientific_claim(PayloadKind::LlmOutput, ScientificRole::EstimatorResult),
        Err(PayloadSemanticsError::UntrustedPayloadIsNotEstimator)
    );
    assert_eq!(
        refuse_untrusted_scientific_claim(PayloadKind::LlmOutput, ScientificRole::PosteriorSummary),
        Err(PayloadSemanticsError::UntrustedPayloadIsNotEstimator)
    );
    assert_eq!(
        refuse_bounds_as_semantics(),
        Err(PayloadSemanticsError::BoundsAreNotSemantics)
    );
}

#[test]
fn recovered_roles_match_known_truth_better_than_estimator_collapse() {
    let truth = [
        ScientificRole::EvidenceContext,
        ScientificRole::InterpretationNarrative,
        ScientificRole::EstimatorResult,
    ];
    let recovered = truth;
    let collapsed = [
        ScientificRole::EstimatorResult,
        ScientificRole::EstimatorResult,
        ScientificRole::EstimatorResult,
    ];
    let recovered_rate = semantics_recovery_rate(&truth, &recovered).expect("recovered");
    let collapsed_rate = semantics_recovery_rate(&truth, &collapsed).expect("collapsed");
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
        semantics_recovery_rate(&[], &[]),
        Err(PayloadSemanticsError::InvalidSemanticsPayload)
    );
    assert_eq!(
        semantics_recovery_rate(&[ScientificRole::EvidenceContext], &[]),
        Err(PayloadSemanticsError::InvalidSemanticsPayload)
    );
    assert_eq!(
        semantics_recovery_rate(
            &[
                ScientificRole::EvidenceContext,
                ScientificRole::InterpretationNarrative
            ],
            &[ScientificRole::EvidenceContext]
        ),
        Err(PayloadSemanticsError::InvalidSemanticsPayload)
    );
}
