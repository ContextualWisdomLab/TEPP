//! Evidence-identity contracts for longitudinal CWC composition.

use analysis_engine::{
    AnalysisEngineError, LONGITUDINAL_CWC_MODEL_CONTRACT_VERSION, LONGITUDINAL_CWC_OUTPUT_PROFILE,
    LongitudinalClusterScore, execute_longitudinal_cwc_run,
};
use temporal_core::{AvailableTime, KnowledgeCutoff};
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest};

const SNAPSHOT_ID: &str = "snapshot-longitudinal-cwc";

fn available(stamp: &str) -> AvailableTime {
    AvailableTime::parse_rfc3339(stamp).expect("available")
}

fn cutoff() -> KnowledgeCutoff {
    KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff")
}

fn row(
    evidence_id: &str,
    cluster_key: u64,
    predictor: f64,
    outcome: f64,
    available_at: &str,
) -> LongitudinalClusterScore {
    LongitudinalClusterScore::new(
        evidence_id,
        SNAPSHOT_ID,
        cluster_key,
        predictor,
        outcome,
        available(available_at),
    )
    .expect("row")
}

fn request() -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "longitudinal-cwc-evidence-identity".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: SNAPSHOT_ID.into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: LONGITUDINAL_CWC_MODEL_CONTRACT_VERSION.into(),
        output_profile: LONGITUDINAL_CWC_OUTPUT_PROFILE.into(),
    }
}

fn accepted(request: &AnalysisRunRequest) -> AnalysisRunAccepted {
    AnalysisRunAccepted::new("run-longitudinal-cwc-identity", "accepted", &request.idempotency_key)
        .expect("accepted")
}

fn baseline_rows() -> Vec<LongitudinalClusterScore> {
    vec![
        row("evidence-1", 1, 0.0, 2.0, "2026-07-01T00:00:00Z"),
        row("evidence-2", 1, 2.0, 3.0, "2026-07-01T00:00:00Z"),
        row("evidence-3", 2, 4.0, 10.0, "2026-07-01T00:00:00Z"),
        row("evidence-4", 2, 6.0, 11.0, "2026-07-01T00:00:00Z"),
    ]
}

#[test]
fn duplicate_evidence_identity_fails_closed_before_cwc_composition() {
    let request = request();
    let accepted = accepted(&request);
    let mut rows = baseline_rows();
    rows.push(row(
        "evidence-2",
        1,
        9.0,
        -7.0,
        "2026-07-02T00:00:00Z",
    ));

    assert_eq!(
        execute_longitudinal_cwc_run(
            &request,
            &accepted,
            SNAPSHOT_ID,
            cutoff(),
            &rows,
            "2026-08-02T00:00:00Z",
        ),
        Err(AnalysisEngineError::DuplicateEvidence)
    );
}

#[test]
fn duplicate_identity_after_cutoff_is_still_a_snapshot_contract_failure() {
    let request = request();
    let accepted = accepted(&request);
    let mut rows = baseline_rows();
    rows.push(row(
        "evidence-2",
        3,
        100.0,
        100.0,
        "2026-08-15T00:00:00Z",
    ));

    assert_eq!(
        execute_longitudinal_cwc_run(
            &request,
            &accepted,
            SNAPSHOT_ID,
            cutoff(),
            &rows,
            "2026-08-02T00:00:00Z",
        ),
        Err(AnalysisEngineError::DuplicateEvidence)
    );
}

#[test]
fn numerically_equal_rows_with_distinct_identity_remain_distinct_evidence() {
    let request = request();
    let accepted = accepted(&request);
    let mut rows = baseline_rows();
    rows.push(row(
        "evidence-5",
        1,
        2.0,
        3.0,
        "2026-07-01T00:00:00Z",
    ));

    let execution = execute_longitudinal_cwc_run(
        &request,
        &accepted,
        SNAPSHOT_ID,
        cutoff(),
        &rows,
        "2026-08-02T00:00:00Z",
    )
    .expect("distinct evidence identity must not be tuple-deduplicated");

    assert_eq!(execution.artifact.row_count, 5);
    assert_eq!(rows[1].evidence_id(), "evidence-2");
    assert_eq!(rows[4].evidence_id(), "evidence-5");
}

#[test]
fn evidence_identity_is_bounded_by_the_existing_analysis_identifier_contract() {
    assert_eq!(
        LongitudinalClusterScore::new(
            "",
            SNAPSHOT_ID,
            1,
            0.0,
            1.0,
            available("2026-07-01T00:00:00Z"),
        ),
        Err(AnalysisEngineError::InvalidEvidence)
    );
}
