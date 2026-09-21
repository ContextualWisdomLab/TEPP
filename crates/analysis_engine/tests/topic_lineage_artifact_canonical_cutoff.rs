use analysis_engine::{
    TOPIC_LINEAGE_ARTIFACT_SCHEMA_VERSION, TopicLineageArtifact, TopicLineageArtifactEdge,
};

fn artifact() -> TopicLineageArtifact {
    TopicLineageArtifact {
        schema_version: TOPIC_LINEAGE_ARTIFACT_SCHEMA_VERSION.into(),
        run_id: "run-1".into(),
        snapshot_id: "snapshot-1".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        selected_seed: 7,
        iterations: 4,
        objective: -1.0,
        topic_count: 2,
        evidence_count: 2,
        connected_post_count: 2,
        lineage_count: 1,
        sequence_edges: vec![TopicLineageArtifactEdge {
            predecessor_document_id: "00000000-0000-0000-0000-000000000001".into(),
            successor_document_id: "00000000-0000-0000-0000-000000000002".into(),
            topic_index: 0,
            association_strength: 0.8,
        }],
        inference_status: "fitted_topic_association_not_causation".into(),
    }
}

#[test]
fn artifact_cutoff_requires_one_canonical_utc_byte_identity() {
    let canonical = artifact();
    assert!(canonical.to_json().is_ok());

    let mut offset_alias = canonical;
    offset_alias.knowledge_cutoff = "2026-08-01T09:00:00+09:00".into();
    assert!(offset_alias.to_json().is_err());
}
