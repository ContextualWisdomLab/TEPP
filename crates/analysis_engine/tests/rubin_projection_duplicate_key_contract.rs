use analysis_engine::{
    AnalysisEngineError, RUBIN_LOADING_MODEL_CONTRACT_VERSION,
    RubinProjectionActivationReceiptV1,
};
use temporal_core::{AvailableTime, KnowledgeCutoff};

const SNAPSHOT_DIGEST: &str =
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const DRAW_PAYLOAD_DIGEST: &str =
    "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const OTHER_DRAW_PAYLOAD_DIGEST: &str =
    "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
const EVIDENCE_DIGEST: &str =
    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const OTHER_EVIDENCE_DIGEST: &str =
    "fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210";

fn receipt() -> RubinProjectionActivationReceiptV1 {
    RubinProjectionActivationReceiptV1::new(
        ("gaussian_complete_data_draws", "candidate-v1"),
        (
            "rubin_loading_uncertainty",
            RUBIN_LOADING_MODEL_CONTRACT_VERSION,
        ),
        (
            "validation-evidence-rubin-candidate-v1",
            EVIDENCE_DIGEST,
            AvailableTime::parse_rfc3339("2026-07-31T23:59:59Z").expect("availability"),
        ),
        "snapshot-rubin-activation",
        SNAPSHOT_DIGEST,
        DRAW_PAYLOAD_DIGEST,
        KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff"),
        "rubin-gaussian-single-level-candidate-v1",
    )
    .expect("receipt")
}

fn inject_duplicate_member(canonical: &str, field: &str, first: &str, second: &str) -> String {
    let unique_member = format!("\"{field}\":\"{second}\"");
    let duplicate_members = format!("\"{field}\":\"{first}\",\"{field}\":\"{second}\"");
    let ambiguous = canonical.replacen(&unique_member, &duplicate_members, 1);
    assert_ne!(ambiguous, canonical, "fixture must inject the duplicate member");
    ambiguous
}

#[test]
fn duplicate_draw_digest_member_is_rejected_before_authority_interpretation() {
    let canonical = receipt().to_json().expect("canonical receipt");
    let ambiguous = inject_duplicate_member(
        &canonical,
        "complete_data_draws_sha256",
        OTHER_DRAW_PAYLOAD_DIGEST,
        DRAW_PAYLOAD_DIGEST,
    );

    assert_eq!(
        RubinProjectionActivationReceiptV1::from_json(&ambiguous),
        Err(AnalysisEngineError::InvalidEvidence)
    );
}

#[test]
fn duplicate_nested_authority_member_is_rejected_before_inner_receipt_reparse() {
    let canonical = receipt().to_json().expect("canonical receipt");
    let ambiguous = inject_duplicate_member(
        &canonical,
        "validation_evidence_sha256",
        OTHER_EVIDENCE_DIGEST,
        EVIDENCE_DIGEST,
    );

    assert_eq!(
        RubinProjectionActivationReceiptV1::from_json(&ambiguous),
        Err(AnalysisEngineError::InvalidEvidence)
    );
}
