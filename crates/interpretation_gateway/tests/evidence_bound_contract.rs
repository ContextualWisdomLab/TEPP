//! LLM interpretations stay hypothetical and require evidence spans.

use interpretation_gateway::{
    ClaimSupport, EvidenceBoundInterpretation, InterpretationError, InterpretationId,
    refuse_interpretation_as_estimator_result, refuse_interpretation_as_observed_fact,
    unsupported_claim_rate,
};
use uuid::Uuid;

#[test]
fn missing_spans_and_hostile_identities_fail_closed() {
    assert_eq!(
        EvidenceBoundInterpretation::propose(InterpretationId::from_uuid(Uuid::nil()), &[]),
        Err(InterpretationError::MissingEvidenceSpan)
    );
}

#[test]
fn interpretation_cannot_become_an_estimator_or_observed_fact() {
    let span = Uuid::from_u128(7);
    let interpretation = EvidenceBoundInterpretation::propose(
        InterpretationId::from_uuid(Uuid::from_u128(1)),
        &[span],
    )
    .expect("cited interpretation");
    assert!(interpretation.is_hypothetical());
    assert_eq!(
        refuse_interpretation_as_estimator_result(interpretation.interpretation_id()),
        Err(InterpretationError::InterpretationIsNotEstimatorResult)
    );
    assert_eq!(
        refuse_interpretation_as_observed_fact(interpretation.interpretation_id()),
        Err(InterpretationError::InterpretationIsNotObservedFact)
    );
}

#[test]
fn cited_interpreter_has_lower_unsupported_claim_rate_than_uncited_promotion() {
    let truth = [
        ClaimSupport::Unsupported,
        ClaimSupport::Unsupported,
        ClaimSupport::Supported,
    ];
    let cited = [
        ClaimSupport::Unsupported,
        ClaimSupport::Unsupported,
        ClaimSupport::Supported,
    ];
    let uncited_promotion = [
        ClaimSupport::Supported,
        ClaimSupport::Supported,
        ClaimSupport::Supported,
    ];

    let cited_rate = unsupported_claim_rate(&truth, &cited).expect("cited");
    let uncited_rate = unsupported_claim_rate(&truth, &uncited_promotion).expect("uncited");
    let cited_expected = {
        let mut unsupported_truth = 0_u32;
        let mut false_support = 0_u32;
        for (truth_label, decided_label) in truth.iter().zip(cited.iter()) {
            if *truth_label == ClaimSupport::Unsupported {
                unsupported_truth += 1;
                if *decided_label == ClaimSupport::Supported {
                    false_support += 1;
                }
            }
        }
        f64::from(false_support) / f64::from(unsupported_truth)
    };
    assert!((cited_rate - cited_expected).abs() < f64::EPSILON);
    assert!(cited_rate < uncited_rate);
}
