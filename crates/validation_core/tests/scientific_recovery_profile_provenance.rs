//! Scientific recovery authority stays bound to one immutable, versioned design profile.

use validation_core::{
    ClaimAuthority, ScientificRecoveryExactHeadReceiptStatusV1,
    ScientificRecoveryExactHeadReceiptV1, ScientificRecoveryExecutionLedgerEntryV1,
    ScientificRecoveryFailurePolicyV1, ScientificRecoveryProfileChronologyV1,
    ScientificRecoveryProfileRegistrationStatusV1, ScientificRecoveryProfileV1,
    ScientificRecoveryReplicationReceiptV1, ScientificRecoverySeedManifestV1, ValidationError,
    promote_scientific_recovery, scientific_recovery_replication_payload_sha256,
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
    .expect("valid recovery profile")
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
fn recovery_profile_identity_changes_with_predeclared_design_and_target() {
    let baseline = profile(2, 0.08);
    let baseline_manifest = seed_manifest(2);
    let different_denominator = profile(3, 0.08);
    let different_target = profile(2, 0.081);

    assert_eq!(baseline.planned_replications(), 2);
    assert_eq!(baseline.max_rmse().to_bits(), 0.08_f64.to_bits());
    assert_eq!(baseline.se_multiplier().to_bits(), 3.0_f64.to_bits());
    assert_eq!(baseline.dgp_sha256(), DGP);
    assert_eq!(baseline.seed_manifest_sha256(), baseline_manifest.sha256());
    assert_eq!(baseline.estimand_sha256(), ESTIMAND);
    assert_eq!(baseline.state_composition_sha256(), STATE);
    assert_eq!(
        baseline.failure_policy(),
        ScientificRecoveryFailurePolicyV1::RequireAllPlannedRecovered
    );
    assert_eq!(
        baseline.failure_policy().wire_name(),
        "require_all_planned_recovered"
    );

    assert_ne!(baseline.sha256(), different_denominator.sha256());
    assert_ne!(baseline.sha256(), different_target.sha256());

    let encoded = baseline.to_json().expect("canonical profile json");
    let decoded = ScientificRecoveryProfileV1::from_json(&encoded).expect("profile roundtrip");
    assert_eq!(decoded, baseline);
    assert_eq!(decoded.sha256(), baseline.sha256());
}

#[test]
fn malformed_or_noncanonical_profile_input_fails_closed() {
    let uppercase = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
    assert_eq!(
        ScientificRecoveryProfileV1::new(
            2,
            0.08,
            3.0,
            uppercase,
            SEEDS,
            ESTIMAND,
            STATE,
            ScientificRecoveryFailurePolicyV1::RequireAllPlannedRecovered,
        ),
        Err(ValidationError::InvalidInput)
    );
    assert_eq!(
        ScientificRecoveryProfileV1::new(
            2,
            0.08,
            3.0,
            "abcd",
            SEEDS,
            ESTIMAND,
            STATE,
            ScientificRecoveryFailurePolicyV1::RequireAllPlannedRecovered,
        ),
        Err(ValidationError::InvalidInput)
    );
    assert_eq!(
        ScientificRecoveryProfileV1::from_json("not-json"),
        Err(ValidationError::InvalidInput)
    );

    let baseline = profile(2, 0.08);
    let encoded = baseline.to_json().expect("profile json");
    let wrong_schema = encoded.replace(
        "tepp.scientific_recovery_profile.v1",
        "tepp.scientific_recovery_profile.v2",
    );
    assert_eq!(
        ScientificRecoveryProfileV1::from_json(&wrong_schema),
        Err(ValidationError::InvalidInput)
    );

    let unknown_field = encoded.replacen('{', "{\"unexpected\":true,", 1);
    assert_eq!(
        ScientificRecoveryProfileV1::from_json(&unknown_field),
        Err(ValidationError::InvalidInput)
    );
}

#[test]
fn negative_zero_uncertainty_multiplier_is_not_a_second_canonical_identity() {
    let canonical_zero = ScientificRecoveryProfileV1::new(
        2,
        0.08,
        0.0,
        DGP,
        SEEDS,
        ESTIMAND,
        STATE,
        ScientificRecoveryFailurePolicyV1::RequireAllPlannedRecovered,
    )
    .expect("positive zero is the canonical zero multiplier");
    assert_eq!(canonical_zero.se_multiplier().to_bits(), 0.0_f64.to_bits());

    assert_eq!(
        ScientificRecoveryProfileV1::new(
            2,
            0.08,
            -0.0,
            DGP,
            SEEDS,
            ESTIMAND,
            STATE,
            ScientificRecoveryFailurePolicyV1::RequireAllPlannedRecovered,
        ),
        Err(ValidationError::InvalidConfiguration)
    );
}

#[test]
fn promoted_scientific_authority_retains_profile_and_exact_head_receipt_identity() {
    let profile = profile(2, 0.08);
    let truth_rows = [[0.0], [0.0]];
    let recovered_rows = [[0.0], [0.0]];
    let truth: Vec<&[f64]> = truth_rows.iter().map(|row| row.as_slice()).collect();
    let recovered: Vec<&[f64]> = recovered_rows.iter().map(|row| row.as_slice()).collect();
    let receipt = exact_head_receipt();
    let replication_receipts = replication_receipts(&profile, &truth, &recovered);
    let chronology = chronology(&profile, &replication_receipts);

    let promotion = promote_scientific_recovery(
        HEAD,
        HEAD,
        &truth,
        &recovered,
        &profile,
        &receipt,
        &replication_receipts,
        &chronology,
    )
    .expect("exact recovery under immutable profile, chronology and exact-head receipt");

    assert_eq!(
        promotion.claim().authority(),
        ClaimAuthority::ScientificallySupported
    );
    assert_eq!(promotion.profile_sha256(), profile.sha256());
    assert_eq!(receipt.artifact_sha256(), RECEIPT);
    assert_eq!(
        promotion.exact_head_receipt_sha256(),
        receipt.receipt_sha256()
    );
    assert_eq!(promotion.profile_chronology_sha256(), chronology.sha256());
}
