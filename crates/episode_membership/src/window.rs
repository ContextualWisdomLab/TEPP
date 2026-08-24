//! Event-time windows and episode-containment refusal.

use crate::EpisodeMembershipError;

/// A closed event-time window with inclusive integer bounds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EventWindow {
    start: i64,
    end: i64,
}

impl EventWindow {
    /// Construct a non-inverted event-time window.
    ///
    /// # Errors
    ///
    /// Returns [`EpisodeMembershipError::InvertedEventWindow`] when `end` is
    /// earlier than `start`.
    pub const fn new(start: i64, end: i64) -> Result<Self, EpisodeMembershipError> {
        if end < start {
            return Err(EpisodeMembershipError::InvertedEventWindow);
        }
        Ok(Self { start, end })
    }

    /// Return the inclusive start instant.
    #[must_use]
    pub const fn start(self) -> i64 {
        self.start
    }

    /// Return the inclusive end instant.
    #[must_use]
    pub const fn end(self) -> i64 {
        self.end
    }
}

/// Refuse a membership window that starts before or ends after the episode.
///
/// Equal bounds are contained. This is membership containment, not
/// subevent-versus-parent event containment.
///
/// # Errors
///
/// Returns [`EpisodeMembershipError::MembershipEscapesEpisode`] when the
/// membership is not contained in `episode`.
pub fn refuse_membership_outside_episode(
    membership: EventWindow,
    episode: EventWindow,
) -> Result<(), EpisodeMembershipError> {
    if membership.start() < episode.start() || membership.end() > episode.end() {
        return Err(EpisodeMembershipError::MembershipEscapesEpisode);
    }
    Ok(())
}

/// Fraction of recovered containment flags that match known truth.
///
/// # Errors
///
/// Returns [`EpisodeMembershipError::InvalidEpisodePayload`] when either
/// slice is empty or the lengths differ.
pub fn identity_recovery_rate(
    truth: &[bool],
    decided: &[bool],
) -> Result<f64, EpisodeMembershipError> {
    if truth.is_empty() || truth.len() != decided.len() {
        return Err(EpisodeMembershipError::InvalidEpisodePayload);
    }
    let matches = truth
        .iter()
        .zip(decided)
        .filter(|(truth_flag, decided_flag)| truth_flag == decided_flag)
        .count();
    Ok(matches as f64 / truth.len() as f64)
}

#[cfg(test)]
mod tests {
    use super::{EventWindow, identity_recovery_rate, refuse_membership_outside_episode};
    use crate::EpisodeMembershipError;

    #[test]
    fn local_branches_cover_windows_and_payloads() {
        assert_eq!(
            EventWindow::new(20, 10),
            Err(EpisodeMembershipError::InvertedEventWindow)
        );
        let episode = EventWindow::new(10, 20).expect("episode");
        assert_eq!(episode.start(), 10);
        assert_eq!(episode.end(), 20);
        let inner = EventWindow::new(11, 19).expect("inner");
        refuse_membership_outside_episode(inner, episode).expect("inner");
        refuse_membership_outside_episode(episode, episode).expect("equal");
        assert_eq!(
            refuse_membership_outside_episode(EventWindow::new(9, 15).expect("early"), episode),
            Err(EpisodeMembershipError::MembershipEscapesEpisode)
        );
        assert_eq!(
            refuse_membership_outside_episode(EventWindow::new(12, 21).expect("late"), episode),
            Err(EpisodeMembershipError::MembershipEscapesEpisode)
        );
        let matched = identity_recovery_rate(&[true], &[true]).expect("rate");
        assert!((matched - 1.0).abs() < f64::EPSILON);
        assert_eq!(
            identity_recovery_rate(&[], &[]),
            Err(EpisodeMembershipError::InvalidEpisodePayload)
        );
        assert_eq!(
            identity_recovery_rate(&[true], &[]),
            Err(EpisodeMembershipError::InvalidEpisodePayload)
        );
    }
}
