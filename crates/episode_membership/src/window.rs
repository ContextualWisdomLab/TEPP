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
/// The match tally accumulates in [`usize`] and converts to [`f64`] only at
/// the final division, so all-match inputs longer than [`u32::MAX`] neither
/// wrap the tally nor panic under overflow-checked builds.
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
    let matches = count_matching_decisions(truth.iter().copied().zip(decided.iter().copied()));
    Ok(recovery_rate_from_tally(matches, truth.len()))
}

/// Count positions where a recovered decision equals its known-truth flag.
///
/// Generic over any pair stream so contract tests can inject a small
/// synthetic iterator; the tally type is [`usize`], which keeps counting
/// exact beyond [`u32::MAX`].
fn count_matching_decisions<I>(pairs: I) -> usize
where
    I: IntoIterator<Item = (bool, bool)>,
{
    pairs
        .into_iter()
        .filter(|(truth_flag, decided_flag)| truth_flag == decided_flag)
        .count()
}

/// Divide an exact match tally by the total length in [`f64`].
///
/// The tally parameter is [`usize`] on purpose: a mocked count source beyond
/// [`u32::MAX`] is representable, and conversion to [`f64`] happens exactly
/// once here instead of accumulating in [`u32`].
const fn recovery_rate_from_tally(matches: usize, total: usize) -> f64 {
    matches as f64 / total as f64
}

#[cfg(test)]
mod tests {
    use super::{
        EventWindow, count_matching_decisions, identity_recovery_rate, recovery_rate_from_tally,
        refuse_membership_outside_episode,
    };
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

    /// Mocked count source reporting a tally beyond [`u32::MAX`].
    ///
    /// The overflow contract is enforced twice: at compile time, because the
    /// tally parameters of [`recovery_rate_from_tally`] are [`usize`] (a
    /// revert to `u32` fails this assertion to compile), and at run time,
    /// because the boundary value must divide exactly in [`f64`].
    const BEYOND_U32_MAX_MATCHES: usize = u32::MAX as usize + 3;
    const BEYOND_U32_MAX_TOTAL: usize = 2 * (u32::MAX as usize) + 6;

    #[test]
    fn match_counting_is_generic_over_injected_iterators() {
        let injected = [(true, true), (false, true), (true, false), (false, false)];
        assert_eq!(count_matching_decisions(injected), 2);
        let boundary_rate = recovery_rate_from_tally(BEYOND_U32_MAX_MATCHES, BEYOND_U32_MAX_TOTAL);
        assert!((boundary_rate - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn recovery_rate_agrees_with_reference_count_on_synthetic_stream() {
        let total = 200_000_usize;
        let truth: Vec<bool> = (0..total).map(|index| index % 3 == 0).collect();
        let decided: Vec<bool> = (0..total).map(|index| index % 4 == 0).collect();
        let expected_matches = truth
            .iter()
            .zip(decided.iter())
            .filter(|(truth_flag, decided_flag)| truth_flag == decided_flag)
            .count();
        let rate = identity_recovery_rate(&truth, &decided).expect("rate");
        assert!((rate - expected_matches as f64 / total as f64).abs() < 1e-9);
    }
}
