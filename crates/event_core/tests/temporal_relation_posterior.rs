//! Synthetic exact-recovery tests for qualitative temporal relation draws.

use event_core::{
    DrawTemporalRelation, TemporalRelationPosteriorError, infer_temporal_relation_posterior,
};
use temporal_core::EventTime;

fn time(day: u8) -> EventTime {
    EventTime::parse_rfc3339(&format!("2026-01-{day:02}T00:00:00Z")).expect("synthetic time")
}

#[test]
fn relation_posterior_exactly_recovers_before_tie_and_after_draws() {
    let posterior = infer_temporal_relation_posterior(
        &[time(1), time(2), time(3), time(4)],
        &[time(2), time(2), time(2), time(5)],
    )
    .expect("identified common draws");
    assert_eq!(
        posterior.relation_draws,
        vec![
            DrawTemporalRelation::Before,
            DrawTemporalRelation::Simultaneous,
            DrawTemporalRelation::After,
            DrawTemporalRelation::Before,
        ]
    );
    assert!((posterior.before_probability - 0.5).abs() < f64::EPSILON);
    assert!((posterior.simultaneous_probability - 0.25).abs() < f64::EPSILON);
    assert!((posterior.after_probability - 0.25).abs() < f64::EPSILON);
}

#[test]
fn relation_posterior_fails_closed_without_common_draws() {
    assert_eq!(
        infer_temporal_relation_posterior(&[], &[]),
        Err(TemporalRelationPosteriorError::EmptyDraws)
    );
    assert_eq!(
        infer_temporal_relation_posterior(&[time(1)], &[]),
        Err(TemporalRelationPosteriorError::DrawCountMismatch)
    );
}
