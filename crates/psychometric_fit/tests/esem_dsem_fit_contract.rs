//! True-parameter ESEM/DSEM fit recovery and fail-closed interpretation gates.

#![allow(clippy::cast_precision_loss)]

use psychometric_fit::{
    FitConstructClass, FitCoordinateKind, PsychometricFitError, admit_fit_coordinates,
    interpret_fit_as_reflective, loading_recovery_rmse, recover_dsem_lagged_path,
    recover_esem_loadings,
};

fn centered_scores(count: usize) -> Vec<f64> {
    let mean = (count as f64 - 1.0) / 2.0;
    (0..count).map(|index| index as f64 - mean).collect()
}

#[test]
fn two_factor_esem_recovers_known_cross_loadings_better_than_zero() {
    let factor_one = centered_scores(16);
    let factor_two: Vec<f64> = factor_one
        .iter()
        .map(|score| score * score - 21.25)
        .collect();
    let indicator_one: Vec<f64> = factor_one
        .iter()
        .zip(&factor_two)
        .map(|(one, two)| 0.8 * one + 0.2 * two)
        .collect();
    let indicator_two: Vec<f64> = factor_one
        .iter()
        .zip(&factor_two)
        .map(|(one, two)| 0.1 * one + 0.7 * two)
        .collect();

    let recovered = recover_esem_loadings(
        &[factor_one, factor_two],
        &[indicator_one, indicator_two],
        FitCoordinateKind::AdditiveLogRatio,
    )
    .expect("noiseless ESEM");
    let truth = [vec![0.8, 0.2], vec![0.1, 0.7]];
    let recovered_rmse = loading_recovery_rmse(&truth, &recovered).expect("rmse");
    let zeroed = [vec![0.0, 0.0], vec![0.0, 0.0]];
    let collapsed_rmse = loading_recovery_rmse(&truth, &zeroed).expect("zero");
    assert!(
        recovered_rmse < 1e-12,
        "noiseless ESEM RMSE {recovered_rmse} exceeded machine-scale bound"
    );
    assert!(recovered_rmse < collapsed_rmse);
}

#[test]
fn single_factor_ilr_and_logistic_normal_recover_the_known_loading() {
    let factor = centered_scores(8);
    let indicator: Vec<f64> = factor.iter().map(|score| 0.6 * score).collect();
    for kind in [
        FitCoordinateKind::IsometricLogRatio,
        FitCoordinateKind::LogisticNormal,
    ] {
        let recovered = recover_esem_loadings(
            std::slice::from_ref(&factor),
            std::slice::from_ref(&indicator),
            kind,
        )
        .expect("loading");
        assert_eq!(recovered.len(), 1);
        assert_eq!(recovered[0].len(), 1);
        assert!((recovered[0][0] - 0.6).abs() < 1e-12);
    }
}

#[test]
fn forward_dsem_lag_recovers_and_non_forward_event_time_fails_closed() {
    let lag = centered_scores(10);
    let outcome: Vec<f64> = lag.iter().map(|score| 0.45 * score).collect();
    let recovered = recover_dsem_lagged_path(10, 20, &lag, &outcome).expect("forward lag");
    assert!((recovered - 0.45).abs() < 1e-12);
    assert_eq!(
        recover_dsem_lagged_path(20, 10, &lag, &outcome),
        Err(PsychometricFitError::ReverseEventTimePath)
    );
    assert_eq!(
        recover_dsem_lagged_path(10, 10, &lag, &outcome),
        Err(PsychometricFitError::ReverseEventTimePath)
    );
}

#[test]
#[allow(clippy::too_many_lines)]
fn raw_proportions_and_invalid_payloads_fail_closed() {
    assert_eq!(
        admit_fit_coordinates(FitCoordinateKind::RawProportion),
        Err(PsychometricFitError::RawProportionForbidden)
    );
    assert_eq!(
        recover_esem_loadings(
            &[vec![0.2, 0.3]],
            &[vec![0.8, 0.7]],
            FitCoordinateKind::RawProportion
        ),
        Err(PsychometricFitError::RawProportionForbidden)
    );
    assert_eq!(
        recover_esem_loadings(
            &[] as &[Vec<f64>],
            &[vec![1.0, 2.0]],
            FitCoordinateKind::AdditiveLogRatio
        ),
        Err(PsychometricFitError::InvalidNumericInput)
    );
    assert_eq!(
        recover_esem_loadings(
            &[vec![1.0, 2.0]],
            &[] as &[Vec<f64>],
            FitCoordinateKind::AdditiveLogRatio
        ),
        Err(PsychometricFitError::InvalidNumericInput)
    );
    assert_eq!(
        recover_esem_loadings(
            &[vec![1.0]],
            &[vec![1.0]],
            FitCoordinateKind::AdditiveLogRatio
        ),
        Err(PsychometricFitError::InvalidNumericInput)
    );
    assert_eq!(
        recover_esem_loadings(
            &[vec![1.0, 2.0]],
            &[vec![1.0]],
            FitCoordinateKind::AdditiveLogRatio
        ),
        Err(PsychometricFitError::InvalidNumericInput)
    );
    assert_eq!(
        recover_esem_loadings(
            &[vec![1.0, f64::NAN]],
            &[vec![1.0, 2.0]],
            FitCoordinateKind::AdditiveLogRatio
        ),
        Err(PsychometricFitError::InvalidNumericInput)
    );
    assert_eq!(
        recover_esem_loadings(
            &[vec![1.0, 1.0]],
            &[vec![2.0, 3.0]],
            FitCoordinateKind::AdditiveLogRatio
        ),
        Err(PsychometricFitError::SingularDesign)
    );
    let three = [vec![0.0, 1.0], vec![1.0, 0.0], vec![0.5, 0.5]];
    assert_eq!(
        recover_esem_loadings(
            &three,
            &[vec![1.0, 2.0]],
            FitCoordinateKind::AdditiveLogRatio
        ),
        Err(PsychometricFitError::InvalidNumericInput)
    );
    assert_eq!(
        recover_esem_loadings(
            &[vec![1.0, 2.0], vec![3.0]],
            &[vec![1.0, 2.0]],
            FitCoordinateKind::AdditiveLogRatio
        ),
        Err(PsychometricFitError::InvalidNumericInput)
    );
    assert_eq!(
        recover_esem_loadings(
            &[vec![1.0, 2.0, 3.0], vec![2.0, 4.0, 6.0]],
            &[vec![1.0, 2.0, 3.0]],
            FitCoordinateKind::AdditiveLogRatio
        ),
        Err(PsychometricFitError::SingularDesign)
    );
    assert_eq!(
        recover_dsem_lagged_path(1, 2, &[1.0], &[2.0]),
        Err(PsychometricFitError::InvalidNumericInput)
    );
    assert_eq!(
        loading_recovery_rmse(&[], &[]),
        Err(PsychometricFitError::InvalidNumericInput)
    );
    assert_eq!(
        loading_recovery_rmse(&[vec![1.0]], &[vec![1.0, 2.0]]),
        Err(PsychometricFitError::InvalidNumericInput)
    );
    assert_eq!(
        loading_recovery_rmse(&[vec![], vec![1.0]], &[vec![], vec![1.0]]),
        Err(PsychometricFitError::InvalidNumericInput)
    );
    assert_eq!(
        loading_recovery_rmse(&[vec![1.0]], &[]),
        Err(PsychometricFitError::InvalidNumericInput)
    );
}

#[test]
fn good_global_fit_cannot_reclassify_formative_or_unresolved_constructs() {
    assert!(FitConstructClass::Reflective.admits_esem_fit());
    assert!(!FitConstructClass::Formative.admits_esem_fit());
    assert!(!FitConstructClass::Network.admits_esem_fit());
    assert!(!FitConstructClass::Unresolved.admits_esem_fit());
    assert_eq!(
        interpret_fit_as_reflective(FitConstructClass::Reflective, true).expect("reflective"),
        FitConstructClass::Reflective
    );
    assert_eq!(
        interpret_fit_as_reflective(FitConstructClass::Formative, true),
        Err(PsychometricFitError::FormativeReinterpretationForbidden)
    );
    assert_eq!(
        interpret_fit_as_reflective(FitConstructClass::Network, false),
        Err(PsychometricFitError::FormativeReinterpretationForbidden)
    );
    assert_eq!(
        interpret_fit_as_reflective(FitConstructClass::Unresolved, true),
        Err(PsychometricFitError::UnresolvedConstruct)
    );
    assert_eq!(FitCoordinateKind::AdditiveLogRatio.as_str(), "alr");
    assert_eq!(FitCoordinateKind::IsometricLogRatio.as_str(), "ilr");
    assert_eq!(
        FitCoordinateKind::LogisticNormal.as_str(),
        "logistic_normal"
    );
    assert_eq!(FitCoordinateKind::RawProportion.as_str(), "raw_proportion");
    assert_eq!(FitConstructClass::Reflective.as_str(), "reflective");
    assert_eq!(FitConstructClass::Formative.as_str(), "formative");
    assert_eq!(FitConstructClass::Network.as_str(), "network");
    assert_eq!(FitConstructClass::Unresolved.as_str(), "unresolved");
    assert_eq!(
        PsychometricFitError::RawProportionForbidden.to_string(),
        "raw topic proportions are forbidden psychometric fit inputs"
    );
    assert_eq!(
        PsychometricFitError::InvalidNumericInput.to_string(),
        "invalid psychometric fit numeric input"
    );
    assert_eq!(
        PsychometricFitError::SingularDesign.to_string(),
        "singular psychometric fit design matrix"
    );
    assert_eq!(
        PsychometricFitError::ReverseEventTimePath.to_string(),
        "DSEM lagged paths cannot move backward in event time"
    );
    assert_eq!(
        PsychometricFitError::FormativeReinterpretationForbidden.to_string(),
        "formative or network constructs cannot be reinterpreted as reflective"
    );
    assert_eq!(
        PsychometricFitError::UnresolvedConstruct.to_string(),
        "construct class is unresolved"
    );
}
