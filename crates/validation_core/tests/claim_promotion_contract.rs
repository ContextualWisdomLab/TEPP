//! ADR 0014 claim authorities cannot be promoted from unusable evidence.

use validation_core::{
    ClaimAuthority, ClaimEvidence, ClaimEvidenceKind, PromotionRequest,
    ScientificRecoveryExactHeadReceiptStatusV1, ScientificRecoveryExactHeadReceiptV1,
    ScientificRecoveryExecutionLedgerEntryV1, ScientificRecoveryFailurePolicyV1,
    ScientificRecoveryProfileChronologyV1, ScientificRecoveryProfileRegistrationStatusV1,
    ScientificRecoveryProfileV1, ScientificRecoveryReplicationReceiptV1,
    ScientificRecoverySeedManifestV1, ValidationError, parse_commit_head, promote_claim,
    promote_scientific_recovery, rmse_standard_error, root_mean_square_error,
    scientific_recovery_replication_payload_sha256,
};

const PROTECTED_HEAD: &str = "b2a3f879ca61daefa534f122647074666d5604bc";
const OTHER_HEAD: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const RECEIPT: &str = "5555555555555555555555555555555555555555555555555555555555555555";
const DGP: &str = "1111111111111111111111111111111111111111111111111111111111111111";
const SEEDS: &str = "2222222222222222222222222222222222222222222222222222222222222222";
const ESTIMAND: &str = "3333333333333333333333333333333333333333333333333333333333333333";
const STATE: &str = "4444444444444444444444444444444444444444444444444444444444444444";
const LEDGER: &str = "6666666666666666666666666666666666666666666666666666666666666666";
const REGISTRATION_ENTRY: &str = "7777777777777777777777777777777777777777777777777777777777777777";

fn implemented_main_evidence() -> [ClaimEvidence; 1] {
    [ClaimEvidence::new(ClaimEvidenceKind::ExactHeadTests, true)]
}

fn scientifically_supported_evidence() -> [ClaimEvidence; 2] {
    [
        ClaimEvidence::new(ClaimEvidenceKind::ExactHeadTests, true),
        ClaimEvidence::new(ClaimEvidenceKind::ScientificRecovery, true),
    ]
}

fn released_evidence() -> [ClaimEvidence; 6] {
    [
        ClaimEvidence::new(ClaimEvidenceKind::ExactHeadTests, true),
        ClaimEvidence::new(ClaimEvidenceKind::ScientificRecovery, true),
        ClaimEvidence::new(ClaimEvidenceKind::SecuritySupplyChain, true),
        ClaimEvidence::new(ClaimEvidenceKind::QualifyingReview, true),
        ClaimEvidence::new(ClaimEvidenceKind::OperationalReadiness, true),
        ClaimEvidence::new(ClaimEvidenceKind::SbomProvenance, true),
    ]
}

fn request<'evidence>(
    target: ClaimAuthority,
    candidate_head: &str,
    evidence: &'evidence [ClaimEvidence],
) -> PromotionRequest<'evidence> {
    PromotionRequest::new(target, candidate_head, PROTECTED_HEAD, evidence).expect("request")
}

fn as_slices<const N: usize, const M: usize>(rows: &[[f64; M]; N]) -> Vec<&[f64]> {
    rows.iter().map(|row| row.as_slice()).collect()
}

fn seed_state(index: usize) -> String {
    format!("{:064x}", index + 1)
}

fn seed_manifest(planned_replications: usize) -> ScientificRecoverySeedManifestV1 {
    let states: Vec<String> = (0..planned_replications).map(seed_state).collect();
    let refs: Vec<&str> = states.iter().map(String::as_str).collect();
    ScientificRecoverySeedManifestV1::new(&refs).expect("valid planned seed manifest")
}

fn recovery_profile(
    planned_replications: usize,
    max_rmse: f64,
    se_multiplier: f64,
) -> ScientificRecoveryProfileV1 {
    let manifest = seed_manifest(planned_replications);
    ScientificRecoveryProfileV1::new(
        planned_replications,
        max_rmse,
        se_multiplier,
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
        PROTECTED_HEAD,
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
    .expect("approved profile registration predates execution entries")
}

#[test]
fn commit_heads_are_forty_hex_bytes() {
    let parsed = parse_commit_head(PROTECTED_HEAD).expect("head");
    assert_eq!(parsed.len(), 20);
    assert_eq!(parse_commit_head(""), Err(ValidationError::InvalidInput));
    assert_eq!(
        parse_commit_head("not-a-commit-sha"),
        Err(ValidationError::InvalidInput)
    );
    assert_eq!(
        parse_commit_head("B2A3F879CA61DAEFA534F122647074666D5604BC"),
        parse_commit_head(PROTECTED_HEAD)
    );
    assert_eq!(
        parse_commit_head("b2a3f879ca61daefa534f122647074666d5604bg"),
        Err(ValidationError::InvalidInput)
    );
}

#[test]
fn decision_accepted_does_not_require_implementation_evidence() {
    let promoted =
        promote_claim(&request(ClaimAuthority::DecisionAccepted, OTHER_HEAD, &[])).expect("design");
    assert_eq!(promoted.authority(), ClaimAuthority::DecisionAccepted);
    assert_eq!(
        promoted.bound_head(),
        parse_commit_head(OTHER_HEAD).unwrap()
    );
    assert_eq!(
        ClaimAuthority::DecisionAccepted.wire_name(),
        "decision_accepted"
    );
}

#[test]
fn implemented_main_requires_exact_protected_head_and_tests() {
    let promoted = promote_claim(&request(
        ClaimAuthority::ImplementedMain,
        PROTECTED_HEAD,
        &implemented_main_evidence(),
    ))
    .expect("implemented");
    assert_eq!(promoted.authority(), ClaimAuthority::ImplementedMain);
    assert_eq!(
        promoted.bound_head(),
        parse_commit_head(PROTECTED_HEAD).unwrap()
    );

    assert_eq!(
        promote_claim(&request(
            ClaimAuthority::ImplementedMain,
            OTHER_HEAD,
            &implemented_main_evidence(),
        )),
        Err(ValidationError::ClaimHeadMismatch)
    );
    assert_eq!(
        promote_claim(&request(
            ClaimAuthority::ImplementedMain,
            PROTECTED_HEAD,
            &[]
        )),
        Err(ValidationError::ClaimEvidenceMissing)
    );
    assert_eq!(
        promote_claim(&request(
            ClaimAuthority::ImplementedMain,
            PROTECTED_HEAD,
            &[ClaimEvidence::new(ClaimEvidenceKind::ExactHeadTests, false)],
        )),
        Err(ValidationError::ClaimEvidenceFailed)
    );
}

#[test]
fn unusable_evidence_kinds_never_promote() {
    let cases = [
        (
            ClaimEvidenceKind::QueuedCheck,
            ValidationError::ClaimQueuedEvidence,
        ),
        (
            ClaimEvidenceKind::PredecessorHead,
            ValidationError::ClaimPredecessorHead,
        ),
        (
            ClaimEvidenceKind::LlmJudgment,
            ValidationError::ClaimLlmJudgment,
        ),
        (
            ClaimEvidenceKind::SkippedRequired,
            ValidationError::ClaimSkippedRequired,
        ),
    ];
    for (kind, expected) in cases {
        assert!(!kind.is_promotable());
        assert!(!kind.wire_name().is_empty());
        let evidence = [
            ClaimEvidence::new(ClaimEvidenceKind::ExactHeadTests, true),
            ClaimEvidence::new(kind, true),
        ];
        assert_eq!(
            promote_claim(&request(
                ClaimAuthority::ImplementedMain,
                PROTECTED_HEAD,
                &evidence,
            )),
            Err(expected)
        );
    }
}

#[test]
fn scientific_and_release_authorities_require_their_gates() {
    assert_eq!(
        promote_claim(&request(
            ClaimAuthority::ScientificallySupported,
            PROTECTED_HEAD,
            &implemented_main_evidence(),
        )),
        Err(ValidationError::ClaimEvidenceMissing)
    );
    let scientific = promote_claim(&request(
        ClaimAuthority::ScientificallySupported,
        PROTECTED_HEAD,
        &scientifically_supported_evidence(),
    ))
    .expect("scientific");
    assert_eq!(
        scientific.authority(),
        ClaimAuthority::ScientificallySupported
    );

    assert_eq!(
        promote_claim(&request(
            ClaimAuthority::Released,
            PROTECTED_HEAD,
            &scientifically_supported_evidence(),
        )),
        Err(ValidationError::ClaimEvidenceMissing)
    );
    let released = promote_claim(&request(
        ClaimAuthority::Released,
        PROTECTED_HEAD,
        &released_evidence(),
    ))
    .expect("released");
    assert_eq!(released.authority(), ClaimAuthority::Released);
    assert_eq!(ClaimAuthority::Released.wire_name(), "released");
    assert_eq!(
        ClaimEvidenceKind::ScientificRecovery.wire_name(),
        "scientific_recovery"
    );
}

#[test]
fn scientific_recovery_requires_explicit_accuracy_target_plus_uncertainty() {
    let truth_flat = [0.70, 0.55, 0.40, -0.20, 0.85];
    let recovered_flat = [0.72, 0.53, 0.41, -0.18, 0.84];
    let rmse = root_mean_square_error(&truth_flat, &recovered_flat).expect("rmse");
    let rmse_se = rmse_standard_error(&truth_flat, &recovered_flat).expect("se");
    let max_rmse = rmse + 3.0 * rmse_se + 0.001;
    let exact_head = exact_head_receipt();

    let truth_rows = [[0.70], [0.55], [0.40], [-0.20], [0.85]];
    let recovered_rows = [[0.72], [0.53], [0.41], [-0.18], [0.84]];
    let truth = as_slices(&truth_rows);
    let recovered = as_slices(&recovered_rows);
    let profile = recovery_profile(5, max_rmse, 3.0);
    let receipts = replication_receipts(&profile, &truth, &recovered);
    let profile_chronology = chronology(&profile, &receipts);
    let promoted = promote_scientific_recovery(
        PROTECTED_HEAD,
        PROTECTED_HEAD,
        &truth,
        &recovered,
        &profile,
        &exact_head,
        &receipts,
        &profile_chronology,
    )
    .expect("promote");
    assert_eq!(
        promoted.claim().authority(),
        ClaimAuthority::ScientificallySupported
    );
    assert_eq!(promoted.profile_sha256(), profile.sha256());
    assert_eq!(promoted.exact_head_receipt_sha256(), RECEIPT);
    assert!(rmse.is_finite());
    assert!(rmse_se.is_finite() && rmse_se > 0.0);

    let exact_profile = recovery_profile(5, 0.001, 3.0);
    let exact_receipts = replication_receipts(&exact_profile, &truth, &truth);
    let exact_chronology = chronology(&exact_profile, &exact_receipts);
    promote_scientific_recovery(
        PROTECTED_HEAD,
        PROTECTED_HEAD,
        &truth,
        &truth,
        &exact_profile,
        &exact_head,
        &exact_receipts,
        &exact_chronology,
    )
    .expect("exact");

    let biased_rows = [[1.70], [1.55], [1.40], [0.80], [1.85]];
    let biased = as_slices(&biased_rows);
    let biased_profile = recovery_profile(5, 0.10, 3.0);
    let biased_receipts = replication_receipts(&biased_profile, &truth, &biased);
    let biased_chronology = chronology(&biased_profile, &biased_receipts);
    assert_eq!(
        promote_scientific_recovery(
            PROTECTED_HEAD,
            PROTECTED_HEAD,
            &truth,
            &biased,
            &biased_profile,
            &exact_head,
            &biased_receipts,
            &biased_chronology,
        ),
        Err(ValidationError::ClaimRecoveryRejected)
    );
    assert_eq!(
        promote_scientific_recovery(
            OTHER_HEAD,
            PROTECTED_HEAD,
            &truth,
            &recovered,
            &profile,
            &exact_head,
            &receipts,
            &profile_chronology,
        ),
        Err(ValidationError::ClaimHeadMismatch)
    );
    let empty: [&[f64]; 0] = [];
    let two_rep_profile = recovery_profile(2, 0.10, 3.0);
    let dummy_truth_rows = [[0.0], [1.0]];
    let dummy_truth = as_slices(&dummy_truth_rows);
    let dummy_receipts = replication_receipts(&two_rep_profile, &dummy_truth, &dummy_truth);
    let dummy_chronology = chronology(&two_rep_profile, &dummy_receipts);
    assert_eq!(
        promote_scientific_recovery(
            PROTECTED_HEAD,
            PROTECTED_HEAD,
            &empty,
            &empty,
            &two_rep_profile,
            &exact_head,
            &dummy_receipts,
            &dummy_chronology,
        ),
        Err(ValidationError::InvalidInput)
    );
    assert_eq!(
        ScientificRecoveryProfileV1::new(
            5,
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
}

#[test]
fn promoted_claim_and_request_reject_invalid_heads() {
    assert_eq!(
        PromotionRequest::new(ClaimAuthority::DecisionAccepted, "bad", PROTECTED_HEAD, &[],).err(),
        Some(ValidationError::InvalidInput)
    );
    // Promoted claims can no longer be minted directly: `PromotedClaim::new`
    // is crate-internal, so the only external path to a promoted claim is the
    // validated `promote_claim` / `promote_scientific_recovery` flow.
    let promoted = promote_claim(&request(
        ClaimAuthority::DecisionAccepted,
        PROTECTED_HEAD,
        &[],
    ))
    .expect("design");
    assert_eq!(promoted.authority(), ClaimAuthority::DecisionAccepted);
    assert_eq!(
        promoted.bound_head(),
        parse_commit_head(PROTECTED_HEAD).unwrap()
    );
}

#[test]
fn required_evidence_refuses_co_present_failing_items() {
    let mixed_passing_first = [
        ClaimEvidence::new(ClaimEvidenceKind::ExactHeadTests, true),
        ClaimEvidence::new(ClaimEvidenceKind::ExactHeadTests, false),
    ];
    assert_eq!(
        promote_claim(&request(
            ClaimAuthority::ImplementedMain,
            PROTECTED_HEAD,
            &mixed_passing_first,
        )),
        Err(ValidationError::ClaimEvidenceFailed)
    );
    let mixed_failing_first = [
        ClaimEvidence::new(ClaimEvidenceKind::ExactHeadTests, false),
        ClaimEvidence::new(ClaimEvidenceKind::ExactHeadTests, true),
    ];
    assert_eq!(
        promote_claim(&request(
            ClaimAuthority::ImplementedMain,
            PROTECTED_HEAD,
            &mixed_failing_first,
        )),
        Err(ValidationError::ClaimEvidenceFailed)
    );
}
