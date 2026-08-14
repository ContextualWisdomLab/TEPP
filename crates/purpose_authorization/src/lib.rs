#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! Purpose-bound authorization grants that refuse blanket PII masking.
//!
//! A grant authorizes one processing purpose for one principal. It cannot be
//! reused for another purpose, and masking identifiers is not authorization
//! (ADR 0009).

mod error;
mod grant;
mod purpose;

/// Fail-closed purpose-authorization errors.
pub use error::PurposeAuthorizationError;
/// One purpose-bound grant.
pub use grant::AuthorizationGrant;
/// Opaque principal identity.
pub use grant::PrincipalId;
/// Closed processing-purpose vocabulary.
pub use purpose::PurposeCode;
/// Fraction of recovered purposes that match known truth.
pub use purpose::purpose_recovery_rate;
/// Refuse to treat blanket PII masking as authorization.
pub use purpose::refuse_blanket_mask_as_authorization;
/// Refuse to use a grant for a different purpose.
pub use purpose::refuse_cross_purpose_use;
