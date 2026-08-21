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
    let truth = [true, false, false];
    let recovered = [true, false, false];
    let collapsed = [true, true, true];
    let recovered_rate = identity_recovery_rate(&truth, &recovered).expect("recovered");
    let collapsed_rate = identity_recovery_rate(&truth, &collapsed).expect("collapsed");
    let expected = {
        let mut matches = 0_u32;
        for (truth_flag, decided_flag) in truth.iter().zip(recovered.iter()) {
            if truth_flag == decided_flag {
                matches += 1;
            }
        }
        f64::from(matches) / f64::from(u32::try_from(truth.len()).expect("len"))
    };
    assert!((recovered_rate - expected).abs() < f64::EPSILON);
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
