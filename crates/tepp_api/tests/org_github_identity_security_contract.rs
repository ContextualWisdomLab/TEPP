//! Org workflow identities must be canonical and immutable.

use tepp_api::{ApiError, bind_org_github_workflow};

const PINNED_WORKFLOW: &str = "ContextualWisdomLab/.github/.github/workflows/security-scan.yml@f070c504c1cb06891b800d7ab0cf6ac7d3cf8eae";

#[test]
fn canonical_workflow_requires_an_exact_owner_path_and_full_commit_sha() {
    let binding = bind_org_github_workflow(PINNED_WORKFLOW).expect("pinned org workflow");
    assert_eq!(binding.workflow_identity(), PINNED_WORKFLOW);

    for invalid in [
        "evil.example/ContextualWisdomLab/.github/.github/workflows/security-scan.yml@f070c504c1cb06891b800d7ab0cf6ac7d3cf8eae",
        "contextualwisdomlab/.github/.github/workflows/security-scan.yml@f070c504c1cb06891b800d7ab0cf6ac7d3cf8eae",
        "ContextualWisdomLab/.github/.github/workflows/security-scan.yml@main",
        "ContextualWisdomLab/.github/.github/workflows/security-scan.yml@f070c504",
        "ContextualWisdomLab/.github/.github/workflows/../security-scan.yml@f070c504c1cb06891b800d7ab0cf6ac7d3cf8eae",
        "ContextualWisdomLab/.github/.github/workflows/subdir/security-scan.yml@f070c504c1cb06891b800d7ab0cf6ac7d3cf8eae",
        "ContextualWisdomLab/.github/.github/workflows/security-scan.txt@f070c504c1cb06891b800d7ab0cf6ac7d3cf8eae",
        "ContextualWisdomLab/.github/.github/workflows/security-scan.yml@F070C504C1CB06891B800D7AB0CF6AC7D3CF8EAE",
    ] {
        assert_eq!(
            bind_org_github_workflow(invalid),
            Err(ApiError::AuthorizationDenied),
            "hostile or mutable workflow reference must fail closed: {invalid}"
        );
    }
}
