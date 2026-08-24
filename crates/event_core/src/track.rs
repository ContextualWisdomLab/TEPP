//! TDT track assignments stay distinct from instances and transitions.

use crate::{EventConfidence, EventError, EventInstanceId, EventMentionId};
use std::collections::{BTreeMap, BTreeSet};

/// Opaque TDT track identity.
///
/// A track is a hypothesized cluster of mentions over time. It is never a
/// promoted event instance and cannot create a forward state transition.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EventTrackId(u32);

impl EventTrackId {
    /// Reconstruct a track identity from a raw fixture or estimator label.
    #[must_use]
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    /// Return the raw track label.
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }
}

/// TDT continue-versus-switch label for a mention relative to the prior track.
///
/// A continue/switch decision is tracking evidence. It is never a promoted
/// event instance and cannot create a forward state transition by itself.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EventTrackLabel {
    /// The mention is scored as continuing the previous track.
    Continue,
    /// The mention is scored as a switch onto a different track.
    Switch,
}

impl EventTrackLabel {
    /// Return the stable wire label name.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Continue => "continue",
            Self::Switch => "switch",
        }
    }

    /// Parse a stable wire track label.
    ///
    /// # Errors
    ///
    /// Returns [`EventError::UnknownEventTrackLabel`] for unrecognized names.
    pub fn from_wire_name(name: &str) -> Result<Self, EventError> {
        match name {
            "continue" => Ok(Self::Continue),
            "switch" => Ok(Self::Switch),
            _ => Err(EventError::UnknownEventTrackLabel),
        }
    }

    /// Return whether this label continues the previous track.
    #[must_use]
    pub const fn is_continue(self) -> bool {
        matches!(self, Self::Continue)
    }

    /// Return the binary probability target used for RMSE.
    ///
    /// Continue truth is `1.0`; switch truth is `0.0`.
    #[must_use]
    pub const fn as_probability_target(self) -> f64 {
        match self {
            Self::Continue => 1.0,
            Self::Switch => 0.0,
        }
    }
}

/// Assignment of one mention to one hypothesized TDT track.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EventTrackAssignment {
    mention_id: EventMentionId,
    track_id: EventTrackId,
}

impl EventTrackAssignment {
    /// Bind a mention to a hypothesized track.
    #[must_use]
    pub const fn new(mention_id: EventMentionId, track_id: EventTrackId) -> Self {
        Self {
            mention_id,
            track_id,
        }
    }

    /// Return the assigned mention identity.
    #[must_use]
    pub const fn mention_id(self) -> EventMentionId {
        self.mention_id
    }

    /// Return the hypothesized track identity.
    #[must_use]
    pub const fn track_id(self) -> EventTrackId {
        self.track_id
    }
}

/// Threshold a same-track probability into a continue/switch label.
///
/// The threshold is inclusive: `probability >= threshold` continues the track.
#[must_use]
pub fn decide_track_continue(
    probability: EventConfidence,
    threshold: EventConfidence,
) -> EventTrackLabel {
    if probability.value() >= threshold.value() {
        EventTrackLabel::Continue
    } else {
        EventTrackLabel::Switch
    }
}

/// Explicit refusal to treat a TDT track as an event instance.
///
/// # Errors
///
/// Always returns [`EventError::EventTrackIsNotEventInstance`].
pub fn refuse_track_as_instance(_track: EventTrackId) -> Result<EventInstanceId, EventError> {
    Err(EventError::EventTrackIsNotEventInstance)
}

/// Explicit refusal to treat a TDT track as a state transition.
///
/// # Errors
///
/// Always returns [`EventError::EventTrackIsNotStateTransition`].
pub fn refuse_track_as_transition(_track: EventTrackId) -> Result<(), EventError> {
    Err(EventError::EventTrackIsNotStateTransition)
}

/// Precision of recovered same-track mention pairs against known truth.
///
/// # Errors
///
/// Returns [`EventError::InvalidWirePayload`] when assignments are empty,
/// mention identities collide, lengths differ, or either pair set is empty.
pub fn tracking_pair_precision(
    truth: &[EventTrackAssignment],
    recovered: &[EventTrackAssignment],
) -> Result<f64, EventError> {
    let truth_pairs = same_track_pairs(truth)?;
    let recovered_pairs = same_track_pairs(recovered)?;
    if truth.len() != recovered.len() {
        return Err(EventError::InvalidWirePayload);
    }
    counted_rate(
        recovered_pairs.intersection(&truth_pairs).count(),
        recovered_pairs.len(),
    )
}

/// Recall of recovered same-track mention pairs against known truth.
///
/// # Errors
///
/// Returns [`EventError::InvalidWirePayload`] when assignments are empty,
/// mention identities collide, lengths differ, or either pair set is empty.
pub fn tracking_pair_recall(
    truth: &[EventTrackAssignment],
    recovered: &[EventTrackAssignment],
) -> Result<f64, EventError> {
    let truth_pairs = same_track_pairs(truth)?;
    let recovered_pairs = same_track_pairs(recovered)?;
    if truth.len() != recovered.len() {
        return Err(EventError::InvalidWirePayload);
    }
    counted_rate(
        recovered_pairs.intersection(&truth_pairs).count(),
        truth_pairs.len(),
    )
}

/// Identity-switch rate among consecutive mentions that share a truth track.
///
/// # Errors
///
/// Returns [`EventError::InvalidWirePayload`] when streams are empty, lengths
/// differ, mention identities collide or disagree, or no consecutive truth
/// pair stays on the same track.
pub fn tracking_identity_switch_rate(
    truth: &[EventTrackAssignment],
    recovered: &[EventTrackAssignment],
) -> Result<f64, EventError> {
    if truth.is_empty() || truth.len() != recovered.len() {
        return Err(EventError::InvalidWirePayload);
    }
    let truth_map = unique_assignment_map(truth)?;
    let recovered_map = unique_assignment_map(recovered)?;
    let mut stay_count = 0_u32;
    let mut switch_count = 0_u32;
    for window in truth.windows(2) {
        let left = window[0].mention_id();
        let right = window[1].mention_id();
        if truth_map.get(&left) != truth_map.get(&right) {
            continue;
        }
        stay_count += 1;
        let recovered_left = recovered_map
            .get(&left)
            .ok_or(EventError::InvalidWirePayload)?;
        let recovered_right = recovered_map
            .get(&right)
            .ok_or(EventError::InvalidWirePayload)?;
        if recovered_left != recovered_right {
            switch_count += 1;
        }
    }
    if stay_count == 0 {
        return Err(EventError::InvalidWirePayload);
    }
    Ok(f64::from(switch_count) / f64::from(stay_count))
}

fn unique_assignment_map(
    assignments: &[EventTrackAssignment],
) -> Result<BTreeMap<EventMentionId, EventTrackId>, EventError> {
    if assignments.is_empty() {
        return Err(EventError::InvalidWirePayload);
    }
    let mut map = BTreeMap::new();
    for assignment in assignments {
        if map
            .insert(assignment.mention_id(), assignment.track_id())
            .is_some()
        {
            return Err(EventError::InvalidWirePayload);
        }
    }
    Ok(map)
}

fn same_track_pairs(
    assignments: &[EventTrackAssignment],
) -> Result<BTreeSet<(EventMentionId, EventMentionId)>, EventError> {
    let map = unique_assignment_map(assignments)?;
    let mut pairs = BTreeSet::new();
    let mentions: Vec<EventMentionId> = map.keys().copied().collect();
    for (index, left) in mentions.iter().enumerate() {
        for right in mentions.iter().skip(index + 1) {
            if map.get(left) == map.get(right) {
                pairs.insert((*left, *right));
            }
        }
    }
    if pairs.is_empty() {
        return Err(EventError::InvalidWirePayload);
    }
    Ok(pairs)
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
        EventTrackAssignment, EventTrackId, EventTrackLabel, counted_rate, decide_track_continue,
        refuse_track_as_instance, refuse_track_as_transition, tracking_identity_switch_rate,
        tracking_pair_precision, tracking_pair_recall,
    };
    use crate::{EventConfidence, EventError, EventMentionId};

    fn assigned(mention_id: EventMentionId, track: u32) -> EventTrackAssignment {
        EventTrackAssignment::new(mention_id, EventTrackId::from_raw(track))
    }

    #[test]
    fn track_helpers_cover_local_branches() {
        let track = EventTrackId::from_raw(3);
        assert_eq!(
            refuse_track_as_instance(track),
            Err(EventError::EventTrackIsNotEventInstance)
        );
        assert_eq!(
            refuse_track_as_transition(track),
            Err(EventError::EventTrackIsNotStateTransition)
        );
        let high = EventConfidence::new(0.8).expect("high");
        let low = EventConfidence::new(0.2).expect("low");
        assert_eq!(decide_track_continue(high, low), EventTrackLabel::Continue);
        assert_eq!(decide_track_continue(low, high), EventTrackLabel::Switch);
        let left = EventMentionId::new();
        let right = EventMentionId::new();
        let truth = [assigned(left, 1), assigned(right, 1)];
        assert!((tracking_pair_precision(&truth, &truth).expect("p") - 1.0).abs() < f64::EPSILON);
        assert!((tracking_pair_recall(&truth, &truth).expect("r") - 1.0).abs() < f64::EPSILON);
        assert!(
            (tracking_identity_switch_rate(&truth, &truth).expect("s") - 0.0).abs() < f64::EPSILON
        );
        let switched = [assigned(left, 1), assigned(right, 2)];
        assert!(
            (tracking_identity_switch_rate(&truth, &switched).expect("sw") - 1.0).abs()
                < f64::EPSILON
        );
        assert_eq!(
            counted_rate(0, usize::MAX),
            Err(EventError::InvalidWirePayload)
        );
        assert_eq!(
            counted_rate(usize::MAX, 1),
            Err(EventError::InvalidWirePayload)
        );
        assert_eq!(counted_rate(1, 0), Err(EventError::InvalidWirePayload));
        cover_fail_closed_assignment_streams(left, right);
    }

    fn cover_fail_closed_assignment_streams(left: EventMentionId, right: EventMentionId) {
        let truth = [assigned(left, 1), assigned(right, 1)];
        let switched = [assigned(left, 1), assigned(right, 2)];
        let third = EventMentionId::new();
        let fourth = EventMentionId::new();
        let three = [assigned(left, 1), assigned(right, 1), assigned(third, 2)];
        let four = [
            assigned(left, 1),
            assigned(right, 1),
            assigned(third, 2),
            assigned(fourth, 2),
        ];
        assert_eq!(
            tracking_pair_precision(&truth, &[]),
            Err(EventError::InvalidWirePayload)
        );
        assert_eq!(
            tracking_identity_switch_rate(&truth, &[]),
            Err(EventError::InvalidWirePayload)
        );
        assert_eq!(
            unique_missing_recovered_switch(),
            Err(EventError::InvalidWirePayload)
        );
        assert_eq!(
            tracking_pair_precision(&three, &four),
            Err(EventError::InvalidWirePayload)
        );
        assert_eq!(
            tracking_pair_recall(&three, &four),
            Err(EventError::InvalidWirePayload)
        );
        assert_eq!(
            tracking_pair_recall(&truth, &switched),
            Err(EventError::InvalidWirePayload)
        );
        assert_eq!(
            tracking_identity_switch_rate(&[assigned(left, 1), assigned(left, 2)], &truth),
            Err(EventError::InvalidWirePayload)
        );
        assert_eq!(
            tracking_identity_switch_rate(&truth, &[assigned(left, 1), assigned(left, 1)]),
            Err(EventError::InvalidWirePayload)
        );
        assert_eq!(
            tracking_identity_switch_rate(
                &truth,
                &[
                    assigned(EventMentionId::new(), 1),
                    assigned(EventMentionId::new(), 1)
                ]
            ),
            Err(EventError::InvalidWirePayload)
        );
        assert_eq!(
            tracking_identity_switch_rate(&[], &[]),
            Err(EventError::InvalidWirePayload)
        );
        assert!(
            (tracking_identity_switch_rate(&three, &three).expect("changed") - 0.0).abs()
                < f64::EPSILON
        );
        assert_eq!(
            tracking_identity_switch_rate(&[assigned(left, 1)], &[assigned(left, 1)]),
            Err(EventError::InvalidWirePayload)
        );
    }

    fn unique_missing_recovered_switch() -> Result<f64, EventError> {
        let left = EventMentionId::new();
        let right = EventMentionId::new();
        let extra = EventMentionId::new();
        let truth = [
            EventTrackAssignment::new(left, EventTrackId::from_raw(1)),
            EventTrackAssignment::new(right, EventTrackId::from_raw(1)),
        ];
        let recovered = [
            EventTrackAssignment::new(left, EventTrackId::from_raw(1)),
            EventTrackAssignment::new(extra, EventTrackId::from_raw(1)),
        ];
        tracking_identity_switch_rate(&truth, &recovered)
    }
}
