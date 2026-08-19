#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! Operational logs that refuse source text and source identity.
//!
//! Privileged and ordinary operations may be logged with a closed action code
//! and a generated opaque analytical subject. Raw source text and source
//! identity cannot enter the log, and blanket PII masking is not a log grant
//! (ADR 0009).

mod error;
mod record;

/// Fail-closed operational-log errors.
pub use error::OperationalLogError;
/// Closed operational action codes.
pub use record::ActionCode;
/// Opaque analytical subject generated independently from source identity.
pub use record::AnalyticalSubject;
/// One operational-log line without source text or source identity.
pub use record::OperationalLogRecord;
/// Source identity that must never become an operational-log subject.
pub use record::SourceIdentity;
/// Fraction of replayed log records that match known truth.
pub use record::log_recovery_rate;
/// Refuse to treat blanket PII masking as operational-log authorization.
pub use record::refuse_blanket_mask_as_log_grant;
/// Refuse to reinterpret source identity as an analytical subject.
pub use record::refuse_source_identity_as_subject;
/// Refuse to place source identity in the operational log.
pub use record::refuse_source_identity_in_log;
/// Refuse to place raw source text in the operational log.
pub use record::refuse_source_text_in_log;
/// Replay an operational log without rewriting lines.
pub use record::replay_operational_log;
