#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! Multiple-membership rows cannot use evidence available after cutoff.
//!
//! A document may belong to several units at once. Historical estimation may
//! keep only those memberships whose availability time does not exceed the
//! knowledge cutoff (ADR 0002/0003).

mod eligibility;
mod error;

/// One membership observation stamped with availability time.
pub use eligibility::MembershipObservation;
/// Fraction of recovered eligibility flags that match known truth.
pub use eligibility::eligibility_recovery_rate;
/// Keep memberships whose availability does not exceed the cutoff.
pub use eligibility::eligible_memberships;
/// Refuse a single membership whose availability exceeds the cutoff.
pub use eligibility::refuse_membership_after_cutoff;
/// Fail-closed membership-cutoff errors.
pub use error::MembershipCutoffError;
