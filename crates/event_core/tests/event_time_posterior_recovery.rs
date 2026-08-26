//! Synthetic recovery tests for exact CHRONOS event-time posterior draws.

use event_core::{
    EventTimePosteriorAtom, EventTimePosteriorError, materialize_event_time_posterior,
};
use temporal_core::EventTime;

fn event_time(value: &str) -> EventTime {
    EventTime::parse_rfc3339(value).expect("synthetic event time must parse")
}

#[test]
fn recovers_exact_posterior_mass_and_canonical_order() {
    let early = event_time("2026-01-01T00:00:00Z");
    let late = event_time("2026-01-03T00:00:00Z");
    let posterior = materialize_event_time_posterior(&[
        EventTimePosteriorAtom {
            event_time: late,
            multiplicity: 1,
        },
        EventTimePosteriorAtom {
            event_time: early,
            multiplicity: 3,
        },
    ])
    .expect("identified posterior must materialize");
    assert_eq!(posterior.draws, vec![early, early, early, late]);
    assert_eq!(
        posterior
            .draws
            .iter()
            .filter(|draw| **draw == early)
            .count(),
        3
    );
}

#[test]
fn refuses_unidentified_or_noncanonical_mass() {
    let instant = event_time("2026-01-01T00:00:00Z");
    assert_eq!(
        materialize_event_time_posterior(&[]),
        Err(EventTimePosteriorError::EmptyPosterior)
    );
    assert_eq!(
        materialize_event_time_posterior(&[EventTimePosteriorAtom {
            event_time: instant,
            multiplicity: 0,
        }]),
        Err(EventTimePosteriorError::ZeroMassAtom)
    );
    assert_eq!(
        materialize_event_time_posterior(&[
            EventTimePosteriorAtom {
                event_time: instant,
                multiplicity: 1,
            },
            EventTimePosteriorAtom {
                event_time: instant,
                multiplicity: 1,
            },
        ]),
        Err(EventTimePosteriorError::DuplicateEventTime)
    );
}
