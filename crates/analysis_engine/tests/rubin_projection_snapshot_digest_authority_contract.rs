//! Contract for content-addressed Rubin projection snapshot authority.

use analysis_engine::{
    AnalysisEngineError, RUBIN_LOADING_ANALYSIS_CONTRACT_ID, RUBIN_LOADING_MODEL_CONTRACT_VERSION,
    RubinProjectionActivationDecision, RubinProjectionActivationReceiptV1,
    decide_rubin_projection_activation,
};
use psychometric_core::IndicatorKind;
use temporal_core::{AvailableTime, KnowledgeCutoff};

const SNAPSHOT_ID: &str = "snapshot-rubin-activation";
const SNAPSHOT_DIGEST: &str =
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const OTHER_SNAPSHOT_DIGEST: &str =
    "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const EVIDENCE_DIGEST: &str =
    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const OBSERVATION_COUNT: u64 = 48;
const DRAW_COUNT: u64 = 8;

fn cutoff() -> KnowledgeCutoff {
    KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff")
}

fn receipt() -> RubinProjectionActivationReceiptV1 {
    RubinProjectionActivationReceiptV1::new(
        ("gaussian_complete_data_draws", "approved-v1"),
        (
            RUBIN_LOADING_ANALYSIS_CONTRACT_ID,
            RUBIN_LOADING_MODEL_CONTRACT_VERSION,
        ),
        (
            "validation-evidence-rubin-approved-v1",
            EVIDENCE_DIGEST,
            AvailableTime::parse_rfc3339("2026-07-31T23:59:59Z").expect("availability"),
        ),
        SNAPSHOT_ID,
        SNAPSHOT_DIGEST,
        cutoff(),
        "rubin-gaussian-single-level-approved-v1",
    )
    .expect("receipt")
}

#[test]
fn receipt_binds_canonical_source_snapshot_digest_into_wire_and_digest() {
    let receipt = receipt();
    assert_eq!(receipt.source_snapshot_id(), SNAPSHOT_ID);
    assert_eq!(receipt.source_snapshot_sha256(), SNAPSHOT_DIGEST);

    let json = receipt.to_json().expect("json");
    assert!(json.contains(SNAPSHOT_DIGEST));
    assert_eq!(
        RubinProjectionActivationReceiptV1::from_json(&json),
        Ok(receipt)
    );
}

#[test]
fn receipt_refuses_noncanonical_source_snapshot_digest() {
    assert_eq!(
        RubinProjectionActivationReceiptV1::new(
            ("gaussian_complete_data_draws", "approved-v1"),
            (
                RUBIN_LOADING_ANALYSIS_CONTRACT_ID,
                RUBIN_LOADING_MODEL_CONTRACT_VERSION,
            ),
            (
                "validation-evidence-rubin-approved-v1",
                EVIDENCE_DIGEST,
                AvailableTime::parse_rfc3339("2026-07-31T23:59:59Z").expect("availability"),
            ),
            SNAPSHOT_ID,
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            cutoff(),
            "rubin-gaussian-single-level-approved-v1",
        ),
        Err(AnalysisEngineError::InvalidEvidence)
    );
}

#[test]
fn runtime_snapshot_digest_is_an_independent_activation_input() {
    let receipt = receipt();
    assert_eq!(
        decide_rubin_projection_activation(
            None,
            SNAPSHOT_ID,
            OTHER_SNAPSHOT_DIGEST,
            cutoff(),
            OBSERVATION_COUNT,
            DRAW_COUNT,
            IndicatorKind::AdditiveLogRatio,
        ),
        RubinProjectionActivationDecision::DescriptiveOnly
    );
    assert_eq!(
        decide_rubin_projection_activation(
            Some(&receipt),
            SNAPSHOT_ID,
            OTHER_SNAPSHOT_DIGEST,
            cutoff(),
            OBSERVATION_COUNT,
            DRAW_COUNT,
            IndicatorKind::AdditiveLogRatio,
        ),
        RubinProjectionActivationDecision::Rejected
    );
}
