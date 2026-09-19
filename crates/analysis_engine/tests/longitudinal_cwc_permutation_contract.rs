//! Consumer RED for enumeration-order invariance of longitudinal CWC composition.
//!
//! The same cutoff-visible evidence population must produce the same digest-bound
//! artifact regardless of source row enumeration. Generic correctly-rounded
//! binary64 mean arithmetic belongs to the released fast-mlsirm owner contract;
//! this test deliberately does not implement a local summation workaround.

use analysis_engine::{
    LONGITUDINAL_CWC_MODEL_CONTRACT_VERSION, LONGITUDINAL_CWC_OUTPUT_PROFILE,
    LongitudinalClusterScore, execute_longitudinal_cwc_run,
};
use temporal_core::{AvailableTime, KnowledgeCutoff};
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest};

const SNAPSHOT_ID: &str = "snapshot-cwc-permutation";

fn available() -> AvailableTime {
    AvailableTime::parse_rfc3339("2026-07-01T00:00:00Z").expect("available")
}

fn cutoff() -> KnowledgeCutoff {
    KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff")
}

fn request() -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "cwc-permutation-idem".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: SNAPSHOT_ID.into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: LONGITUDINAL_CWC_MODEL_CONTRACT_VERSION.into(),
        output_profile: LONGITUDINAL_CWC_OUTPUT_PROFILE.into(),
    }
}

fn accepted(request: &AnalysisRunRequest) -> AnalysisRunAccepted {
    AnalysisRunAccepted::new("run-cwc-permutation", "accepted", &request.idempotency_key)
        .expect("accepted")
}

fn row(
    evidence_id: &str,
    cluster_key: u64,
    predictor: f64,
    outcome: f64,
) -> LongitudinalClusterScore {
    LongitudinalClusterScore::new(
        evidence_id,
        SNAPSHOT_ID,
        cluster_key,
        predictor,
        outcome,
        available(),
    )
    .expect("finite row")
}

fn baseline_rows() -> Vec<LongitudinalClusterScore> {
    vec![
        row("c1-high", 1, 10_000_000_000_000_000.0, 5_000_000_000_000_000.0),
        row("c1-tail", 1, 1.0, 0.5),
        row("c1-low", 1, -10_000_000_000_000_000.0, -5_000_000_000_000_000.0),
        row("c2-high", 2, 10_000_000_000_000_010.0, 5_000_000_000_000_020.0),
        row("c2-tail", 2, 11.0, 20.5),
        row("c2-low", 2, -9_999_999_999_999_990.0, -4_999_999_999_999_980.0),
    ]
}

fn permuted_rows() -> Vec<LongitudinalClusterScore> {
    vec![
        row("c1-high", 1, 10_000_000_000_000_000.0, 5_000_000_000_000_000.0),
        row("c1-low", 1, -10_000_000_000_000_000.0, -5_000_000_000_000_000.0),
        row("c1-tail", 1, 1.0, 0.5),
        row("c2-high", 2, 10_000_000_000_000_010.0, 5_000_000_000_000_020.0),
        row("c2-tail", 2, 11.0, 20.5),
        row("c2-low", 2, -9_999_999_999_999_990.0, -4_999_999_999_999_980.0),
    ]
}

#[test]
fn same_evidence_population_is_bit_identical_under_row_permutation() {
    let request = request();
    let accepted = accepted(&request);
    let baseline = execute_longitudinal_cwc_run(
        &request,
        &accepted,
        SNAPSHOT_ID,
        cutoff(),
        &baseline_rows(),
        "2026-08-02T00:00:00Z",
    )
    .expect("baseline execution");
    let permuted = execute_longitudinal_cwc_run(
        &request,
        &accepted,
        SNAPSHOT_ID,
        cutoff(),
        &permuted_rows(),
        "2026-08-02T00:00:00Z",
    )
    .expect("permuted execution");

    assert_eq!(
        permuted.artifact, baseline.artifact,
        "CWC artifact changed when only evidence enumeration order changed"
    );
    assert_eq!(
        permuted.terminal_result, baseline.terminal_result,
        "digest-bound terminal result changed when only evidence enumeration order changed"
    );
}
