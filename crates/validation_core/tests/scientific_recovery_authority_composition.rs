//! Computed recovery is one evidence class; scientific authority also needs a bound exact-head receipt.

use validation_core::{
    ClaimAuthority, ScientificRecoveryExactHeadReceiptStatusV1,
    ScientificRecoveryExactHeadReceiptV1, ScientificRecoveryFailurePolicyV1,
    ScientificRecoveryProfileV1, ValidationError, promote_scientific_recovery,
};

const HEAD: &str = "b2a3f879ca61daefa534f122647074666d5604bc";
const OTHER_HEAD: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const RECEIPT: &str = "5555555555555555555555555555555555555555555555555555555555555555";
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

fn receipt(
    head: &str,
    status: ScientificRecoveryExactHeadReceiptStatusV1,
) -> ScientificRecoveryExactHeadReceiptV1 {
    ScientificRecoveryExactHeadReceiptV1::new(head, RECEIPT, status)
        .expect("valid exact-head receipt")
}

#[test]
fn passing_exact_head_receipt_composes_with_computed_recovery_and_is_retained() {
    let profile = exact_profile();
    let (truth, recovered) = exact_rows();
    let exact_head = receipt(HEAD, ScientificRecoveryExactHeadReceiptStatusV1::Passed);

    let promotion = promote_scientific_recovery(
        HEAD,
        HEAD,
        &truth,
        &recovered,
        &profile,
        &exact_head,
    )
    .expect("exact-head receipt plus computed recovery");

    assert_eq!(
        promotion.claim().authority(),
        ClaimAuthority::ScientificallySupported
    );
    assert_eq!(promotion.profile_sha256(), profile.sha256());
    assert_eq!(promotion.exact_head_receipt_sha256(), RECEIPT);
}

#[test]
fn predecessor_exact_head_receipt_cannot_be_relabelled_as_candidate_evidence() {
    let profile = exact_profile();
    let (truth, recovered) = exact_rows();
    let predecessor = receipt(
        OTHER_HEAD,
        ScientificRecoveryExactHeadReceiptStatusV1::Passed,
    );

    assert_eq!(
        promote_scientific_recovery(HEAD, HEAD, &truth, &recovered, &profile, &predecessor),
        Err(ValidationError::ClaimPredecessorHead)
    );
}

#[test]
fn nonpassing_exact_head_receipt_states_stay_fail_closed() {
    let profile = exact_profile();
    let (truth, recovered) = exact_rows();

    for (status, expected) in [
        (
            ScientificRecoveryExactHeadReceiptStatusV1::Failed,
            ValidationError::ClaimEvidenceFailed,
        ),
        (
            ScientificRecoveryExactHeadReceiptStatusV1::Queued,
            ValidationError::ClaimQueuedEvidence,
        ),
        (
            ScientificRecoveryExactHeadReceiptStatusV1::Skipped,
            ValidationError::ClaimSkippedRequired,
        ),
    ] {
        let exact_head = receipt(HEAD, status);
        assert_eq!(
            promote_scientific_recovery(HEAD, HEAD, &truth, &recovered, &profile, &exact_head),
            Err(expected)
        );
    }
}

#[test]
fn exact_head_receipt_requires_canonical_sha256_identity() {
    assert_eq!(
        ScientificRecoveryExactHeadReceiptV1::new(
            HEAD,
            "not-a-sha256",
            ScientificRecoveryExactHeadReceiptStatusV1::Passed,
        ),
        Err(ValidationError::InvalidInput)
    );
    assert_eq!(
        ScientificRecoveryExactHeadReceiptV1::new(
            HEAD,
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            ScientificRecoveryExactHeadReceiptStatusV1::Passed,
        ),
        Err(ValidationError::InvalidInput)
    );
}
