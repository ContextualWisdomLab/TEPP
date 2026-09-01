#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Loopback live HTTP/1.1 listener for contextual-orchestrator interpretation.
//!
//! The listener accepts `POST /v1/interpretation-runs` and
//! `GET /v1/interpretation-runs` on loopback only. Accepted output is always
//! hypothetical and never scientific authority. Collection GET enumerates
//! metric-free identities so operators do not guess idempotency keys.
//! GET-by-id returns one of those identities without POST replay.
//! `GET /v1/interpretation-runs/{idempotency_key}/request` returns the stored
//! create request without POST replay. Published `tepp-interpretation-run-request`
//! mints that GET onto spawned `tepp-orchestrator-loopback` TCP.
//! `GET /v1/interpretation-runs/by-run-id/{interpretation_run_id}` returns the
//! metric-free identity of the unique accepted run without POST replay.
//! Published `tepp-interpretation-run-lookup` mints that GET onto spawned
//! `tepp-orchestrator-loopback` TCP.
//! `GET /v1/interpretation-runs/by-run-id/{interpretation_run_id}/request`
//! returns the stored create request of that unique accepted run.
//! Published `tepp-interpretation-run-lookup-request` mints that GET onto
//! spawned `tepp-orchestrator-loopback` TCP.
//! Table-access hosts, review/Copilot/GitHub credentials, and
//! `COPILOT_GITHUB_TOKEN` fail closed. This crate does not implement TLS
//! termination or call a model provider (ADR 0010; ADR 0011). The published
//! `tepp-interpretation-runs` CLI mints typed contextual-orchestrator
//! interpretation-run POST exchanges onto spawned `tepp-orchestrator-loopback`
//! TCP.

mod error;
mod http;
mod interpretation_run_cli;
mod interpretation_run_collection_http;
mod interpretation_run_lookup_cli;
mod interpretation_run_lookup_http;
mod interpretation_run_lookup_stored_request_cli;
mod interpretation_run_lookup_stored_request_http;
mod interpretation_run_retrieval_http;
mod interpretation_run_stored_request_cli;
mod interpretation_run_stored_request_http;
mod mode;
mod request;
mod service;

/// Fail-closed orchestrator live-listener errors.
pub use error::OrchestratorLiveError;
/// Maximum live HTTP header-block size in bytes.
pub use http::LIVE_HEADER_BYTE_LIMIT;
/// Maximum live HTTP header count.
pub use http::LIVE_HEADER_COUNT_LIMIT;
/// Loopback live HTTP/1.1 response.
pub use http::OrchestratorLiveResponse;
/// Published modular consumer for interpretation-run POST.
pub use interpretation_run_cli::CONTEXTUAL_ORCHESTRATOR_CONSUMER_CODE;
/// Loopback interpretation-run CLI invocation.
pub use interpretation_run_cli::InterpretationRunCliInvocation;
/// Loopback interpretation-run CLI verb.
pub use interpretation_run_cli::InterpretationRunCliVerb;
/// Typed HTTPS interpretation-run exchange.
pub use interpretation_run_cli::InterpretationRunHttpExchange;
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
/// Maximum opaque cursor length on interpretation-run collection GET.
pub use interpretation_run_collection_http::INTERPRETATION_RUN_COLLECTION_CURSOR_MAX_LEN;
/// Default page size for interpretation-run collection GET.
pub use interpretation_run_collection_http::INTERPRETATION_RUN_COLLECTION_DEFAULT_LIMIT;
/// Maximum page size for interpretation-run collection GET.
pub use interpretation_run_collection_http::INTERPRETATION_RUN_COLLECTION_MAX_LIMIT;
/// Metric-free interpretation-run collection page.
pub use interpretation_run_collection_http::InterpretationRunCollection;
/// Typed GET exchange for interpretation-run collection.
pub use interpretation_run_collection_http::InterpretationRunCollectionHttpExchange;
/// One metric-free interpretation-run collection row.
pub use interpretation_run_collection_http::InterpretationRunCollectionItem;
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
/// Maximum opaque `interpretation_run_id` length on interpretation-run lookup GET.
pub use interpretation_run_lookup_http::INTERPRETATION_RUN_LOOKUP_ID_MAX_LEN;
/// Reserved collection-relative prefix that names the lookup resource.
pub use interpretation_run_lookup_http::INTERPRETATION_RUN_LOOKUP_PREFIX;
/// Typed GET exchange for interpretation-run lookup by server-assigned id.
pub use interpretation_run_lookup_http::InterpretationRunLookupHttpExchange;
/// Build a credential-free contextual-orchestrator lookup GET exchange.
pub use interpretation_run_lookup_http::contextual_orchestrator_interpretation_run_lookup_exchange;
/// Extract the opaque `interpretation_run_id` from a lookup GET path.
pub use interpretation_run_lookup_http::interpretation_run_lookup_path_id;
/// Whether a path is the lookup-by-run-id resource.
pub use interpretation_run_lookup_http::is_interpretation_run_lookup_path;
/// Extra-segment that names the stored create on lookup stored-request GET.
pub use interpretation_run_lookup_stored_request_http::INTERPRETATION_RUN_LOOKUP_STORED_REQUEST_SEGMENT;
/// Typed GET exchange for stored-request lookup by server-assigned id.
pub use interpretation_run_lookup_stored_request_http::InterpretationRunLookupStoredRequestHttpExchange;
/// Build a credential-free contextual-orchestrator lookup stored-request GET.
pub use interpretation_run_lookup_stored_request_http::contextual_orchestrator_interpretation_run_lookup_stored_request_exchange;
/// Extract the opaque `interpretation_run_id` from a lookup stored-request path.
pub use interpretation_run_lookup_stored_request_http::interpretation_run_lookup_stored_request_path_id;
/// Whether a path is the lookup stored-request extra-segment resource.
pub use interpretation_run_lookup_stored_request_http::is_interpretation_run_lookup_stored_request_path;
/// Loopback lookup stored-request CLI invocation.
pub use interpretation_run_lookup_stored_request_cli::InterpretationRunLookupStoredRequestCliInvocation;
/// Loopback lookup stored-request CLI verb.
pub use interpretation_run_lookup_stored_request_cli::InterpretationRunLookupStoredRequestCliVerb;
/// Compose HTTP/1.1 lookup stored-request GET from a CLI invocation.
pub use interpretation_run_lookup_stored_request_cli::compose_interpretation_run_lookup_stored_request_cli_http;
/// Dispatch a lookup stored-request CLI invocation against an in-process listener.
pub use interpretation_run_lookup_stored_request_cli::dispatch_interpretation_run_lookup_stored_request_cli;
/// Execute a lookup stored-request CLI invocation over loopback TCP.
pub use interpretation_run_lookup_stored_request_cli::execute_interpretation_run_lookup_stored_request_cli;
/// Render a typed lookup stored-request exchange onto a loopback HTTP/1.1 request.
pub use interpretation_run_lookup_stored_request_cli::loopback_http1_from_interpretation_run_lookup_stored_request_exchange;
/// Read leftover stdin for the lookup stored-request CLI.
pub use interpretation_run_lookup_stored_request_cli::read_interpretation_run_lookup_stored_request_cli_stdin;
/// Filter lookup stored-request CLI stdout so the stored create stays hypothetical.
pub use interpretation_run_lookup_stored_request_cli::render_interpretation_run_lookup_stored_request_cli_stdout;
/// Loopback lookup CLI invocation.
pub use interpretation_run_lookup_cli::InterpretationRunLookupCliInvocation;
/// Loopback lookup CLI verb.
pub use interpretation_run_lookup_cli::InterpretationRunLookupCliVerb;
/// Compose HTTP/1.1 lookup GET from a CLI invocation.
pub use interpretation_run_lookup_cli::compose_interpretation_run_lookup_cli_http;
/// Dispatch a lookup CLI invocation against an in-process listener.
pub use interpretation_run_lookup_cli::dispatch_interpretation_run_lookup_cli;
/// Execute a lookup CLI invocation over loopback TCP.
pub use interpretation_run_lookup_cli::execute_interpretation_run_lookup_cli;
/// Render a typed lookup exchange onto a loopback HTTP/1.1 request.
pub use interpretation_run_lookup_cli::loopback_http1_from_interpretation_run_lookup_exchange;
/// Read leftover stdin for the lookup CLI.
pub use interpretation_run_lookup_cli::read_interpretation_run_lookup_cli_stdin;
/// Filter lookup CLI stdout so the identity stays hypothetical.
pub use interpretation_run_lookup_cli::render_interpretation_run_lookup_cli_stdout;
/// Maximum opaque idempotency-key length on interpretation-run GET-by-id.
pub use interpretation_run_retrieval_http::INTERPRETATION_RUN_RETRIEVAL_ID_MAX_LEN;
/// Typed GET exchange for interpretation-run GET-by-id.
pub use interpretation_run_retrieval_http::InterpretationRunRetrievalHttpExchange;
/// Build a credential-free contextual-orchestrator GET-by-id exchange.
pub use interpretation_run_retrieval_http::contextual_orchestrator_interpretation_run_retrieval_exchange;
/// Serialize one metric-free GET-by-id identity.
pub use interpretation_run_retrieval_http::interpretation_run_retrieval_item_json;
/// Extract the opaque idempotency key from a GET-by-id path.
pub use interpretation_run_retrieval_http::interpretation_run_retrieval_path_id;
/// Loopback stored-request CLI invocation.
pub use interpretation_run_stored_request_cli::InterpretationRunStoredRequestCliInvocation;
/// Loopback stored-request CLI verb.
pub use interpretation_run_stored_request_cli::InterpretationRunStoredRequestCliVerb;
/// Compose HTTP/1.1 stored-request GET from a CLI invocation.
pub use interpretation_run_stored_request_cli::compose_interpretation_run_stored_request_cli_http;
/// Dispatch a stored-request CLI invocation against an in-process listener.
pub use interpretation_run_stored_request_cli::dispatch_interpretation_run_stored_request_cli;
/// Execute a stored-request CLI invocation over loopback TCP.
pub use interpretation_run_stored_request_cli::execute_interpretation_run_stored_request_cli;
/// Render a typed stored-request exchange onto a loopback HTTP/1.1 request.
pub use interpretation_run_stored_request_cli::loopback_http1_from_interpretation_run_stored_request_exchange;
/// Read leftover stdin for the stored-request CLI.
pub use interpretation_run_stored_request_cli::read_interpretation_run_stored_request_cli_stdin;
/// Filter stored-request CLI stdout so `scientific_authority` stays false.
pub use interpretation_run_stored_request_cli::render_interpretation_run_stored_request_cli_stdout;
/// Typed GET exchange for interpretation-run stored-request retrieval.
pub use interpretation_run_stored_request_http::InterpretationRunStoredRequestHttpExchange;
/// Build a credential-free contextual-orchestrator stored-request GET exchange.
pub use interpretation_run_stored_request_http::contextual_orchestrator_interpretation_run_stored_request_exchange;
/// Extract the opaque idempotency key from a stored-request GET path.
pub use interpretation_run_stored_request_http::interpretation_run_stored_request_path_id;
/// Whether a path is the stored-request extra-segment resource.
pub use interpretation_run_stored_request_http::is_interpretation_run_stored_request_path;
/// Refuse metric keys on stored-request JSON.
pub use interpretation_run_stored_request_http::refuse_metrics_on_interpretation_run_stored_request_payload;
/// Closed ADR 0010 orchestration-mode vocabulary.
pub use mode::OrchestrationMode;
/// Default maximum interpretation-run JSON payload size in bytes.
pub use request::DEFAULT_INTERPRETATION_BYTE_LIMIT;
/// Canonical hypothetical claim-status label.
pub use request::HYPOTHETICAL_CLAIM_STATUS;
/// Supported interpretation-run contract version.
pub use request::INTERPRETATION_RUN_CONTRACT_VERSION;
/// Versioned path contextual-orchestrator may POST or GET.
pub use request::INTERPRETATION_RUN_PATH;
/// Accepted hypothetical interpretation-run response.
pub use request::InterpretationRunAccepted;
/// Interpretation-run create request.
pub use request::InterpretationRunRequest;
/// Loopback live HTTP/1.1 orchestrator listener.
pub use service::OrchestratorLiveService;
