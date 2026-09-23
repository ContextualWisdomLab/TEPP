//! Latent event truth rows.

use temporal_core::EventTime;
use uuid::Uuid;

use crate::{SimulationError, TruthManifest};

/// Known latent event state for recovery studies.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum LatentEventState {
    /// The event occurred in the generative process.
    Occurred,
    /// The event was scheduled but not realized (held out for incomplete tracking).
    Planned,
}

impl LatentEventState {
    /// Stable wire name for the latent state.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Occurred => "occurred",
            Self::Planned => "planned",
        }
    }
}

/// One latent event instance in the truth corpus.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LatentEvent {
    event_id: Uuid,
    event_time: EventTime,
    ordinal: u32,
    state: LatentEventState,
}

impl LatentEvent {
    /// Construct a latent event truth row.
    #[must_use]
    pub const fn new(
        event_id: Uuid,
        event_time: EventTime,
        ordinal: u32,
        state: LatentEventState,
    ) -> Self {
        Self {
            event_id,
            event_time,
            ordinal,
            state,
        }
    }

    /// Stable event identity.
    #[must_use]
    pub const fn event_id(&self) -> Uuid {
        self.event_id
    }

    /// True event/valid time.
    #[must_use]
    pub const fn event_time(&self) -> EventTime {
        self.event_time
    }

    /// Ordinal index used for deterministic temporal order.
    #[must_use]
    pub const fn ordinal(&self) -> u32 {
        self.ordinal
    }

    /// Known generative state.
    #[must_use]
    pub const fn state(&self) -> LatentEventState {
        self.state
    }
}

impl TruthManifest {
    /// Return the affine physical-time basis used by known prevalence trajectories.
    ///
    /// The tuple is `(origin_event_time, center_seconds_from_origin, scale_seconds)`
    /// and represents the simulator predictor as
    /// `x = (t_seconds_from_origin - center_seconds_from_origin) / scale_seconds`.
    /// For a valid generated manifest, this coordinate equals the topic DGP's
    /// normalized event position at every latent event.
    ///
    /// # Errors
    ///
    /// Returns [`SimulationError::ManifestInvariantViolation`] unless the manifest
    /// contains at least two latent events with contiguous zero-based ordinals and
    /// strictly increasing, equally spaced EventTime values.
    #[allow(clippy::cast_precision_loss)]
    pub fn prevalence_time_basis(&self) -> Result<(EventTime, f64, f64), SimulationError> {
        let events = self.events();
        if events.len() < 2 {
            return Err(SimulationError::ManifestInvariantViolation);
        }
        let origin = events[0].event_time();
        let origin_nanoseconds = origin.instant().as_nanosecond();
        let final_nanoseconds = events
            .last()
            .ok_or(SimulationError::ManifestInvariantViolation)?
            .event_time()
            .instant()
            .as_nanosecond();
        let span_nanoseconds = final_nanoseconds
            .checked_sub(origin_nanoseconds)
            .filter(|span| *span > 0)
            .ok_or(SimulationError::ManifestInvariantViolation)?;
        let interval_count = i128::try_from(events.len() - 1)
            .map_err(|_| SimulationError::ManifestInvariantViolation)?;
        if span_nanoseconds % interval_count != 0 {
            return Err(SimulationError::ManifestInvariantViolation);
        }
        let step_nanoseconds = span_nanoseconds / interval_count;
        if step_nanoseconds <= 0 {
            return Err(SimulationError::ManifestInvariantViolation);
        }
        for (index, event) in events.iter().enumerate() {
            let expected_ordinal = u32::try_from(index)
                .map_err(|_| SimulationError::ManifestInvariantViolation)?;
            let index_i128 = i128::try_from(index)
                .map_err(|_| SimulationError::ManifestInvariantViolation)?;
            let expected_delta = step_nanoseconds
                .checked_mul(index_i128)
                .ok_or(SimulationError::ManifestInvariantViolation)?;
            let actual_delta = event
                .event_time()
                .instant()
                .as_nanosecond()
                .checked_sub(origin_nanoseconds)
                .ok_or(SimulationError::ManifestInvariantViolation)?;
            if event.ordinal() != expected_ordinal || actual_delta != expected_delta {
                return Err(SimulationError::ManifestInvariantViolation);
            }
        }

        let span_seconds = span_nanoseconds as f64 / 1_000_000_000.0;
        let center_seconds_from_origin = span_seconds / 2.0;
        let scale_seconds = center_seconds_from_origin;
        if !center_seconds_from_origin.is_finite() || scale_seconds <= 0.0 {
            return Err(SimulationError::ManifestInvariantViolation);
        }
        Ok((origin, center_seconds_from_origin, scale_seconds))
    }

    /// Express one EventTime on the known prevalence trajectory coordinate.
    ///
    /// # Errors
    ///
    /// Returns [`SimulationError::ManifestInvariantViolation`] when the manifest
    /// does not expose a valid affine prevalence-time basis or the derived
    /// coordinate is non-finite.
    #[allow(clippy::cast_precision_loss)]
    pub fn prevalence_time_coordinate_at(
        &self,
        event_time: EventTime,
    ) -> Result<f64, SimulationError> {
        let (origin, center_seconds, scale_seconds) = self.prevalence_time_basis()?;
        let delta_nanoseconds = event_time
            .instant()
            .as_nanosecond()
            .checked_sub(origin.instant().as_nanosecond())
            .ok_or(SimulationError::ManifestInvariantViolation)?;
        let seconds_from_origin = delta_nanoseconds as f64 / 1_000_000_000.0;
        let coordinate = (seconds_from_origin - center_seconds) / scale_seconds;
        if !coordinate.is_finite() {
            return Err(SimulationError::ManifestInvariantViolation);
        }
        Ok(coordinate)
    }
}

#[cfg(test)]
mod tests {
    use super::{LatentEvent, LatentEventState};
    use temporal_core::EventTime;
    use uuid::Uuid;

    #[test]
    fn latent_event_accessors_and_wire_names() {
        let event_time = EventTime::parse_rfc3339("2026-01-02T03:00:00Z").expect("time");
        let id = Uuid::nil();
        let event = LatentEvent::new(id, event_time, 3, LatentEventState::Occurred);
        assert_eq!(event.event_id(), id);
        assert_eq!(event.event_time(), event_time);
        assert_eq!(event.ordinal(), 3);
        assert_eq!(event.state(), LatentEventState::Occurred);
        assert_eq!(LatentEventState::Occurred.wire_name(), "occurred");
        assert_eq!(LatentEventState::Planned.wire_name(), "planned");
        let planned = LatentEvent::new(id, event_time, 4, LatentEventState::Planned);
        assert_eq!(planned.state(), LatentEventState::Planned);
    }
}
