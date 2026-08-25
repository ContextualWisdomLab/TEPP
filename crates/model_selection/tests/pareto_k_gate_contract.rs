//! Statistical/Pareto K gates are deterministic and recover known K without LLM authority.

use model_selection::{
    ModelCandidate, ModelSelectionError, select_candidate_k, selected_k_root_mean_square_error,
};

fn candidate(k: u32, log_likelihood: f64, complexity: f64) -> ModelCandidate {
    ModelCandidate::statistical(k, log_likelihood, complexity).expect("statistical candidate")
}

fn synthetic_candidates(truth_k: u32, noisy_replication: bool) -> [ModelCandidate; 3] {
    let true_log_likelihood = if noisy_replication { -20.0 } else { -10.0 };
    [
        candidate(truth_k, true_log_likelihood, f64::from(truth_k)),
        candidate(truth_k - 1, -30.0, f64::from(truth_k - 1)),
        candidate(
            truth_k + 1,
            if noisy_replication { -19.0 } else { -25.0 },
            f64::from(truth_k + 1),
        ),
    ]
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
    let only_llm = ModelCandidate::llm_vote_only(5).expect("valid llm candidate");
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
        ModelCandidate::llm_vote_only(6).expect("valid llm candidate"),
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
fn repeated_synthetic_truth_recovers_k_with_bounded_error_and_bias() {
    let truth = [3_u32, 4, 5, 6, 7, 8];
    let selected: Vec<u32> = truth
        .iter()
        .enumerate()
        .map(|(replication, truth_k)| {
            select_candidate_k(&synthetic_candidates(*truth_k, replication == 4))
                .expect("synthetic statistical front")
        })
        .collect();

    let matching = truth
        .iter()
        .zip(&selected)
        .filter(|(truth_k, selected_k)| truth_k == selected_k)
        .count();
    let sum_squared_error: f64 = truth
        .iter()
        .zip(&selected)
        .map(|(truth_k, selected_k)| {
            let residual = f64::from(*selected_k) - f64::from(*truth_k);
            residual * residual
        })
        .sum();
    let bias: f64 = truth
        .iter()
        .zip(&selected)
        .map(|(truth_k, selected_k)| f64::from(*selected_k) - f64::from(*truth_k))
        .sum::<f64>()
        / f64::from(u32::try_from(truth.len()).expect("small fixture"));

    assert_eq!(selected, vec![3, 4, 5, 6, 8, 8]);
    assert_eq!(matching, 5);
    let replication_count = f64::from(u32::try_from(truth.len()).expect("small fixture"));
    assert!((sum_squared_error / replication_count).sqrt() < 0.5);
    assert!((bias - (1.0 / 6.0)).abs() < f64::EPSILON);
    for (truth_k, selected_k) in truth.iter().zip(&selected) {
        assert!(
            selected_k_root_mean_square_error(&[*selected_k], *truth_k).expect("replication RMSE")
                <= 1.0
        );
    }
}

#[test]
fn empty_candidate_sets_abstain() {
    assert_eq!(
        select_candidate_k(&[]),
        Err(ModelSelectionError::EmptyCandidateSet)
    );
}
