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
