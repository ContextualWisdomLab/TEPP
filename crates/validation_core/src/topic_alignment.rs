//! Deterministic one-to-one alignment of fitted topics to known simulation truth.
//!
//! Topic-model component indexes are not stable scientific identities under label
//! permutation. Recovery metrics therefore need an explicit alignment step before
//! fitted content, prevalence, or document-state coordinates are compared with a
//! known generating basis. This module uses squared Hellinger distance between
//! topic-term probability rows and solves the global square assignment problem.
//! It does not rename, reorder, or mutate fitted model state.

use crate::ValidationError;

const PROBABILITY_SUM_TOLERANCE: f64 = 1.0e-10;

/// One deterministic one-to-one mapping from known-truth topics to fitted topics.
#[derive(Clone, Debug, PartialEq)]
pub struct TopicAlignment {
    truth_to_fitted: Vec<usize>,
    total_squared_hellinger_distance: f64,
}

impl TopicAlignment {
    /// Return the fitted topic index assigned to every truth topic index.
    #[must_use]
    pub fn truth_to_fitted(&self) -> &[usize] {
        &self.truth_to_fitted
    }

    /// Return the sum of squared Hellinger distances under the optimal assignment.
    #[must_use]
    pub const fn total_squared_hellinger_distance(&self) -> f64 {
        self.total_squared_hellinger_distance
    }
}

/// Align fitted topic-term probability rows to a known-truth topic basis.
///
/// Both matrices must contain the same number of topics and vocabulary columns,
/// with at least two topics and two terms. Every row must be a finite,
/// nonnegative probability distribution whose mass sums to one within the
/// validation tolerance. Pairwise row costs are squared Hellinger distances,
/// `0.5 * Σ_v (sqrt(p_v) - sqrt(q_v))²`. A deterministic Hungarian assignment
/// then minimizes total cost over all one-to-one truth/fitted topic mappings.
///
/// The returned mapping is a validation coordinate map only. It does not mutate
/// the fitted basis, establish semantic topic identity, authenticate vocabulary
/// provenance, or authorize a release projection.
///
/// # Errors
///
/// Returns [`ValidationError::InvalidInput`] for empty, one-topic, ragged,
/// dimension-mismatched, non-finite, negative, zero-mass, or non-normalized
/// probability matrices.
pub fn align_topic_probability_rows(
    truth: &[Vec<f64>],
    fitted: &[Vec<f64>],
) -> Result<TopicAlignment, ValidationError> {
    let vocabulary_size = validate_probability_basis(truth)?;
    if validate_probability_basis(fitted)? != vocabulary_size || truth.len() != fitted.len() {
        return Err(ValidationError::InvalidInput);
    }

    let costs: Vec<Vec<f64>> = truth
        .iter()
        .map(|truth_row| {
            fitted
                .iter()
                .map(|fitted_row| squared_hellinger(truth_row, fitted_row))
                .collect()
        })
        .collect();
    let truth_to_fitted = minimum_cost_assignment(&costs);
    let total_squared_hellinger_distance = truth_to_fitted
        .iter()
        .enumerate()
        .map(|(truth_index, fitted_index)| costs[truth_index][*fitted_index])
        .sum();

    Ok(TopicAlignment {
        truth_to_fitted,
        total_squared_hellinger_distance,
    })
}

fn validate_probability_basis(rows: &[Vec<f64>]) -> Result<usize, ValidationError> {
    if rows.len() < 2 {
        return Err(ValidationError::InvalidInput);
    }
    let vocabulary_size = rows.first().map_or(0, Vec::len);
    if vocabulary_size < 2 {
        return Err(ValidationError::InvalidInput);
    }
    for row in rows {
        if row.len() != vocabulary_size
            || row.iter().any(|value| !value.is_finite() || *value < 0.0)
        {
            return Err(ValidationError::InvalidInput);
        }
        let mass = row.iter().sum::<f64>();
        if !mass.is_finite()
            || mass <= 0.0
            || (mass - 1.0).abs() > PROBABILITY_SUM_TOLERANCE
        {
            return Err(ValidationError::InvalidInput);
        }
    }
    Ok(vocabulary_size)
}

fn squared_hellinger(left: &[f64], right: &[f64]) -> f64 {
    0.5 * left
        .iter()
        .zip(right)
        .map(|(left_value, right_value)| (left_value.sqrt() - right_value.sqrt()).powi(2))
        .sum::<f64>()
}

/// Solve a validated square minimum-cost assignment with deterministic tie order.
///
/// This is the shortest-path/potential form of the Hungarian method. Rows are
/// truth topics, columns are fitted topics, and ascending column order breaks
/// exact floating-point ties without post-hoc topic reordering. Callers validate
/// square finite nonnegative geometry before constructing `costs`.
fn minimum_cost_assignment(costs: &[Vec<f64>]) -> Vec<usize> {
    let n = costs.len();
    let mut row_potential = vec![0.0_f64; n + 1];
    let mut column_potential = vec![0.0_f64; n + 1];
    let mut matched_row_for_column = vec![0_usize; n + 1];
    let mut predecessor_column = vec![0_usize; n + 1];

    for row in 1..=n {
        matched_row_for_column[0] = row;
        let mut column = 0_usize;
        let mut minimum_reduced_cost = vec![f64::INFINITY; n + 1];
        let mut used_column = vec![false; n + 1];

        loop {
            used_column[column] = true;
            let active_row = matched_row_for_column[column];
            let mut delta = f64::INFINITY;
            let mut next_column = 0_usize;

            for candidate_column in 1..=n {
                if used_column[candidate_column] {
                    continue;
                }
                let reduced_cost = costs[active_row - 1][candidate_column - 1]
                    - row_potential[active_row]
                    - column_potential[candidate_column];
                if reduced_cost < minimum_reduced_cost[candidate_column] {
                    minimum_reduced_cost[candidate_column] = reduced_cost;
                    predecessor_column[candidate_column] = column;
                }
                if minimum_reduced_cost[candidate_column] < delta {
                    delta = minimum_reduced_cost[candidate_column];
                    next_column = candidate_column;
                }
            }

            for candidate_column in 0..=n {
                if used_column[candidate_column] {
                    row_potential[matched_row_for_column[candidate_column]] += delta;
                    column_potential[candidate_column] -= delta;
                } else {
                    minimum_reduced_cost[candidate_column] -= delta;
                }
            }
            column = next_column;
            if matched_row_for_column[column] == 0 {
                break;
            }
        }

        loop {
            let previous_column = predecessor_column[column];
            matched_row_for_column[column] = matched_row_for_column[previous_column];
            column = previous_column;
            if column == 0 {
                break;
            }
        }
    }

    let mut assignment = vec![0_usize; n];
    for column in 1..=n {
        assignment[matched_row_for_column[column] - 1] = column - 1;
    }
    assignment
}

#[cfg(test)]
mod tests {
    use super::minimum_cost_assignment;

    #[test]
    fn exact_assignment_ties_use_stable_column_order() {
        let costs = vec![vec![0.0, 0.0], vec![0.0, 0.0]];
        assert_eq!(minimum_cost_assignment(&costs), vec![0, 1]);
    }
}
