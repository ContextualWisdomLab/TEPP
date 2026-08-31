//! GAP-003A scientific-acceptance validation-run contract.

use analysis_engine::{
    AnalysisCorpus, AnalysisEngineError, AnalysisEvidenceUnit, RecoveryObservation,
    SCIENTIFIC_ACCEPTANCE_OUTPUT_PROFILE, SCIENTIFIC_ACCEPTANCE_SCHEMA_VERSION,
    VALIDATION_CPU_F64_MODEL, complete_validation_run, submit_validation_run,
};
use temporal_core::{AvailableTime, EventTime};
use tepp_api::AnalysisRunRequest;

fn request() -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "idem-validation-contract".into(),
        tenant_workspace_id: "tenant-workspace-contract".into(),
        snapshot_id: "snapshot-contract".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: VALIDATION_CPU_F64_MODEL.into(),
        output_profile: SCIENTIFIC_ACCEPTANCE_OUTPUT_PROFILE.into(),
    }
}

fn unit(id: &str, available: &str) -> AnalysisEvidenceUnit {
    AnalysisEvidenceUnit::new(
        id,
        EventTime::parse_rfc3339("2026-07-01T00:00:00Z").expect("event"),
        AvailableTime::parse_rfc3339(available).expect("available"),
        1,
    )
    .expect("unit")
}

#[test]
fn durable_run_binds_cutoff_eligible_evidence_and_emits_acceptance_evidence() {
    let corpus = AnalysisCorpus::new(
        "snapshot-contract",
        vec![
            unit("evidence-2", "2026-07-20T00:00:00Z"),
            unit("evidence-1", "2026-07-10T00:00:00Z"),
            unit("future", "2026-08-02T00:00:00Z"),
        ],
    )
    .expect("corpus");
    let receipt = submit_validation_run(&request(), &corpus, 42).expect("submit");
    assert_eq!(receipt.eligible_evidence_count(), 2);
    assert!(!receipt.to_json().expect("json").contains("rmse"));
    let observation = RecoveryObservation::new(
        &receipt,
        "contract-recovery",
        vec![0.70, 0.55, 0.40, -0.20, 0.85],
        vec![0.70, 0.55, 0.40, -0.20, 0.85],
        vec![0.50, 0.35, 0.20, -0.40, 0.65],
        vec![0.90, 0.75, 0.60, 0.00, 1.00],
        vec![1.0, 2.0, 3.0, 4.0, 5.0],
        vec![1.1, 1.9, 3.2, 3.8, 5.1],
        3.0,
        false,
    )
    .expect("observation");
    let evidence =
        complete_validation_run(&receipt, &request(), &corpus, &observation).expect("complete");
    assert_eq!(
        evidence.schema_version(),
        SCIENTIFIC_ACCEPTANCE_SCHEMA_VERSION
    );
    assert_eq!(evidence.run_id(), receipt.run_id());
    assert_eq!(evidence.recovery_sha256().len(), 64);
    assert!(evidence.se_gate_accepted());
    assert!(evidence.to_json().expect("json").contains("rmse"));
}

#[test]
fn llm_authored_recovery_and_cutoff_empty_corpora_fail_closed() {
    let corpus = AnalysisCorpus::new(
        "snapshot-contract",
        vec![unit("only-future", "2026-08-02T00:00:00Z")],
    )
    .expect("corpus");
    assert_eq!(
        submit_validation_run(&request(), &corpus, 1),
        Err(AnalysisEngineError::NoEligibleEvidence)
    );
    let eligible = AnalysisCorpus::new(
        "snapshot-contract",
        vec![unit("evidence-1", "2026-07-10T00:00:00Z")],
    )
    .expect("eligible");
    let receipt = submit_validation_run(&request(), &eligible, 1).expect("submit");
    let llm = RecoveryObservation::new(
        &receipt,
        "llm-contract",
        vec![1.0, 2.0, 3.0],
        vec![1.0, 2.0, 3.0],
        vec![0.5, 1.5, 2.5],
        vec![1.5, 2.5, 3.5],
        vec![1.0, 2.0, 3.0],
        vec![1.0, 2.0, 3.0],
        3.0,
        true,
    )
    .expect("llm");
    assert_eq!(
        complete_validation_run(&receipt, &request(), &eligible, &llm),
        Err(AnalysisEngineError::LlmAuthoredRecovery)
    );
}

#[test]
fn recovery_from_a_different_run_or_tenant_fails_closed() {
    let first = AnalysisCorpus::new(
        "snapshot-contract",
        vec![unit("evidence-1", "2026-07-10T00:00:00Z")],
    )
    .expect("first");
    let second = AnalysisCorpus::new(
        "snapshot-contract",
        vec![
            unit("evidence-1", "2026-07-10T00:00:00Z"),
            unit("evidence-2", "2026-07-20T00:00:00Z"),
        ],
    )
    .expect("second");
    let receipt_a = submit_validation_run(&request(), &first, 1).expect("a");
    let receipt_b = submit_validation_run(&request(), &second, 1).expect("b");
    let foreign = RecoveryObservation::new(
        &receipt_a,
        "foreign-recovery",
        vec![1.0, 2.0, 3.0],
        vec![1.0, 2.0, 3.0],
        vec![0.5, 1.5, 2.5],
        vec![1.5, 2.5, 3.5],
        vec![1.0, 2.0, 3.0],
        vec![1.0, 2.0, 3.0],
        3.0,
        false,
    )
    .expect("foreign");
    assert_eq!(
        complete_validation_run(&receipt_b, &request(), &second, &foreign),
        Err(AnalysisEngineError::BindingMismatch)
    );
    let mut other_tenant = request();
    other_tenant.tenant_workspace_id = "tenant-workspace-other".into();
    let other_receipt = submit_validation_run(&other_tenant, &first, 1).expect("tenant");
    assert_ne!(receipt_a.run_id(), other_receipt.run_id());
    assert_eq!(
        complete_validation_run(&other_receipt, &other_tenant, &first, &foreign),
        Err(AnalysisEngineError::BindingMismatch)
    );
}
