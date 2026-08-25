//! TDT story-segmentation scores stay distinct from instances and transitions.

use crate::{EventConfidence, EventError, EventInstanceId};
use std::collections::BTreeSet;

/// TDT story-boundary versus continuation label.
///
/// A boundary decision is detection evidence. It is never a promoted event
/// instance and cannot create a forward state transition by itself.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StoryBoundaryLabel {
    /// A new story starts after this unit.
    Boundary,
    /// The next unit continues the current story.
    Continuation,
}

impl StoryBoundaryLabel {
    /// Return the stable wire label name.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Boundary => "boundary",
            Self::Continuation => "continuation",
        }
    }

    /// Parse a stable wire story-boundary label.
    ///
    /// # Errors
    ///
    /// Returns [`EventError::UnknownStoryBoundaryLabel`] for unrecognized names.
    pub fn from_wire_name(name: &str) -> Result<Self, EventError> {
        match name {
            "boundary" => Ok(Self::Boundary),
            "continuation" => Ok(Self::Continuation),
            _ => Err(EventError::UnknownStoryBoundaryLabel),
        }
    }

    /// Return whether this label marks a story boundary.
    #[must_use]
    pub const fn is_boundary(self) -> bool {
        matches!(self, Self::Boundary)
    }

    /// Return the binary probability target used for RMSE.
    ///
    /// Boundary truth is `1.0`; continuation truth is `0.0`.
    #[must_use]
    pub const fn as_probability_target(self) -> f64 {
        match self {
            Self::Boundary => 1.0,
            Self::Continuation => 0.0,
        }
    }
}

/// Ordered TDT story/event segmentation of a measurement-unit sequence.
///
/// `boundary_after[i]` is true when a new story starts after unit `i`. There
/// are `unit_count - 1` interior candidate boundaries. The partition is
/// detection evidence, not a promoted event instance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StorySegmentation {
    unit_count: u32,
    boundary_after: Vec<bool>,
}

impl StorySegmentation {
    /// Construct a validated segmentation of `unit_count` ordered units.
    ///
    /// # Errors
    ///
    /// Returns [`EventError::InvalidWirePayload`] when fewer than two units
    /// are supplied or `boundary_after` is not exactly `unit_count - 1` long.
    pub fn new(unit_count: u32, boundary_after: Vec<bool>) -> Result<Self, EventError> {
        if unit_count < 2 {
            return Err(EventError::InvalidWirePayload);
        }
        let expected = (unit_count - 1) as usize;
        if boundary_after.len() != expected {
            return Err(EventError::InvalidWirePayload);
        }
        Ok(Self {
            unit_count,
            boundary_after,
        })
    }

    /// Return the number of ordered measurement units.
    #[must_use]
    pub const fn unit_count(&self) -> u32 {
        self.unit_count
    }

    /// Return interior boundary decisions after each unit except the last.
    #[must_use]
    pub fn boundary_after(&self) -> &[bool] {
        &self.boundary_after
    }
}

/// Threshold a boundary probability into a detection label.
///
/// The threshold is inclusive: `probability >= threshold` is a boundary.
#[must_use]
pub fn decide_story_boundary(
    probability: EventConfidence,
    threshold: EventConfidence,
) -> StoryBoundaryLabel {
    if probability.value() >= threshold.value() {
        StoryBoundaryLabel::Boundary
    } else {
        StoryBoundaryLabel::Continuation
    }
}

/// Explicit refusal to treat a TDT story segmentation as an event instance.
///
/// # Errors
///
/// Always returns [`EventError::StorySegmentationIsNotEventInstance`].
pub fn refuse_story_segmentation_as_instance(
    _segmentation: &StorySegmentation,
) -> Result<EventInstanceId, EventError> {
    Err(EventError::StorySegmentationIsNotEventInstance)
}

/// Explicit refusal to treat a TDT story segmentation as a state transition.
///
/// # Errors
///
/// Always returns [`EventError::StorySegmentationIsNotStateTransition`].
pub fn refuse_story_segmentation_as_transition(
    _segmentation: &StorySegmentation,
) -> Result<(), EventError> {
    Err(EventError::StorySegmentationIsNotStateTransition)
}

/// Precision of recovered interior story boundaries against known truth.
///
/// # Errors
///
/// Returns [`EventError::InvalidWirePayload`] when the sequences differ in
/// length or the recovered set contains no boundary.
pub fn story_boundary_precision(
    truth: &StorySegmentation,
    recovered: &StorySegmentation,
) -> Result<f64, EventError> {
    let (truth_set, recovered_set) = aligned_boundary_sets(truth, recovered)?;
    counted_rate(
        truth_set.intersection(&recovered_set).count(),
        recovered_set.len(),
    )
}

/// Recall of recovered interior story boundaries against known truth.
///
/// # Errors
///
/// Returns [`EventError::InvalidWirePayload`] when the sequences differ in
/// length or the truth set contains no boundary.
pub fn story_boundary_recall(
    truth: &StorySegmentation,
    recovered: &StorySegmentation,
) -> Result<f64, EventError> {
    let (truth_set, recovered_set) = aligned_boundary_sets(truth, recovered)?;
    counted_rate(
        truth_set.intersection(&recovered_set).count(),
        truth_set.len(),
    )
}

/// Pevzner–Hearst `WindowDiff` between a known-truth and recovered segmentation.
///
/// The window counts interior boundaries in each aligned span of `window`
/// units. A window mismatches when the counts differ.
///
/// # Errors
///
/// Returns [`EventError::InvalidWirePayload`] when the sequences differ in
/// length or `window` is zero or at least the unit count.
pub fn story_window_diff(
    truth: &StorySegmentation,
    recovered: &StorySegmentation,
    window: u32,
) -> Result<f64, EventError> {
    window_probe(truth, recovered, window, |truth_count, recovered_count| {
        truth_count != recovered_count
    })
}

/// Beeferman Pk between a known-truth and recovered segmentation.
///
/// Probe pairs `window` units apart disagree when one partition places them
/// in the same story and the other does not.
///
/// # Errors
///
/// Returns [`EventError::InvalidWirePayload`] when the sequences differ in
/// length or `window` is zero or at least the unit count.
pub fn story_pk(
    truth: &StorySegmentation,
    recovered: &StorySegmentation,
    window: u32,
) -> Result<f64, EventError> {
    window_probe(truth, recovered, window, |truth_count, recovered_count| {
        (truth_count == 0) != (recovered_count == 0)
    })
}

fn aligned_boundary_sets(
    truth: &StorySegmentation,
    recovered: &StorySegmentation,
) -> Result<(BTreeSet<usize>, BTreeSet<usize>), EventError> {
    if truth.unit_count != recovered.unit_count {
        return Err(EventError::InvalidWirePayload);
    }
    Ok((
        boundary_indices(&truth.boundary_after),
        boundary_indices(&recovered.boundary_after),
    ))
}

fn boundary_indices(boundary_after: &[bool]) -> BTreeSet<usize> {
    boundary_after
        .iter()
        .enumerate()
        .filter_map(|(index, is_boundary)| is_boundary.then_some(index))
        .collect()
}

fn window_probe(
    truth: &StorySegmentation,
    recovered: &StorySegmentation,
    window: u32,
    disagree: impl Fn(usize, usize) -> bool,
) -> Result<f64, EventError> {
    if truth.unit_count != recovered.unit_count || window == 0 || window >= truth.unit_count {
        return Err(EventError::InvalidWirePayload);
    }
    let probe_count = (truth.unit_count - window) as usize;
    let window = window as usize;
    let truth_prefix = boundary_prefix_counts(&truth.boundary_after);
    let recovered_prefix = boundary_prefix_counts(&recovered.boundary_after);
    let mut disagreements = 0_usize;
    for start in 0..probe_count {
        let end = start + window;
        if disagree(
            usize::try_from(truth_prefix[end] - truth_prefix[start])
                .map_err(|_| EventError::InvalidWirePayload)?,
            usize::try_from(recovered_prefix[end] - recovered_prefix[start])
                .map_err(|_| EventError::InvalidWirePayload)?,
        ) {
            disagreements += 1;
        }
    }
    counted_rate(disagreements, probe_count)
}

fn boundary_prefix_counts(boundary_after: &[bool]) -> Vec<u32> {
    let mut prefix = Vec::with_capacity(boundary_after.len() + 1);
    prefix.push(0);
    for &is_boundary in boundary_after {
        let next = prefix.last().copied().unwrap_or(0) + u32::from(is_boundary);
        prefix.push(next);
    }
    prefix
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
        StoryBoundaryLabel, StorySegmentation, counted_rate, decide_story_boundary,
        refuse_story_segmentation_as_instance, refuse_story_segmentation_as_transition,
        story_boundary_precision, story_boundary_recall, story_pk, story_window_diff,
    };
    use crate::{EventConfidence, EventError};

    #[test]
    fn segmentation_helpers_cover_local_branches() {
        let story = StorySegmentation::new(4, vec![false, true, false]).expect("story");
        assert_eq!(story.unit_count(), 4);
        assert_eq!(story.boundary_after(), &[false, true, false]);
        assert_eq!(
            refuse_story_segmentation_as_instance(&story),
            Err(EventError::StorySegmentationIsNotEventInstance)
        );
        assert_eq!(
            refuse_story_segmentation_as_transition(&story),
            Err(EventError::StorySegmentationIsNotStateTransition)
        );
        let high = EventConfidence::new(0.8).expect("high");
        let low = EventConfidence::new(0.2).expect("low");
        assert_eq!(
            decide_story_boundary(high, low),
            StoryBoundaryLabel::Boundary
        );
        assert_eq!(
            decide_story_boundary(low, high),
            StoryBoundaryLabel::Continuation
        );
        assert!((story_boundary_precision(&story, &story).expect("p") - 1.0).abs() < f64::EPSILON);
        assert!((story_boundary_recall(&story, &story).expect("r") - 1.0).abs() < f64::EPSILON);
        assert!(story_window_diff(&story, &story, 2).expect("wd").abs() < f64::EPSILON);
        assert!(story_pk(&story, &story, 2).expect("pk").abs() < f64::EPSILON);
        assert_eq!(
            StorySegmentation::new(1, vec![]),
            Err(EventError::InvalidWirePayload)
        );
        assert_eq!(
            StorySegmentation::new(4, vec![true]),
            Err(EventError::InvalidWirePayload)
        );
        let other = StorySegmentation::new(5, vec![false, true, false, false]).expect("other");
        assert_eq!(
            story_boundary_precision(&story, &other),
            Err(EventError::InvalidWirePayload)
        );
        assert_eq!(
            story_window_diff(&story, &other, 2),
            Err(EventError::InvalidWirePayload)
        );
        assert_eq!(
            story_pk(&story, &story, 0),
            Err(EventError::InvalidWirePayload)
        );
        assert_eq!(
            story_window_diff(&story, &story, 4),
            Err(EventError::InvalidWirePayload)
        );
        assert_eq!(
            story_boundary_precision(
                &story,
                &StorySegmentation::new(4, vec![false, false, false]).expect("empty")
            ),
            Err(EventError::InvalidWirePayload)
        );
        assert_eq!(
            story_boundary_recall(
                &StorySegmentation::new(4, vec![false, false, false]).expect("empty"),
                &story
            ),
            Err(EventError::InvalidWirePayload)
        );
        assert_eq!(
            counted_rate(0, usize::MAX),
            Err(EventError::InvalidWirePayload)
        );
        assert_eq!(counted_rate(1, 0), Err(EventError::InvalidWirePayload));
        assert!((counted_rate(1, 2).expect("half") - 0.5).abs() < f64::EPSILON);
        assert_eq!(
            StoryBoundaryLabel::from_wire_name("cut"),
            Err(EventError::UnknownStoryBoundaryLabel)
        );
        assert_eq!(StoryBoundaryLabel::Boundary.wire_name(), "boundary");
        assert_eq!(StoryBoundaryLabel::Continuation.wire_name(), "continuation");
        assert!(StoryBoundaryLabel::Boundary.is_boundary());
        assert!(!StoryBoundaryLabel::Continuation.is_boundary());
        assert!((StoryBoundaryLabel::Boundary.as_probability_target() - 1.0).abs() < f64::EPSILON);
        assert!(
            (StoryBoundaryLabel::Continuation.as_probability_target() - 0.0).abs() < f64::EPSILON
        );
    }
}
