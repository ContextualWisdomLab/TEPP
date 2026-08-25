//! TDT link-detection scores stay distinct from instances and transitions.

use crate::{EventConfidence, EventError, EventInstanceId, EventMentionId};
use std::collections::BTreeSet;

/// TDT same-event versus distinct-event link label.
///
/// A link decision is detection evidence. It is never a promoted event instance
/// and cannot create a forward state transition by itself.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EventLinkLabel {
    /// The mention pair is scored as the same event or story.
    Linked,
    /// The mention pair is scored as distinct events.
    Unlinked,
}

impl EventLinkLabel {
    /// Return the stable wire label name.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Linked => "linked",
            Self::Unlinked => "unlinked",
        }
    }

    /// Parse a stable wire link label.
    ///
    /// # Errors
    ///
    /// Returns [`EventError::UnknownEventLinkLabel`] for unrecognized names.
    pub fn from_wire_name(name: &str) -> Result<Self, EventError> {
        match name {
            "linked" => Ok(Self::Linked),
            "unlinked" => Ok(Self::Unlinked),
            _ => Err(EventError::UnknownEventLinkLabel),
        }
    }

    /// Return whether this label is a positive link detection.
    #[must_use]
    pub const fn is_linked(self) -> bool {
        matches!(self, Self::Linked)
    }

    /// Return the binary probability target used for RMSE.
    ///
    /// Linked truth is `1.0`; unlinked truth is `0.0`.
    #[must_use]
    pub const fn as_probability_target(self) -> f64 {
        match self {
            Self::Linked => 1.0,
            Self::Unlinked => 0.0,
        }
    }
}

/// An undirected TDT link hypothesis between two mentions.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EventLinkPair {
    left: EventMentionId,
    right: EventMentionId,
}

impl EventLinkPair {
    /// Construct a normalized undirected mention pair.
    ///
    /// # Errors
    ///
    /// Returns [`EventError::InvalidWirePayload`] when both mentions are the
    /// same identity. A mention cannot link to itself.
    pub fn new(left: EventMentionId, right: EventMentionId) -> Result<Self, EventError> {
        if left == right {
            return Err(EventError::InvalidWirePayload);
        }
        if left <= right {
            Ok(Self { left, right })
        } else {
            Ok(Self {
                left: right,
                right: left,
            })
        }
    }

    /// Return the lexicographically smaller mention identifier.
    #[must_use]
    pub const fn left(self) -> EventMentionId {
        self.left
    }

    /// Return the lexicographically larger mention identifier.
    #[must_use]
    pub const fn right(self) -> EventMentionId {
        self.right
    }
}

/// Threshold a link probability into a detection label.
///
/// The threshold is inclusive: `probability >= threshold` is linked.
#[must_use]
pub fn decide_event_link(
    probability: EventConfidence,
    threshold: EventConfidence,
) -> EventLinkLabel {
    if probability.value() >= threshold.value() {
        EventLinkLabel::Linked
    } else {
        EventLinkLabel::Unlinked
    }
}

/// Explicit refusal to treat a TDT link as an event instance.
///
/// # Errors
///
/// Always returns [`EventError::EventLinkIsNotEventInstance`].
pub fn refuse_event_link_as_instance(_link: EventLinkPair) -> Result<EventInstanceId, EventError> {
    Err(EventError::EventLinkIsNotEventInstance)
}

/// Explicit refusal to treat a TDT link as a state transition.
///
/// # Errors
///
/// Always returns [`EventError::EventLinkIsNotStateTransition`].
pub fn refuse_event_link_as_transition(_link: EventLinkPair) -> Result<(), EventError> {
    Err(EventError::EventLinkIsNotStateTransition)
}

/// Precision of recovered TDT links against the known-truth pair set.
///
/// # Errors
///
/// Returns [`EventError::InvalidWirePayload`] when the recovered set is empty.
pub fn event_link_precision(
    truth: &[EventLinkPair],
    recovered: &[EventLinkPair],
) -> Result<f64, EventError> {
    let truth_set: BTreeSet<_> = truth.iter().copied().collect();
    let recovered_set: BTreeSet<_> = recovered.iter().copied().collect();
    counted_rate(
        recovered_set.intersection(&truth_set).count(),
        recovered_set.len(),
    )
}

/// Recall of recovered TDT links against the known-truth pair set.
///
/// # Errors
///
/// Returns [`EventError::InvalidWirePayload`] when the truth set is empty.
pub fn event_link_recall(
    truth: &[EventLinkPair],
    recovered: &[EventLinkPair],
) -> Result<f64, EventError> {
    let truth_set: BTreeSet<_> = truth.iter().copied().collect();
    let recovered_set: BTreeSet<_> = recovered.iter().copied().collect();
    counted_rate(
        recovered_set.intersection(&truth_set).count(),
        truth_set.len(),
    )
}

fn counted_rate(numerator: usize, denominator: usize) -> Result<f64, EventError> {
    let numerator = u32::try_from(numerator).map_err(|_| EventError::InvalidWirePayload)?;
    let denominator = u32::try_from(denominator).map_err(|_| EventError::InvalidWirePayload)?;
    if denominator == 0 {
        return Err(EventError::InvalidWirePayload);
    }
    Ok(f64::from(numerator) / f64::from(denominator))
}

#[cfg(test)]
mod tests {
    use super::{
        EventLinkLabel, EventLinkPair, counted_rate, decide_event_link, event_link_precision,
        event_link_recall, refuse_event_link_as_instance, refuse_event_link_as_transition,
    };
    use crate::{EventConfidence, EventError, EventMentionId};

    #[test]
    fn link_helpers_cover_local_branches() {
        let left = EventMentionId::new();
        let right = EventMentionId::new();
        let link = EventLinkPair::new(left, right).expect("pair");
        assert_eq!(
            refuse_event_link_as_instance(link),
            Err(EventError::EventLinkIsNotEventInstance)
        );
        assert_eq!(
            refuse_event_link_as_transition(link),
            Err(EventError::EventLinkIsNotStateTransition)
        );
        let high = EventConfidence::new(0.8).expect("high");
        let low = EventConfidence::new(0.2).expect("low");
        assert_eq!(decide_event_link(high, low), EventLinkLabel::Linked);
        assert_eq!(decide_event_link(low, high), EventLinkLabel::Unlinked);
        let truth = [link];
        let recovered = [link];
        assert!((event_link_precision(&truth, &recovered).expect("p") - 1.0).abs() < f64::EPSILON);
        assert!((event_link_recall(&truth, &recovered).expect("r") - 1.0).abs() < f64::EPSILON);
        assert_eq!(
            EventLinkPair::new(left, left),
            Err(EventError::InvalidWirePayload)
        );
        assert_eq!(
            event_link_precision(&truth, &[]),
            Err(EventError::InvalidWirePayload)
        );
        assert_eq!(
            event_link_recall(&[], &recovered),
            Err(EventError::InvalidWirePayload)
        );
        assert_eq!(
            counted_rate(0, usize::MAX),
            Err(EventError::InvalidWirePayload)
        );
        assert_eq!(counted_rate(1, 0), Err(EventError::InvalidWirePayload));
    }
}
