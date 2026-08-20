//! True-parameter recovery and fail-closed ESEM/DSEM input gates.
#![allow(clippy::cast_precision_loss)]

use psychometric_core::{
    CausalHeuristic, ConstructClass, IndicatorKind, PsychometricError, claim_causal_effect,
    compare_latent_means, interpret_as_reflective, ordinary_least_squares_slope,
    pearson_correlation, posterior_draw_point_estimate_mean, recover_loading_point_estimate_mean,
    recover_reflective_loading, require_valid_indicator,
};

fn rmse(truth: &[f64], recovered: &[f64]) -> f64 {
    let n = truth.len() as f64;
    let sum_sq: f64 = truth
        .iter()
        .zip(recovered)
        .map(|(left, right)| {
            let residual = left - right;
            residual * residual
        })
        .sum();
    (sum_sq / n).sqrt()
}

fn centered_scores(count: usize) -> Vec<f64> {
    let mean = (count as f64 - 1.0) / 2.0;
    (0..count).map(|index| index as f64 - mean).collect()
}

#[test]
fn known_loading_recovers_through_ols_with_computed_rmse() {
    let true_loading = 0.8_f64;
    let factor_scores = centered_scores(16);
    let indicators: Vec<f64> = factor_scores
        .iter()
        .map(|score| true_loading * score)
        .collect();

    let recovered =
        recover_reflective_loading(&factor_scores, &indicators, IndicatorKind::AdditiveLogRatio)
            .expect("noiseless reflective loading");
    let error = rmse(&[true_loading], &[recovered]);
    assert!(
        error < 1e-12,
        "noiseless OLS RMSE {error} exceeded machine-scale bound"
    );
}

#[test]
fn posterior_draw_point_estimate_mean_recovers_true_loading_under_symmetric_draw_noise() {
    let true_loading = 0.8_f64;
    let factor_scores = centered_scores(16);
    let mut indicator_draws = Vec::with_capacity(5);
    for draw in 0..5 {
        let draw_loading = true_loading + 0.01 * (f64::from(draw) - 2.0);
        indicator_draws.push(
            factor_scores
                .iter()
                .map(|score| draw_loading * score)
                .collect::<Vec<f64>>(),
        );
    }

    let pooled = recover_loading_point_estimate_mean(
        &factor_scores,
        &indicator_draws,
        IndicatorKind::LogisticNormal,
    )
    .expect("posterior-draw point-estimate loading");
    let pooled_error = rmse(&[true_loading], &[pooled]);
    assert!(
        pooled_error < 1e-12,
        "symmetric posterior-draw point-estimate RMSE {pooled_error} should cancel"
    );

    let single = recover_reflective_loading(
        &factor_scores,
        &indicator_draws[0],
        IndicatorKind::IsometricLogRatio,
    )
    .expect("single draw");
    let single_error = rmse(&[true_loading], &[single]);
    assert!(
        single_error > pooled_error,
        "single-draw RMSE {single_error} should exceed pooled RMSE {pooled_error}"
    );
}

#[test]
fn raw_proportions_and_invalid_numeric_inputs_fail_closed() {
    assert_eq!(
        require_valid_indicator(IndicatorKind::RawProportion),
        Err(PsychometricError::RawProportionForbidden)
    );
    assert_eq!(
        pearson_correlation(&[0.2, 0.3], &[0.8, 0.7], IndicatorKind::RawProportion),
        Err(PsychometricError::RawProportionForbidden)
    );
    assert_eq!(
        recover_reflective_loading(&[1.0, 2.0], &[0.5, 0.5], IndicatorKind::RawProportion),
        Err(PsychometricError::RawProportionForbidden)
    );
    assert_eq!(
        recover_loading_point_estimate_mean(
            &[1.0, 2.0],
            &[vec![0.5, 0.5]],
            IndicatorKind::RawProportion
        ),
        Err(PsychometricError::RawProportionForbidden)
    );

    assert_eq!(
        pearson_correlation(&[1.0], &[1.0], IndicatorKind::AdditiveLogRatio),
        Err(PsychometricError::InvalidNumericInput)
    );
    assert_eq!(
        pearson_correlation(&[1.0, 2.0], &[1.0], IndicatorKind::AdditiveLogRatio),
        Err(PsychometricError::InvalidNumericInput)
    );
    assert_eq!(
        pearson_correlation(
            &[1.0, f64::NAN],
            &[1.0, 2.0],
            IndicatorKind::AdditiveLogRatio
        ),
        Err(PsychometricError::InvalidNumericInput)
    );
    assert_eq!(
        ordinary_least_squares_slope(&[1.0, 1.0], &[2.0, 3.0]),
        Err(PsychometricError::SingularDesign)
    );
    assert_eq!(
        ordinary_least_squares_slope(&[0.0, f64::MAX], &[0.0, f64::MAX]),
        Err(PsychometricError::InvalidNumericInput)
    );
    assert_eq!(
        pearson_correlation(&[1.0, 1.0], &[2.0, 3.0], IndicatorKind::AdditiveLogRatio),
        Err(PsychometricError::SingularDesign)
    );
    assert_eq!(
        pearson_correlation(&[1.0, 2.0], &[3.0, 3.0], IndicatorKind::AdditiveLogRatio),
        Err(PsychometricError::SingularDesign)
    );
    assert_eq!(
        pearson_correlation(
            &[1.0, 2.0],
            &[1.0, f64::NAN],
            IndicatorKind::AdditiveLogRatio
        ),
        Err(PsychometricError::InvalidNumericInput)
    );
    assert_eq!(
        posterior_draw_point_estimate_mean(&[]),
        Err(PsychometricError::InvalidNumericInput)
    );
    assert_eq!(
        posterior_draw_point_estimate_mean(&[1.0, f64::INFINITY]),
        Err(PsychometricError::InvalidNumericInput)
    );
    assert_eq!(
        recover_loading_point_estimate_mean(&[1.0, 2.0], &[], IndicatorKind::AdditiveLogRatio),
        Err(PsychometricError::InvalidNumericInput)
    );
    assert_eq!(
        recover_loading_point_estimate_mean(
            &[1.0, 2.0],
            &[vec![1.0]],
            IndicatorKind::AdditiveLogRatio
        ),
        Err(PsychometricError::InvalidNumericInput)
    );
}

#[test]
fn construct_class_and_causal_heuristics_refuse_overclaim() {
    assert!(ConstructClass::Reflective.admits_reflective_esem());
    assert!(!ConstructClass::Formative.admits_reflective_esem());
    assert!(!ConstructClass::Network.admits_reflective_esem());
    assert!(!ConstructClass::Unresolved.admits_reflective_esem());
    assert_eq!(ConstructClass::Reflective.as_str(), "reflective");
    assert_eq!(ConstructClass::Formative.as_str(), "formative");
    assert_eq!(ConstructClass::Network.as_str(), "network");
    assert_eq!(ConstructClass::Unresolved.as_str(), "unresolved");

    assert_eq!(
        interpret_as_reflective(ConstructClass::Reflective, false).expect("reflective"),
        ConstructClass::Reflective
    );
    assert_eq!(
        interpret_as_reflective(ConstructClass::Reflective, true).expect("fit unused"),
        ConstructClass::Reflective
    );
    assert_eq!(
        interpret_as_reflective(ConstructClass::Formative, true),
        Err(PsychometricError::FormativeReinterpretationForbidden)
    );
    assert_eq!(
        interpret_as_reflective(ConstructClass::Formative, false),
        Err(PsychometricError::FormativeReinterpretationForbidden)
    );
    assert_eq!(
        interpret_as_reflective(ConstructClass::Network, true),
        Err(PsychometricError::FormativeReinterpretationForbidden)
    );
    assert_eq!(
        interpret_as_reflective(ConstructClass::Unresolved, true),
        Err(PsychometricError::UnresolvedConstruct)
    );
    assert_eq!(
        interpret_as_reflective(ConstructClass::Unresolved, false),
        Err(PsychometricError::UnresolvedConstruct)
    );

    compare_latent_means(true).expect("invariance met");
    assert_eq!(
        compare_latent_means(false),
        Err(PsychometricError::InvarianceRequired)
    );

    for heuristic in [
        CausalHeuristic::TemporalPrecedence,
        CausalHeuristic::DocumentLinkage,
        CausalHeuristic::EventTracking,
        CausalHeuristic::ModelPrediction,
    ] {
        assert_eq!(
            claim_causal_effect(heuristic),
            Err(PsychometricError::CausalUnderidentified)
        );
        assert!(!heuristic.as_str().is_empty());
    }

    assert!(IndicatorKind::AdditiveLogRatio.is_valid_structural_input());
    assert!(IndicatorKind::IsometricLogRatio.is_valid_structural_input());
    assert!(IndicatorKind::LogisticNormal.is_valid_structural_input());
    assert!(!IndicatorKind::RawProportion.is_valid_structural_input());
    assert_eq!(IndicatorKind::AdditiveLogRatio.as_str(), "alr");
    assert_eq!(IndicatorKind::IsometricLogRatio.as_str(), "ilr");
    assert_eq!(IndicatorKind::LogisticNormal.as_str(), "logistic_normal");
    assert_eq!(IndicatorKind::RawProportion.as_str(), "raw_proportion");
}

#[test]
fn finite_alr_correlation_and_error_messages_are_stable() {
    let left = [0.0_f64, 1.0, 2.0];
    let right = [0.0_f64, 2.0, 4.0];
    let correlation = pearson_correlation(&left, &right, IndicatorKind::AdditiveLogRatio)
        .expect("perfect positive");
    assert!((correlation - 1.0).abs() < 1e-12);

    let slope = ordinary_least_squares_slope(&left, &right).expect("slope");
    assert!((slope - 2.0).abs() < 1e-12);
    let mean = posterior_draw_point_estimate_mean(&[0.7, 0.8, 0.9]).expect("mean");
    assert!((mean - 0.8).abs() < 1e-15);

    assert_eq!(
        PsychometricError::RawProportionForbidden.to_string(),
        "raw topic proportions are forbidden psychometric indicators"
    );
    assert_eq!(
        PsychometricError::InvalidNumericInput.to_string(),
        "invalid psychometric numeric input"
    );
    assert_eq!(
        PsychometricError::SingularDesign.to_string(),
        "singular psychometric design matrix"
    );
    assert_eq!(
        PsychometricError::FormativeReinterpretationForbidden.to_string(),
        "formative or network constructs cannot be reinterpreted as reflective"
    );
    assert_eq!(
        PsychometricError::CausalUnderidentified.to_string(),
        "temporal precedence is not causal identification"
    );
    assert_eq!(
        PsychometricError::UnresolvedConstruct.to_string(),
        "construct class is unresolved"
    );
    assert_eq!(
        PsychometricError::InvarianceRequired.to_string(),
        "latent-mean comparison requires invariance evidence"
    );
}
