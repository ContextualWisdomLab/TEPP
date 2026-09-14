//! Pareto admission and selection among statistically supported candidates.

use crate::{ModelCandidate, ModelSelectionError};

/// Non-empty collection of validated statistical candidates.
pub(crate) struct StatisticalCandidateFront {
    first_candidate: ModelCandidate,
    remaining_candidates: Vec<ModelCandidate>,
}

impl StatisticalCandidateFront {
    pub(crate) fn new(first: ModelCandidate) -> Self {
        Self {
            first_candidate: first,
            remaining_candidates: Vec::new(),
        }
    }

    pub(crate) fn push(&mut self, candidate: ModelCandidate) {
        self.remaining_candidates.push(candidate);
    }

    pub(crate) fn selected_k(&self) -> u32 {
        let all_candidates = || {
            std::iter::once(self.first_candidate).chain(self.remaining_candidates.iter().copied())
        };
        let mut selected = self.first_candidate;
        for candidate in all_candidates() {
            if all_candidates().any(|other| other.dominates(candidate)) {
                continue;
            }
            let selected_is_dominated = all_candidates().any(|other| other.dominates(selected));
            let likelihood_order = candidate
                .held_out_log_likelihood()
                .partial_cmp(&selected.held_out_log_likelihood())
                .unwrap_or(std::cmp::Ordering::Equal);
            if selected_is_dominated
                || likelihood_order.is_gt()
                || (likelihood_order.is_eq() && candidate.candidate_k() < selected.candidate_k())
            {
                selected = candidate;
            }
        }
        selected.candidate_k()
    }
}

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
    let mut statistical = candidates
        .iter()
        .copied()
        .filter(|candidate| candidate.is_statistically_supported());
    let Some(first) = statistical.next() else {
        return Err(ModelSelectionError::LlmVoteIsNotStatisticalAuthority);
    };
    let mut front = StatisticalCandidateFront::new(first);
    for candidate in statistical {
        front.push(candidate);
    }
    Ok(front.selected_k())
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
        assert_eq!(select_candidate_k(&[b, a]).expect("reverse tie"), 2);
        let higher_likelihood = ModelCandidate::statistical(8, -20.0, 9.0).expect("likelihood");
        assert_eq!(
            select_candidate_k(&[a, higher_likelihood]).expect("likelihood tie-break"),
            8
        );
        let dominating = ModelCandidate::statistical(3, -20.0, 7.0).expect("dominating");
        assert_eq!(select_candidate_k(&[a, dominating]).expect("dominated"), 3);

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
            select_candidate_k(&[ModelCandidate::llm_vote_only(3).expect("valid llm candidate")]),
            Err(ModelSelectionError::LlmVoteIsNotStatisticalAuthority)
        );
    }
}
