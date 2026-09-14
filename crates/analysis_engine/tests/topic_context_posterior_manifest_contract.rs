//! Wire contract for the topic-context posterior snapshot manifest.

use std::collections::BTreeMap;

use analysis_engine::{
    AnalysisEngineError, TOPIC_CONTEXT_POSTERIOR_BYTE_LIMIT, TopicContextPosteriorSnapshotManifest,
};

fn manifest() -> TopicContextPosteriorSnapshotManifest {
    TopicContextPosteriorSnapshotManifest {
        snapshot_id: "snapshot-topic-context-posterior".into(),
        source_snapshot_sha256: "0".repeat(64),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        artifact_sha256: "1".repeat(64),
        document_available_at: BTreeMap::from([(
            "018f3f7a-7b7c-7d00-8000-000000000001".into(),
            "2026-07-20T00:00:00Z".into(),
        )]),
        support_evidence_available_at: BTreeMap::from([(
            "evidence-relation-1".into(),
            "2026-07-20T00:00:00Z".into(),
        )]),
    }
}

#[test]
fn manifest_round_trips_through_the_bounded_validated_wire_contract() {
    let expected = manifest();
    let json = expected.to_json().expect("manifest json");
    assert!(json.len() < TOPIC_CONTEXT_POSTERIOR_BYTE_LIMIT);
    assert_eq!(
        TopicContextPosteriorSnapshotManifest::from_json(&json).expect("manifest parse"),
        expected
    );
}

#[test]
fn manifest_requires_an_explicit_support_evidence_availability_ledger() {
    let json = manifest().to_json().expect("manifest json");
    let mut value: serde_json::Value = serde_json::from_str(&json).expect("manifest value");
    value
        .as_object_mut()
        .expect("manifest object")
        .remove("support_evidence_available_at");
    let payload = serde_json::to_string(&value).expect("manifest payload");

    assert_eq!(
        TopicContextPosteriorSnapshotManifest::from_json(&payload),
        Err(AnalysisEngineError::InvalidEvidence)
    );
}

#[test]
fn manifest_parser_rejects_unknown_fields_and_oversized_payloads() {
    let json = manifest().to_json().expect("manifest json");
    let unknown = json.replacen('{', "{\"unexpected\":true,", 1);
    assert_eq!(
        TopicContextPosteriorSnapshotManifest::from_json(&unknown),
        Err(AnalysisEngineError::InvalidEvidence)
    );

    let oversized = "x".repeat(TOPIC_CONTEXT_POSTERIOR_BYTE_LIMIT + 1);
    assert_eq!(
        TopicContextPosteriorSnapshotManifest::from_json(&oversized),
        Err(AnalysisEngineError::LimitExceeded)
    );
}

#[test]
fn manifest_rejects_malformed_support_availability() {
    let mut malformed = manifest();
    *malformed
        .support_evidence_available_at
        .first_entry()
        .expect("support evidence")
        .get_mut() = "not-a-time".into();
    assert_eq!(
        malformed.to_json(),
        Err(AnalysisEngineError::InvalidEvidence)
    );
}
