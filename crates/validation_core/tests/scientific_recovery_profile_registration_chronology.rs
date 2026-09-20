//! Scientific recovery authority requires owner-ledger profile approval before execution entries.

use validation_core::{
    ClaimAuthority, ScientificRecoveryExactHeadReceiptStatusV1,
    ScientificRecoveryExactHeadReceiptV1, ScientificRecoveryExecutionLedgerEntryV1,
    ScientificRecoveryFailurePolicyV1, ScientificRecoveryProfileChronologyV1,
    ScientificRecoveryProfileRegistrationStatusV1, ScientificRecoveryProfileV1,
    ScientificRecoveryReplicationReceiptV1, ScientificRecoverySeedManifestV1, ValidationError,
    promote_scientific_recovery, scientific_recovery_replication_payload_sha256,
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
const EXEC_0: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
const EXEC_1: &str = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
const LEDGER: &str = "6666666666666666666666666666666666666666666666666666666666666666";
const REGISTRATION_ENTRY: &str =
    "7777777777777777777777777777777777777777777777777777777777777777";
const EXEC_ENTRY_0: &str =
    "8888888888888888888888888888888888888888888888888888888888888888";
const EXEC_ENTRY_1: &str =
    "9999999999999999999999999999999999999999999999999999999999999999";

fn profile() -> ScientificRecoveryProfileV1 {
    let manifest = ScientificRecoverySeedManifestV1::new(&[SEED_0, SEED_1])
        .expect("valid planned seed manifest");
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

fn rows() -> (Vec<&'static [f64]>, Vec<&'static [f64]>) {
    static TRUTH: [[f64; 1]; 2] = [[0.0], [1.0]];
    static RECOVERED: [[f64; 1]; 2] = [[0.0], [1.0]];
    (
        TRUTH.iter().map(|row| row.as_slice()).collect(),
        RECOVERED.iter().map(|row| row.as_slice()).collect(),
    )
}

fn replication_receipts(
    profile: &ScientificRecoveryProfileV1,
    truth: &[&[f64]],
    recovered: &[&[f64]],
) -> Vec<ScientificRecoveryReplicationReceiptV1> {
    [SEED_0, SEED_1]
        .into_iter()
        .zip([EXEC_0, EXEC_1])
        .enumerate()
        .map(|(index, (seed, execution))| {
            let payload = scientific_recovery_replication_payload_sha256(
                truth[index],
                recovered[index],
            )
            .expect("valid represented payload");
            ScientificRecoveryReplicationReceiptV1::new(
                index,
                &profile.sha256(),
                profile.seed_manifest_sha256(),
                seed,
                execution,
                &payload,
            )
            .expect("valid replication receipt")
        })
        .collect()
}

fn chronology(
    profile: &ScientificRecoveryProfileV1,
    status: ScientificRecoveryProfileRegistrationStatusV1,
) -> ScientificRecoveryProfileChronologyV1 {
    let executions = [
        ScientificRecoveryExecutionLedgerEntryV1::new(EXEC_0, EXEC_ENTRY_0, 11)
            .expect("execution entry 0"),
        ScientificRecoveryExecutionLedgerEntryV1::new(EXEC_1, EXEC_ENTRY_1, 12)
            .expect("execution entry 1"),
    ];
    ScientificRecoveryProfileChronologyV1::new(
        profile,
        LEDGER,
        REGISTRATION_ENTRY,
        10,
        status,
        &executions,
    )
    .expect("valid owner-ledger chronology")
}

#[test]
fn approved_registration_before_all_execution_entries_is_retained_by_authority() {
    let profile = profile();
    let (truth, recovered) = rows();
    let receipts = replication_receipts(&profile, &truth, &recovered);
    let chronology = chronology(
        &profile,
        ScientificRecoveryProfileRegistrationStatusV1::Approved,
    );

    let promoted = promote_scientific_recovery(
        HEAD,
        HEAD,
        &truth,
        &recovered,
        &profile,
        &exact_head_receipt(),
        &receipts,
        &chronology,
    )
    .expect("approved owner-ledger registration predates every execution entry");

    assert_eq!(
        promoted.claim().authority(),
        ClaimAuthority::ScientificallySupported
    );
    assert_eq!(promoted.profile_chronology_sha256(), chronology.sha256());
}

#[test]
fn pending_or_rejected_registration_cannot_promote_scientific_authority() {
    let profile = profile();
    let (truth, recovered) = rows();
    let receipts = replication_receipts(&profile, &truth, &recovered);

    for (status, expected) in [
        (
            ScientificRecoveryProfileRegistrationStatusV1::Pending,
            ValidationError::ClaimQueuedEvidence,
        ),
        (
            ScientificRecoveryProfileRegistrationStatusV1::Rejected,
            ValidationError::ClaimEvidenceFailed,
        ),
    ] {
        let chronology = chronology(&profile, status);
        assert_eq!(
            promote_scientific_recovery(
                HEAD,
                HEAD,
                &truth,
                &recovered,
                &profile,
                &exact_head_receipt(),
                &receipts,
                &chronology,
            ),
            Err(expected)
        );
    }
}

#[test]
fn registration_must_precede_every_unique_execution_ledger_position() {
    let profile = profile();
    let not_after_registration = [
        ScientificRecoveryExecutionLedgerEntryV1::new(EXEC_0, EXEC_ENTRY_0, 10)
            .expect("canonical execution entry"),
        ScientificRecoveryExecutionLedgerEntryV1::new(EXEC_1, EXEC_ENTRY_1, 12)
            .expect("canonical execution entry"),
    ];
    assert_eq!(
        ScientificRecoveryProfileChronologyV1::new(
            &profile,
            LEDGER,
            REGISTRATION_ENTRY,
            10,
            ScientificRecoveryProfileRegistrationStatusV1::Approved,
            &not_after_registration,
        ),
        Err(ValidationError::InvalidInput)
    );

    let duplicate_sequence = [
        ScientificRecoveryExecutionLedgerEntryV1::new(EXEC_0, EXEC_ENTRY_0, 11)
            .expect("canonical execution entry"),
        ScientificRecoveryExecutionLedgerEntryV1::new(EXEC_1, EXEC_ENTRY_1, 11)
            .expect("canonical execution entry"),
    ];
    assert_eq!(
        ScientificRecoveryProfileChronologyV1::new(
            &profile,
            LEDGER,
            REGISTRATION_ENTRY,
            10,
            ScientificRecoveryProfileRegistrationStatusV1::Approved,
            &duplicate_sequence,
        ),
        Err(ValidationError::InvalidInput)
    );
}
