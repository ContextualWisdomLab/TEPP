#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Versioned TEPP service DTOs, error envelopes, and export contracts.
//!
//! These pure wire contracts let TEPP operate standalone and as a modular CWL
//! component without sharing application tables. Domain estimation remains in
//! scientific crates; this crate only defines fail-closed interchange shapes.
//! Naruon and `LineageWeave` use the versioned analysis-run contract; `LineageWeave`
//! may also request a cutoff-safe project-history projection from explicit
//! source evidence. Naruon owns the current purpose-bound export adapter.
//! Loopback listeners prove the HTTP boundary without claiming production TLS,
//! causality, or completed psychometric model results.

mod analysis_result;
mod analysis_run;
mod analysis_run_cancel_http;
mod analysis_run_collection_http;
mod analysis_run_idempotency_lookup_http;
mod analysis_run_live;
mod analysis_run_retry_http;
mod analysis_run_retry_lineage_http;
mod analysis_run_retry_parent_http;
mod analysis_run_status_http;
mod analysis_run_stored_request_http;
mod authorization;
mod corpus_split_manifest;
mod envelope;
mod error;
mod export;
mod lineage_criterion_anchor;
mod lineage_pair_criterion;
mod lineageweave_http;
mod live_http;
mod naruon_http;
mod naruon_live;
mod orchestration;
mod project_history;
mod project_journey;
mod provider_payload;
mod temporal_context;
mod wire;

/// Terminal analysis-result contract version constant.
pub use analysis_result::ANALYSIS_RESULT_CONTRACT_VERSION;
/// Bounded identity-free terminal result summary.
pub use analysis_result::AnalysisResultSummary;
/// Request-bound terminal analysis outcome.
pub use analysis_result::AnalysisRunTerminalResult;
/// Canonical terminal analysis-run state.
pub use analysis_result::AnalysisRunTerminalState;
/// Default terminal analysis-result payload byte limit.
pub use analysis_result::DEFAULT_ANALYSIS_RESULT_BYTE_LIMIT;
/// Require exact terminal result binding to request and accepted receipt.
pub use analysis_result::require_terminal_binding;
/// Compare a terminal result with an accepted receipt.
pub use analysis_result::terminal_result_matches_accepted;
/// Compare a terminal result with its submitted request.
pub use analysis_result::terminal_result_matches_request;
/// Analysis-run contract version constant.
pub use analysis_run::ANALYSIS_RUN_CONTRACT_VERSION;
/// Analysis-run status/read contract version constant.
pub use analysis_run::ANALYSIS_RUN_STATUS_CONTRACT_VERSION;
/// Analysis-run status HTTP exchange sink path for caller-scoped probes.
pub use analysis_run::ANALYSIS_RUN_STATUS_PATH;
/// Accepted analysis-run response.
pub use analysis_run::AnalysisRunAccepted;
/// Analysis-run create request.
pub use analysis_run::AnalysisRunRequest;
/// Typed analysis-run status/read response.
pub use analysis_run::AnalysisRunStatus;
/// Analysis-run status/read lifecycle state.
pub use analysis_run::AnalysisRunStatusState;
/// Default analysis-run payload byte limit.
pub use analysis_run::DEFAULT_ANALYSIS_RUN_BYTE_LIMIT;
/// Idempotent request equality helper.
pub use analysis_run::requests_are_idempotent_matches;
/// Require exact status binding to a request and accepted receipt.
pub use analysis_run::require_status_binding;
/// Analysis-run cancel contract version constant.
pub use analysis_run_cancel_http::ANALYSIS_RUN_CANCEL_CONTRACT_VERSION;
/// Maximum opaque run identity length on the cancel path.
pub use analysis_run_cancel_http::ANALYSIS_RUN_CANCEL_ID_MAX_LEN;
/// Versioned analysis-run cancel request.
pub use analysis_run_cancel_http::AnalysisRunCancelRequest;
/// Build a Naruon analysis-run cancel exchange.
pub use analysis_run_cancel_http::naruon_analysis_run_cancel_exchange;
/// Refuse scientific-metric keys on a cancel payload.
pub use analysis_run_cancel_http::refuse_metrics_on_cancel_payload;
/// Analysis-run collection contract version constant.
pub use analysis_run_collection_http::ANALYSIS_RUN_COLLECTION_CONTRACT_VERSION;
/// Maximum exclusive collection cursor length.
pub use analysis_run_collection_http::ANALYSIS_RUN_COLLECTION_CURSOR_MAX_LEN;
/// Default collection page size.
pub use analysis_run_collection_http::ANALYSIS_RUN_COLLECTION_DEFAULT_LIMIT;
/// Maximum collection page size.
pub use analysis_run_collection_http::ANALYSIS_RUN_COLLECTION_MAX_LIMIT;
/// Versioned metric-free analysis-run collection page.
pub use analysis_run_collection_http::AnalysisRunCollection;
/// One metric-free collection row.
pub use analysis_run_collection_http::AnalysisRunCollectionItem;
/// True when the path is exactly the analysis-run collection resource.
pub use analysis_run_collection_http::is_analysis_run_collection_path;
/// Build a Naruon analysis-run collection GET exchange.
pub use analysis_run_collection_http::naruon_analysis_run_collection_exchange;
/// Parse the exclusive collection page cursor.
pub use analysis_run_collection_http::parse_collection_page_cursor;
/// Parse the collection page limit.
pub use analysis_run_collection_http::parse_collection_page_limit;
/// Refuse scientific-metric keys on a collection payload.
pub use analysis_run_collection_http::refuse_metrics_on_collection_payload;
/// Analysis-run idempotency-lookup contract version constant.
pub use analysis_run_idempotency_lookup_http::ANALYSIS_RUN_IDEMPOTENCY_LOOKUP_CONTRACT_VERSION;
/// Maximum opaque idempotency-key length on the lookup path.
pub use analysis_run_idempotency_lookup_http::ANALYSIS_RUN_IDEMPOTENCY_LOOKUP_KEY_MAX_LEN;
/// Reserved collection-relative prefix for idempotency-key lookup.
pub use analysis_run_idempotency_lookup_http::ANALYSIS_RUN_IDEMPOTENCY_LOOKUP_PREFIX;
/// Versioned metric-free identity of one analysis run found by idempotency key.
pub use analysis_run_idempotency_lookup_http::AnalysisRunIdempotencyLookup;
/// Build a Naruon analysis-run idempotency-lookup GET exchange.
pub use analysis_run_idempotency_lookup_http::naruon_analysis_run_idempotency_lookup_exchange;
/// Refuse scientific-metric keys on an idempotency-lookup payload.
pub use analysis_run_idempotency_lookup_http::refuse_metrics_on_idempotency_lookup_payload;
/// Consumer-neutral loopback analysis-run service.
pub use analysis_run_live::AnalysisRunLiveService;
/// Analysis-run retry contract version constant.
pub use analysis_run_retry_http::ANALYSIS_RUN_RETRY_CONTRACT_VERSION;
/// Maximum opaque run identity length on the retry path.
pub use analysis_run_retry_http::ANALYSIS_RUN_RETRY_ID_MAX_LEN;
/// Versioned analysis-run retry request.
pub use analysis_run_retry_http::AnalysisRunRetryRequest;
/// Build a Naruon analysis-run retry exchange.
pub use analysis_run_retry_http::naruon_analysis_run_retry_exchange;
/// Refuse scientific-metric keys on a retry payload.
pub use analysis_run_retry_http::refuse_metrics_on_retry_payload;
/// Analysis-run retry-lineage contract version constant.
pub use analysis_run_retry_lineage_http::ANALYSIS_RUN_RETRY_LINEAGE_CONTRACT_VERSION;
/// Maximum opaque run identity length on the retry-lineage path.
pub use analysis_run_retry_lineage_http::ANALYSIS_RUN_RETRY_LINEAGE_ID_MAX_LEN;
/// Maximum number of direct retry children on one lineage page.
pub use analysis_run_retry_lineage_http::ANALYSIS_RUN_RETRY_LINEAGE_MAX_RETRIES;
/// Versioned metric-free retry lineage of one parent analysis run.
pub use analysis_run_retry_lineage_http::AnalysisRunRetryLineage;
/// One metric-free retry-lineage child row.
pub use analysis_run_retry_lineage_http::AnalysisRunRetryLineageItem;
/// Build a Naruon analysis-run retry-lineage GET exchange.
pub use analysis_run_retry_lineage_http::naruon_analysis_run_retry_lineage_exchange;
/// Refuse scientific-metric keys on a retry-lineage payload.
pub use analysis_run_retry_lineage_http::refuse_metrics_on_retry_lineage_payload;
/// Analysis-run retry-parent contract version constant.
pub use analysis_run_retry_parent_http::ANALYSIS_RUN_RETRY_PARENT_CONTRACT_VERSION;
/// Maximum opaque run identity length on the retry-parent path.
pub use analysis_run_retry_parent_http::ANALYSIS_RUN_RETRY_PARENT_ID_MAX_LEN;
/// Versioned metric-free retry parent of one analysis run.
pub use analysis_run_retry_parent_http::AnalysisRunRetryParent;
/// One metric-free retry-parent identity row.
pub use analysis_run_retry_parent_http::AnalysisRunRetryParentItem;
/// Build a Naruon analysis-run retry-parent GET exchange.
pub use analysis_run_retry_parent_http::naruon_analysis_run_retry_parent_exchange;
/// Refuse scientific-metric keys on a retry-parent payload.
pub use analysis_run_retry_parent_http::refuse_metrics_on_retry_parent_payload;
/// Analysis-run status HTTP exchange re-exports.
pub use analysis_run_status_http::{ANALYSIS_RUN_ID_MAX_LEN, naruon_analysis_run_status_exchange};
/// Analysis-run stored-request contract version constant.
pub use analysis_run_stored_request_http::ANALYSIS_RUN_STORED_REQUEST_CONTRACT_VERSION;
/// Maximum opaque run identity length on the stored-request path.
pub use analysis_run_stored_request_http::ANALYSIS_RUN_STORED_REQUEST_ID_MAX_LEN;
/// Versioned metric-free stored analysis-run create fields.
pub use analysis_run_stored_request_http::AnalysisRunStoredRequest;
/// Build a Naruon analysis-run stored-request GET exchange.
pub use analysis_run_stored_request_http::naruon_analysis_run_stored_request_exchange;
/// Refuse scientific-metric keys on a stored-request payload.
pub use analysis_run_stored_request_http::refuse_metrics_on_stored_request_payload;
/// Corpus-split leakage-audit contract version.
pub use corpus_split_manifest::CORPUS_SPLIT_MANIFEST_CONTRACT_VERSION;
/// Versioned corpus-split leakage-audit manifest.
pub use corpus_split_manifest::CorpusSplitManifest;
/// Train/validation/test partition identities for a split manifest.
pub use corpus_split_manifest::CorpusSplitPartitions;
/// Content-redacting error envelope.
pub use envelope::ErrorEnvelope;
/// Fail-closed API errors.
pub use error::ApiError;
/// TDT/CHRONOS model-artifact type for immutable persistence.
pub use event_core::INTERVAL_CONSISTENCY_ARTIFACT_TYPE;
/// Durable typed JSON and `GraphML` bounded-consistency artifact.
pub use event_core::{IntervalConsistencyArtifact, IntervalConsistencyArtifactRelation};
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
/// TEPP-owned criterion-validity outcome for one Event Lineage weight run.
pub use lineage_criterion_anchor::CriterionValidityStatus;
/// Default maximum lineage criterion-anchor artifact size.
pub use lineage_criterion_anchor::DEFAULT_LINEAGE_CRITERION_ANCHOR_BYTE_LIMIT;
/// Semantic version of the lineage criterion-anchor artifact.
pub use lineage_criterion_anchor::LINEAGE_CRITERION_ANCHOR_CONTRACT_VERSION;
/// Analysis-run model contract that requests a lineage criterion anchor.
pub use lineage_criterion_anchor::LINEAGE_CRITERION_MODEL_CONTRACT;
/// Analysis-run output profile that requests a lineage criterion anchor.
pub use lineage_criterion_anchor::LINEAGE_CRITERION_OUTPUT_PROFILE;
/// Terminal result-schema identity for a lineage criterion anchor.
pub use lineage_criterion_anchor::LINEAGE_CRITERION_RESULT_SCHEMA;
/// Versioned TEPP criterion-validity artifact for one Event Lineage weight run.
pub use lineage_criterion_anchor::LineageCriterionAnchor;
/// Default bounded posterior artifact size.
pub use lineage_pair_criterion::DEFAULT_LINEAGE_PAIR_CRITERION_BYTE_LIMIT;
/// Exact pair-criterion posterior schema consumed by fast-mlsirm.
pub use lineage_pair_criterion::LINEAGE_PAIR_CRITERION_POSTERIOR_SCHEMA;
/// Independent anchor basis for pair-criterion interpretation.
pub use lineage_pair_criterion::LineageAnchorBasis;
/// CPU or GPU execution receipt.
pub use lineage_pair_criterion::LineageComputeReceipt;
/// Method-derived CPU/GPU parity receipts.
pub use lineage_pair_criterion::LineageComputeReceipts;
/// Posterior draw-generation provenance.
pub use lineage_pair_criterion::LineageDrawProvenance;
/// One pair's criterion and event-time posterior draws.
pub use lineage_pair_criterion::LineagePairCriterionPosterior;
/// Complete TEPP pair-criterion producer artifact.
pub use lineage_pair_criterion::LineagePairCriterionPosteriorArtifact;
/// TDT/CHRONOS temporal inference provenance.
pub use lineage_pair_criterion::LineageTemporalProvenance;
/// Published `LineageWeave` modular-consumer identity.
pub use lineageweave_http::LINEAGEWEAVE_CONSUMER_CODE;
/// Published Naruon modular-consumer identity.
pub use lineageweave_http::NARUON_CONSUMER_CODE;
/// Build a `LineageWeave` analysis-run exchange without provider credentials.
pub use lineageweave_http::lineageweave_analysis_run_exchange;
/// Build a `LineageWeave` project-history exchange without provider credentials.
pub use lineageweave_http::lineageweave_project_history_exchange;
/// Build a credential-free `LineageWeave` temporal-context exchange.
pub use lineageweave_http::lineageweave_temporal_context_exchange;
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
/// Maximum posterior Project Journey artifact size.
pub use project_journey::DEFAULT_PROJECT_JOURNEY_BYTE_LIMIT;
/// Exact posterior Project Journey schema identity.
pub use project_journey::PROJECT_JOURNEY_POSTERIOR_SCHEMA;
/// One evidence-grounded journey event with event-time draws.
pub use project_journey::ProjectJourneyEventPosterior;
/// Complete posterior Project Journey graph.
pub use project_journey::ProjectJourneyPosteriorArtifact;
/// One posterior temporal dependency, branch, or transition.
pub use project_journey::ProjectJourneyRelationPosterior;
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
/// Temporal association claim boundary.
pub use temporal_context::TEMPORAL_ASSOCIATION_CLAIM_BOUNDARY;
/// Temporal-context contract version constant.
pub use temporal_context::TEMPORAL_CONTEXT_CONTRACT_VERSION;
/// Versioned temporal-context HTTP path.
pub use temporal_context::TEMPORAL_CONTEXT_PATH;
/// One opaque event in a temporal-context request.
pub use temporal_context::TemporalContextEvent;
/// One adjacent temporal relation.
pub use temporal_context::TemporalContextRelation;
/// Temporal-context request.
pub use temporal_context::TemporalContextRequest;
/// Temporal-context response.
pub use temporal_context::TemporalContextResponse;
/// One ordered event in a temporal-context response.
pub use temporal_context::TemporalContextTimelineEvent;
/// One non-causal transition-gap candidate.
pub use temporal_context::TemporalTransitionGapCandidate;
/// Build a cutoff-safe, non-causal temporal context.
pub use temporal_context::build_temporal_context;
