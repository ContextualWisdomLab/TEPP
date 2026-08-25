//! Interval-aware historical eligibility against a knowledge cutoff.

use crate::{
    AvailableTime, KnowledgeCutoff, TemporalBoundary, TemporalCertainty, TemporalError,
    TemporalInterval,
};

/// Decide whether an availability interval is fully eligible under `knowledge_cutoff`.
///
/// Evidence may enter a historical analysis only when every possible
/// availability instant is at or before the cutoff. Unknown availability and
/// open-ended upper bounds fail closed because they can extend past the cutoff.
/// Event time and document time cannot be substituted: the interval is typed as
/// [`AvailableTime`].
///
/// ```compile_fail,E0308
/// use temporal_core::{
///     EventTime, KnowledgeCutoff, TemporalInterval, TemporalPrecision,
///     evaluate_historical_eligibility,
/// };
///
/// let event = TemporalInterval::exact(
///     EventTime::parse_rfc3339("2026-01-01T00:00:00Z")?,
///     TemporalPrecision::Second,
/// )?;
/// let cutoff = KnowledgeCutoff::parse_rfc3339("2026-06-01T00:00:00Z")?;
/// evaluate_historical_eligibility(&event, &cutoff)?;
/// # Ok::<(), temporal_core::TemporalError>(())
/// ```
///
/// # Errors
///
/// Returns [`TemporalError::UncertainAvailability`] when the interval cannot
/// prove an upper bound, or [`TemporalError::IneligibleAtCutoff`] when the
/// latest possible availability is after the cutoff.
pub fn evaluate_historical_eligibility(
    availability: &TemporalInterval<AvailableTime>,
    knowledge_cutoff: &KnowledgeCutoff,
) -> Result<(), TemporalError> {
    if matches!(availability.certainty(), TemporalCertainty::Unknown) {
        return Err(TemporalError::UncertainAvailability);
    }

    let latest = match availability.upper() {
        TemporalBoundary::Unbounded => return Err(TemporalError::UncertainAvailability),
        TemporalBoundary::Included(value) => value.instant().as_nanosecond(),
        TemporalBoundary::Excluded(value) => value.instant().as_nanosecond() - 1,
    };
    if latest <= knowledge_cutoff.instant().as_nanosecond() {
        Ok(())
    } else {
        Err(TemporalError::IneligibleAtCutoff)
    }
}

#[cfg(test)]
mod tests {
    use super::evaluate_historical_eligibility;
    use crate::{
        AvailableTime, KnowledgeCutoff, TemporalBoundary, TemporalInterval, TemporalPrecision,
    };

    fn available(stamp: &str) -> AvailableTime {
        AvailableTime::parse_rfc3339(stamp).expect("available")
    }

    #[test]
    fn excluded_upper_one_nanosecond_after_cutoff_is_eligible() {
        let cutoff = KnowledgeCutoff::parse_rfc3339("2026-06-01T00:00:00Z").expect("cutoff");
        let just_after = available("2026-06-01T00:00:00.000000001Z");
        let interval = TemporalInterval::bounded(
            TemporalBoundary::Unbounded,
            TemporalBoundary::Excluded(just_after),
            TemporalPrecision::Nanosecond,
        )
        .expect("interval");
        assert_eq!(evaluate_historical_eligibility(&interval, &cutoff), Ok(()));
    }
}
