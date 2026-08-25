#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Loopback live HTTP/1.1 listener for contextual-orchestrator interpretation.
//!
//! The listener accepts `POST /v1/interpretation-runs` on loopback only.
//! Accepted output is always hypothetical and never scientific authority.
//! Table-access hosts, review/Copilot/GitHub credentials, and
//! `COPILOT_GITHUB_TOKEN` fail closed. This crate does not implement TLS
//! termination or call a model provider (ADR 0010; ADR 0011).

mod error;
mod http;
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
/// Closed ADR 0010 orchestration-mode vocabulary.
pub use mode::OrchestrationMode;
/// Default maximum interpretation-run JSON payload size in bytes.
pub use request::DEFAULT_INTERPRETATION_BYTE_LIMIT;
/// Canonical hypothetical claim-status label.
pub use request::HYPOTHETICAL_CLAIM_STATUS;
/// Supported interpretation-run contract version.
pub use request::INTERPRETATION_RUN_CONTRACT_VERSION;
/// Versioned path contextual-orchestrator may POST.
pub use request::INTERPRETATION_RUN_PATH;
/// Accepted hypothetical interpretation-run response.
pub use request::InterpretationRunAccepted;
/// Interpretation-run create request.
pub use request::InterpretationRunRequest;
/// Loopback live HTTP/1.1 orchestrator listener.
pub use service::OrchestratorLiveService;
