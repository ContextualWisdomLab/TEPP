#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! Purpose-bound selective disclosure without blanket PII masking.
//!
//! Authorized field grants may emit only the requested classes. Scientific
//! linkage (authorship, event time, membership) cannot be stripped under a
//! scientific purpose. Direct identity and source text require re-identification
//! purpose. Blanket masking is not a disclosure grant (ADR 0009).

mod disclosure;
mod error;

/// One purpose-bound set of disclosed field codes.
pub use disclosure::DisclosedFieldSet;
/// Closed processing purpose for a disclosure decision.
pub use disclosure::DisclosurePurpose;
/// Closed field: author or authorship role linkage.
pub use disclosure::FIELD_AUTHOR_ROLE;
/// Closed field: direct source identity.
pub use disclosure::FIELD_DIRECT_IDENTITY;
/// Closed field: event or valid time.
pub use disclosure::FIELD_EVENT_TIME;
/// Closed field: membership or contextual role.
pub use disclosure::FIELD_MEMBERSHIP_ROLE;
/// Closed field: opaque analytical identifier.
pub use disclosure::FIELD_OPAQUE_ID;
/// Closed field: raw source text.
pub use disclosure::FIELD_SOURCE_TEXT;
/// Disclose the intersection of requested fields and the purpose grant.
pub use disclosure::disclose;
/// Fraction of disclosed field sets that match known truth.
pub use disclosure::disclosure_recovery_rate;
/// Refuse to treat a blanket PII mask as a disclosure grant.
pub use disclosure::refuse_blanket_mask;
/// Fail-closed selective-disclosure errors.
pub use error::SelectiveDisclosureError;
