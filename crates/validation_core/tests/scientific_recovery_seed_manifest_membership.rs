//! Scientific recovery binds every repetition to the exact ordered seed manifest committed by the profile.

use validation_core::{
    ClaimAuthority, ScientificRecoveryExactHeadReceiptStatusV1,
    ScientificRecoveryExactHeadReceiptV1, ScientificRecoveryFailurePolicyV1,
    ScientificRecoveryProfileV1, ScientificRecoveryReplicationReceiptV1,
    ScientificRecoverySeedManifestV1, ValidationError, promote_scientific_recovery,
    scientific_recovery_replication_payload_sha256,
};

const HEAD: &str = "b2a3f879ca61daefa534f122647074666d5604bc";
const TEST_ARTIFACT: &str =
    "5555555555555555555555555555555555555555555555555555555555555555";
const DGP: &str = "1111111111111111111111111111111111111111111111111111111111111111";
const ESTIMAND: &str =
    "3333333333333333333333333333333333333333333333333333333333333333";
const STATE: &str = "4444444444444444444444444444444444444444444444444444444444444444";
const SEED_0: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const SEED_1: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const SEED_OUTSIDE: &str =
    "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
const EXEC_0: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
const EXEC_1: &str = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";

fn seed_manifest() -> ScientificRecoverySeedManifestV1 {
    ScientificRecoverySeedManifestV1::new(&[SEED_0, SEED_1]).expect("valid ordered seed manifest")
}

fn profile(manifest: &ScientificRecoverySeedManifestV1) -> ScientificRecoveryProfileV1 {
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
    .expect("valid recovery profile")
}

fn exact_head_receipt() -> ScientificRecoveryExactHeadReceiptV1 {
    ScientificRecoveryExactHeadReceiptV1::new(
        HEAD,
        TEST_ARTIFACT,
        ScientificRecoveryExactHeadReceiptStatusV1::Passed,
    )
    .expect("valid exact-head receipt")
}

fn replication_receipt(
    profile: &ScientificRecoveryProfileV1,
    index: usize,
    seed_state_sha256: &str,
    execution_artifact_sha256: &str,
    truth: &[f64],
    recovered: &[f64],
) -> ScientificRecoveryReplicationReceiptV1 {
    let payload_sha256 = scientific_recovery_replication_payload_sha256(truth, recovered)
        .expect("valid represented replication payload");
    ScientificRecoveryReplicationReceiptV1::new(
        index,
        &profile.sha256(),
        profile.seed_manifest_sha256(),
        seed_state_sha256,
        execution_artifact_sha256,
        &payload_sha256,
    )
    .expect("valid replication receipt")
}

#[test]
fn promotion_requires_exact_ordered_seed_manifest_membership() {
    let manifest = seed_manifest();
    let profile = profile(&manifest);
    let truth_rows = [[0.0], [1.0]];
    let recovered_rows = truth_rows;
    let truth: Vec<&[f64]> = truth_rows.iter().map(|row| row.as_slice()).collect();
    let recovered: Vec<&[f64]> = recovered_rows.iter().map(|row| row.as_slice()).collect();
    let exact_head = exact_head_receipt();
    let receipts = vec![
        replication_receipt(&profile, 0, SEED_0, EXEC_0, truth[0], recovered[0]),
        replication_receipt(&profile, 1, SEED_1, EXEC_1, truth[1], recovered[1]),
    ];

    let promoted = promote_scientific_recovery(
        HEAD,
        HEAD,
        &truth,
        &recovered,
        &profile,
        &exact_head,
        &receipts,
    )
    .expect("each receipt seed is the exact manifest entry at its repetition index");
    assert_eq!(
        promoted.claim().authority(),
        ClaimAuthority::ScientificallySupported
    );

    let substituted = vec![
        replication_receipt(&profile, 0, SEED_0, EXEC_0, truth[0], recovered[0]),
        replication_receipt(
            &profile,
            1,
            SEED_OUTSIDE,
            EXEC_1,
            truth[1],
            recovered[1],
        ),
    ];
    assert_eq!(
        promote_scientific_recovery(
            HEAD,
            HEAD,
            &truth,
            &recovered,
            &profile,
            &exact_head,
            &substituted,
        ),
        Err(ValidationError::InvalidInput)
    );
}

#[test]
fn manifest_identity_is_ordered_unique_and_canonical() {
    let manifest = seed_manifest();
    assert_eq!(manifest.len(), 2);
    assert!(!manifest.is_empty());
    assert_eq!(manifest.seed_state_sha256(0), Some(SEED_0));
    assert_eq!(manifest.seed_state_sha256(1), Some(SEED_1));
    assert_eq!(manifest.seed_state_sha256(2), None);
    assert_eq!(manifest.sha256().len(), 64);

    let reversed = ScientificRecoverySeedManifestV1::new(&[SEED_1, SEED_0])
        .expect("same members in a different planned order are a different manifest");
    assert_ne!(manifest.sha256(), reversed.sha256());

    assert_eq!(
        ScientificRecoverySeedManifestV1::new(&[SEED_0, SEED_0]),
        Err(ValidationError::InvalidInput)
    );
    assert_eq!(
        ScientificRecoverySeedManifestV1::new(&["short", SEED_1]),
        Err(ValidationError::InvalidInput)
    );
    assert_eq!(
        ScientificRecoverySeedManifestV1::new(&[SEED_0]),
        Err(ValidationError::InvalidInput)
    );
}
