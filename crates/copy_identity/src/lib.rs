#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! A template copy is not the source document and not a state transition.
//!
//! Copy variants keep a distinct identity for relation-aware splits. They
//! never become input-process-outcome edges and never reuse the source
//! identity (ADR 0003).

mod error;
mod kind;

/// Fail-closed copy-identity errors.
pub use error::CopyIdentityError;
/// Fraction of recovered copy kinds that match known truth.
pub use kind::identity_recovery_rate;
/// Refuse to treat a template copy as the source document identity.
pub use kind::refuse_copy_as_source_identity;
/// Refuse to treat a template copy as a forward state transition.
pub use kind::refuse_copy_as_transition;
/// Closed vocabulary of copy-related document identities.
pub use kind::CopyKind;
