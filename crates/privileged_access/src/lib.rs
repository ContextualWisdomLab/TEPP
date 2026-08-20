#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! Privileged-access audit records that refuse source identity.
//!
//! Re-identification and grant use are privileged. The audit log records the
//! decision, purpose, and opaque analytical subject. It cannot carry source
//! identity, and blanket PII masking is not an audit grant (ADR 0009).

mod audit;
mod error;

/// Privileged action recorded without source identity.
pub use audit::PrivilegedAction;
/// One privileged-access decision bound to an opaque analytical subject.
pub use audit::PrivilegedAuditRecord;
/// Fraction of replayed audit records that match known truth.
pub use audit::audit_recovery_rate;
/// Refuse to treat blanket PII masking as privileged-access authorization.
pub use audit::refuse_blanket_mask_as_audit_grant;
/// Refuse to place source identity in the privileged-access audit log.
pub use audit::refuse_source_identity_in_audit;
/// Replay a privileged-access log without rewriting decisions.
pub use audit::replay_privileged_audit;
/// Fail-closed privileged-access errors.
pub use error::PrivilegedAccessError;
