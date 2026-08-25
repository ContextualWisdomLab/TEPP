#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! Customer and competitor cannot occupy the same group at once.
//!
//! Customer, partner, and competitor are time-varying roles, not permanent
//! entity classes. A customer/competitor pair in one group fails closed
//! (ADR 0003).

mod error;
mod role;

/// Fail-closed role-contradiction errors.
pub use error::RoleContradictionError;
/// Closed vocabulary of commercial roles that can change over time.
pub use role::ContextualRole;
/// Fraction of recovered contextual roles that match known truth.
pub use role::identity_recovery_rate;
/// Refuse a contradictory customer/competitor pair in one group.
pub use role::refuse_contradictory_roles;
/// Refuse to treat a contextual role as a permanent entity class.
pub use role::refuse_role_as_entity_class;
/// Return whether two roles contradict in the same group.
pub use role::roles_contradict;
