#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Versioned TEPP service DTOs, error envelopes, and export contracts.
//!
//! These pure wire contracts let TEPP operate standalone and as a modular CWL
//! component without sharing application tables. Domain estimation remains in
//! scientific crates; this crate only defines fail-closed interchange shapes.
//! Naruon and `LineageWeave` use the versioned analysis-run contract; Naruon also
//! owns the current purpose-bound export adapter. Loopback listeners prove the
//! HTTP boundary without claiming production TLS or completed model results.

mod analysis_run;
mod analysis_run_live;
mod authorization;
mod envelope;
mod error;
mod export;
mod lineageweave_http;
mod naruon_http;
mod naruon_live;
mod temporal_context;
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
/// Published `LineageWeave` modular-consumer identity.
pub use lineageweave_http::LINEAGEWEAVE_CONSUMER_CODE;
/// Published Naruon modular-consumer identity.
pub use lineageweave_http::NARUON_CONSUMER_CODE;
/// Build a credential-free `LineageWeave` analysis-run exchange.
pub use lineageweave_http::lineageweave_analysis_run_exchange;
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
