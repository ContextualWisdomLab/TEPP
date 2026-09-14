//! Regression contract for immutable source-snapshot authority in Rubin projection.
//!
//! A projection receipt claims to bind an immutable source snapshot. Mutable
//! branch and pull-request locators therefore cannot be admitted as snapshot
//! identities even when the expected snapshot argument repeats the same alias.
//! Snapshot identity alone is not a content commitment, so the receipt also
//! carries the canonical source snapshot SHA-256. Concrete draw bytes have a
//! separate digest and do not weaken snapshot authority.

use analysis_engine::{
    AnalysisEngineError, RUBIN_LOADING_MODEL_CONTRACT_VERSION,
    RubinProjectionActivationReceiptV1,
};
use temporal_core::{AvailableTime, KnowledgeCutoff};

const EVIDENCE_DIGEST: &str =
    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const SNAPSHOT_DIGEST: &str =
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const DRAW_PAYLOAD_DIGEST: &str =
    "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

fn receipt_with_snapshot(
    snapshot_id: &str,
) -> Result<RubinProjectionActivationReceiptV1, AnalysisEngineError> {
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
        snapshot_id,
        SNAPSHOT_DIGEST,
        DRAW_PAYLOAD_DIGEST,
        KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff"),
        "rubin-gaussian-single-level-candidate-v1",
    )
}

#[test]
fn mutable_snapshot_aliases_fail_closed_at_receipt_construction() {
    for mutable_snapshot in [
        "main",
        "master",
        "latest",
        "refs/heads/main",
        "PR-504",
        "pull/504",
        "#504",
        "github.com/ContextualWisdomLab/TEPP/tree/main",
    ] {
        assert_eq!(
            receipt_with_snapshot(mutable_snapshot),
            Err(AnalysisEngineError::InvalidEvidence),
            "mutable source snapshot must fail closed: {mutable_snapshot}"
        );
    }
}

#[test]
fn immutable_snapshot_identifier_and_digest_remain_admissible() {
    let receipt = receipt_with_snapshot("snapshot-rubin-activation-v1")
        .expect("immutable snapshot identifier");
    assert_eq!(receipt.source_snapshot_id(), "snapshot-rubin-activation-v1");
    assert_eq!(receipt.source_snapshot_sha256(), SNAPSHOT_DIGEST);
    assert_eq!(receipt.complete_data_draws_sha256(), DRAW_PAYLOAD_DIGEST);
}

#[test]
fn malformed_snapshot_digest_fails_closed_at_receipt_construction() {
    assert_eq!(
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
            "snapshot-rubin-activation-v1",
            "not-a-digest",
            DRAW_PAYLOAD_DIGEST,
            KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff"),
            "rubin-gaussian-single-level-candidate-v1",
        ),
        Err(AnalysisEngineError::InvalidEvidence)
    );
}
