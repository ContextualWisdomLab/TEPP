//! Episode membership cannot escape the episode event-time interval.

use episode_membership::{
    EpisodeMembershipError, EventWindow, identity_recovery_rate, refuse_membership_outside_episode,
};

#[test]
fn membership_cannot_escape_the_episode_interval() {
    let episode = EventWindow::new(10, 20).expect("episode");
    let contained = EventWindow::new(10, 20).expect("equal");
    let starts_early = EventWindow::new(9, 15).expect("early");
    let ends_late = EventWindow::new(12, 21).expect("late");
    refuse_membership_outside_episode(contained, episode).expect("contained");
    assert_eq!(
        refuse_membership_outside_episode(starts_early, episode),
        Err(EpisodeMembershipError::MembershipEscapesEpisode)
    );
    assert_eq!(
        refuse_membership_outside_episode(ends_late, episode),
        Err(EpisodeMembershipError::MembershipEscapesEpisode)
    );
    assert_eq!(
        EventWindow::new(20, 10),
        Err(EpisodeMembershipError::InvertedEventWindow)
    );
}

#[test]
fn recovered_containment_matches_known_truth_better_than_accepting_every_membership() {
    let cases = [
        (10, 20, 10, 18, true),
        (30, 40, 32, 40, true),
        (50, 60, 50, 60, true),
        (70, 80, 69, 75, false),
        (90, 100, 95, 101, false),
        (110, 120, 112, 118, true),
    ];
    let mut truth = Vec::with_capacity(cases.len());
    let mut recovered = Vec::with_capacity(cases.len());
    for (episode_start, episode_end, member_start, member_end, expected) in cases {
        let episode = EventWindow::new(episode_start, episode_end).expect("episode");
        let membership = EventWindow::new(member_start, member_end).expect("membership");
        assert!(episode.start() <= episode.end());
        truth.push(expected);
        recovered.push(refuse_membership_outside_episode(membership, episode).is_ok());
    }
    let collapsed = vec![true; truth.len()];
    let recovered_rate = identity_recovery_rate(&truth, &recovered).expect("recovered");
    let collapsed_rate = identity_recovery_rate(&truth, &collapsed).expect("collapsed");
    assert!((recovered_rate - 1.0).abs() < f64::EPSILON);
    assert!((collapsed_rate - (2.0 / 3.0)).abs() < f64::EPSILON);
    assert!(recovered_rate > collapsed_rate);
}

#[test]
fn empty_or_mismatched_containment_payloads_fail_closed() {
    assert_eq!(
        identity_recovery_rate(&[], &[]),
        Err(EpisodeMembershipError::InvalidEpisodePayload)
    );
    assert_eq!(
        identity_recovery_rate(&[true], &[]),
        Err(EpisodeMembershipError::InvalidEpisodePayload)
    );
    assert_eq!(
        identity_recovery_rate(&[true, false], &[true]),
        Err(EpisodeMembershipError::InvalidEpisodePayload)
    );
}
