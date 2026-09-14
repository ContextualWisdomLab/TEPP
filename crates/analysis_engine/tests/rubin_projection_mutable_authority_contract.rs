//! Regression contract for mutable Rubin projection authority locators.
//!
//! Activation authority is digest-bound scientific provenance. Mutable Git
//! branch, pull-request, issue, repository-locator, or latest-release aliases
//! must never be admitted as immutable contract/evidence identities.

use analysis_engine::{
    AnalysisEngineError, RUBIN_LOADING_MODEL_CONTRACT_VERSION,
    RubinProjectionActivationReceiptV1,
};
use temporal_core::{AvailableTime, KnowledgeCutoff};

const EVIDENCE_DIGEST: &str =
    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn canonical_receipt_json() -> String {
    RubinProjectionActivationReceiptV1::new(
        ("gaussian_complete_data_draws", "candidate-v1"),
        ("rubin_loading_uncertainty", RUBIN_LOADING_MODEL_CONTRACT_VERSION),
        (
            "validation-evidence-rubin-candidate-v1",
            EVIDENCE_DIGEST,
            AvailableTime::parse_rfc3339("2026-07-31T23:59:59Z").expect("availability"),
        ),
        "snapshot-rubin-activation",
        KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff"),
        "rubin-gaussian-single-level-candidate-v1",
    )
    .expect("receipt")
    .to_json()
    .expect("json")
}

#[test]
fn mutable_authority_aliases_fail_closed_on_public_wire() {
    let canonical = canonical_receipt_json();
    let mutable_aliases = [
        "Latest",
        "PR-504",
        "pull/504",
        "issue-505",
        "#504",
        "github.com/ContextualWisdomLab/TEPP/pull/504",
        "git://github.com/ContextualWisdomLab/TEPP.git",
        "ssh://git@github.com/ContextualWisdomLab/TEPP.git",
        "latest-release",
        "release-latest",
    ];

    for mutable_alias in mutable_aliases {
        let mut payload: serde_json::Value =
            serde_json::from_str(&canonical).expect("canonical json");
        payload["validation_evidence_id"] = serde_json::json!(mutable_alias);
        assert_eq!(
            RubinProjectionActivationReceiptV1::from_json(&payload.to_string()),
            Err(AnalysisEngineError::InvalidEvidence),
            "mutable authority alias must fail closed: {mutable_alias}"
        );
    }
}

#[test]
fn immutable_scientific_identifiers_remain_admissible_without_keyword_heuristics() {
    let canonical = canonical_receipt_json();
    let immutable_identifiers = [
        "validation-evidence-rubin-candidate-v1",
        "validation-evidence-latest-model-v1",
        "main-effect-loading-model-v1",
        "pr-",
        "pr-model-v1",
        "issue-analysis-v1",
    ];

    for immutable_identifier in immutable_identifiers {
        let mut payload: serde_json::Value =
            serde_json::from_str(&canonical).expect("canonical json");
        payload["validation_evidence_id"] = serde_json::json!(immutable_identifier);
        assert!(
            RubinProjectionActivationReceiptV1::from_json(&payload.to_string()).is_ok(),
            "non-locator scientific identifier must remain admissible: {immutable_identifier}"
        );
    }
}
