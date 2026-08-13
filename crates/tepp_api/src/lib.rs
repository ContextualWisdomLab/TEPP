#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Versioned TEPP service DTOs, error envelopes, and export contracts.
//!
//! These pure wire contracts let TEPP operate standalone and as a modular CWL
//! component without sharing application tables. Domain estimation remains in
//! scientific crates; this crate only defines fail-closed interchange shapes.

mod analysis_run;
mod authorization;
mod envelope;
mod error;
mod export;
mod org_github;
mod wire;

/// Analysis-run contract version constant.
pub use analysis_run::ANALYSIS_RUN_CONTRACT_VERSION;
/// Accepted analysis-run response.
pub use analysis_run::AnalysisRunAccepted;
/// Analysis-run create request.
pub use analysis_run::AnalysisRunRequest;
/// Default analysis-run payload byte limit.
pub use analysis_run::DEFAULT_ANALYSIS_RUN_BYTE_LIMIT;
/// Idempotent request equality helper.
pub use analysis_run::requests_are_idempotent_matches;
/// Content-redacting error envelope.
pub use envelope::ErrorEnvelope;
/// Fail-closed API errors.
pub use error::ApiError;
/// Export contract version constant.
pub use export::EXPORT_CONTRACT_VERSION;
/// Minimal `GraphML` export builder.
pub use export::GraphMlExport;
/// `JSON-LD` export envelope.
pub use export::JsonLdExport;
/// Reproducibility manifest.
pub use export::ReproducibilityManifest;

/// Analytical export purpose.
pub use authorization::AnalyticalPurpose;
/// Purpose-bound export authorization decision.
pub use authorization::ExportAuthorizationDecision;
/// Purpose-bound export authorization request.
pub use authorization::ExportAuthorizationRequest;
/// Authorize an export under purpose-bound policy.
pub use authorization::authorize_export;
/// Fail closed when an export decision is denied.
pub use authorization::require_export_allowed;
/// Org reusable-workflow contract version.
pub use org_github::ORG_GITHUB_WORKFLOW_CONTRACT_VERSION;
/// Organization control-plane repository identity.
pub use org_github::ORG_GITHUB_WORKFLOW_OWNER;
/// Fail-closed org reusable-workflow binding.
pub use org_github::OrgGithubWorkflowBinding;
/// Authority an org reusable workflow may hold.
pub use org_github::OrgWorkflowAuthority;
/// Bind an org reusable workflow as CI/review/security only.
pub use org_github::bind_org_github_workflow;
/// Check conclusions never promote scientific claims.
pub use org_github::refuse_check_conclusion_as_scientific_claim;
/// Refuse Copilot, GitHub, or review-agent secret names.
pub use org_github::refuse_org_workflow_secret;
/// Org workflows never receive application-table access.
pub use org_github::refuse_org_workflow_table_access;
