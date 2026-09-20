//! Scientific recovery authority must retain the exact represented payload identity.

use validation_core::{
    ScientificRecoveryExactHeadReceiptStatusV1, ScientificRecoveryExactHeadReceiptV1,
    ScientificRecoveryFailurePolicyV1, ScientificRecoveryProfileV1, promote_scientific_recovery,
};

const HEAD: &str = "b2a3f879ca61daefa534f122647074666d5604bc";
const RECEIPT: &str = "5555555555555555555555555555555555555555555555555555555555555555";
const DGP: &str = "1111111111111111111111111111111111111111111111111111111111111111";
const SEEDS: &str = "2222222222222222222222222222222222222222222222222222222222222222";
const ESTIMAND: &str = "3333333333333333333333333333333333333333333333333333333333333333";
const STATE: &str = "4444444444444444444444444444444444444444444444444444444444444444";

fn profile() -> ScientificRecoveryProfileV1 {
    ScientificRecoveryProfileV1::new(
        2,
        0.08,
        3.0,
        DGP,
        SEEDS,
        ESTIMAND,
        STATE,
        ScientificRecoveryFailurePolicyV1::RequireAllPlannedRecovered,
    )
    .expect("valid profile")
}

fn receipt() -> ScientificRecoveryExactHeadReceiptV1 {
    ScientificRecoveryExactHeadReceiptV1::new(
        HEAD,
        RECEIPT,
        ScientificRecoveryExactHeadReceiptStatusV1::Passed,
    )
    .expect("valid exact-head receipt")
}

fn promote(
    truth_rows: &[[f64; 2]; 2],
    recovered_rows: &[[f64; 2]; 2],
) -> validation_core::ScientificRecoveryPromotionV1 {
    let truth: Vec<&[f64]> = truth_rows.iter().map(|row| row.as_slice()).collect();
    let recovered: Vec<&[f64]> = recovered_rows.iter().map(|row| row.as_slice()).collect();
    promote_scientific_recovery(HEAD, HEAD, &truth, &recovered, &profile(), &receipt())
        .expect("valid exact recovery")
}

#[test]
fn promotion_identity_distinguishes_the_exact_evaluated_payload() {
    let zeros = [[0.0, 0.0], [0.0, 0.0]];
    let ones = [[1.0, 1.0], [1.0, 1.0]];

    let first = promote(&zeros, &zeros);
    let repeated = promote(&zeros, &zeros);
    let different = promote(&ones, &ones);

    assert_eq!(
        first.recovery_evidence_sha256(),
        repeated.recovery_evidence_sha256()
    );
    assert_ne!(
        first.recovery_evidence_sha256(),
        different.recovery_evidence_sha256()
    );
}

#[test]
fn signed_zero_does_not_create_a_second_payload_identity() {
    let positive_zero = [[0.0, 0.0], [0.0, 0.0]];
    let negative_zero = [[-0.0, 0.0], [0.0, -0.0]];

    let positive = promote(&positive_zero, &positive_zero);
    let negative = promote(&negative_zero, &negative_zero);

    assert_eq!(
        positive.recovery_evidence_sha256(),
        negative.recovery_evidence_sha256()
    );
}

#[test]
fn payload_identity_commits_replication_and_coordinate_order() {
    let truth = [[0.0, 1.0], [2.0, 3.0]];
    let recovered = truth;
    let swapped_replications = [[2.0, 3.0], [0.0, 1.0]];
    let swapped_coordinates = [[1.0, 0.0], [3.0, 2.0]];

    let baseline = promote(&truth, &recovered);
    let replication_order = promote(&swapped_replications, &swapped_replications);
    let coordinate_order = promote(&swapped_coordinates, &swapped_coordinates);

    assert_ne!(
        baseline.recovery_evidence_sha256(),
        replication_order.recovery_evidence_sha256()
    );
    assert_ne!(
        baseline.recovery_evidence_sha256(),
        coordinate_order.recovery_evidence_sha256()
    );
}
