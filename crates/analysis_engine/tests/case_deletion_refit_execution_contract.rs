//! End-to-end contract for cutoff-safe exhaustive case-deletion refit.

use std::sync::atomic::{AtomicUsize, Ordering};

use analysis_engine::{
    AnalysisEngineError, CASE_DELETION_REFIT_ARTIFACT_SCHEMA_VERSION,
    CASE_DELETION_REFIT_MODEL_CONTRACT_VERSION, CASE_DELETION_REFIT_OUTPUT_PROFILE,
    CaseDeletionDocument, CaseDeletionFitContext, CaseDeletionRefitInput, CaseDeletionRefitter,
    execute_case_deletion_refit_run,
};
use temporal_core::{AvailableTime, KnowledgeCutoff};
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalState, ApiError};

const SNAPSHOT_ID: &str = "snapshot-case-deletion-refit";

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

fn available_time(value: &str) -> AvailableTime {
    AvailableTime::parse_rfc3339(value).expect("available time")
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

fn visible_snapshot_ids(len: usize) -> Vec<String> {
    vec![SNAPSHOT_ID.to_owned(); len]
}

fn visible_available_times(len: usize) -> Vec<AvailableTime> {
    vec![available_time("2026-07-01T00:00:00Z"); len]
}

fn request() -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "case-deletion-refit-idem".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: SNAPSHOT_ID.into(),
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

fn execute_with_provenance<D, P, F>(
    request: &AnalysisRunRequest,
    documents: &[CaseDeletionDocument<D>],
    snapshot_ids: &[String],
    available_times: &[AvailableTime],
    fitter: &F,
) -> Result<analysis_engine::CaseDeletionRefitExecution, AnalysisEngineError>
where
    D: Clone,
    F: CaseDeletionRefitter<D, P>,
{
    execute_case_deletion_refit_run(
        request,
        &accepted(request),
        SNAPSHOT_ID,
        cutoff(),
        &CaseDeletionRefitInput::new(
            documents,
            snapshot_ids,
            available_times,
            "topic-model-run",
            fitter,
        ),
        "2026-08-02T00:00:00Z",
    )
}

fn execute_with_documents(
    request: &AnalysisRunRequest,
    documents: &[CaseDeletionDocument<f64>],
) -> Result<analysis_engine::CaseDeletionRefitExecution, AnalysisEngineError> {
    let snapshot_ids = visible_snapshot_ids(documents.len());
    let available_times = visible_available_times(documents.len());
    execute_with_provenance(
        request,
        documents,
        &snapshot_ids,
        &available_times,
        &MeanFitter,
    )
}

fn execute(
    request: &AnalysisRunRequest,
) -> Result<analysis_engine::CaseDeletionRefitExecution, AnalysisEngineError> {
    execute_with_documents(request, &documents())
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
fn future_duplicate_evidence_cannot_change_historical_replay() {
    let request = request();
    let baseline_documents = documents();
    let baseline = execute_with_documents(&request, &baseline_documents).expect("baseline");

    let mut replay_documents = baseline_documents;
    replay_documents.push(CaseDeletionDocument {
        document_id: "document-a".into(),
        evidence: 10_000.0,
    });
    let replay_snapshot_ids = visible_snapshot_ids(replay_documents.len());
    let mut replay_available_times = visible_available_times(replay_documents.len());
    *replay_available_times.last_mut().expect("future availability") =
        available_time("2026-08-02T00:00:00Z");
    let replay = execute_with_provenance(
        &request,
        &replay_documents,
        &replay_snapshot_ids,
        &replay_available_times,
        &MeanFitter,
    )
    .expect("future evidence must be censored before duplicate/scientific admission");
    assert_eq!(replay.artifact, baseline.artifact);
    assert_eq!(replay.terminal_result.summary, baseline.terminal_result.summary);
}

#[test]
fn cross_snapshot_and_misaligned_provenance_fail_closed() {
    let request = request();
    let documents = documents();
    let available_times = visible_available_times(documents.len());
    let mut wrong_snapshot_ids = visible_snapshot_ids(documents.len());
    wrong_snapshot_ids[1] = "snapshot-other".into();
    assert_eq!(
        execute_with_provenance(
            &request,
            &documents,
            &wrong_snapshot_ids,
            &available_times,
            &MeanFitter,
        ),
        Err(AnalysisEngineError::InvalidEvidence)
    );

    let short_snapshot_ids = vec![SNAPSHOT_ID.to_owned(); documents.len() - 1];
    assert_eq!(
        execute_with_provenance(
            &request,
            &documents,
            &short_snapshot_ids,
            &available_times,
            &MeanFitter,
        ),
        Err(AnalysisEngineError::InvalidEvidence)
    );
}

#[test]
fn visible_duplicate_identity_still_fails_closed() {
    let request = request();
    let mut duplicate_documents = documents();
    duplicate_documents.push(CaseDeletionDocument {
        document_id: "document-a".into(),
        evidence: 13.0,
    });
    assert_eq!(
        execute_with_documents(&request, &duplicate_documents),
        Err(AnalysisEngineError::InvalidEvidence)
    );
}

#[test]
fn oversized_case_deletion_census_fails_before_any_fitter_call() {
    let request = request();
    let documents = (0..257)
        .map(|index| CaseDeletionDocument {
            document_id: format!("document-{index}"),
            evidence: f64::from(u32::try_from(index).expect("small test index")),
        })
        .collect::<Vec<_>>();
    let snapshot_ids = visible_snapshot_ids(documents.len());
    let available_times = visible_available_times(documents.len());
    let fitter = CountingFitter::default();
    assert_eq!(
        execute_with_provenance(
            &request,
            &documents,
            &snapshot_ids,
            &available_times,
            &fitter,
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
    assert_eq!(
        execute_with_documents(&request, &one),
        Err(AnalysisEngineError::InvalidEvidence)
    );

    let documents = documents();
    let snapshot_ids = visible_snapshot_ids(documents.len());
    let available_times = visible_available_times(documents.len());
    assert_eq!(
        execute_with_provenance(
            &request,
            &documents,
            &snapshot_ids,
            &available_times,
            &RefusingFitter,
        ),
        Err(AnalysisEngineError::CaseDeletionFitFailure)
    );
}

#[test]
fn execution_refuses_snapshot_profile_and_cutoff_mismatch() {
    let request = request();
    let documents = documents();
    let snapshot_ids = visible_snapshot_ids(documents.len());
    let available_times = visible_available_times(documents.len());
    assert_eq!(
        execute_case_deletion_refit_run(
            &request,
            &accepted(&request),
            "other-snapshot",
            cutoff(),
            &CaseDeletionRefitInput::new(
                &documents,
                &snapshot_ids,
                &available_times,
                "topic-model-run",
                &MeanFitter,
            ),
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
    let snapshot_ids = visible_snapshot_ids(documents.len());
    let available_times = visible_available_times(documents.len());
    assert_eq!(
        execute_case_deletion_refit_run(
            &request,
            &accepted(&request),
            SNAPSHOT_ID,
            cutoff(),
            &CaseDeletionRefitInput::new(
                &documents,
                &snapshot_ids,
                &available_times,
                "topic-model-run",
                &MeanFitter,
            ),
            "not-a-timestamp",
        ),
        Err(AnalysisEngineError::Api(ApiError::InvalidWirePayload))
    );
}
