#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Versioned TEPP service DTOs, error envelopes, and export contracts.
//!
//! These pure wire contracts let TEPP operate standalone and as a modular CWL
//! component without sharing application tables. Domain estimation remains in
//! scientific crates; this crate only defines fail-closed interchange shapes.
//! naruon HTTP interchange is a versioned `https` POST to analysis-run and
//! export paths; table-access URLs, review/Copilot/NIM/proxy headers, and
//! lexical inference claims fail closed. A loopback live listener proves
//! those POSTs over TCP without claiming production TLS (ADR 0011).

mod analysis_run;
mod authorization;
mod envelope;
mod error;
mod export;
mod naruon_http;
mod naruon_live;
mod orchestration;
mod orchestrator_http;
mod org_github;
mod provider_payload;
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
/// Versioned analysis-run path naruon may call.
pub use naruon_http::NARUON_ANALYSIS_RUN_PATH;
/// Versioned export path naruon may call.
pub use naruon_http::NARUON_EXPORT_PATH;
/// Allowed TEPP inference method code naruon may claim.
pub use naruon_http::NARUON_TEPP_INFERENCE_METHOD;
/// Fail-closed HTTP exchange naruon may send to TEPP.
pub use naruon_http::NaruonHttpExchange;
/// Build a naruon analysis-run create exchange.
pub use naruon_http::naruon_analysis_run_exchange;
/// Build an analysis-run exchange and refuse credential headers.
pub use naruon_http::naruon_analysis_run_exchange_with_headers;
/// Build a naruon export-authorization exchange.
pub use naruon_http::naruon_export_exchange;
/// Refuse lexical heuristics as TEPP inference claims.
pub use naruon_http::naruon_may_claim_tepp_inference;
/// Maximum live HTTP header-block bytes.
pub use naruon_live::NARUON_LIVE_HEADER_BYTE_LIMIT;
/// Maximum live HTTP header count.
pub use naruon_live::NARUON_LIVE_HEADER_COUNT_LIMIT;
/// Accepted-stream read/write deadline.
pub use naruon_live::NARUON_LIVE_IO_TIMEOUT;
/// HTTP/1.1 response from the naruon live listener.
pub use naruon_live::NaruonLiveResponse;
/// Loopback live HTTP/1.1 service for naruon POSTs.
pub use naruon_live::NaruonLiveService;
/// Comparable-budget ablation record.
pub use orchestration::BudgetAblationRecord;
/// Credential-free contextual-orchestrator binding.
pub use orchestration::ContextualOrchestratorBinding;
/// Document attempt to override TEPP orchestration authority.
pub use orchestration::DocumentControlAttempt;
/// Bounded interpretation task kind.
pub use orchestration::InterpretationTaskKind;
/// Maximum access capabilities on one orchestration request.
pub use orchestration::MAX_ORCHESTRATION_ACCESS_ENTRIES;
/// Maximum UTF-8 bytes in one orchestration access token.
pub use orchestration::MAX_ORCHESTRATION_ACCESS_TOKEN_BYTES;
/// Maximum billable token budget on one orchestration request.
pub use orchestration::MAX_ORCHESTRATION_TOKEN_BUDGET;
/// Orchestration contract version for contextual-orchestrator bindings.
pub use orchestration::ORCHESTRATION_CONTRACT_VERSION;
/// Versioned TEPP orchestration policy identity.
pub use orchestration::ORCHESTRATION_POLICY_VERSION;
/// Versioned orchestration mode.
pub use orchestration::OrchestrationMode;
/// Governed orchestration plan.
pub use orchestration::OrchestrationPlan;
/// Orchestration router request.
pub use orchestration::OrchestrationRequest;
/// Orchestration role identity.
pub use orchestration::OrchestrationRole;
/// Role-specific reasoning effort.
pub use orchestration::ReasoningEffort;
/// Role plus recorded reasoning effort.
pub use orchestration::RoleAssignment;
/// Bind a plan for contextual-orchestrator execution.
pub use orchestration::bind_contextual_orchestrator;
/// Record a comparable-budget ablation against a direct baseline.
pub use orchestration::record_budget_ablation;
/// Route a task onto a versioned orchestration plan.
pub use orchestration::route_orchestration;
/// Maximum orchestrator JSON body size.
pub use orchestrator_http::MAX_ORCHESTRATOR_BODY_BYTES;
/// Maximum orchestrator idempotency-key size.
pub use orchestrator_http::MAX_ORCHESTRATOR_IDEMPOTENCY_KEY_BYTES;
/// Versioned interpretation-run path.
pub use orchestrator_http::ORCHESTRATOR_INTERPRETATION_PATH;
/// Credential-free orchestrator HTTPS exchange.
pub use orchestrator_http::OrchestratorHttpExchange;
/// Build an interpretation request for contextual-orchestrator.
pub use orchestrator_http::orchestrator_interpretation_exchange;
/// Orchestrator output is never scientific acceptance.
pub use orchestrator_http::refuse_orchestrator_as_scientific_acceptance;
/// Refuse repository-write or review-agent secret names.
pub use orchestrator_http::refuse_repository_write_secret;
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
/// Elevated re-identification result.
pub use provider_payload::DisclosedIdentityMapping;
/// Separately protected identity mapping.
pub use provider_payload::IdentityMappingRecord;
/// Minimized provider payload without direct identity.
pub use provider_payload::MinimizedProviderPayload;
/// Log-safe provider disclosure record.
pub use provider_payload::ProviderDisclosureLog;
/// Evidence offered to a model provider.
pub use provider_payload::ProviderEvidenceOffer;
/// Time-bounded purpose grant.
pub use provider_payload::PurposeGrant;
/// Redacted re-identification decision outcome.
pub use provider_payload::ReidentificationAuditOutcome;
/// Redacted append-only re-identification audit record.
pub use provider_payload::ReidentificationAuditRecord;
/// Append-only persistence port for re-identification audit evidence.
pub use provider_payload::ReidentificationAuditSink;
/// Disclose a mapping on the elevated scientific path.
pub use provider_payload::disclose_identity_mapping;
/// Minimize evidence for a model provider.
pub use provider_payload::minimize_provider_payload;
