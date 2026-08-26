//! Deterministic recovery for independent Event Lineage criterion posteriors.

use event_core::{
    CriterionPosteriorError, IndependentCriterionCounts, fit_independent_criterion_posterior,
};

#[test]
fn jeffreys_posterior_recovers_known_relation_probabilities() {
    let truth_counts = [
        (0.1_f64, 1_000_u32),
        (0.25, 2_500),
        (0.5, 5_000),
        (0.75, 7_500),
        (0.9, 9_000),
    ];
    let trials = 10_000_u32;
    let estimates = truth_counts.map(|(_, successes)| {
        fit_independent_criterion_posterior(IndependentCriterionCounts { successes, trials }, 64)
            .expect("identified posterior")
            .mean
    });
    let rmse = estimates
        .iter()
        .zip(truth_counts)
        .map(|(estimate, (truth, _))| (estimate - truth).powi(2))
        .sum::<f64>()
        / 5.0;
    let rmse = rmse.sqrt();
    assert!(rmse < 0.000_05, "known-probability RMSE was {rmse}");
}

#[test]
fn plausible_values_preserve_symmetry_order_and_uncertainty() {
    let posterior = fit_independent_criterion_posterior(
        IndependentCriterionCounts {
            successes: 50,
            trials: 100,
        },
        32,
    )
    .expect("posterior");
    assert!((posterior.mean - 0.5).abs() < f64::EPSILON);
    assert!(posterior.variance > 0.0);
    assert!(
        posterior
            .plausible_values
            .windows(2)
            .all(|pair| pair[0] < pair[1])
    );
    for index in 0..posterior.plausible_values.len() {
        let reflected = posterior.plausible_values[posterior.plausible_values.len() - 1 - index];
        assert!((posterior.plausible_values[index] + reflected - 1.0).abs() < 1.0e-12);
    }
}

#[test]
fn invalid_observation_and_draw_contracts_fail_closed() {
    assert_eq!(
        fit_independent_criterion_posterior(
            IndependentCriterionCounts {
                successes: 0,
                trials: 0,
            },
            2,
        ),
        Err(CriterionPosteriorError::EmptyObservations)
    );
    assert_eq!(
        fit_independent_criterion_posterior(
            IndependentCriterionCounts {
                successes: 2,
                trials: 1,
            },
            2,
        ),
        Err(CriterionPosteriorError::SuccessesExceedTrials)
    );
    assert_eq!(
        fit_independent_criterion_posterior(
            IndependentCriterionCounts {
                successes: 1,
                trials: 2,
            },
            1,
        ),
        Err(CriterionPosteriorError::InsufficientDraws)
    );
}
