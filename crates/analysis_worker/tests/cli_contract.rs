//! Executable trust-boundary coverage without a live database.

use analysis_worker::{AnalysisWorkerInput, WORKER_INPUT_CONTRACT_VERSION, WorkerEvidenceUnit};
use std::{fs, process::Command};
use uuid::Uuid;

#[test]
fn executable_reaches_the_fail_closed_live_transport_boundary() {
    let mut input = AnalysisWorkerInput {
        contract_version: WORKER_INPUT_CONTRACT_VERSION,
        reproducibility_manifest_id: Uuid::nil(),
        snapshot_id: "snapshot-1".into(),
        source_snapshot_sha256: String::new(),
        evidence_units: vec![WorkerEvidenceUnit {
            evidence_id: "evidence-1".into(),
            event_time: "2026-08-27T00:00:00Z".into(),
            available_time: "2026-08-27T00:00:00Z".into(),
            membership_count: 1,
        }],
    };
    input.source_snapshot_sha256 = input.evidence_digest().expect("digest");
    let path = std::env::temp_dir().join(format!("tepp-worker-{}.json", Uuid::now_v7()));
    fs::write(&path, input.to_json().expect("input JSON")).expect("temporary input");

    let output = Command::new(env!("CARGO_BIN_EXE_analysis_worker"))
        .args([
            Uuid::nil().to_string(),
            Uuid::nil().to_string(),
            path.to_string_lossy().into_owned(),
            "2026-08-28T00:00:00Z".into(),
        ])
        .env("DATABASE_URL", "postgres://127.0.0.1:1/tepp_no_listener")
        .env("TEPP_CODE_COMMIT_SHA", "a".repeat(40))
        .env("TEPP_DEPENDENCY_LOCK_SHA256", "b".repeat(64))
        .output()
        .expect("worker process");
    fs::remove_file(path).expect("remove temporary input");

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
}
