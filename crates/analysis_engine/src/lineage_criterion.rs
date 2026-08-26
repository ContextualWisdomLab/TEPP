//! Rust-owned posterior fitting for independently observed TDT link criteria.
//!
//! This slice fits the identified independent criterion probability. Event-time
//! draws remain producer evidence from the temporal model; it does not infer a
//! date from record order or promote CHRONOS predictions to observed facts.

use std::collections::BTreeSet;

use event_core::{
    CriterionPosteriorError, IndependentCriterionCounts, fit_independent_criterion_posterior,
};

/// Independent criterion counts and temporal draws for one exact pair.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LineageCriterionObservation {
    /// Opaque exact pair identity.
    pub pair_id: String,
    /// Independently observed positive TDT link outcomes.
    pub successes: u32,
    /// Total independently observed TDT link outcomes.
    pub trials: u32,
    /// Predecessor event-time draws supplied by the temporal model.
    pub predecessor_event_time_draws: Vec<String>,
    /// Successor event-time draws supplied by the temporal model.
    pub successor_event_time_draws: Vec<String>,
}

/// Identified posterior output for one exact pair.
#[derive(Clone, Debug, PartialEq)]
pub struct LineageCriterionFit {
    /// Opaque exact pair identity.
    pub pair_id: String,
    /// Jeffreys posterior mean.
    pub posterior_mean: f64,
    /// Jeffreys posterior variance.
    pub posterior_variance: f64,
    /// Deterministic posterior quadrature draws.
    pub criterion_draws: Vec<f64>,
    /// Unmodified predecessor event-time draws.
    pub predecessor_event_time_draws: Vec<String>,
    /// Unmodified successor event-time draws.
    pub successor_event_time_draws: Vec<String>,
}

/// Fail-closed independent criterion fitting errors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LineageCriterionFitError {
    /// No exact pair was supplied.
    EmptyInput,
    /// Pair identity was empty, padded, too long, or duplicated.
    InvalidPairIdentity,
    /// Event-time draw counts differ from the requested common draw count.
    TemporalDrawMismatch,
    /// The independently identified scientific estimator rejected the input.
    Criterion(CriterionPosteriorError),
}

/// Fit independent TDT link-criterion posteriors without local weights.
///
/// Input order is preserved. Temporal draws are carried without alteration;
/// record creation timestamps and nearest dates are never substituted.
///
/// # Errors
///
/// Fails closed for empty or duplicate pair identities, mixed draw counts, or
/// invalid independent criterion observations.
pub fn fit_lineage_criterion_posteriors(
    observations: &[LineageCriterionObservation],
    draw_count: usize,
) -> Result<Vec<LineageCriterionFit>, LineageCriterionFitError> {
    if observations.is_empty() {
        return Err(LineageCriterionFitError::EmptyInput);
    }
    let identities = observations
        .iter()
        .map(|observation| observation.pair_id.as_str())
        .collect::<BTreeSet<_>>();
    if identities.len() != observations.len()
        || observations.iter().any(|observation| {
            observation.pair_id.is_empty()
                || observation.pair_id.len() > 256
                || observation.pair_id.trim() != observation.pair_id
        })
    {
        return Err(LineageCriterionFitError::InvalidPairIdentity);
    }
    observations
        .iter()
        .map(|observation| {
            if observation.predecessor_event_time_draws.len() != draw_count
                || observation.successor_event_time_draws.len() != draw_count
            {
                return Err(LineageCriterionFitError::TemporalDrawMismatch);
            }
            let posterior = fit_independent_criterion_posterior(
                IndependentCriterionCounts {
                    successes: observation.successes,
                    trials: observation.trials,
                },
                draw_count,
            )
            .map_err(LineageCriterionFitError::Criterion)?;
            Ok(LineageCriterionFit {
                pair_id: observation.pair_id.clone(),
                posterior_mean: posterior.mean,
                posterior_variance: posterior.variance,
                criterion_draws: posterior.plausible_values,
                predecessor_event_time_draws: observation.predecessor_event_time_draws.clone(),
                successor_event_time_draws: observation.successor_event_time_draws.clone(),
            })
        })
        .collect()
}
