#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! Language, episode, template, department, and opportunity-pool targets are not entities.
//!
//! Persistence currently stores only an entity or a project. Those two
//! columns cannot stand in for the other ADR 0003 membership targets.

mod error;
mod kind;

/// Fail-closed membership-target errors.
pub use error::MembershipTargetError;
/// Closed vocabulary of membership targets.
pub use kind::MembershipTargetKind;
/// Fraction of recovered target kinds that match known truth.
pub use kind::identity_recovery_rate;
/// Refuse to treat one target kind as another.
pub use kind::refuse_collapsed_target;
