//! Regression contract for semantic knowledge-cutoff equality in relation-absence runs.

use analysis_engine::{
    RELATION_ABSENCE_MODEL_CONTRACT_VERSION, RELATION_ABSENCE_OUTPUT_PROFILE,
    RelationAbsencePair, execute_relation_absence_run,
};
use relation_absence::ObservationStatus;
use temporal_core::{AvailableTime, KnowledgeCutoff};
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest};

fn available() -> AvailableTime {
    AvailableTime::parse_rfc3339("2026-07-01T00:00:00Z").expect("available time")
}

fn pair(pair_id: &str, status: ObservationStatus) -> RelationAbsencePair {
    RelationAbsencePair::new(pair_id, status, available()).expect("pair")
}

#[test]
fn equivalent_rfc3339_cutoff_offsets_are_admitted() {
    let request = AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "relation-absence-equivalent-cutoff".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: "snapshot-relation-absence".into(),
        knowledge_cutoff: "2026-08-01T09:00:00+09:00".into(),
        model_contract_version: RELATION_ABSENCE_MODEL_CONTRACT_VERSION.into(),
        output_profile: RELATION_ABSENCE_OUTPUT_PROFILE.into(),
    };
    let accepted = AnalysisRunAccepted::new(
        "run-relation-absence",
        "accepted",
        &request.idempotency_key,
    )
    .expect("accepted");
    let execution_cutoff =
        KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("execution cutoff");
    let pairs = vec![
        pair("observed-a", ObservationStatus::Observed),
        pair("inferred-b", ObservationStatus::Inferred),
        pair("unobserved-c", ObservationStatus::Unobserved),
    ];

    let execution = execute_relation_absence_run(
        &request,
        &accepted,
        "snapshot-relation-absence",
        execution_cutoff,
        &pairs,
        "2026-08-02T00:00:00Z",
    )
    .expect("equivalent RFC 3339 spellings denote the same cutoff instant");

    assert_eq!(
        execution.artifact.knowledge_cutoff,
        execution_cutoff.to_rfc3339()
    );
}
