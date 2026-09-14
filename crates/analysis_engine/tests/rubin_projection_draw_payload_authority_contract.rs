//! Concrete draw-payload provenance contract for Rubin projection activation.
//!
//! Snapshot identity, design dimensions, and an approved generator class do not
//! identify the actual complete-data draw matrix consumed by one run. Activation
//! therefore binds the receipt to the canonical SHA-256 of that concrete payload
//! and receives the independently computed runtime digest.

use analysis_engine::{
    RUBIN_LOADING_MODEL_CONTRACT_VERSION, RubinProjectionActivationDecision,
    RubinProjectionActivationReceiptV1, decide_rubin_projection_activation,
};
use psychometric_core::IndicatorKind;
use temporal_core::{AvailableTime, KnowledgeCutoff};

const SNAPSHOT_ID: &str = "snapshot-rubin-activation";
const SNAPSHOT_DIGEST: &str =
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const DRAW_PAYLOAD_DIGEST: &str =
    "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const OTHER_DRAW_PAYLOAD_DIGEST: &str =
    "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
const EVIDENCE_DIGEST: &str =
    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn cutoff() -> KnowledgeCutoff {
    KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff")
}

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
        SNAPSHOT_ID,
        SNAPSHOT_DIGEST,
        DRAW_PAYLOAD_DIGEST,
        cutoff(),
        "rubin-gaussian-single-level-candidate-v1",
    )
    .expect("receipt")
}

#[test]
fn receipt_wire_binds_the_concrete_complete_data_draw_payload() {
    let receipt = receipt();
    assert_eq!(receipt.complete_data_draws_sha256(), DRAW_PAYLOAD_DIGEST);

    let json = receipt.to_json().expect("receipt json");
    let reparsed = RubinProjectionActivationReceiptV1::from_json(&json).expect("receipt parse");
    assert_eq!(reparsed.complete_data_draws_sha256(), DRAW_PAYLOAD_DIGEST);

    let mut forged: serde_json::Value = serde_json::from_str(&json).expect("json");
    forged["complete_data_draws_sha256"] = serde_json::json!("not-a-digest");
    assert!(RubinProjectionActivationReceiptV1::from_json(&forged.to_string()).is_err());
}

#[test]
fn production_decision_receives_runtime_draw_payload_digest_independently() {
    let receipt = receipt();

    // Production remains rejected because the approved-pairing registry is
    // intentionally empty, but the API must carry the concrete runtime digest
    // independently so a future approved pairing cannot authorize substitution.
    assert_eq!(
        decide_rubin_projection_activation(
            Some(&receipt),
            SNAPSHOT_ID,
            SNAPSHOT_DIGEST,
            DRAW_PAYLOAD_DIGEST,
            cutoff(),
            48,
            8,
            IndicatorKind::AdditiveLogRatio,
        ),
        RubinProjectionActivationDecision::Rejected
    );
    assert_eq!(
        decide_rubin_projection_activation(
            Some(&receipt),
            SNAPSHOT_ID,
            SNAPSHOT_DIGEST,
            OTHER_DRAW_PAYLOAD_DIGEST,
            cutoff(),
            48,
            8,
            IndicatorKind::AdditiveLogRatio,
        ),
        RubinProjectionActivationDecision::Rejected
    );
}
