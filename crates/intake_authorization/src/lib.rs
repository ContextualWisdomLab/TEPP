#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! Untrusted intake fails closed without a grant; bounds are not authorization.
//!
//! Documents, serialized records, checkpoints, and LLM outputs require a
//! purpose-bound grant at the intake boundary. Passing size or identity
//! bounds is not that grant (ADR 0009; AGENTS.md).

mod error;
mod intake;

/// Fail-closed intake-authorization errors.
pub use error::IntakeAuthorizationError;
/// Fraction of recovered grant-presence flags that match known truth.
pub use intake::identity_recovery_rate;
/// Refuse to treat size, identity, or provenance bounds as authorization.
pub use intake::refuse_bounds_as_authorization;
/// Refuse untrusted intake that has no purpose-bound grant.
pub use intake::refuse_intake_without_grant;
/// Whether a purpose-bound grant is present at intake.
pub use intake::GrantPresence;
/// Closed vocabulary of untrusted inbound kinds that require a grant.
pub use intake::IntakeKind;
