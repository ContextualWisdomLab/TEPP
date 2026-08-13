#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! Global topic identity that survives dormancy and reactivation.
//!
//! A P0 topic identity is selected once for the modeled period. Activity may
//! change without minting a new identity. Reactivation is not a new topic
//! (ADR 0012).

mod activity;
mod error;
mod identity;

/// Activity of one global topic identity.
pub use activity::TopicActivity;
/// Topic identity together with its current activity.
pub use activity::TopicLineageRecord;
/// Fail-closed topic-lineage errors.
pub use error::TopicLineageError;
/// Opaque global topic identity.
pub use identity::TopicIdentity;
/// Fraction of recovered identities that match known truth.
pub use identity::identity_recovery_rate;
/// Refuse to treat reactivation as a newly minted topic.
pub use identity::refuse_new_identity_on_reactivation;
