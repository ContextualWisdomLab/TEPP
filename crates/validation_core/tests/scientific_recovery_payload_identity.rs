//! Scientific recovery authority must retain the exact represented payload identity.

use validation_core::{
    ScientificRecoveryExactHeadReceiptStatusV1, ScientificRecoveryExactHeadReceiptV1,
    ScientificRecoveryExecutionLedgerEntryV1, ScientificRecoveryFailurePolicyV1,
    ScientificRecoveryProfileChronologyV1, ScientificRecoveryProfileRegistrationStatusV1,
    ScientificRecoveryProfileV1, ScientificRecoveryReplicationReceiptV1,
    ScientificRecoverySeedManifestV1, promote_scientific_recovery,
    scientific_recovery_replication_payload_sha256,
};

const HEAD: &str = "b2a3f879ca61daefa534f122647074666d5604bc";
const RECEIPT: &str = "5555555555555555555555555555555555555555555555555555555555555555";
const DGP: &str = "1111111111111111111111111111111111111111111111111111111111111111";
const ESTIMAND: &str = "3333333333333333333333333333333333333333333333333333333333333333";
const STATE: &str = "4444444444444444444444444444444444444444444444444444444444444444";
const LEDGER: &str = "6666666666666666666666666666666666666666666666666666666666666666";
const REGISTRATION_ENTRY: &str = "7777777777777777777777777777777777777777777777777777777777777777";

fn seed_state(index: usize) -> String {
    format!("{:064x}", index + 1)
}

fn seed_manifest() -> ScientificRecoverySeedManifestV1 {
    let states: Vec<String> = (0..2).map(seed_state).collect();
    let refs: Vec<&str> = states.iter().map(String::as_str).collect();
    ScientificRecoverySeedManifestV1::new(&refs).expect("valid seed manifest")
}

fn profile() -> ScientificRecoveryProfileV1 {
    let manifest = seed_manifest();
    ScientificRecoveryProfileV1::new(
        2,
        0.08,
        3.0,
        DGP,
        manifest.sha256(),
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
            let seed_state = seed_state(index);
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

fn chronology(
    profile: &ScientificRecoveryProfileV1,
    receipts: &[ScientificRecoveryReplicationReceiptV1],
) -> ScientificRecoveryProfileChronologyV1 {
    let entries: Vec<_> = receipts
        .iter()
        .enumerate()
        .map(|(index, receipt)| {
            let entry_sha = format!("{:064x}", index + 4096);
            ScientificRecoveryExecutionLedgerEntryV1::new(
                receipt.execution_artifact_sha256(),
                &entry_sha,
                101 + index as u64,
            )
            .expect("valid execution ledger entry")
        })
        .collect();
    ScientificRecoveryProfileChronologyV1::new(
        profile,
        LEDGER,
        REGISTRATION_ENTRY,
        100,
        ScientificRecoveryProfileRegistrationStatusV1::Approved,
        &entries,
    )
    .expect("approved chronology")
}

fn promote(
    truth_rows: &[[f64; 2]; 2],
    recovered_rows: &[[f64; 2]; 2],
) -> validation_core::ScientificRecoveryPromotionV1 {
    let truth: Vec<&[f64]> = truth_rows.iter().map(|row| row.as_slice()).collect();
    let recovered: Vec<&[f64]> = recovered_rows.iter().map(|row| row.as_slice()).collect();
    let profile = profile();
    let replication_receipts = replication_receipts(&profile, &truth, &recovered);
    let chronology = chronology(&profile, &replication_receipts);
    promote_scientific_recovery(
        HEAD,
        HEAD,
        &truth,
        &recovered,
        &profile,
        &receipt(),
        &replication_receipts,
        &chronology,
    )
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
