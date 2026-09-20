//! Scientific recovery authority binds every independent repetition to its planned RNG state and execution artifact.

use validation_core::{
    ClaimAuthority, ScientificRecoveryExactHeadReceiptStatusV1,
    ScientificRecoveryExactHeadReceiptV1, ScientificRecoveryFailurePolicyV1,
    ScientificRecoveryProfileV1, ScientificRecoveryReplicationReceiptV1, ValidationError,
    promote_scientific_recovery, scientific_recovery_replication_payload_sha256,
};

const HEAD: &str = "b2a3f879ca61daefa534f122647074666d5604bc";
const TEST_ARTIFACT: &str =
    "5555555555555555555555555555555555555555555555555555555555555555";
const DGP: &str = "1111111111111111111111111111111111111111111111111111111111111111";
const SEED_MANIFEST: &str =
    "2222222222222222222222222222222222222222222222222222222222222222";
const ESTIMAND: &str =
    "3333333333333333333333333333333333333333333333333333333333333333";
const STATE: &str = "4444444444444444444444444444444444444444444444444444444444444444";
const SEED_0: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const SEED_1: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const EXEC_0: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
const EXEC_1: &str = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
const EXEC_0_ALT: &str =
    "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";

fn profile() -> ScientificRecoveryProfileV1 {
    ScientificRecoveryProfileV1::new(
        2,
        0.08,
        3.0,
        DGP,
        SEED_MANIFEST,
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
fn promoted_authority_retains_ordered_replication_execution_provenance() {
    let profile = profile();
    let truth_rows = [[0.0, 1.0], [2.0, 3.0]];
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
    .expect("planned seed-state and execution receipts match exact recovery payloads");

    assert_eq!(
        promoted.claim().authority(),
        ClaimAuthority::ScientificallySupported
    );
    assert_eq!(promoted.replication_provenance_sha256().len(), 64);

    let repeated = promote_scientific_recovery(
        HEAD,
        HEAD,
        &truth,
        &recovered,
        &profile,
        &exact_head,
        &receipts,
    )
    .expect("same represented execution provenance is deterministic");
    assert_eq!(
        promoted.replication_provenance_sha256(),
        repeated.replication_provenance_sha256()
    );

    let changed_execution = vec![
        replication_receipt(&profile, 0, SEED_0, EXEC_0_ALT, truth[0], recovered[0]),
        replication_receipt(&profile, 1, SEED_1, EXEC_1, truth[1], recovered[1]),
    ];
    let changed = promote_scientific_recovery(
        HEAD,
        HEAD,
        &truth,
        &recovered,
        &profile,
        &exact_head,
        &changed_execution,
    )
    .expect("different immutable execution artifact may contain the same accepted payload");
    assert_ne!(
        promoted.replication_provenance_sha256(),
        changed.replication_provenance_sha256()
    );
}

#[test]
fn replication_receipts_fail_closed_on_order_payload_or_seed_reuse() {
    let profile = profile();
    let truth_rows = [[0.0], [1.0]];
    let recovered_rows = truth_rows;
    let truth: Vec<&[f64]> = truth_rows.iter().map(|row| row.as_slice()).collect();
    let recovered: Vec<&[f64]> = recovered_rows.iter().map(|row| row.as_slice()).collect();
    let exact_head = exact_head_receipt();

    let mut reversed = vec![
        replication_receipt(&profile, 0, SEED_0, EXEC_0, truth[0], recovered[0]),
        replication_receipt(&profile, 1, SEED_1, EXEC_1, truth[1], recovered[1]),
    ];
    reversed.swap(0, 1);
    assert_eq!(
        promote_scientific_recovery(
            HEAD,
            HEAD,
            &truth,
            &recovered,
            &profile,
            &exact_head,
            &reversed,
        ),
        Err(ValidationError::InvalidInput)
    );

    let wrong_payload = vec![
        ScientificRecoveryReplicationReceiptV1::new(
            0,
            &profile.sha256(),
            profile.seed_manifest_sha256(),
            SEED_0,
            EXEC_0,
            "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
        )
        .expect("canonical but incorrect payload digest"),
        replication_receipt(&profile, 1, SEED_1, EXEC_1, truth[1], recovered[1]),
    ];
    assert_eq!(
        promote_scientific_recovery(
            HEAD,
            HEAD,
            &truth,
            &recovered,
            &profile,
            &exact_head,
            &wrong_payload,
        ),
        Err(ValidationError::InvalidInput)
    );

    let duplicate_seed_state = vec![
        replication_receipt(&profile, 0, SEED_0, EXEC_0, truth[0], recovered[0]),
        replication_receipt(&profile, 1, SEED_0, EXEC_1, truth[1], recovered[1]),
    ];
    assert_eq!(
        promote_scientific_recovery(
            HEAD,
            HEAD,
            &truth,
            &recovered,
            &profile,
            &exact_head,
            &duplicate_seed_state,
        ),
        Err(ValidationError::InvalidInput)
    );
}
