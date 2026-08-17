#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! Episode membership cannot escape the episode event-time interval.
//!
//! A document may belong to an episode only while that episode is active
//! in event time. This is not subevent-versus-parent containment
//! (ADR 0003).

mod error;
mod window;

/// Fail-closed episode-membership errors.
pub use error::EpisodeMembershipError;
/// Fraction of recovered containment flags that match known truth.
pub use window::identity_recovery_rate;
/// Refuse a membership window that starts before or ends after the episode.
pub use window::refuse_membership_outside_episode;
/// A closed event-time window with inclusive integer bounds.
pub use window::EventWindow;
