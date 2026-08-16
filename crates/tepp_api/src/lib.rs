#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Versioned TEPP service DTOs, error envelopes, and export contracts.
//!
//! These pure wire contracts let TEPP operate standalone and as a modular CWL
//! component without sharing application tables. Domain estimation remains in
//! scientific crates; this crate only defines fail-closed interchange shapes.
//! naruon HTTP interchange is a versioned `https` POST to analysis-run and
//! export paths; table-access URLs, review/Copilot/NIM headers, and lexical
//! inference claims fail closed (ADR 0011). A loopback-only live HTTP/1.1
//! listener serves those POSTs without TLS termination or table access.

mod analysis_run;
mod authorization;
mod envelope;
mod error;
mod export;
mod naruon_http;
mod naruon_live;
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
/// Maximum live HTTP header-block size in bytes.
pub use naruon_live::NARUON_LIVE_HEADER_BYTE_LIMIT;
/// Maximum live HTTP header count.
pub use naruon_live::NARUON_LIVE_HEADER_COUNT_LIMIT;
/// Read/write deadline applied to each accepted live stream.
pub use naruon_live::NARUON_LIVE_IO_TIMEOUT;
/// Loopback live HTTP/1.1 response.
pub use naruon_live::NaruonLiveResponse;
/// Loopback live HTTP/1.1 naruon listener.
pub use naruon_live::NaruonLiveService;
