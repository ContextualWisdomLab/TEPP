#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Versioned TEPP service DTOs, error envelopes, and export contracts.
//!
//! These pure wire contracts let TEPP operate standalone and as a modular CWL
//! component without sharing application tables. Domain estimation remains in
//! scientific crates; this crate only defines fail-closed interchange shapes.
//! Naruon and LineageWeave use the versioned analysis-run contract; LineageWeave
//! may also request a cutoff-safe project-history projection from explicit
//! source evidence. Naruon owns the current purpose-bound export adapter.
//! Loopback listeners prove the HTTP boundary without claiming production TLS,
//! causality, or completed psychometric model results.

mod analysis_run;
mod analysis_run_live;
mod authorization;
mod envelope;
mod error;
mod export;
mod lineageweave_http;
mod naruon_http;
mod naruon_live;
mod orchestration;
mod project_history;
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
/// Consumer-neutral loopback analysis-run service.
pub use analysis_run_live::AnalysisRunLiveService;
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
/// Published LineageWeave modular-consumer identity.
pub use lineageweave_http::LINEAGEWEAVE_CONSUMER_CODE;
/// Published Naruon modular-consumer identity.
pub use lineageweave_http::NARUON_CONSUMER_CODE;
/// Build a LineageWeave analysis-run exchange without provider credentials.
pub use lineageweave_http::lineageweave_analysis_run_exchange;
/// Build a LineageWeave project-history exchange without provider credentials.
pub use lineageweave_http::lineageweave_project_history_exchange;
/// Versioned analysis-run path modular consumers may call.
pub use naruon_http::NARUON_ANALYSIS_RUN_PATH;
/// Versioned export path Naruon may call.
pub use naruon_http::NARUON_EXPORT_PATH;
/// Allowed TEPP inference method code Naruon may claim.
pub use naruon_http::NARUON_TEPP_INFERENCE_METHOD;
/// Fail-closed HTTP exchange a modular consumer may send to TEPP.
pub use naruon_http::NaruonHttpExchange;
/// Build a Naruon analysis-run create exchange.
pub use naruon_http::naruon_analysis_run_exchange;
/// Build a Naruon analysis-run exchange and refuse credential headers.
pub use naruon_http::naruon_analysis_run_exchange_with_headers;
/// Build a Naruon export-authorization exchange.
pub use naruon_http::naruon_export_exchange;
/// Refuse lexical heuristics as TEPP inference claims.
pub use naruon_http::naruon_may_claim_tepp_inference;
/// Maximum live HTTP header-block bytes.
pub use naruon_live::NARUON_LIVE_HEADER_BYTE_LIMIT;
/// Maximum live HTTP header count.
pub use naruon_live::NARUON_LIVE_HEADER_COUNT_LIMIT;
/// Accepted-stream read/write deadline.
pub use naruon_live::NARUON_LIVE_IO_TIMEOUT;
/// HTTP/1.1 response from the loopback listener.
pub use naruon_live::NaruonLiveResponse;
/// Backward-compatible Naruon loopback HTTP/1.1 service.
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
/// Default maximum serialized project-history request bytes.
pub use project_history::DEFAULT_PROJECT_HISTORY_BYTE_LIMIT;
/// Default maximum project-history event count.
pub use project_history::DEFAULT_PROJECT_HISTORY_EVENT_LIMIT;
/// Supported project-history contract version.
pub use project_history::PROJECT_HISTORY_CONTRACT_VERSION;
/// Versioned project-history path.
pub use project_history::PROJECT_HISTORY_PATH;
/// Explicit source-grounded project event.
pub use project_history::ProjectHistoryEvent;
/// One non-causal temporal finding.
pub use project_history::ProjectHistoryFinding;
/// Project-history HTTP exchange.
pub use project_history::ProjectHistoryHttpExchange;
/// Deterministic TEPP project-history projection.
pub use project_history::ProjectHistoryProjection;
/// Versioned project-history request.
pub use project_history::ProjectHistoryRequest;
/// Build a cutoff-safe project-history projection.
pub use project_history::project_history_projection;
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
