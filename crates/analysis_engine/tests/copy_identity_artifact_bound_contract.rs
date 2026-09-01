//! Execution and serialization bounds for the copy-identity profile.

use analysis_engine::{
    AnalysisEngineError, CopyIdentityArtifact, CopyIdentityDocument, MAX_EVIDENCE_UNITS,
    COPY_IDENTITY_ARTIFACT_SCHEMA_VERSION, COPY_IDENTITY_MODEL_CONTRACT_VERSION,
    COPY_IDENTITY_OUTPUT_PROFILE, execute_copy_identity_run,
};
use copy_identity::CopyKind;
use temporal_core::{AvailableTime, KnowledgeCutoff};
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest};

fn cutoff() -> KnowledgeCutoff {
    KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff")
}

fn document(id: String, kind: CopyKind) -> CopyIdentityDocument {
    CopyIdentityDocument::new(
        id,
        kind,
        AvailableTime::parse_rfc3339("2026-07-01T00:00:00Z").expect("available"),
    )
    .expect("document")
}

fn request() -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "copy-bound-idem".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: "snapshot-copy-bound".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: COPY_IDENTITY_MODEL_CONTRACT_VERSION.into(),
        output_profile: COPY_IDENTITY_OUTPUT_PROFILE.into(),
    }
}

#[test]
fn compact_oversized_copy_artifact_fails_closed() {
    let document_count = MAX_EVIDENCE_UNITS as u64 + 1;
    let template_copy_count = document_count - 1;
    let artifact = CopyIdentityArtifact {
        schema_version: COPY_IDENTITY_ARTIFACT_SCHEMA_VERSION.into(),
        run_id: "run-compact-oversize".into(),
        snapshot_id: "snapshot-compact-oversize".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        document_count,
        source_document_count: 1,
        template_copy_count,
        refused_as_source_count: template_copy_count,
        refused_as_transition_count: template_copy_count,
        inference_status: "template_copy_is_not_source_identity_not_transition".into(),
    };
    let raw_payload = serde_json::to_string(&artifact).expect("raw json");

    assert_eq!(
        artifact.to_json(),
        Err(AnalysisEngineError::InvalidCopyIdentityArtifact)
    );
    assert_eq!(
        CopyIdentityArtifact::from_json(&raw_payload),
        Err(AnalysisEngineError::InvalidCopyIdentityArtifact)
    );
}

#[test]
fn oversized_copy_execution_fails_before_census() {
    let request = request();
    let accepted = AnalysisRunAccepted::new("run-copy-bound", "accepted", &request.idempotency_key)
        .expect("accepted");
    let documents = (0..=MAX_EVIDENCE_UNITS)
        .map(|index| {
            let kind = if index == 0 {
                CopyKind::SourceDocument
            } else {
                CopyKind::TemplateCopy
            };
            document(format!("document-{index}"), kind)
        })
        .collect::<Vec<_>>();

    assert_eq!(
        execute_copy_identity_run(
            &request,
            &accepted,
            "snapshot-copy-bound",
            cutoff(),
            &documents,
            "2026-08-02T00:00:00Z",
        ),
        Err(AnalysisEngineError::LimitExceeded)
    );
}
