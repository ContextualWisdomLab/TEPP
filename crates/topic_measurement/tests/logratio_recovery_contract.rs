//! True-parameter recovery of logistic-normal topic coordinates.
#![allow(clippy::cast_precision_loss)]

use topic_measurement::{
    TopicMeasurementError, additive_log_ratio, from_additive_log_ratio,
    refuse_lexical_inferential_weight,
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

#[test]
fn known_simplex_recovers_through_alr_with_computed_rmse() {
    // Closed-form simplex: (2, 3, 1) / 6. ALR is (ln 2, ln 3).
    let truth = [2.0 / 6.0, 3.0 / 6.0, 1.0 / 6.0];
    let coordinates = additive_log_ratio(&truth).expect("alr");
    let true_parameters = [2.0_f64.ln(), 3.0_f64.ln()];
    assert_eq!(coordinates.len(), 2);
    let parameter_rmse = rmse(&true_parameters, &coordinates);
    assert!(
        parameter_rmse < 1e-15,
        "true-parameter ALR RMSE {parameter_rmse} exceeded machine-scale bound"
    );

    let recovered = from_additive_log_ratio(&coordinates).expect("inverse");
    let simplex_rmse = rmse(&truth, &recovered);
    assert!(
        simplex_rmse < 1e-15,
        "ALR round-trip RMSE {simplex_rmse} exceeded machine-scale bound"
    );
    let sum: f64 = recovered.iter().sum();
    assert!((sum - 1.0).abs() < 1e-15);
}

#[test]
fn large_finite_coordinates_round_trip_without_exponential_overflow() {
    let truth = [710.0, 709.0];
    let simplex = from_additive_log_ratio(&truth)
        .expect("finite representable ALR coordinates must use a stable inverse");
    assert!(simplex.iter().all(|part| part.is_finite() && *part > 0.0));
    assert!((simplex.iter().sum::<f64>() - 1.0).abs() < 1e-15);

    let recovered = additive_log_ratio(&simplex)
        .expect("forward ALR must subtract logs instead of overflowing the ratio");
    assert!(
        rmse(&truth, &recovered) < 1e-10,
        "large-coordinate round trip must retain the true parameters"
    );
}

#[test]
fn equal_shares_map_to_zero_alr_and_refuse_raw_euclidean_use() {
    let thirds = [1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0];
    let coordinates = additive_log_ratio(&thirds).expect("equal");
    assert!(coordinates.iter().all(|value| value.abs() < 1e-15));
    let recovered = from_additive_log_ratio(&[0.0, 0.0]).expect("zeros");
    assert!(rmse(&thirds, &recovered) < 1e-15);
}

#[test]
fn invalid_compositions_and_lexical_weights_fail_closed() {
    // K=2 is valid; zero/negative/non-unit-sum/non-finite/K<2 are not.
    assert_eq!(
        additive_log_ratio(&[0.0, 1.0]),
        Err(TopicMeasurementError::InvalidComposition)
    );
    assert_eq!(
        additive_log_ratio(&[-0.1, 1.1]),
        Err(TopicMeasurementError::InvalidComposition)
    );
    assert_eq!(
        additive_log_ratio(&[0.2, 0.2, 0.2]),
        Err(TopicMeasurementError::InvalidComposition)
    );
    assert_eq!(
        additive_log_ratio(&[f64::NAN, 1.0]),
        Err(TopicMeasurementError::InvalidComposition)
    );
    assert_eq!(
        additive_log_ratio(&[]),
        Err(TopicMeasurementError::InvalidComposition)
    );
    assert_eq!(
        additive_log_ratio(&[1.0]),
        Err(TopicMeasurementError::InvalidComposition)
    );
    assert_eq!(
        from_additive_log_ratio(&[]),
        Err(TopicMeasurementError::InvalidLogRatioDimension)
    );
    assert_eq!(
        from_additive_log_ratio(&[f64::INFINITY]),
        Err(TopicMeasurementError::InvalidLogRatioDimension)
    );
    assert_eq!(
        from_additive_log_ratio(&[1.0e9]),
        Err(TopicMeasurementError::InvalidLogRatioDimension),
        "max-shifted reference weight must fail closed when it underflows to zero"
    );
    assert_eq!(
        from_additive_log_ratio(&[-1.0e9]),
        Err(TopicMeasurementError::InvalidLogRatioDimension),
        "inverse must not return a zero simplex part after underflow"
    );
    assert_eq!(
        from_additive_log_ratio(&[0.0, 0.0, 0.0, -744.0]),
        Err(TopicMeasurementError::InvalidLogRatioDimension),
        "normalization must not underflow a nonzero weight to a zero simplex part"
    );
    assert_eq!(
        additive_log_ratio(&[f64::MAX, f64::MAX]),
        Err(TopicMeasurementError::InvalidComposition),
        "overflowing finite parts must fail closed because compensated mass is non-finite"
    );

    for method in ["tfidf", "bm25", "keyword", "TF-IDF", ""] {
        assert_eq!(
            refuse_lexical_inferential_weight(method),
            Err(TopicMeasurementError::LexicalWeightForbidden)
        );
    }
}
