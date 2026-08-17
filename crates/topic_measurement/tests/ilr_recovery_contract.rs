//! True-parameter recovery of isometric log-ratio topic coordinates.
#![allow(clippy::cast_precision_loss)]

use topic_measurement::{
    TopicMeasurementError, additive_log_ratio, from_isometric_log_ratio, isometric_log_ratio,
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
fn known_simplex_recovers_through_ilr_with_computed_rmse() {
    // Closed-form simplex: (2, 3, 1) / 6.
    // Sequential Egozcue ILR: y1 = √(2/3) ln(2√3 / 3), y2 = √(1/2) ln 3.
    let truth = [2.0 / 6.0, 3.0 / 6.0, 1.0 / 6.0];
    let true_parameters = [
        (2.0_f64 / 3.0).sqrt() * (2.0 * 3.0_f64.sqrt() / 3.0).ln(),
        (1.0_f64 / 2.0).sqrt() * 3.0_f64.ln(),
    ];
    let coordinates = isometric_log_ratio(&truth).expect("ilr");
    assert_eq!(coordinates.len(), 2);
    let parameter_rmse = rmse(&true_parameters, &coordinates);
    assert!(
        parameter_rmse < 1e-15,
        "true-parameter ILR RMSE {parameter_rmse} exceeded machine-scale bound"
    );

    let recovered = from_isometric_log_ratio(&coordinates).expect("inverse");
    let simplex_rmse = rmse(&truth, &recovered);
    assert!(
        simplex_rmse < 1e-15,
        "ILR round-trip RMSE {simplex_rmse} exceeded machine-scale bound"
    );
    let sum: f64 = recovered.iter().sum();
    assert!((sum - 1.0).abs() < 1e-15);

    let alr = additive_log_ratio(&truth).expect("alr");
    assert!(
        (alr[0] - coordinates[0]).abs() > 1e-6,
        "ILR must not collapse to the reference-dependent ALR map"
    );
}

#[test]
fn equal_shares_are_the_ilr_origin_and_preserve_aitchison_distance() {
    let halves = [0.5, 0.5];
    let origin = isometric_log_ratio(&halves).expect("origin");
    assert_eq!(origin.len(), 1);
    assert!(origin[0].abs() < 1e-15);

    let unbalanced = [0.8, 0.2];
    let coordinates = isometric_log_ratio(&unbalanced).expect("pair");
    let expected = (0.5_f64).sqrt() * 4.0_f64.ln();
    assert!((coordinates[0] - expected).abs() < 1e-15);

    let recovered = from_isometric_log_ratio(&coordinates).expect("inverse");
    assert!(rmse(&unbalanced, &recovered) < 1e-15);
}

#[test]
fn large_finite_ilr_coordinates_round_trip_or_fail_closed() {
    let representable = from_isometric_log_ratio(&[40.0]).expect("representable");
    assert!(
        representable
            .iter()
            .all(|part| part.is_finite() && *part > 0.0)
    );
    let recovered = isometric_log_ratio(&representable).expect("forward");
    assert!(rmse(&[40.0], &recovered) < 1e-12);

    assert_eq!(
        from_isometric_log_ratio(&[1000.0]),
        Err(TopicMeasurementError::InvalidLogRatioDimension),
        "ILR inverse must not return a zero simplex part after underflow"
    );
    assert_eq!(
        from_isometric_log_ratio(&[-f64::MAX, f64::MAX]),
        Err(TopicMeasurementError::InvalidLogRatioDimension),
        "overflowing CLR reconstruction must fail closed"
    );
}

#[test]
fn invalid_ilr_inputs_fail_closed() {
    assert_eq!(
        isometric_log_ratio(&[]),
        Err(TopicMeasurementError::InvalidComposition)
    );
    assert_eq!(
        from_isometric_log_ratio(&[]),
        Err(TopicMeasurementError::InvalidLogRatioDimension)
    );
    assert_eq!(
        from_isometric_log_ratio(&[f64::NAN]),
        Err(TopicMeasurementError::InvalidLogRatioDimension)
    );
    assert_eq!(
        from_isometric_log_ratio(&[f64::INFINITY]),
        Err(TopicMeasurementError::InvalidLogRatioDimension)
    );
}
