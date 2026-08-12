#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Time-varying cross-classified and multiple-membership assignments.
//!
//! TEPP models documents and other observations as members of many simultaneous
//! contexts—authors, departments, customers, partners, competitors, projects,
//! templates, languages, locations, and episodes—without forcing a single
//! hierarchy. Membership weights and event-time validity intervals are first
//! class so multilevel and multiple-membership estimators can avoid atomistic
//! fallacy.

mod assignment;
mod error;
mod identifier;
mod network;
mod role;
mod weight;

/// One weighted, role-typed, time-varying membership assignment.
pub use assignment::MembershipAssignment;
/// Fail-closed membership-domain validation errors.
pub use error::MembershipError;
/// Opaque analytical group identifier.
pub use identifier::GroupId;
/// Opaque analytical member identifier.
pub use identifier::MemberId;
/// In-memory multiple-membership network for estimation inputs.
pub use network::MembershipNetwork;
/// Contextual membership roles (not permanent entity classes).
pub use role::MembershipRole;
/// Finite non-negative membership weight.
pub use weight::MembershipWeight;
