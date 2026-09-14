//! Runtime design-envelope contract for Rubin projection activation.
//!
//! A design-envelope identifier is not evidence that the current run lies in
//! the validated design. Activation therefore receives the actual observation
//! and draw counts independently of the receipt. Production remains fail closed
//! while the approved-pairing registry is empty.

use analysis_engine::{
    RUBIN_LOADING_MODEL_CONTRACT_VERSION, RubinProjectionActivationDecision,
    RubinProjectionActivationReceiptV1, decide_rubin_projection_activation,
};
use psychometric_core::IndicatorKind;
use temporal_core::{AvailableTime, KnowledgeCutoff};

const SNAPSHOT_ID: &str = "snapshot-rubin-activation";
const SNAPSHOT_DIGEST: &str =
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
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
        cutoff(),
        "rubin-gaussian-single-level-candidate-v1",
    )
    .expect("receipt")
}

#[test]
fn production_activation_receives_actual_runtime_design_dimensions() {
    assert_eq!(
        decide_rubin_projection_activation(
            Some(&receipt()),
            SNAPSHOT_ID,
            SNAPSHOT_DIGEST,
            cutoff(),
            48,
            8,
            IndicatorKind::AdditiveLogRatio,
        ),
        RubinProjectionActivationDecision::Rejected
    );

    assert_eq!(
        decide_rubin_projection_activation(
            Some(&receipt()),
            SNAPSHOT_ID,
            SNAPSHOT_DIGEST,
            cutoff(),
            49,
            8,
            IndicatorKind::AdditiveLogRatio,
        ),
        RubinProjectionActivationDecision::Rejected
    );
}
