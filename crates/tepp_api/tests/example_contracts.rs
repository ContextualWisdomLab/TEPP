//! Example payloads under `/examples` must parse through the live contracts.

use std::path::PathBuf;
use tepp_api::{AnalysisRunRequest, ReproducibilityManifest};

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

    let manifest =
        ReproducibilityManifest::from_json(&repo_example("reproducibility_manifest_v1.json"))
            .expect("manifest example");
    assert_eq!(manifest.engine_version, "0.1.0");
}
