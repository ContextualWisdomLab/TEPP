//! Org-central `.github` reusable workflows own CI/review/security only.

use tepp_api::{
    ApiError, ORG_GITHUB_WORKFLOW_CONTRACT_VERSION, ORG_GITHUB_WORKFLOW_OWNER,
    OrgWorkflowAuthority, bind_org_github_workflow, refuse_check_conclusion_as_scientific_claim,
    refuse_org_workflow_secret, refuse_org_workflow_table_access,
};

const PINNED_SHA: &str = "f070c504c1cb06891b800d7ab0cf6ac7d3cf8eae";

#[test]
fn reusable_workflow_binds_as_ci_review_security_only() {
    let identity = format!(
        "ContextualWisdomLab/.github/.github/workflows/security-scan.yml@{PINNED_SHA}"
    );
    let binding = bind_org_github_workflow(&identity).expect("org reusable workflow");
    assert_eq!(
        binding.contract_version(),
        ORG_GITHUB_WORKFLOW_CONTRACT_VERSION
    );
    assert_eq!(ORG_GITHUB_WORKFLOW_CONTRACT_VERSION, 1);
    assert_eq!(ORG_GITHUB_WORKFLOW_OWNER, "ContextualWisdomLab/.github");
    assert!(
        binding
            .workflow_identity()
            .starts_with(ORG_GITHUB_WORKFLOW_OWNER)
    );
    assert!(
        binding
            .workflow_identity()
            .contains("workflows/security-scan.yml")
    );
    assert!(binding.workflow_identity().ends_with(PINNED_SHA));
    assert_eq!(binding.authority(), OrgWorkflowAuthority::CiReviewSecurity);
    assert_eq!(
        refuse_check_conclusion_as_scientific_claim("SUCCESS"),
        Err(ApiError::AuthorizationDenied)
    );
}

#[test]
fn table_access_and_hostile_workflow_refs_fail_closed() {
    for workflow_ref in [
        "",
        "   ",
        "postgres://tepp/application_table",
        "jdbc:postgresql://db/tepp",
        "sql.internal/tables",
        "tables.example/workflows/ci.yml",
    ] {
        assert_eq!(
            bind_org_github_workflow(workflow_ref),
            Err(if workflow_ref.trim().is_empty() {
                ApiError::InvalidWirePayload
            } else {
                ApiError::AuthorizationDenied
            })
        );
    }
    assert_eq!(
        refuse_org_workflow_table_access("postgres://tepp/application_table"),
        Err(ApiError::AuthorizationDenied)
    );
    assert_eq!(
        refuse_org_workflow_table_access(""),
        Err(ApiError::InvalidWirePayload)
    );
}

#[test]
fn review_agent_and_copilot_secrets_are_refused() {
    assert_eq!(
        refuse_org_workflow_secret("COPILOT_GITHUB_TOKEN"),
        Err(ApiError::AuthorizationDenied)
    );
    assert_eq!(
        refuse_org_workflow_secret("review-agent-github-token"),
        Err(ApiError::AuthorizationDenied)
    );
    refuse_org_workflow_secret("NVIDIA_NIM_API_KEY").expect("nim allowed as name");
    assert_eq!(
        refuse_org_workflow_secret(""),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        refuse_check_conclusion_as_scientific_claim(""),
        Err(ApiError::InvalidWirePayload)
    );
}
