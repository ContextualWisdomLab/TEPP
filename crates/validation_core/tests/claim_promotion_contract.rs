//! ADR 0014 claim authorities cannot be promoted from unusable evidence.

use validation_core::{
    ClaimAuthority, ClaimEvidence, ClaimEvidenceKind, PromotedClaim, PromotionRequest,
    ValidationError, parse_commit_head, promote_claim, promote_scientific_recovery,
    rmse_standard_error, root_mean_square_error,
};

const PROTECTED_HEAD: &str = "b2a3f879ca61daefa534f122647074666d5604bc";
const OTHER_HEAD: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

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
        Err(ValidationError::ClaimEvidenceMissing)
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
fn scientific_recovery_uses_computed_rmse_not_hardcoded_thresholds() {
    let truth = [0.70, 0.55, 0.40, -0.20, 0.85];
    let recovered = [0.72, 0.53, 0.41, -0.18, 0.84];
    let rmse = root_mean_square_error(&truth, &recovered).expect("rmse");
    let rmse_se = rmse_standard_error(&truth, &recovered).expect("se");
    let computed_k = (rmse / rmse_se) + 1.0;
    let promoted = promote_scientific_recovery(
        PROTECTED_HEAD,
        PROTECTED_HEAD,
        &truth,
        &recovered,
        computed_k,
    )
    .expect("promote");
    assert_eq!(
        promoted.authority(),
        ClaimAuthority::ScientificallySupported
    );
    assert!(rmse.is_finite());
    assert!(rmse_se.is_finite() && rmse_se > 0.0);
    promote_scientific_recovery(PROTECTED_HEAD, PROTECTED_HEAD, &truth, &truth, 3.0)
        .expect("exact");

    let biased = [1.70, 1.55, 1.40, 0.80, 1.85];
    assert_eq!(
        promote_scientific_recovery(PROTECTED_HEAD, PROTECTED_HEAD, &truth, &biased, 3.0),
        Err(ValidationError::ClaimRecoveryRejected)
    );
    assert_eq!(
        promote_scientific_recovery(OTHER_HEAD, PROTECTED_HEAD, &truth, &recovered, 3.0),
        Err(ValidationError::ClaimHeadMismatch)
    );
    assert_eq!(
        promote_scientific_recovery(PROTECTED_HEAD, PROTECTED_HEAD, &[], &[], 3.0),
        Err(ValidationError::InvalidInput)
    );
}

#[test]
fn promoted_claim_and_request_reject_invalid_heads() {
    assert_eq!(
        PromotionRequest::new(ClaimAuthority::DecisionAccepted, "bad", PROTECTED_HEAD, &[],).err(),
        Some(ValidationError::InvalidInput)
    );
    let _ = PromotedClaim::new(
        ClaimAuthority::DecisionAccepted,
        parse_commit_head(PROTECTED_HEAD).unwrap(),
    );
}
