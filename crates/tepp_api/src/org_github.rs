//! Versioned org-central `.github` reusable-workflow interchange.

use crate::ApiError;
use crate::wire::require_nonempty;

/// Contract version for org-central reusable-workflow bindings.
pub const ORG_GITHUB_WORKFLOW_CONTRACT_VERSION: u16 = 1;

/// Organization control-plane repository that may own reusable workflows.
pub const ORG_GITHUB_WORKFLOW_OWNER: &str = "ContextualWisdomLab/.github";

const ORG_GITHUB_WORKFLOW_PREFIX: &str = "ContextualWisdomLab/.github/.github/workflows/";

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

    /// Canonical reusable-workflow identity (`owner/.github/workflows/…@sha`).
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

/// Bind an immutable org reusable workflow as CI/review/security control only.
///
/// The identity must use the exact [`ORG_GITHUB_WORKFLOW_OWNER`] spelling, a
/// top-level `.github/workflows/*.yml` or `*.yaml` file, and a lowercase 40-hex
/// commit SHA. Mutable branches, tags, path traversal, nested workflow paths,
/// and look-alike owner prefixes fail closed. The workflow never becomes
/// scientific authority.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] for an empty reference and
/// [`ApiError::AuthorizationDenied`] for hostile, mutable, or non-org
/// identities.
pub fn bind_org_github_workflow(workflow_ref: &str) -> Result<OrgGithubWorkflowBinding, ApiError> {
    require_nonempty(workflow_ref)?;
    refuse_org_workflow_table_access(workflow_ref)?;
    require_org_workflow_identity(workflow_ref)?;
    Ok(OrgGithubWorkflowBinding {
        contract_version: ORG_GITHUB_WORKFLOW_CONTRACT_VERSION,
        workflow_identity: workflow_ref.to_owned(),
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
/// [`ApiError::AuthorizationDenied`] for every name except the NVIDIA NIM key.
pub fn refuse_org_workflow_secret(secret_name: &str) -> Result<(), ApiError> {
    require_nonempty(secret_name)?;
    let folded: String = secret_name
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .flat_map(char::to_lowercase)
        .collect();
    if folded == "nvidianimapikey" {
        Ok(())
    } else {
        Err(ApiError::AuthorizationDenied)
    }
}

fn require_org_workflow_identity(workflow_ref: &str) -> Result<(), ApiError> {
    if workflow_ref.trim() != workflow_ref {
        return Err(ApiError::AuthorizationDenied);
    }
    let Some(remainder) = workflow_ref.strip_prefix(ORG_GITHUB_WORKFLOW_PREFIX) else {
        return Err(ApiError::AuthorizationDenied);
    };
    let Some((workflow_file, commit_sha)) = remainder.split_once('@') else {
        return Err(ApiError::AuthorizationDenied);
    };
    if workflow_file.is_empty()
        || workflow_file.contains('/')
        || workflow_file.contains("..")
        || !workflow_file.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
        })
        || !std::path::Path::new(workflow_file)
            .extension()
            .is_some_and(|extension| {
                extension.eq_ignore_ascii_case("yml") || extension.eq_ignore_ascii_case("yaml")
            })
    {
        return Err(ApiError::AuthorizationDenied);
    }
    if commit_sha.len() != 40
        || !commit_sha
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(ApiError::AuthorizationDenied);
    }
    let lowered_file = workflow_file.to_ascii_lowercase();
    if lowered_file.contains("scientific") || lowered_file.contains("recovery") {
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

    const SHA: &str = "f070c504c1cb06891b800d7ab0cf6ac7d3cf8eae";

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
            require_org_workflow_identity(&format!(
                "ContextualWisdomLab/.github/.github/workflows/scientific-acceptance.yml@{SHA}"
            )),
            Err(ApiError::AuthorizationDenied)
        );
        assert_eq!(
            require_org_workflow_identity(&format!(
                "ContextualWisdomLab/.github/.github/workflows/recovery-claim.yml@{SHA}"
            )),
            Err(ApiError::AuthorizationDenied)
        );
        require_org_workflow_identity(&format!(
            "ContextualWisdomLab/.github/.github/workflows/security-scan.yml@{SHA}"
        ))
        .expect("org workflow");
        require_org_workflow_identity(&format!(
            "ContextualWisdomLab/.github/.github/workflows/security-scan.yaml@{SHA}"
        ))
        .expect("yaml org workflow");
        assert_eq!(
            refuse_org_workflow_secret("AWS_SECRET"),
            Err(ApiError::AuthorizationDenied)
        );
        refuse_org_workflow_table_access(&format!(
            "ContextualWisdomLab/.github/.github/workflows/security-scan.yml@{SHA}"
        ))
        .expect("not a table host");
        assert_eq!(
            refuse_check_conclusion_as_scientific_claim("FAILURE"),
            Err(ApiError::AuthorizationDenied)
        );
        let binding = bind_org_github_workflow(&format!(
            "ContextualWisdomLab/.github/.github/workflows/noema-review.yml@{SHA}"
        ))
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
            bind_org_github_workflow(&format!(
                "ContextualWisdomLab/.github/.github/workflows/scientific-acceptance.yml@{SHA}"
            )),
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
