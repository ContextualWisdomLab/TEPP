#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! Location is a time-varying market membership, not entity identity.
//!
//! Geographic and market assignments stay explicit multiple-membership
//! structure. They are not permanent entity classes and are not language
//! channels (ADR 0003).

mod error;
mod kind;

/// Fail-closed location-membership errors.
pub use error::LocationMembershipError;
/// Closed vocabulary of location-related membership treatments.
pub use kind::LocationKind;
/// Fraction of recovered location kinds that match known truth.
pub use kind::identity_recovery_rate;
/// Refuse to treat location membership as permanent entity identity.
pub use kind::refuse_location_as_entity_identity;
/// Refuse to treat location membership as a language channel.
pub use kind::refuse_location_as_language_channel;
