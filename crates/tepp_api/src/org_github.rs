//! Versioned org-central `.github` reusable-workflow interchange.

use crate::ApiError;
use crate::wire::require_nonempty;

/// Contract version for org-central reusable-workflow bindings.
pub const ORG_GITHUB_WORKFLOW_CONTRACT_VERSION: u16 = 1;

/// Organization control-plane repository that may own reusable workflows.
pub const ORG_GITHUB_WORKFLOW_OWNER: &str = "ContextualWisdomLab/.github";

/// Authority an org reusable workflow may hold.
///
/// Scientific acceptance is intentionally absent: CI conclusions cannot
/// promote recovery, invariance, or implemented-main claims (ADR 0014).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrgWorkflowAuthority {
    /// CI, review, security, and release-control only.
    CiReviewSecurity,
}

/// Fail-closed binding of an org reusable workflow identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrgGithubWorkflowBinding {
    contract_version: u16,
    workflow_identity: String,
    authority: OrgWorkflowAuthority,
}

impl OrgGithubWorkflowBinding {
    /// Contract version accepted for this binding.
    #[must_use]
    pub const fn contract_version(&self) -> u16 {
        self.contract_version
    }

    /// Canonical reusable-workflow identity (`owner/.github/workflows/…@ref`).
    #[must_use]
    pub fn workflow_identity(&self) -> &str {
        &self.workflow_identity
    }

    /// Bound authority; always [`OrgWorkflowAuthority::CiReviewSecurity`].
    #[must_use]
    pub const fn authority(&self) -> OrgWorkflowAuthority {
        self.authority
    }
}

/// Bind an org reusable workflow as CI/review/security control only.
///
/// The reference must name [`ORG_GITHUB_WORKFLOW_OWNER`] and a `workflows/`
/// path. Table-access hosts and scientific-claim names fail closed.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] for an empty reference and
/// [`ApiError::AuthorizationDenied`] for hostile or non-org identities.
pub fn bind_org_github_workflow(workflow_ref: &str) -> Result<OrgGithubWorkflowBinding, ApiError> {
    require_nonempty(workflow_ref)?;
    refuse_org_workflow_table_access(workflow_ref)?;
    require_org_workflow_identity(workflow_ref)?;
    Ok(OrgGithubWorkflowBinding {
        contract_version: ORG_GITHUB_WORKFLOW_CONTRACT_VERSION,
        workflow_identity: workflow_ref.trim().to_owned(),
        authority: OrgWorkflowAuthority::CiReviewSecurity,
    })
}

/// GitHub Check conclusions never promote scientific or implemented-main claims.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] for an empty conclusion and
/// [`ApiError::AuthorizationDenied`] for every nonempty conclusion.
pub fn refuse_check_conclusion_as_scientific_claim(conclusion: &str) -> Result<(), ApiError> {
    require_nonempty(conclusion)?;
    Err(ApiError::AuthorizationDenied)
}

/// Org workflows never receive TEPP application-table access.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] for an empty target and
/// [`ApiError::AuthorizationDenied`] for SQL, JDBC, or table hosts.
pub fn refuse_org_workflow_table_access(target: &str) -> Result<(), ApiError> {
    require_nonempty(target)?;
    let lowered = target.to_ascii_lowercase();
    if ["postgres", "jdbc", "sql", "tables"]
        .iter()
        .any(|needle| lowered.contains(needle))
    {
        return Err(ApiError::AuthorizationDenied);
    }
    Ok(())
}

/// Refuse repository-write or review-agent secret names on this port.
///
/// `NVIDIA_NIM_API_KEY` is the only allowed model-credential name.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] for an empty name and
/// [`ApiError::AuthorizationDenied`] for Copilot, GitHub, or review-agent names.
pub fn refuse_org_workflow_secret(secret_name: &str) -> Result<(), ApiError> {
    require_nonempty(secret_name)?;
    let folded: String = secret_name
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .flat_map(char::to_lowercase)
        .collect();
    if folded == "nvidianimapikey" {
        return Ok(());
    }
    if folded.contains("copilot") || folded.contains("github") || folded.contains("reviewagent") {
        return Err(ApiError::AuthorizationDenied);
    }
    Err(ApiError::AuthorizationDenied)
}

fn require_org_workflow_identity(workflow_ref: &str) -> Result<(), ApiError> {
    let folded = workflow_ref.to_ascii_lowercase();
    if !folded.contains("contextualwisdomlab/.github") || !folded.contains("workflows/") {
        return Err(ApiError::AuthorizationDenied);
    }
    if folded.contains("scientific") || folded.contains("recovery") {
        return Err(ApiError::AuthorizationDenied);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        OrgGithubWorkflowBinding, OrgWorkflowAuthority, bind_org_github_workflow,
        refuse_check_conclusion_as_scientific_claim, refuse_org_workflow_secret,
        refuse_org_workflow_table_access, require_org_workflow_identity,
    };
    use crate::ApiError;

    #[test]
    fn identity_and_secret_branches_are_covered() {
        assert_eq!(
            require_org_workflow_identity("other/.github/workflows/ci.yml"),
            Err(ApiError::AuthorizationDenied)
        );
        assert_eq!(
            require_org_workflow_identity("ContextualWisdomLab/.github/readme.md"),
            Err(ApiError::AuthorizationDenied)
        );
        assert_eq!(
            require_org_workflow_identity(
                "ContextualWisdomLab/.github/.github/workflows/scientific-acceptance.yml"
            ),
            Err(ApiError::AuthorizationDenied)
        );
        assert_eq!(
            require_org_workflow_identity(
                "ContextualWisdomLab/.github/.github/workflows/recovery-claim.yml"
            ),
            Err(ApiError::AuthorizationDenied)
        );
        require_org_workflow_identity(
            "ContextualWisdomLab/.github/.github/workflows/security-scan.yml@main",
        )
        .expect("org workflow");
        assert_eq!(
            refuse_org_workflow_secret("AWS_SECRET"),
            Err(ApiError::AuthorizationDenied)
        );
        refuse_org_workflow_table_access(
            "ContextualWisdomLab/.github/.github/workflows/security-scan.yml@main",
        )
        .expect("not a table host");
        assert_eq!(
            refuse_check_conclusion_as_scientific_claim("FAILURE"),
            Err(ApiError::AuthorizationDenied)
        );
        let binding = bind_org_github_workflow(
            "ContextualWisdomLab/.github/.github/workflows/noema-review.yml@main",
        )
        .expect("review workflow");
        assert_eq!(binding.authority(), OrgWorkflowAuthority::CiReviewSecurity);
        let constructed = OrgGithubWorkflowBinding {
            contract_version: 1,
            workflow_identity: "constructed".into(),
            authority: OrgWorkflowAuthority::CiReviewSecurity,
        };
        assert_eq!(constructed.workflow_identity(), "constructed");
        assert_eq!(constructed.contract_version(), 1);
        assert_eq!(
            bind_org_github_workflow("other/.github/workflows/ci.yml"),
            Err(ApiError::AuthorizationDenied)
        );
        assert_eq!(
            bind_org_github_workflow(
                "ContextualWisdomLab/.github/.github/workflows/scientific-acceptance.yml"
            ),
            Err(ApiError::AuthorizationDenied)
        );
        assert_eq!(
            refuse_org_workflow_secret("REVIEW_AGENT"),
            Err(ApiError::AuthorizationDenied)
        );
        assert_eq!(
            refuse_org_workflow_secret("GITHUB_TOKEN"),
            Err(ApiError::AuthorizationDenied)
        );
    }
}
