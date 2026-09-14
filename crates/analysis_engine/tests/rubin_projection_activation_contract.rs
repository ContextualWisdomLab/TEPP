//! Fail-closed authority contract for Rubin inferential projection.
//!
//! A valid-looking receipt is not sufficient authority: production approval
//! requires an immutable pairing that is independently registered by the
//! Validation Evidence owner path. Until that registry contains a pairing, the
//! decision remains descriptive-only or rejected without changing Rubin
//! arithmetic.

use analysis_engine::{
    RUBIN_LOADING_MODEL_CONTRACT_VERSION, RubinProjectionActivationDecision,
    RubinProjectionActivationReceiptV1, decide_rubin_projection_activation,
};
use psychometric_core::IndicatorKind;
use temporal_core::{AvailableTime, KnowledgeCutoff};

const SNAPSHOT_ID: &str = "snapshot-rubin-activation";
const SNAPSHOT_DIGEST: &str =
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const CUTOFF: &str = "2026-08-01T00:00:00Z";
const EVIDENCE_DIGEST: &str =
    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn cutoff() -> KnowledgeCutoff {
    KnowledgeCutoff::parse_rfc3339(CUTOFF).expect("cutoff")
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
        cutoff(),
        "rubin-gaussian-single-level-candidate-v1",
    )
    .expect("receipt")
}

#[test]
fn noncanonical_analysis_identity_is_not_admitted_by_rubin_receipt() {
    let evidence = (
        "validation-evidence-rubin-candidate-v1",
        EVIDENCE_DIGEST,
        AvailableTime::parse_rfc3339("2026-07-31T23:59:59Z").expect("availability"),
    );

    assert!(
        RubinProjectionActivationReceiptV1::new(
            ("gaussian_complete_data_draws", "candidate-v1"),
            ("different_analysis_contract", RUBIN_LOADING_MODEL_CONTRACT_VERSION),
            evidence.clone(),
            SNAPSHOT_ID,
            SNAPSHOT_DIGEST,
            cutoff(),
            "rubin-gaussian-single-level-candidate-v1",
        )
        .is_err()
    );
    assert!(
        RubinProjectionActivationReceiptV1::new(
            ("gaussian_complete_data_draws", "candidate-v1"),
            ("rubin_loading_uncertainty", "noncanonical-version"),
            evidence,
            SNAPSHOT_ID,
            SNAPSHOT_DIGEST,
            cutoff(),
            "rubin-gaussian-single-level-candidate-v1",
        )
        .is_err()
    );
}

#[test]
fn no_receipt_remains_descriptive_only() {
    assert_eq!(
        decide_rubin_projection_activation(
            None,
            SNAPSHOT_ID,
            SNAPSHOT_DIGEST,
            cutoff(),
            IndicatorKind::AdditiveLogRatio,
        ),
        RubinProjectionActivationDecision::DescriptiveOnly
    );
}

#[test]
fn production_registry_does_not_preapprove_candidate_evidence() {
    assert_eq!(
        decide_rubin_projection_activation(
            Some(&receipt()),
            SNAPSHOT_ID,
            SNAPSHOT_DIGEST,
            cutoff(),
            IndicatorKind::AdditiveLogRatio,
        ),
        RubinProjectionActivationDecision::Rejected
    );
}

#[test]
fn receipt_wire_is_bounded_digest_bound_and_canonical() {
    let receipt = receipt();
    let json = receipt.to_json().expect("receipt json");
    let reparsed = RubinProjectionActivationReceiptV1::from_json(&json).expect("receipt parse");

    assert_eq!(reparsed, receipt);
    assert_eq!(reparsed.knowledge_cutoff(), CUTOFF);
    assert_eq!(reparsed.source_snapshot_id(), SNAPSHOT_ID);
    assert_eq!(reparsed.source_snapshot_sha256(), SNAPSHOT_DIGEST);
    assert_eq!(reparsed.sha256().expect("digest").len(), 64);
}

#[test]
fn receipt_wire_refuses_forged_digest_and_late_evidence() {
    let canonical = receipt().to_json().expect("receipt json");
    let mut forged: serde_json::Value = serde_json::from_str(&canonical).expect("json");
    forged["validation_evidence_sha256"] = serde_json::json!("not-a-digest");
    assert!(RubinProjectionActivationReceiptV1::from_json(&forged.to_string()).is_err());

    let mut forged_snapshot: serde_json::Value = serde_json::from_str(&canonical).expect("json");
    forged_snapshot["source_snapshot_sha256"] = serde_json::json!("not-a-digest");
    assert!(RubinProjectionActivationReceiptV1::from_json(&forged_snapshot.to_string()).is_err());

    let late = RubinProjectionActivationReceiptV1::new(
        ("gaussian_complete_data_draws", "candidate-v1"),
        (
            "rubin_loading_uncertainty",
            RUBIN_LOADING_MODEL_CONTRACT_VERSION,
        ),
        (
            "validation-evidence-rubin-candidate-v1",
            EVIDENCE_DIGEST,
            AvailableTime::parse_rfc3339("2026-08-01T00:00:01Z").expect("availability"),
        ),
        SNAPSHOT_ID,
        SNAPSHOT_DIGEST,
        cutoff(),
        "rubin-gaussian-single-level-candidate-v1",
    )
    .expect("syntactically valid late receipt");
    assert_eq!(
        decide_rubin_projection_activation(
            Some(&late),
            SNAPSHOT_ID,
            SNAPSHOT_DIGEST,
            cutoff(),
            IndicatorKind::AdditiveLogRatio,
        ),
        RubinProjectionActivationDecision::Rejected
    );
}
