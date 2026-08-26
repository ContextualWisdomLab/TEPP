//! Deterministic recovery and fail-closed tests for the Rust criterion path.

use analysis_engine::{
    LineageCriterionFitError, LineageCriterionObservation, fit_lineage_criterion_posteriors,
};
use event_core::CriterionPosteriorError;

fn observation(pair_id: &str, successes: u32, trials: u32) -> LineageCriterionObservation {
    LineageCriterionObservation {
        pair_id: pair_id.into(),
        successes,
        trials,
        predecessor_event_time_draws: vec!["2026-01-01T00:00:00Z".into(); 32],
        successor_event_time_draws: vec!["2026-01-02T00:00:00Z".into(); 32],
    }
}

#[test]
fn rust_path_recovers_known_independent_pair_probabilities() {
    let truth_counts = [
        (0.1_f64, 1_000_u32),
        (0.25, 2_500),
        (0.5, 5_000),
        (0.75, 7_500),
        (0.9, 9_000),
    ];
    let observations = truth_counts
        .iter()
        .enumerate()
        .map(|(index, (_, successes))| observation(&format!("pair-{index}"), *successes, 10_000))
        .collect::<Vec<_>>();
    let fits = fit_lineage_criterion_posteriors(&observations, 32).expect("identified fits");
    let rmse = fits
        .iter()
        .zip(truth_counts)
        .map(|(fit, (truth, _))| (fit.posterior_mean - truth).powi(2))
        .sum::<f64>()
        / 5.0;
    assert!(rmse.sqrt() < 0.000_05);
    assert_eq!(
        fits[0].predecessor_event_time_draws,
        observations[0].predecessor_event_time_draws
    );
}

#[test]
fn rust_path_rejects_identity_draw_and_criterion_failures() {
    assert_eq!(
        fit_lineage_criterion_posteriors(&[], 32),
        Err(LineageCriterionFitError::EmptyInput)
    );

    let duplicate = vec![observation("pair", 1, 2), observation("pair", 1, 2)];
    assert_eq!(
        fit_lineage_criterion_posteriors(&duplicate, 32),
        Err(LineageCriterionFitError::InvalidPairIdentity)
    );

    let mut mixed = observation("pair", 1, 2);
    mixed.successor_event_time_draws.pop();
    assert_eq!(
        fit_lineage_criterion_posteriors(&[mixed], 32),
        Err(LineageCriterionFitError::TemporalDrawMismatch)
    );

    assert_eq!(
        fit_lineage_criterion_posteriors(&[observation("pair", 3, 2)], 32),
        Err(LineageCriterionFitError::Criterion(
            CriterionPosteriorError::SuccessesExceedTrials
        ))
    );
}
