//! Latent event truth rows.

use temporal_core::EventTime;
use uuid::Uuid;

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
