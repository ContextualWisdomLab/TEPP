//! Qualitative CHRONOS relation posterior from common event-time draws.
//!
//! The operation compares event clocks draw by draw. It preserves exact ties
//! and posterior uncertainty; record creation time, nearest dates, thresholds,
//! and causal labels are absent by construction.

use temporal_core::EventTime;

/// Exact qualitative relation in one common posterior draw.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DrawTemporalRelation {
    /// The predecessor event time is earlier.
    Before,
    /// The event times are exactly equal at source precision.
    Simultaneous,
    /// The predecessor event time is later.
    After,
}

/// Posterior relation frequencies and complete draw sequence.
#[derive(Clone, Debug, PartialEq)]
pub struct TemporalRelationPosterior {
    /// Complete relation draw sequence.
    pub relation_draws: Vec<DrawTemporalRelation>,
    /// Posterior mass for `Before`.
    pub before_probability: f64,
    /// Posterior mass for `Simultaneous`.
    pub simultaneous_probability: f64,
    /// Posterior mass for `After`.
    pub after_probability: f64,
}

/// Fail-closed temporal relation posterior errors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TemporalRelationPosteriorError {
    /// No posterior draws were supplied.
    EmptyDraws,
    /// The two event clocks do not share a common draw count.
    DrawCountMismatch,
}

/// Infer a qualitative temporal-relation posterior from common event-time draws.
///
/// # Errors
///
/// Fails closed for empty or unequal draw sets.
pub fn infer_temporal_relation_posterior(
    predecessor_draws: &[EventTime],
    successor_draws: &[EventTime],
) -> Result<TemporalRelationPosterior, TemporalRelationPosteriorError> {
    if predecessor_draws.is_empty() {
        return Err(TemporalRelationPosteriorError::EmptyDraws);
    }
    if predecessor_draws.len() != successor_draws.len() {
        return Err(TemporalRelationPosteriorError::DrawCountMismatch);
    }
    let relation_draws = predecessor_draws
        .iter()
        .zip(successor_draws)
        .map(
            |(predecessor, successor)| match predecessor.cmp(successor) {
                std::cmp::Ordering::Less => DrawTemporalRelation::Before,
                std::cmp::Ordering::Equal => DrawTemporalRelation::Simultaneous,
                std::cmp::Ordering::Greater => DrawTemporalRelation::After,
            },
        )
        .collect::<Vec<_>>();
    let total_u32 = u32::try_from(relation_draws.len())
        .map_err(|_| TemporalRelationPosteriorError::DrawCountMismatch)?;
    let probability = |relation| {
        let count = relation_draws
            .iter()
            .filter(|value| **value == relation)
            .count();
        let count_u32 = u32::try_from(count).expect("count is bounded by converted draw length");
        f64::from(count_u32) / f64::from(total_u32)
    };
    Ok(TemporalRelationPosterior {
        before_probability: probability(DrawTemporalRelation::Before),
        simultaneous_probability: probability(DrawTemporalRelation::Simultaneous),
        after_probability: probability(DrawTemporalRelation::After),
        relation_draws,
    })
}
