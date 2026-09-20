//! Computed recovery is one evidence class; ADR 0014 authority still needs exact-head tests.

use validation_core::{
    ClaimAuthority, ClaimEvidence, ClaimEvidenceKind, ScientificRecoveryFailurePolicyV1,
    ScientificRecoveryProfileV1, ValidationError, promote_scientific_recovery,
};

const HEAD: &str = "b2a3f879ca61daefa534f122647074666d5604bc";
const DGP: &str = "1111111111111111111111111111111111111111111111111111111111111111";
const SEEDS: &str = "2222222222222222222222222222222222222222222222222222222222222222";
const ESTIMAND: &str = "3333333333333333333333333333333333333333333333333333333333333333";
const STATE: &str = "4444444444444444444444444444444444444444444444444444444444444444";

fn exact_profile() -> ScientificRecoveryProfileV1 {
    ScientificRecoveryProfileV1::new(
        2,
        0.01,
        3.0,
        DGP,
        SEEDS,
        ESTIMAND,
        STATE,
        ScientificRecoveryFailurePolicyV1::RequireAllPlannedRecovered,
    )
    .expect("valid recovery profile")
}

fn exact_rows() -> (Vec<&'static [f64]>, Vec<&'static [f64]>) {
    static TRUTH: [[f64; 1]; 2] = [[0.0], [0.0]];
    static RECOVERED: [[f64; 1]; 2] = [[0.0], [0.0]];
    (
        TRUTH.iter().map(|row| row.as_slice()).collect(),
        RECOVERED.iter().map(|row| row.as_slice()).collect(),
    )
}

#[test]
fn recovery_metrics_without_exact_head_tests_do_not_mint_scientific_authority() {
    let profile = exact_profile();
    let (truth, recovered) = exact_rows();

    assert_eq!(
        promote_scientific_recovery(HEAD, HEAD, &truth, &recovered, &profile, &[]),
        Err(ValidationError::ClaimEvidenceMissing)
    );
}

#[test]
fn passing_exact_head_tests_compose_with_computed_recovery() {
    let profile = exact_profile();
    let (truth, recovered) = exact_rows();
    let exact_head = [ClaimEvidence::new(ClaimEvidenceKind::ExactHeadTests, true)];

    let promotion = promote_scientific_recovery(
        HEAD,
        HEAD,
        &truth,
        &recovered,
        &profile,
        &exact_head,
    )
    .expect("exact-head tests plus computed recovery");

    assert_eq!(
        promotion.claim().authority(),
        ClaimAuthority::ScientificallySupported
    );
    assert_eq!(promotion.profile_sha256(), profile.sha256());
}

#[test]
fn unusable_or_failing_exact_head_evidence_stays_fail_closed() {
    let profile = exact_profile();
    let (truth, recovered) = exact_rows();

    for (evidence, expected) in [
        (
            ClaimEvidence::new(ClaimEvidenceKind::ExactHeadTests, false),
            ValidationError::ClaimEvidenceFailed,
        ),
        (
            ClaimEvidence::new(ClaimEvidenceKind::QueuedCheck, true),
            ValidationError::ClaimQueuedEvidence,
        ),
        (
            ClaimEvidence::new(ClaimEvidenceKind::PredecessorHead, true),
            ValidationError::ClaimPredecessorHead,
        ),
        (
            ClaimEvidence::new(ClaimEvidenceKind::SkippedRequired, true),
            ValidationError::ClaimSkippedRequired,
        ),
        (
            ClaimEvidence::new(ClaimEvidenceKind::LlmJudgment, true),
            ValidationError::ClaimLlmJudgment,
        ),
    ] {
        assert_eq!(
            promote_scientific_recovery(
                HEAD,
                HEAD,
                &truth,
                &recovered,
                &profile,
                &[evidence],
            ),
            Err(expected)
        );
    }
}
