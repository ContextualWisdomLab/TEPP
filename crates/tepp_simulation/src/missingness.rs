//! Controlled missingness for uncertain or unobserved fields.

use crate::SimulationError;
use temporal_core::EventTime;

/// Apply event-time missingness while preserving the true value separately.
///
/// Returns `None` when the observation is masked; otherwise returns the true
/// event time unchanged.
#[must_use]
pub fn mask_event_time(true_event_time: EventTime, is_missing: bool) -> Option<EventTime> {
    if is_missing {
        None
    } else {
        Some(true_event_time)
    }
}

/// Validate a missingness rate expressed in basis points.
///
/// # Errors
///
/// Returns [`SimulationError::InvalidConfiguration`] when the rate exceeds
/// `10_000` basis points.
pub fn validate_missingness_rate_bps(rate_bps: u32) -> Result<(), SimulationError> {
    if rate_bps > 10_000 {
        return Err(SimulationError::InvalidConfiguration);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{mask_event_time, validate_missingness_rate_bps};
    use crate::SimulationError;
    use temporal_core::EventTime;

    #[test]
    fn missingness_masks_or_preserves_event_time() {
        let stamp = EventTime::parse_rfc3339("2026-01-05T00:00:00Z").expect("time");
        assert_eq!(mask_event_time(stamp, true), None);
        assert_eq!(mask_event_time(stamp, false), Some(stamp));
        assert_eq!(
            validate_missingness_rate_bps(10_001),
            Err(SimulationError::InvalidConfiguration)
        );
        validate_missingness_rate_bps(0).expect("zero ok");
        validate_missingness_rate_bps(10_000).expect("full ok");
    }
}
