//! Pareto admission and selection among statistically supported candidates.

use crate::{ModelCandidate, ModelSelectionError};

/// Select the unique admissible `K` from a Pareto-filtered statistical front.
///
/// LLM-only candidates are ignored as recommenders and never become the
/// numerical optimum. Among non-dominated statistical candidates the gate
/// prefers higher held-out log-likelihood, then smaller `K`; complexity is
/// applied while constructing the Pareto front.
///
/// # Errors
///
/// Returns [`ModelSelectionError::EmptyCandidateSet`] when no candidates are
/// supplied or [`ModelSelectionError::LlmVoteIsNotStatisticalAuthority`] when
/// every candidate is an LLM vote.
pub fn select_candidate_k(candidates: &[ModelCandidate]) -> Result<u32, ModelSelectionError> {
    if candidates.is_empty() {
        return Err(ModelSelectionError::EmptyCandidateSet);
    }
    if candidates
        .iter()
        .all(|candidate| candidate.is_llm_vote_only())
    {
        return Err(ModelSelectionError::LlmVoteIsNotStatisticalAuthority);
    }

    let statistical: Vec<ModelCandidate> = candidates
        .iter()
        .copied()
        .filter(|candidate| candidate.is_statistically_supported())
        .collect();
    let mut front: Vec<ModelCandidate> = statistical
        .iter()
        .copied()
        .filter(|candidate| !statistical.iter().any(|other| other.dominates(*candidate)))
        .collect();
    front.sort_by(|left, right| {
        let ll_ord = right
            .held_out_log_likelihood()
            .partial_cmp(&left.held_out_log_likelihood())
            .unwrap_or(std::cmp::Ordering::Equal);
        if ll_ord != std::cmp::Ordering::Equal {
            return ll_ord;
        }
        left.candidate_k().cmp(&right.candidate_k())
    });
    Ok(front[0].candidate_k())
}

/// RMSE of selected `K` replications against a known-truth topic count.
///
/// # Errors
///
/// Returns [`ModelSelectionError::EmptyCandidateSet`] when `selected` is
/// empty, [`ModelSelectionError::NonPositiveCandidateK`] when `truth_k` is
/// less than two, or [`ModelSelectionError::InvalidDiagnostic`] when a
/// selected replication is less than two.
pub fn selected_k_root_mean_square_error(
    selected: &[u32],
    truth_k: u32,
) -> Result<f64, ModelSelectionError> {
    if selected.is_empty() {
        return Err(ModelSelectionError::EmptyCandidateSet);
    }
    if truth_k < 2 {
        return Err(ModelSelectionError::NonPositiveCandidateK);
    }
    let mut sum_squares = 0.0_f64;
    for selected_k in selected {
        if *selected_k < 2 {
            return Err(ModelSelectionError::InvalidDiagnostic);
        }
        let residual = f64::from(*selected_k) - f64::from(truth_k);
        sum_squares += residual * residual;
    }
    Ok((sum_squares / selected.len() as f64).sqrt())
}

#[cfg(test)]
mod tests {
    use super::{select_candidate_k, selected_k_root_mean_square_error};
    use crate::{ModelCandidate, ModelSelectionError};

    #[test]
    fn gate_helpers_cover_local_branches() {
        let a = ModelCandidate::statistical(2, -30.0, 8.0).expect("a");
        let b = ModelCandidate::statistical(4, -30.0, 8.0).expect("b");
        assert_eq!(
            select_candidate_k(&[]),
            Err(ModelSelectionError::EmptyCandidateSet)
        );
        assert_eq!(select_candidate_k(&[a, b]).expect("tie"), 2);
        let higher_likelihood = ModelCandidate::statistical(8, -20.0, 9.0).expect("likelihood");
        assert_eq!(
            select_candidate_k(&[a, higher_likelihood]).expect("likelihood tie-break"),
            8
        );

        assert_eq!(
            selected_k_root_mean_square_error(&[], 4),
            Err(ModelSelectionError::EmptyCandidateSet)
        );
        assert_eq!(
            selected_k_root_mean_square_error(&[4], 1),
            Err(ModelSelectionError::NonPositiveCandidateK)
        );
        assert_eq!(
            selected_k_root_mean_square_error(&[1], 4),
            Err(ModelSelectionError::InvalidDiagnostic)
        );
        assert!(
            selected_k_root_mean_square_error(&[4], 4)
                .expect("valid rmse")
                .abs()
                < f64::EPSILON
        );
        assert_eq!(
            select_candidate_k(&[ModelCandidate::llm_vote_only(3)]),
            Err(ModelSelectionError::LlmVoteIsNotStatisticalAuthority)
        );
    }
}
