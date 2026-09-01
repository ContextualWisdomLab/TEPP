#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Loopback live HTTP/1.1 listener for contextual-orchestrator interpretation.
//!
//! The listener accepts `POST /v1/interpretation-runs` and
//! `GET /v1/interpretation-runs` on loopback only. Accepted output is always
//! hypothetical and never scientific authority. Collection GET enumerates
//! metric-free identities so operators do not guess idempotency keys.
//! GET-by-id returns one of those identities without POST replay.
//! `POST /v1/interpretation-runs/{idempotency_key}/cancel` removes one
//! accepted identity from the in-memory registry.
//! `tepp-interpretation-run-cancel` mints that POST onto spawned loopback TCP.
//! Table-access hosts, review/Copilot/GitHub credentials, and
//! `COPILOT_GITHUB_TOKEN` fail closed. This crate does not implement TLS
//! termination or call a model provider (ADR 0010; ADR 0011). The published
//! `tepp-interpretation-runs` CLI mints typed contextual-orchestrator
//! interpretation-run POST exchanges onto spawned `tepp-orchestrator-loopback`
//! TCP.

mod error;
mod http;
mod interpretation_run_cancel_cli;
mod interpretation_run_cancel_http;
mod interpretation_run_cli;
mod interpretation_run_collection_http;
mod interpretation_run_retrieval_http;
mod mode;
mod request;
mod service;

/// Fail-closed orchestrator live-listener errors.
pub use error::OrchestratorLiveError;
/// Loopback live HTTP/1.1 response.
pub use http::OrchestratorLiveResponse;
/// Maximum live HTTP header-block size in bytes.
pub use http::LIVE_HEADER_BYTE_LIMIT;
/// Maximum live HTTP header count.
pub use http::LIVE_HEADER_COUNT_LIMIT;
/// Compose one HTTP/1.1 cancel POST from the typed consumer exchange.
pub use interpretation_run_cancel_cli::compose_interpretation_run_cancel_cli_http;
/// Dispatch one cancel CLI invocation against an in-process listener.
pub use interpretation_run_cancel_cli::dispatch_interpretation_run_cancel_cli;
/// Execute one cancel CLI invocation over loopback TCP.
pub use interpretation_run_cancel_cli::execute_interpretation_run_cancel_cli;
/// Render a typed cancel POST exchange as HTTP/1.1 for a loopback listener.
pub use interpretation_run_cancel_cli::loopback_http1_from_interpretation_run_cancel_exchange;
/// Read stdin leftover bytes; cancel POST admits empty.
pub use interpretation_run_cancel_cli::read_interpretation_run_cancel_cli_stdin;
/// Filter CLI stdout so cancel never prints scientific acceptance.
pub use interpretation_run_cancel_cli::render_interpretation_run_cancel_cli_stdout;
/// One operator CLI invocation against a loopback cancel listener.
pub use interpretation_run_cancel_cli::InterpretationRunCancelCliInvocation;
/// Supported operator verbs for the loopback interpretation-run cancel CLI.
pub use interpretation_run_cancel_cli::InterpretationRunCancelCliVerb;
/// Build a credential-free contextual-orchestrator cancel exchange.
pub use interpretation_run_cancel_http::contextual_orchestrator_interpretation_run_cancel_exchange;
/// Extract the opaque idempotency key from a cancel path.
pub use interpretation_run_cancel_http::interpretation_run_cancel_path_id;
/// Refuse metric, evidence, and causal-score keys on cancel JSON.
pub use interpretation_run_cancel_http::refuse_metrics_on_interpretation_run_cancel_payload;
/// Typed POST exchange for interpretation-run cancel.
pub use interpretation_run_cancel_http::InterpretationRunCancelHttpExchange;
/// Metric-free cancelled interpretation-run identity.
pub use interpretation_run_cancel_http::InterpretationRunCancelled;
/// Maximum opaque idempotency-key length on interpretation-run cancel.
pub use interpretation_run_cancel_http::INTERPRETATION_RUN_CANCEL_ID_MAX_LEN;
/// Compose HTTP/1.1 interpretation-run POST from a CLI invocation.
pub use interpretation_run_cli::compose_interpretation_run_cli_http;
/// Build a credential-free contextual-orchestrator interpretation-run exchange.
pub use interpretation_run_cli::contextual_orchestrator_interpretation_run_exchange;
/// Dispatch an interpretation-run CLI invocation against an in-process listener.
pub use interpretation_run_cli::dispatch_interpretation_run_cli;
/// Execute an interpretation-run CLI invocation over loopback TCP.
pub use interpretation_run_cli::execute_interpretation_run_cli;
/// Render a typed interpretation-run exchange onto a loopback HTTP/1.1 request.
pub use interpretation_run_cli::loopback_http1_from_interpretation_run_exchange;
/// Read leftover stdin for the interpretation-run CLI.
pub use interpretation_run_cli::read_interpretation_run_cli_stdin;
/// Refuse scientific-metric keys on interpretation-run CLI JSON.
pub use interpretation_run_cli::refuse_metrics_on_interpretation_run_cli_payload;
/// Filter interpretation-run CLI stdout so the accepted run stays hypothetical.
pub use interpretation_run_cli::render_interpretation_run_cli_stdout;
/// Loopback interpretation-run CLI invocation.
pub use interpretation_run_cli::InterpretationRunCliInvocation;
/// Loopback interpretation-run CLI verb.
pub use interpretation_run_cli::InterpretationRunCliVerb;
/// Typed HTTPS interpretation-run exchange.
pub use interpretation_run_cli::InterpretationRunHttpExchange;
/// Published modular consumer for interpretation-run POST.
pub use interpretation_run_cli::CONTEXTUAL_ORCHESTRATOR_CONSUMER_CODE;
/// Build a credential-free contextual-orchestrator collection GET exchange.
pub use interpretation_run_collection_http::contextual_orchestrator_interpretation_run_collection_exchange;
/// Whether a path is the interpretation-run collection resource.
pub use interpretation_run_collection_http::is_interpretation_run_collection_path;
/// Page stored collection rows with an exclusive idempotency-key cursor.
pub use interpretation_run_collection_http::page_interpretation_run_collection_items;
/// Parse the optional exclusive `tepp-page-cursor` header.
pub use interpretation_run_collection_http::parse_interpretation_run_collection_page_cursor;
/// Parse the optional `tepp-page-limit` header.
pub use interpretation_run_collection_http::parse_interpretation_run_collection_page_limit;
/// Refuse metric, evidence, and causal-score keys on collection JSON.
pub use interpretation_run_collection_http::refuse_metrics_on_interpretation_run_collection_payload;
/// Metric-free interpretation-run collection page.
pub use interpretation_run_collection_http::InterpretationRunCollection;
/// Typed GET exchange for interpretation-run collection.
pub use interpretation_run_collection_http::InterpretationRunCollectionHttpExchange;
/// One metric-free interpretation-run collection row.
pub use interpretation_run_collection_http::InterpretationRunCollectionItem;
/// Maximum opaque cursor length on interpretation-run collection GET.
pub use interpretation_run_collection_http::INTERPRETATION_RUN_COLLECTION_CURSOR_MAX_LEN;
/// Default page size for interpretation-run collection GET.
pub use interpretation_run_collection_http::INTERPRETATION_RUN_COLLECTION_DEFAULT_LIMIT;
/// Maximum page size for interpretation-run collection GET.
pub use interpretation_run_collection_http::INTERPRETATION_RUN_COLLECTION_MAX_LIMIT;
/// Build a credential-free contextual-orchestrator GET-by-id exchange.
pub use interpretation_run_retrieval_http::contextual_orchestrator_interpretation_run_retrieval_exchange;
/// Serialize one metric-free GET-by-id identity.
pub use interpretation_run_retrieval_http::interpretation_run_retrieval_item_json;
/// Extract the opaque idempotency key from a GET-by-id path.
pub use interpretation_run_retrieval_http::interpretation_run_retrieval_path_id;
/// Typed GET exchange for interpretation-run GET-by-id.
pub use interpretation_run_retrieval_http::InterpretationRunRetrievalHttpExchange;
/// Maximum opaque idempotency-key length on interpretation-run GET-by-id.
pub use interpretation_run_retrieval_http::INTERPRETATION_RUN_RETRIEVAL_ID_MAX_LEN;
/// Closed ADR 0010 orchestration-mode vocabulary.
pub use mode::OrchestrationMode;
/// Accepted hypothetical interpretation-run response.
pub use request::InterpretationRunAccepted;
/// Interpretation-run create request.
pub use request::InterpretationRunRequest;
/// Default maximum interpretation-run JSON payload size in bytes.
pub use request::DEFAULT_INTERPRETATION_BYTE_LIMIT;
/// Canonical hypothetical claim-status label.
pub use request::HYPOTHETICAL_CLAIM_STATUS;
/// Supported interpretation-run contract version.
pub use request::INTERPRETATION_RUN_CONTRACT_VERSION;
/// Versioned path contextual-orchestrator may POST or GET.
pub use request::INTERPRETATION_RUN_PATH;
/// Loopback live HTTP/1.1 orchestrator listener.
pub use service::OrchestratorLiveService;
