//! Example payloads under `/examples` must parse through the live contracts.

use std::path::PathBuf;
use tepp_api::{
    AnalysisRunLiveService, AnalysisRunRequest, CorpusSplitManifest, NARUON_ANALYSIS_RUN_PATH,
    NaruonLiveService, ReproducibilityManifest,
};

fn repo_example(name: &str) -> String {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.pop();
    path.push("examples");
    path.push(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("missing example {}: {error}", path.display()))
}

#[test]
fn committed_examples_parse_through_live_contracts() {
    let run = AnalysisRunRequest::from_json(&repo_example("analysis_run_request_v1.json"))
        .expect("analysis example");
    assert_eq!(run.contract_version, 1);
    assert_eq!(run.tenant_workspace_id, "tenant-workspace-demo");

    let naruon =
        AnalysisRunRequest::from_json(&repo_example("naruon_modular_analysis_run_request_v1.json"))
            .expect("naruon example");
    assert_eq!(naruon.output_profile, "naruon-consumer-validation-report");

    let manifest =
        ReproducibilityManifest::from_json(&repo_example("reproducibility_manifest_v1.json"))
            .expect("manifest example");
    assert_eq!(manifest.engine_version, "0.1.0");

    let split = CorpusSplitManifest::from_json(&repo_example("corpus_split_manifest_v1.json"))
        .expect("split example");
    assert_eq!(split.excluded_unavailable_at_cutoff_count, 1);
}

#[test]
fn example_contracts_prove_live_idempotency_and_bound_loopback_identity() {
    let mut analysis_service = AnalysisRunLiveService::new();
    let run = AnalysisRunRequest::from_json(&repo_example("analysis_run_request_v1.json"))
        .expect("analysis example");
    let body = run.to_json().expect("run json");
    let request = format!(
        "POST {NARUON_ANALYSIS_RUN_PATH} HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: naruon\r\ntepp-contract-version: 1\r\nidempotency-key: {}\r\ncontent-length: {}\r\n\r\n{body}",
        run.idempotency_key,
        body.len()
    );
    let accepted = analysis_service.handle_http_request(&request);
    assert_eq!(accepted.status_code, 202);
    assert_eq!(
        analysis_service.handle_http_request(&request).body,
        accepted.body
    );
    let mut conflict = run.clone();
    conflict.snapshot_id.push_str("-changed");
    let conflict_body = conflict.to_json().expect("conflict json");
    let conflict_request = request
        .replace(
            &format!("content-length: {}", body.len()),
            &format!("content-length: {}", conflict_body.len()),
        )
        .replace(&body, &conflict_body);
    assert_eq!(
        analysis_service
            .handle_http_request(&conflict_request)
            .status_code,
        400
    );

    let service = NaruonLiveService::bind_loopback().expect("loopback bind");
    let bound = service.local_addr().expect("bound address");
    let bound_request = |host: &str| {
        format!(
            "POST {NARUON_ANALYSIS_RUN_PATH} HTTP/1.1\r\nHost: {host}\r\ncontent-type: application/json\r\ntepp-consumer: naruon\r\ntepp-contract-version: 1\r\nidempotency-key: {}\r\ncontent-length: {}\r\n\r\n{body}",
            run.idempotency_key,
            body.len()
        )
    };
    let mut bound_service = service;
    assert_eq!(
        bound_service
            .handle_http_request(&bound_request(&bound.to_string()))
            .status_code,
        202
    );
    assert_eq!(
        bound_service
            .handle_http_request(&bound_request(&bound.ip().to_string()))
            .status_code,
        202
    );
    let mut conflicting = run.clone();
    conflicting.snapshot_id.push_str("-changed");
    let conflicting_body = conflicting.to_json().expect("bound conflict json");
    let conflicting_request = bound_request(&bound.to_string())
        .replace(
            &format!("content-length: {}", body.len()),
            &format!("content-length: {}", conflicting_body.len()),
        )
        .replace(&body, &conflicting_body);
    assert_eq!(
        bound_service
            .handle_http_request(&conflicting_request)
            .status_code,
        400
    );
    assert_eq!(
        bound_service
            .handle_http_request(&bound_request("8.8.8.8"))
            .status_code,
        403
    );
}
