//! End-to-end contract for cutoff-safe exhaustive case-deletion refit.

use analysis_engine::{
    AnalysisEngineError, CASE_DELETION_REFIT_ARTIFACT_SCHEMA_VERSION,
    CASE_DELETION_REFIT_MODEL_CONTRACT_VERSION, CASE_DELETION_REFIT_OUTPUT_PROFILE,
    CaseDeletionDocument, CaseDeletionFitContext, CaseDeletionRefitInput, CaseDeletionRefitter,
    execute_case_deletion_refit_run,
};
use temporal_core::KnowledgeCutoff;
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalState};

struct MeanFitter;

struct RefusingFitter;

impl CaseDeletionRefitter<f64, f64> for MeanFitter {
    type Error = ();

    fn fit(
        &self,
        retained_documents: &[&CaseDeletionDocument<f64>],
        _context: &CaseDeletionFitContext,
    ) -> Result<f64, Self::Error> {
        let sum = retained_documents
            .iter()
            .map(|document| document.evidence)
            .sum::<f64>();
        let count = u32::try_from(retained_documents.len()).map_err(|_| ())?;
        Ok(sum / f64::from(count))
    }
}

impl CaseDeletionRefitter<f64, f64> for RefusingFitter {
    type Error = &'static str;

    fn fit(
        &self,
        _retained_documents: &[&CaseDeletionDocument<f64>],
        _context: &CaseDeletionFitContext,
    ) -> Result<f64, Self::Error> {
        Err("synthetic refusal")
    }
}

fn cutoff() -> KnowledgeCutoff {
    KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff")
}

fn documents() -> Vec<CaseDeletionDocument<f64>> {
    vec![
        CaseDeletionDocument {
            document_id: "document-a".into(),
            evidence: 1.0,
        },
        CaseDeletionDocument {
            document_id: "document-b".into(),
            evidence: 3.0,
        },
        CaseDeletionDocument {
            document_id: "document-c".into(),
            evidence: 8.0,
        },
    ]
}

fn request() -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "case-deletion-refit-idem".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: "snapshot-case-deletion-refit".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: CASE_DELETION_REFIT_MODEL_CONTRACT_VERSION.into(),
        output_profile: CASE_DELETION_REFIT_OUTPUT_PROFILE.into(),
    }
}

fn accepted(request: &AnalysisRunRequest) -> AnalysisRunAccepted {
    AnalysisRunAccepted::new(
        "run-case-deletion-refit",
        "accepted",
        &request.idempotency_key,
    )
    .expect("accepted")
}

fn execute(
    request: &AnalysisRunRequest,
) -> Result<analysis_engine::CaseDeletionRefitExecution, AnalysisEngineError> {
    let documents = documents();
    let fitter = MeanFitter;
    execute_case_deletion_refit_run(
        request,
        &accepted(request),
        "snapshot-case-deletion-refit",
        cutoff(),
        &CaseDeletionRefitInput::new(&documents, "topic-model-run", &fitter),
        "2026-08-02T00:00:00Z",
    )
}

#[test]
fn exhaustive_refits_emit_digest_bound_counts_without_reweighting() {
    let request = request();
    let execution = execute(&request).expect("execution");
    assert_eq!(
        execution.artifact.schema_version,
        CASE_DELETION_REFIT_ARTIFACT_SCHEMA_VERSION
    );
    assert_eq!(execution.artifact.document_count, 3);
    assert_eq!(execution.artifact.deletion_refit_count, 3);
    assert_eq!(execution.artifact.independent_seed_domain_count, 4);
    assert_eq!(execution.artifact.full_seed_domain, "topic-model-run:full");
    assert_eq!(
        execution.artifact.inference_status,
        "exhaustive_actual_deletion_not_reweighting_approx"
    );
    assert_eq!(
        execution.terminal_result.run_state,
        AnalysisRunTerminalState::Succeeded
    );
    assert_eq!(
        execution.terminal_result.result_sha256.as_deref(),
        Some(execution.artifact.sha256().expect("digest").as_str())
    );
    assert_eq!(
        execution.terminal_result.result_schema_version.as_deref(),
        Some(CASE_DELETION_REFIT_ARTIFACT_SCHEMA_VERSION)
    );
}

#[test]
fn invalid_corpus_and_fitter_refusal_fail_closed() {
    let request = request();
    let one = vec![CaseDeletionDocument {
        document_id: "document-a".into(),
        evidence: 1.0,
    }];
    let fitter = MeanFitter;
    assert_eq!(
        execute_case_deletion_refit_run(
            &request,
            &accepted(&request),
            "snapshot-case-deletion-refit",
            cutoff(),
            &CaseDeletionRefitInput::new(&one, "topic-model-run", &fitter),
            "2026-08-02T00:00:00Z",
        ),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let documents = documents();
    let refusing = RefusingFitter;
    assert_eq!(
        execute_case_deletion_refit_run(
            &request,
            &accepted(&request),
            "snapshot-case-deletion-refit",
            cutoff(),
            &CaseDeletionRefitInput::new(&documents, "topic-model-run", &refusing),
            "2026-08-02T00:00:00Z",
        ),
        Err(AnalysisEngineError::CaseDeletionFitFailure)
    );
}

#[test]
fn execution_refuses_snapshot_profile_and_cutoff_mismatch() {
    let request = request();
    let documents = documents();
    let fitter = MeanFitter;
    assert_eq!(
        execute_case_deletion_refit_run(
            &request,
            &accepted(&request),
            "other-snapshot",
            cutoff(),
            &CaseDeletionRefitInput::new(&documents, "topic-model-run", &fitter),
            "2026-08-02T00:00:00Z",
        ),
        Err(AnalysisEngineError::SnapshotMismatch)
    );
    for invalid_request in [
        {
            let mut value = request.clone();
            value.knowledge_cutoff = "2026-08-02T00:00:00Z".into();
            value
        },
        {
            let mut value = request.clone();
            value.model_contract_version = "other-model".into();
            value
        },
        {
            let mut value = request.clone();
            value.output_profile = "composed_fitted_lineage_v1".into();
            value
        },
        {
            let mut value = request.clone();
            value.output_profile = "fitted_candidate_k_v1".into();
            value
        },
        {
            let mut value = request.clone();
            value.output_profile = "trsl_topic_lineage_v1".into();
            value
        },
        {
            let mut value = request.clone();
            value.output_profile = "pareto_candidate_k_v1".into();
            value
        },
    ] {
        assert_eq!(
            execute(&invalid_request),
            Err(AnalysisEngineError::InvalidEvidence)
        );
    }
}
