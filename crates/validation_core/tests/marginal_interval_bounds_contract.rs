use validation_core::{ValidationError, normal_marginal_interval_bounds};

#[test]
fn marginal_normal_intervals_use_full_validated_covariance_geometry() {
    let bounds = normal_marginal_interval_bounds(
        &[1.0, -2.0],
        &[vec![4.0, 1.0], vec![1.0, 9.0]],
        1.96,
    )
    .expect("valid marginal normal intervals");

    assert_eq!(bounds.lower().len(), 2);
    assert_eq!(bounds.upper().len(), 2);
    assert!((bounds.lower()[0] - (-2.92)).abs() < 1.0e-12);
    assert!((bounds.upper()[0] - 4.92).abs() < 1.0e-12);
    assert!((bounds.lower()[1] - (-7.88)).abs() < 1.0e-12);
    assert!((bounds.upper()[1] - 3.88).abs() < 1.0e-12);

    let singular = normal_marginal_interval_bounds(
        &[3.0, 4.0],
        &[vec![0.0, 0.0], vec![0.0, 1.0]],
        1.0,
    )
    .expect("positive-semidefinite zero-variance coordinate");
    assert_eq!(singular.lower()[0].to_bits(), 3.0_f64.to_bits());
    assert_eq!(singular.upper()[0].to_bits(), 3.0_f64.to_bits());
}

#[test]
fn marginal_normal_intervals_fail_closed_on_invalid_geometry_or_arithmetic() {
    assert_eq!(
        normal_marginal_interval_bounds(&[], &[], 1.96),
        Err(ValidationError::InvalidInput)
    );
    assert_eq!(
        normal_marginal_interval_bounds(&[0.0, 1.0], &[vec![1.0]], 1.96),
        Err(ValidationError::InvalidInput)
    );
    assert_eq!(
        normal_marginal_interval_bounds(
            &[0.0, 1.0],
            &[vec![1.0, 0.2], vec![0.1, 1.0]],
            1.96,
        ),
        Err(ValidationError::InvalidInput)
    );
    assert_eq!(
        normal_marginal_interval_bounds(
            &[0.0, 1.0],
            &[vec![1.0, 2.0], vec![2.0, 1.0]],
            1.96,
        ),
        Err(ValidationError::InvalidInput)
    );
    assert_eq!(
        normal_marginal_interval_bounds(&[f64::NAN], &[vec![1.0]], 1.96),
        Err(ValidationError::InvalidInput)
    );
    assert_eq!(
        normal_marginal_interval_bounds(&[0.0], &[vec![f64::INFINITY]], 1.96),
        Err(ValidationError::InvalidInput)
    );
    assert_eq!(
        normal_marginal_interval_bounds(&[0.0], &[vec![-1.0]], 1.96),
        Err(ValidationError::InvalidInput)
    );
    assert_eq!(
        normal_marginal_interval_bounds(&[0.0], &[vec![1.0]], 0.0),
        Err(ValidationError::InvalidConfiguration)
    );
    assert_eq!(
        normal_marginal_interval_bounds(&[0.0], &[vec![1.0]], f64::NAN),
        Err(ValidationError::InvalidConfiguration)
    );
    // `z` itself is finite, but multiplying it by a positive standard deviation
    // overflows; the owner must reject that arithmetic rather than emit infinities.
    assert_eq!(
        normal_marginal_interval_bounds(&[0.0], &[vec![4.0]], f64::MAX),
        Err(ValidationError::InvalidInput)
    );
}
