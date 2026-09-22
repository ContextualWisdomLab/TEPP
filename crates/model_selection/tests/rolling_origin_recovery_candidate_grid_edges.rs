use model_selection::{
    FittedCandidateKConfig, ModelSelectionError,
    select_declared_rolling_origin_recovery_candidate_k,
};

#[test]
fn declared_recovery_grid_requires_at_least_one_window() {
    let config = FittedCandidateKConfig::new(vec![2, 3], vec![7, 11], 20, 1e-5)
        .expect("candidate grid");
    assert_eq!(
        select_declared_rolling_origin_recovery_candidate_k(&config, &[]),
        Err(ModelSelectionError::EmptyCandidateSet)
    );
}
