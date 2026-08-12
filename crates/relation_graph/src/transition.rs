//! Forward-only temporal validation for state-transition edges.

use crate::RelationError;
use temporal_core::{AllenRelation, EventTime, TemporalInterval, classify_interval_relation};

/// Validate that a forward transition does not move backward in event time.
///
/// Accepts Allen relations that do not place the source wholly after the target.
/// `After` and `MetBy` fail closed as reverse order. Intervals that cannot be
/// classified as proper closed Allen inputs fail closed as uncertain.
///
/// # Errors
///
/// Returns [`RelationError::ReverseTemporalOrder`] or
/// [`RelationError::UncertainTemporalOrder`].
pub fn validate_forward_event_order(
    source_event_time: &TemporalInterval<EventTime>,
    target_event_time: &TemporalInterval<EventTime>,
) -> Result<(), RelationError> {
    match classify_interval_relation(source_event_time, target_event_time) {
        Ok(AllenRelation::After | AllenRelation::MetBy) => Err(RelationError::ReverseTemporalOrder),
        Ok(_) => Ok(()),
        Err(_) => Err(RelationError::UncertainTemporalOrder),
    }
}

#[cfg(test)]
mod tests {
    use super::validate_forward_event_order;
    use crate::RelationError;
    use temporal_core::{EventTime, TemporalBoundary, TemporalInterval, TemporalPrecision};

    fn closed(start: &str, end: &str) -> TemporalInterval<EventTime> {
        TemporalInterval::bounded(
            TemporalBoundary::Included(EventTime::parse_rfc3339(start).expect("start")),
            TemporalBoundary::Included(EventTime::parse_rfc3339(end).expect("end")),
            TemporalPrecision::Second,
        )
        .expect("interval")
    }

    #[test]
    fn before_and_meets_are_forward() {
        let early = closed("2026-01-01T00:00:00Z", "2026-01-02T00:00:00Z");
        let late = closed("2026-01-03T00:00:00Z", "2026-01-04T00:00:00Z");
        let meets = closed("2026-01-02T00:00:00Z", "2026-01-03T00:00:00Z");
        assert!(validate_forward_event_order(&early, &late).is_ok());
        assert!(validate_forward_event_order(&early, &meets).is_ok());
    }

    #[test]
    fn after_is_reverse() {
        let early = closed("2026-01-01T00:00:00Z", "2026-01-02T00:00:00Z");
        let late = closed("2026-01-03T00:00:00Z", "2026-01-04T00:00:00Z");
        assert_eq!(
            validate_forward_event_order(&late, &early),
            Err(RelationError::ReverseTemporalOrder)
        );
    }

    #[test]
    fn unknown_intervals_fail_closed() {
        let known = closed("2026-01-01T00:00:00Z", "2026-01-02T00:00:00Z");
        let unknown = TemporalInterval::<EventTime>::unknown();
        assert_eq!(
            validate_forward_event_order(&known, &unknown),
            Err(RelationError::UncertainTemporalOrder)
        );
    }
}
