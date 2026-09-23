//! Numerical edge coverage for truth-basis ALR covariance propagation.

use validation_core::{
    ValidationError, align_topic_probability_rows, realign_additive_log_ratio_covariance,
};

fn four_topic_alignment() -> validation_core::TopicAlignment {
    let basis = vec![
        vec![0.7, 0.1, 0.1, 0.1],
        vec![0.1, 0.7, 0.1, 0.1],
        vec![0.1, 0.1, 0.7, 0.1],
        vec![0.1, 0.1, 0.1, 0.7],
    ];
    align_topic_probability_rows(&basis, &basis).expect("four-topic identity alignment")
}

#[test]
fn covariance_factorization_refuses_non_finite_residuals() {
    let alignment = four_topic_alignment();
    let covariance = vec![
        vec![1.0, -1.0e154, 1.0e154],
        vec![-1.0e154, f64::MAX, 1.0e308],
        vec![1.0e154, 1.0e308, f64::MAX],
    ];

    assert_eq!(
        realign_additive_log_ratio_covariance(&alignment, &covariance),
        Err(ValidationError::InvalidInput)
    );
}

#[test]
fn covariance_factorization_refuses_non_finite_off_diagonal_factor() {
    let alignment = four_topic_alignment();
    let minimum_positive = f64::from_bits(1);
    let covariance = vec![
        vec![minimum_positive, f64::MAX, 0.0],
        vec![f64::MAX, f64::MAX, 0.0],
        vec![0.0, 0.0, 1.0],
    ];

    assert_eq!(
        realign_additive_log_ratio_covariance(&alignment, &covariance),
        Err(ValidationError::InvalidInput)
    );
}
