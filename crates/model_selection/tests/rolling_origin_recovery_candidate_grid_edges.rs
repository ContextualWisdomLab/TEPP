use model_selection::{
    FittedCandidateKConfig, ModelSelectionError,
    select_declared_rolling_origin_recovery_candidate_k,
    select_declared_rolling_origin_recovery_candidate_k_for_cutoffs,
};
use temporal_core::KnowledgeCutoff;

fn cutoff(day: u8) -> KnowledgeCutoff {
    KnowledgeCutoff::parse_rfc3339(&format!("2026-01-{day:02}T00:00:00Z")).expect("cutoff")
}

#[test]
fn declared_recovery_grid_requires_at_least_one_window() {
    let config = FittedCandidateKConfig::new(vec![2, 3], vec![7, 11], 20, 1e-5)
        .expect("candidate grid");
    assert_eq!(
        select_declared_rolling_origin_recovery_candidate_k(&config, &[]),
        Err(ModelSelectionError::EmptyCandidateSet)
    );
}

#[test]
fn declared_recovery_horizon_cannot_be_silently_dropped() {
    let config = FittedCandidateKConfig::new(vec![2, 3], vec![7, 11], 20, 1e-5)
        .expect("candidate grid");
    let cutoffs = [cutoff(10), cutoff(20), cutoff(30)];

    assert_eq!(
        select_declared_rolling_origin_recovery_candidate_k_for_cutoffs(
            &config,
            &cutoffs,
            &[],
        ),
        Err(ModelSelectionError::RecoveryWindowSetMismatch)
    );
}
