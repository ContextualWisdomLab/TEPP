//! Extreme binary64 contract for scientific recovery's conservative RMSE bound.

use validation_core::{
    ScientificRecoveryExactHeadReceiptStatusV1, ScientificRecoveryExactHeadReceiptV1,
    ScientificRecoveryExecutionLedgerEntryV1, ScientificRecoveryFailurePolicyV1,
    ScientificRecoveryProfileChronologyV1, ScientificRecoveryProfileRegistrationStatusV1,
    ScientificRecoveryProfileV1, ScientificRecoveryReplicationReceiptV1,
    ScientificRecoverySeedManifestV1, ValidationError, promote_scientific_recovery,
    rmse_standard_error, root_mean_square_error, scientific_recovery_replication_payload_sha256,
};

const HEAD: &str = "b2a3f879ca61daefa534f122647074666d5604bc";
const RECEIPT: &str = "5555555555555555555555555555555555555555555555555555555555555555";
const DGP: &str = "1111111111111111111111111111111111111111111111111111111111111111";
const ESTIMAND: &str = "3333333333333333333333333333333333333333333333333333333333333333";
const STATE: &str = "4444444444444444444444444444444444444444444444444444444444444444";
const LEDGER: &str = "6666666666666666666666666666666666666666666666666666666666666666";
const REGISTRATION_ENTRY: &str = "7777777777777777777777777777777777777777777777777777777777777777";

fn as_slices<const N: usize, const M: usize>(rows: &[[f64; M]; N]) -> Vec<&[f64]> {
    rows.iter().map(|row| row.as_slice()).collect()
}

fn seed_state(index: usize) -> String {
    format!("{:064x}", index + 1)
}

fn seed_manifest() -> ScientificRecoverySeedManifestV1 {
    let states: Vec<String> = (0..2).map(seed_state).collect();
    let refs: Vec<&str> = states.iter().map(String::as_str).collect();
    ScientificRecoverySeedManifestV1::new(&refs).expect("valid seed manifest")
}

fn profile(max_rmse: f64) -> ScientificRecoveryProfileV1 {
    let manifest = seed_manifest();
    ScientificRecoveryProfileV1::new(
        2,
        max_rmse,
        3.0,
        DGP,
        manifest.sha256(),
        ESTIMAND,
        STATE,
        ScientificRecoveryFailurePolicyV1::RequireAllPlannedRecovered,
    )
    .expect("valid profile")
}

fn exact_head_receipt() -> ScientificRecoveryExactHeadReceiptV1 {
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

#[test]
fn scaled_projection_must_not_erase_positive_uncertainty_above_the_target() {
    let previous_max = f64::from_bits(f64::MAX.to_bits() - 1);
    let truth_flat = [0.0, 0.0];
    let recovered_flat = [f64::MAX, previous_max];
    let rmse = root_mean_square_error(&truth_flat, &recovered_flat).expect("finite extreme rmse");
    let rmse_se =
        rmse_standard_error(&truth_flat, &recovered_flat).expect("finite extreme rmse se");

    assert!(rmse < f64::MAX);
    assert!(rmse_se > 0.0);

    let scale = f64::MAX;
    let projected_upper = (rmse / scale) + 3.0 * (rmse_se / scale);
    assert_eq!(
        projected_upper.to_bits(),
        1.0_f64.to_bits(),
        "the predecessor scale projection loses the positive uncertainty contribution"
    );

    let truth_rows = [[0.0], [0.0]];
    let recovered_rows = [[f64::MAX], [previous_max]];
    let truth = as_slices(&truth_rows);
    let recovered = as_slices(&recovered_rows);
    let profile = profile(f64::MAX);
    let replication_receipts = replication_receipts(&profile, &truth, &recovered);
    let chronology = chronology(&profile, &replication_receipts);
    assert_eq!(
        promote_scientific_recovery(
            HEAD,
            HEAD,
            &truth,
            &recovered,
            &profile,
            &exact_head_receipt(),
            &replication_receipts,
            &chronology,
        ),
        Err(ValidationError::ClaimRecoveryRejected),
        "scientific authority must not be minted when positive RMSE uncertainty exceeds the remaining practical margin"
    );
}

#[test]
fn practical_target_boundary_is_not_promoted_as_scientific_support() {
    let truth_flat = [0.0, 0.0];
    let recovered_flat = [0.05, -0.05];
    let rmse = root_mean_square_error(&truth_flat, &recovered_flat).expect("rmse");
    let rmse_se = rmse_standard_error(&truth_flat, &recovered_flat).expect("rmse se");

    assert_eq!(rmse.to_bits(), 0.05_f64.to_bits());
    assert_eq!(rmse_se.to_bits(), 0.0_f64.to_bits());

    let truth_rows = [[0.0], [0.0]];
    let recovered_rows = [[0.05], [-0.05]];
    let truth = as_slices(&truth_rows);
    let recovered = as_slices(&recovered_rows);
    let profile = profile(0.05);
    let replication_receipts = replication_receipts(&profile, &truth, &recovered);
    let chronology = chronology(&profile, &replication_receipts);
    assert_eq!(
        promote_scientific_recovery(
            HEAD,
            HEAD,
            &truth,
            &recovered,
            &profile,
            &exact_head_receipt(),
            &replication_receipts,
            &chronology,
        ),
        Err(ValidationError::ClaimRecoveryRejected),
        "promotion requires the conservative RMSE bound to remain strictly inside the caller-owned target"
    );
}
