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

#[test]
fn duplicate_draw_digest_member_is_rejected_before_authority_interpretation() {
    let canonical = receipt().to_json().expect("canonical receipt");
    let unique_member = format!(
        "\"complete_data_draws_sha256\":\"{DRAW_PAYLOAD_DIGEST}\""
    );
    let duplicate_members = format!(
        "\"complete_data_draws_sha256\":\"{OTHER_DRAW_PAYLOAD_DIGEST}\",\"complete_data_draws_sha256\":\"{DRAW_PAYLOAD_DIGEST}\""
    );
    let ambiguous = canonical.replacen(&unique_member, &duplicate_members, 1);

    assert_ne!(ambiguous, canonical, "fixture must inject the duplicate member");
    assert_eq!(
        RubinProjectionActivationReceiptV1::from_json(&ambiguous),
        Err(AnalysisEngineError::InvalidEvidence)
    );
}
