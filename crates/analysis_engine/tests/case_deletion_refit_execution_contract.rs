//! End-to-end contract for cutoff-safe exhaustive case-deletion refit.

use std::sync::atomic::{AtomicUsize, Ordering};

use analysis_engine::{
    AnalysisEngineError, CASE_DELETION_REFIT_ARTIFACT_SCHEMA_VERSION,
    CASE_DELETION_REFIT_MODEL_CONTRACT_VERSION, CASE_DELETION_REFIT_OUTPUT_PROFILE,
    CaseDeletionDocument, CaseDeletionFitContext, CaseDeletionRefitInput, CaseDeletionRefitter,
    execute_case_deletion_refit_run,
};
use temporal_core::KnowledgeCutoff;
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalState, ApiError};

struct MeanFitter;

struct RefusingFitter;

#[derive(Default)]
struct CountingFitter {
    calls: AtomicUsize,
}

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

impl CaseDeletionRefitter<f64, ()> for CountingFitter {
    type Error = ();

    fn fit(
        &self,
        _retained_documents: &[&CaseDeletionDocument<f64>],
        _context: &CaseDeletionFitContext,
    ) -> Result<(), Self::Error> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(())
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
        execution
            .terminal_result
            .summary
            .as_ref()
            .expect("summary")
            .validation_status,
        "validated"
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
fn equivalent_cutoff_spellings_bind_the_same_instant() {
    let canonical_request = request();
    let baseline = execute(&canonical_request).expect("canonical cutoff");
    let mut offset_request = canonical_request;
    offset_request.knowledge_cutoff = "2026-08-01T01:00:00+01:00".into();
    let equivalent = execute(&offset_request).expect("equivalent instant");
    assert_eq!(equivalent.artifact, baseline.artifact);
    assert_eq!(equivalent.terminal_result.summary, baseline.terminal_result.summary);
}

#[test]
fn oversized_case_deletion_census_fails_before_any_fitter_call() {
    let request = request();
    let documents = (0..257)
        .map(|index| CaseDeletionDocument {
            document_id: format!("document-{index}"),
            evidence: f64::from(index),
        })
        .collect::<Vec<_>>();
    let fitter = CountingFitter::default();
    assert_eq!(
        execute_case_deletion_refit_run(
            &request,
            &accepted(&request),
            "snapshot-case-deletion-refit",
            cutoff(),
            &CaseDeletionRefitInput::new(&documents, "topic-model-run", &fitter),
            "2026-08-02T00:00:00Z",
        ),
        Err(AnalysisEngineError::LimitExceeded)
    );
    assert_eq!(fitter.calls.load(Ordering::SeqCst), 0);
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

#[test]
fn invalid_completed_at_fails_terminal_result_construction() {
    let request = request();
    let documents = documents();
    let fitter = MeanFitter;
    assert_eq!(
        execute_case_deletion_refit_run(
            &request,
            &accepted(&request),
            "snapshot-case-deletion-refit",
            cutoff(),
            &CaseDeletionRefitInput::new(&documents, "topic-model-run", &fitter),
            "not-a-timestamp",
        ),
        Err(AnalysisEngineError::Api(ApiError::InvalidWirePayload))
    );
}
