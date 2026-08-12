//! Versioned event instances distinct from mentions.

use crate::{EventConfidence, EventError, EventInstanceId, EventMentionId, EventRoleKind};
use temporal_core::{EventTime, TemporalBoundary, TemporalInterval, TemporalPrecision};

/// A versioned event instance with event-time support and typed roles.
///
/// Instances are the unit of temporal state transition and multilevel modeling.
/// They are created only by explicit promotion from one or more mentions or by
/// authoritative assertion — never by silent cast from a mention identifier.
#[derive(Clone, Debug, PartialEq)]
pub struct EventInstance {
    instance_id: EventInstanceId,
    supporting_mentions: Vec<EventMentionId>,
    event_time: TemporalInterval<EventTime>,
    confidence: EventConfidence,
    roles: Vec<(EventRoleKind, String)>,
}

impl EventInstance {
    /// Promote supporting mentions into a new event instance.
    ///
    /// # Errors
    ///
    /// Returns confidence or validity errors when inputs fail validation.
    /// At least one supporting mention is required.
    pub fn promote_from_mentions(
        supporting_mentions: Vec<EventMentionId>,
        valid_from: EventTime,
        valid_to: EventTime,
        confidence: EventConfidence,
    ) -> Result<Self, EventError> {
        if supporting_mentions.is_empty() {
            return Err(EventError::InvalidWirePayload);
        }
        let event_time = TemporalInterval::bounded(
            TemporalBoundary::Included(valid_from),
            TemporalBoundary::Included(valid_to),
            TemporalPrecision::Second,
        )
        .map_err(|_| EventError::InvalidWirePayload)?;
        Ok(Self {
            instance_id: EventInstanceId::new(),
            supporting_mentions,
            event_time,
            confidence,
            roles: Vec::new(),
        })
    }

    /// Return the instance identifier.
    #[must_use]
    pub const fn instance_id(&self) -> EventInstanceId {
        self.instance_id
    }

    /// Return supporting mention identifiers.
    #[must_use]
    pub fn supporting_mentions(&self) -> &[EventMentionId] {
        &self.supporting_mentions
    }

    /// Return the event-time interval.
    #[must_use]
    pub const fn event_time(&self) -> TemporalInterval<EventTime> {
        self.event_time
    }

    /// Return instance confidence.
    #[must_use]
    pub const fn confidence(&self) -> EventConfidence {
        self.confidence
    }

    /// Attach a typed role argument.
    pub fn assign_role(&mut self, role: EventRoleKind, argument: impl Into<String>) {
        self.roles.push((role, argument.into()));
    }

    /// Return role assignments.
    #[must_use]
    pub fn roles(&self) -> &[(EventRoleKind, String)] {
        &self.roles
    }

    /// Return whether the instance is active at an event time.
    #[must_use]
    pub fn is_active_at(&self, instant: EventTime) -> bool {
        self.event_time.contains(instant)
    }
}

/// Explicit refusal to cast a mention as an instance.
///
/// # Errors
///
/// Always returns [`EventError::MentionIsNotEventInstance`].
pub fn refuse_mention_as_instance(
    _mention_id: EventMentionId,
) -> Result<EventInstanceId, EventError> {
    Err(EventError::MentionIsNotEventInstance)
}

/// Wire schema version for future event instance records.
pub const EVENT_INSTANCE_WIRE_SCHEMA_VERSION: u16 = 1;
