//! Computed recovery is one evidence class; scientific authority also needs a bound exact-head receipt.

use validation_core::{
    ClaimAuthority, ScientificRecoveryExactHeadReceiptStatusV1,
    ScientificRecoveryExactHeadReceiptV1, ScientificRecoveryFailurePolicyV1,
    ScientificRecoveryProfileV1, ScientificRecoveryReplicationReceiptV1, ValidationError,
    promote_scientific_recovery, scientific_recovery_replication_payload_sha256,
};

const HEAD: &str = "b2a3f879ca61daefa534f122647074666d5604bc";
const OTHER_HEAD: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const RECEIPT_ARTIFACT: &str =
    "5555555555555555555555555555555555555555555555555555555555555555";
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
    ScientificRecoveryExactHeadReceiptV1::new(head, RECEIPT_ARTIFACT, status)
        .expect("valid exact-head receipt")
}

fn replication_receipts(
    profile: &ScientificRecoveryProfileV1,
    truth: &[&[f64]],
    recovered: &[&[f64]],
) -> Vec<ScientificRecoveryReplicationReceiptV1> {
    truth
        .iter()
        .zip(recovered)
        .enumerate()
        .map(|(index, (truth, recovered))| {
            let payload = scientific_recovery_replication_payload_sha256(truth, recovered)
                .expect("valid replication payload");
            let seed_state = format!("{:064x}", index + 1);
            let execution_artifact = format!("{:064x}", index + 1024);
            ScientificRecoveryReplicationReceiptV1::new(
                index,
                &profile.sha256(),
                profile.seed_manifest_sha256(),
                &seed_state,
                &execution_artifact,
                &payload,
            )
            .expect("valid replication receipt")
        })
        .collect()
}

#[test]
fn passing_exact_head_receipt_composes_with_computed_recovery_and_is_retained() {
    let profile = exact_profile();
    let (truth, recovered) = exact_rows();
    let exact_head = receipt(HEAD, ScientificRecoveryExactHeadReceiptStatusV1::Passed);
    let replication_receipts = replication_receipts(&profile, &truth, &recovered);

    let promotion = promote_scientific_recovery(
        HEAD,
        HEAD,
        &truth,
        &recovered,
        &profile,
        &exact_head,
        &replication_receipts,
    )
    .expect("exact-head receipt plus computed recovery");

    assert_eq!(
        promotion.claim().authority(),
        ClaimAuthority::ScientificallySupported
    );
    assert_eq!(promotion.profile_sha256(), profile.sha256());
    assert_eq!(exact_head.artifact_sha256(), RECEIPT_ARTIFACT);
    assert_eq!(
        promotion.exact_head_receipt_sha256(),
        exact_head.receipt_sha256()
    );
    assert_ne!(promotion.exact_head_receipt_sha256(), RECEIPT_ARTIFACT);
}

#[test]
fn predecessor_exact_head_receipt_cannot_be_relabelled_as_candidate_evidence() {
    let profile = exact_profile();
    let (truth, recovered) = exact_rows();
    let predecessor = receipt(
        OTHER_HEAD,
        ScientificRecoveryExactHeadReceiptStatusV1::Passed,
    );
    let replication_receipts = replication_receipts(&profile, &truth, &recovered);

    assert_eq!(
        promote_scientific_recovery(
            HEAD,
            HEAD,
            &truth,
            &recovered,
            &profile,
            &predecessor,
            &replication_receipts,
        ),
        Err(ValidationError::ClaimPredecessorHead)
    );
}

#[test]
fn nonpassing_exact_head_receipt_states_stay_fail_closed() {
    let profile = exact_profile();
    let (truth, recovered) = exact_rows();
    let replication_receipts = replication_receipts(&profile, &truth, &recovered);

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
            promote_scientific_recovery(
                HEAD,
                HEAD,
                &truth,
                &recovered,
                &profile,
                &exact_head,
                &replication_receipts,
            ),
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
