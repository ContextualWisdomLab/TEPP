//! Validation-coordinate projection of fitted document topic state into known-truth topic order.
//!
//! Topic-model component indexes are permutation-indeterminate. Document topic proportions
//! therefore cannot be compared with simulation truth by raw column position even after the
//! global topic-term rows have been aligned. This module reuses the owner-issued
//! [`crate::TopicAlignment`] mapping and changes only topic-coordinate order; it does not infer
//! semantic identity, authenticate document provenance, or authorize a release projection.

use crate::{TopicAlignment, ValidationError};

const PROBABILITY_SUM_TOLERANCE: f64 = 1.0e-10;

/// Re-express one fitted simplex topic-state vector in known-truth topic order.
///
/// The input must contain exactly one probability per fitted topic. Values must be finite,
/// nonnegative, and sum to one within the validation tolerance. The output at truth index `t`
/// is the fitted probability at `alignment.truth_to_fitted()[t]`.
///
/// # Errors
///
/// Returns [`ValidationError::InvalidInput`] when the fitted vector has the wrong width,
/// contains a non-finite or negative value, is not a normalized positive-mass simplex, or the
/// projected vector becomes invalid.
pub fn realign_topic_probability_vector(
    alignment: &TopicAlignment,
    fitted_probabilities: &[f64],
) -> Result<Vec<f64>, ValidationError> {
    validate_simplex(alignment, fitted_probabilities)?;

    let aligned: Vec<f64> = alignment
        .truth_to_fitted()
        .iter()
        .map(|fitted_index| fitted_probabilities[*fitted_index])
        .collect();
    validate_simplex(alignment, &aligned)?;
    Ok(aligned)
}

/// Re-express fitted document topic-state rows in known-truth topic order.
///
/// Row order is preserved exactly; this function changes only topic-coordinate order inside
/// each row. Document identity binding remains the caller's responsibility and should come from
/// an owner-issued fitted input rather than a caller-created positional assumption.
///
/// # Errors
///
/// Returns [`ValidationError::InvalidInput`] for an empty row set or when any row fails the
/// simplex geometry required by [`realign_topic_probability_vector`].
pub fn realign_topic_probability_rows(
    alignment: &TopicAlignment,
    fitted_rows: &[Vec<f64>],
) -> Result<Vec<Vec<f64>>, ValidationError> {
    if fitted_rows.is_empty() {
        return Err(ValidationError::InvalidInput);
    }
    fitted_rows
        .iter()
        .map(|row| realign_topic_probability_vector(alignment, row))
        .collect()
}

fn validate_simplex(
    alignment: &TopicAlignment,
    probabilities: &[f64],
) -> Result<(), ValidationError> {
    if probabilities.len() != alignment.truth_to_fitted().len()
        || probabilities.is_empty()
        || probabilities
            .iter()
            .any(|value| !value.is_finite() || *value < 0.0)
    {
        return Err(ValidationError::InvalidInput);
    }
    let mass = probabilities.iter().sum::<f64>();
    if !mass.is_finite()
        || mass <= 0.0
        || (mass - 1.0).abs() > PROBABILITY_SUM_TOLERANCE
    {
        return Err(ValidationError::InvalidInput);
    }
    Ok(())
}
