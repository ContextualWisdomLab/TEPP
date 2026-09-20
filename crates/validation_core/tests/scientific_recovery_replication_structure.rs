//! Scientific recovery uncertainty is counted over independent simulation replications, not state rows.

use validation_core::{
    ScientificRecoveryExactHeadReceiptStatusV1, ScientificRecoveryExactHeadReceiptV1,
    ScientificRecoveryExecutionLedgerEntryV1, ScientificRecoveryFailurePolicyV1,
    ScientificRecoveryProfileChronologyV1, ScientificRecoveryProfileRegistrationStatusV1,
    ScientificRecoveryProfileV1, ScientificRecoveryReplicationReceiptV1,
    ScientificRecoverySeedManifestV1, ValidationError, promote_scientific_recovery,
    rmse_standard_error, root_mean_square_error,
    scientific_recovery_replication_payload_sha256,
};

const HEAD: &str = "b2a3f879ca61daefa534f122647074666d5604bc";
const RECEIPT: &str = "5555555555555555555555555555555555555555555555555555555555555555";
const DGP: &str = "1111111111111111111111111111111111111111111111111111111111111111";
const SEEDS: &str = "2222222222222222222222222222222222222222222222222222222222222222";
const ESTIMAND: &str = "3333333333333333333333333333333333333333333333333333333333333333";
const STATE: &str = "4444444444444444444444444444444444444444444444444444444444444444";
const LEDGER: &str = "6666666666666666666666666666666666666666666666666666666666666666";
const REGISTRATION_ENTRY: &str =
    "7777777777777777777777777777777777777777777777777777777777777777";

fn as_slices<const N: usize, const M: usize>(rows: &[[f64; M]; N]) -> Vec<&[f64]> {
    rows.iter().map(|row| row.as_slice()).collect()
}

fn seed_state(index: usize) -> String {
    format!("{:064x}", index + 1)
}

fn seed_manifest(planned_replications: usize) -> ScientificRecoverySeedManifestV1 {
    let states: Vec<String> = (0..planned_replications).map(seed_state).collect();
    let refs: Vec<&str> = states.iter().map(String::as_str).collect();
    ScientificRecoverySeedManifestV1::new(&refs).expect("valid seed manifest")
}

fn profile(planned_replications: usize, max_rmse: f64) -> ScientificRecoveryProfileV1 {
    let manifest = seed_manifest(planned_replications);
    ScientificRecoveryProfileV1::new(
        planned_replications,
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

fn chronology(profile: &ScientificRecoveryProfileV1) -> ScientificRecoveryProfileChronologyV1 {
    let entries: Vec<_> = (0..profile.planned_replications())
        .map(|index| {
            let execution_artifact = format!("{:064x}", index + 1024);
            let entry_sha = format!("{:064x}", index + 4096);
            ScientificRecoveryExecutionLedgerEntryV1::new(
                &execution_artifact,
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
fn repeated_correlated_states_do_not_masquerade_as_independent_monte_carlo_replications() {
    let flat_truth = [0.0; 128];
    let mut flat_recovered = [0.0; 128];
    flat_recovered[64..].fill(0.08);

    let flat_rmse = root_mean_square_error(&flat_truth, &flat_recovered).expect("flat rmse");
    let flat_rmse_se =
        rmse_standard_error(&flat_truth, &flat_recovered).expect("flat pseudo-replication se");
    assert!(
        flat_rmse + 3.0 * flat_rmse_se < 0.08,
        "the predecessor flat residual path must demonstrate the pseudo-replication false promotion"
    );

    let truth_replications = [[0.0; 64], [0.0; 64]];
    let recovered_replications = [[0.0; 64], [0.08; 64]];
    let truth = as_slices(&truth_replications);
    let recovered = as_slices(&recovered_replications);
    let profile = profile(2, 0.08);
    let replication_receipts = replication_receipts(&profile, &truth, &recovered);
    let chronology = chronology(&profile);

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
        "64 perfectly correlated state coordinates inside each of two runs must still count as only two independent Monte Carlo replications"
    );
}

#[test]
fn replication_structure_must_be_complete_before_promotion() {
    let truth_rows = [[0.0], [0.0]];
    let recovered_rows = [[0.0], [0.0]];
    let truth = as_slices(&truth_rows);
    let recovered = as_slices(&recovered_rows);
    let profile = profile(2, 0.01);
    let one_receipt = replication_receipts(&profile, &truth[..1], &recovered[..1]);
    let chronology = chronology(&profile);

    assert_eq!(
        promote_scientific_recovery(
            HEAD,
            HEAD,
            &truth,
            &recovered[..1],
            &profile,
            &exact_head_receipt(),
            &one_receipt,
            &chronology,
        ),
        Err(ValidationError::InvalidInput),
        "outer replication counts must match the declared design denominator"
    );

    let invalid_truth_rows = [[0.0, 0.0], [0.0, 0.0]];
    let invalid_recovered_rows = [[0.0, 0.0], [0.0, 0.0]];
    let invalid_truth = as_slices(&invalid_truth_rows);
    let full_recovered = as_slices(&invalid_recovered_rows);
    let replication_receipts = replication_receipts(&profile, &invalid_truth, &full_recovered);
    let mut invalid_recovered = full_recovered;
    invalid_recovered[1] = &invalid_recovered_rows[1][..1];
    assert_eq!(
        promote_scientific_recovery(
            HEAD,
            HEAD,
            &invalid_truth,
            &invalid_recovered,
            &profile,
            &exact_head_receipt(),
            &replication_receipts,
            &chronology,
        ),
        Err(ValidationError::InvalidInput),
        "each replication must contain a valid truth/recovery pair"
    );
}

#[test]
fn planned_replication_denominator_prevents_survivor_only_promotion() {
    let truth_rows = [[0.0], [0.0]];
    let recovered_rows = [[0.0], [0.0]];
    let truth = as_slices(&truth_rows);
    let recovered = as_slices(&recovered_rows);
    let profile = profile(3, 0.01);
    let replication_receipts = replication_receipts(&profile, &truth, &recovered);
    let chronology = chronology(&profile);

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
        Err(ValidationError::InvalidInput),
        "two successful survivors must not satisfy a profile that planned three independent replications"
    );

    assert_eq!(
        ScientificRecoveryProfileV1::new(
            1,
            0.01,
            3.0,
            DGP,
            SEEDS,
            ESTIMAND,
            STATE,
            ScientificRecoveryFailurePolicyV1::RequireAllPlannedRecovered,
        ),
        Err(ValidationError::InvalidConfiguration),
        "one exact-recovery repetition cannot define Monte Carlo scientific support"
    );
}
