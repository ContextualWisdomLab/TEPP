//! Scientific claim promotion separates practical recovery accuracy from Monte Carlo precision.

use validation_core::{
    ScientificRecoveryExactHeadReceiptStatusV1, ScientificRecoveryExactHeadReceiptV1,
    ScientificRecoveryFailurePolicyV1, ScientificRecoveryProfileV1,
    ScientificRecoveryReplicationReceiptV1, ValidationError, promote_scientific_recovery,
    rmse_standard_error, root_mean_square_error,
    scientific_recovery_replication_payload_sha256,
};

const HEAD: &str = "b2a3f879ca61daefa534f122647074666d5604bc";
const RECEIPT: &str = "5555555555555555555555555555555555555555555555555555555555555555";
const DGP: &str = "1111111111111111111111111111111111111111111111111111111111111111";
const SEEDS: &str = "2222222222222222222222222222222222222222222222222222222222222222";
const ESTIMAND: &str = "3333333333333333333333333333333333333333333333333333333333333333";
const STATE: &str = "4444444444444444444444444444444444444444444444444444444444444444";

fn as_slices<const N: usize, const M: usize>(rows: &[[f64; M]; N]) -> Vec<&[f64]> {
    rows.iter().map(|row| row.as_slice()).collect()
}

fn profile(
    planned_replications: usize,
    max_rmse: f64,
    se_multiplier: f64,
) -> ScientificRecoveryProfileV1 {
    ScientificRecoveryProfileV1::new(
        planned_replications,
        max_rmse,
        se_multiplier,
        DGP,
        SEEDS,
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
fn practical_rmse_inside_target_is_not_rejected_for_being_precisely_nonzero() {
    let truth_flat = [0.0; 8];
    let recovered_flat = [0.02, -0.02, 0.02, -0.02, 0.02, -0.02, 0.02, -0.02];
    let rmse = root_mean_square_error(&truth_flat, &recovered_flat).expect("rmse");
    let rmse_se = rmse_standard_error(&truth_flat, &recovered_flat).expect("rmse se");

    assert!(rmse < 0.05);
    assert_eq!(rmse_se.to_bits(), 0.0_f64.to_bits());

    let truth_rows = [[0.0]; 8];
    let recovered_rows = [
        [0.02], [-0.02], [0.02], [-0.02], [0.02], [-0.02], [0.02], [-0.02],
    ];
    let truth = as_slices(&truth_rows);
    let recovered = as_slices(&recovered_rows);
    let profile = profile(8, 0.05, 3.0);
    let replication_receipts = replication_receipts(&profile, &truth, &recovered);
    promote_scientific_recovery(
        HEAD,
        HEAD,
        &truth,
        &recovered,
        &profile,
        &exact_head_receipt(),
        &replication_receipts,
    )
    .expect("a precisely estimated RMSE inside the explicit practical target must promote");
}

#[test]
fn monte_carlo_uncertainty_remains_binding_near_practical_rmse_target() {
    let truth_flat = [0.0; 8];
    let recovered_flat = [0.0, 0.06, 0.0, 0.06, 0.0, 0.06, 0.0, 0.06];
    let rmse = root_mean_square_error(&truth_flat, &recovered_flat).expect("rmse");
    let rmse_se = rmse_standard_error(&truth_flat, &recovered_flat).expect("rmse se");

    assert!(rmse < 0.05, "point RMSE must remain inside the target");
    assert!(
        rmse + 3.0 * rmse_se > 0.05,
        "uncertainty must push the conservative bound outside the target"
    );

    let truth_rows = [[0.0]; 8];
    let recovered_rows = [[0.0], [0.06], [0.0], [0.06], [0.0], [0.06], [0.0], [0.06]];
    let truth = as_slices(&truth_rows);
    let recovered = as_slices(&recovered_rows);
    let profile = profile(8, 0.05, 3.0);
    let replication_receipts = replication_receipts(&profile, &truth, &recovered);
    assert_eq!(
        promote_scientific_recovery(
            HEAD,
            HEAD,
            &truth,
            &recovered,
            &profile,
            &exact_head_receipt(),
            &replication_receipts,
        ),
        Err(ValidationError::ClaimRecoveryRejected)
    );
}

#[test]
fn invalid_practical_targets_and_multipliers_fail_closed_at_profile_construction() {
    assert_eq!(
        ScientificRecoveryProfileV1::new(
            2,
            0.0,
            3.0,
            DGP,
            SEEDS,
            ESTIMAND,
            STATE,
            ScientificRecoveryFailurePolicyV1::RequireAllPlannedRecovered,
        ),
        Err(ValidationError::InvalidConfiguration)
    );
    assert_eq!(
        ScientificRecoveryProfileV1::new(
            2,
            0.1,
            -1.0,
            DGP,
            SEEDS,
            ESTIMAND,
            STATE,
            ScientificRecoveryFailurePolicyV1::RequireAllPlannedRecovered,
        ),
        Err(ValidationError::InvalidConfiguration)
    );
}
