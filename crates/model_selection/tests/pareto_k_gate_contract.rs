//! Statistical/Pareto K gates run before any LLM review and recover known K.

use model_selection::{
    ModelCandidate, ModelSelectionError, select_candidate_k, selected_k_root_mean_square_error,
};

fn candidate(k: u32, log_likelihood: f64, complexity: f64) -> ModelCandidate {
    ModelCandidate::statistical(k, log_likelihood, complexity).expect("statistical candidate")
}

#[test]
fn non_positive_k_and_non_finite_diagnostics_fail_closed() {
    assert_eq!(
        ModelCandidate::statistical(1, -10.0, 4.0),
        Err(ModelSelectionError::NonPositiveCandidateK)
    );
    assert_eq!(
        ModelCandidate::statistical(3, f64::NAN, 4.0),
        Err(ModelSelectionError::InvalidDiagnostic)
    );
    assert_eq!(
        ModelCandidate::statistical(3, -10.0, f64::INFINITY),
        Err(ModelSelectionError::InvalidDiagnostic)
    );
    assert_eq!(
        ModelCandidate::statistical(3, -10.0, -0.1),
        Err(ModelSelectionError::InvalidDiagnostic)
    );
}

#[test]
fn llm_vote_cannot_define_the_numerical_optimum() {
    let only_llm = ModelCandidate::llm_vote_only(5);
    assert_eq!(
        select_candidate_k(&[only_llm]),
        Err(ModelSelectionError::LlmVoteIsNotStatisticalAuthority)
    );
}

#[test]
fn pareto_front_selects_known_truth_k_with_computed_rmse() {
    let truth_k = 4_u32;
    let candidates = [
        candidate(2, -100.0, 10.0),
        candidate(truth_k, -40.0, 20.0),
        candidate(8, -45.0, 40.0),
        ModelCandidate::llm_vote_only(6),
    ];

    let selected = select_candidate_k(&candidates).expect("admissible statistical front");
    assert_eq!(selected, truth_k);

    let rmse = selected_k_root_mean_square_error(&[selected], truth_k).expect("rmse");
    let expected = {
        let residual = f64::from(selected) - f64::from(truth_k);
        (residual * residual).sqrt()
    };
    assert!((rmse - expected).abs() < f64::EPSILON);
    assert!(rmse < 0.5);
    assert_eq!(
        select_candidate_k(&[candidate(2, -30.0, 8.0), candidate(4, -30.0, 8.0)]),
        Ok(2)
    );
    assert_eq!(
        selected_k_root_mean_square_error(&[], truth_k),
        Err(ModelSelectionError::EmptyCandidateSet)
    );
    assert_eq!(
        selected_k_root_mean_square_error(&[selected], 1),
        Err(ModelSelectionError::NonPositiveCandidateK)
    );
}

#[test]
fn empty_candidate_sets_abstain() {
    assert_eq!(
        select_candidate_k(&[]),
        Err(ModelSelectionError::EmptyCandidateSet)
    );
}
