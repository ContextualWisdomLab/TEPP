//! Contract for comparing prevalence coefficients across affine EventTime bases.

use validation_core::{
    LinearTimeBasis, ValidationError, reexpress_linear_prevalence_time_basis,
};

fn trajectory_value(intercept: f64, slope: f64, basis: LinearTimeBasis, time_seconds: f64) -> f64 {
    intercept + slope * ((time_seconds - basis.center_seconds()) / basis.scale_seconds())
}

#[test]
fn equivalent_physical_trajectory_survives_training_basis_reexpression() {
    let truth_basis = LinearTimeBasis::new(50.0, 50.0).expect("truth basis");
    let training_basis = LinearTimeBasis::new(25.0, 25.0).expect("training basis");
    let truth_intercepts = [0.2, -0.1];
    let truth_slopes = [0.8, -0.4];

    let aligned = reexpress_linear_prevalence_time_basis(
        &truth_intercepts,
        &truth_slopes,
        truth_basis,
        training_basis,
    )
    .expect("basis transform");

    assert!((aligned.intercepts()[0] - (-0.2)).abs() < 1.0e-12);
    assert!((aligned.intercepts()[1] - 0.1).abs() < 1.0e-12);
    assert!((aligned.event_time_slopes()[0] - 0.4).abs() < 1.0e-12);
    assert!((aligned.event_time_slopes()[1] - (-0.2)).abs() < 1.0e-12);

    for time_seconds in [0.0, 25.0, 50.0, 100.0] {
        for coordinate in 0..truth_intercepts.len() {
            let truth = trajectory_value(
                truth_intercepts[coordinate],
                truth_slopes[coordinate],
                truth_basis,
                time_seconds,
            );
            let transformed = trajectory_value(
                aligned.intercepts()[coordinate],
                aligned.event_time_slopes()[coordinate],
                training_basis,
                time_seconds,
            );
            assert!((truth - transformed).abs() < 1.0e-12);
        }
    }
}

#[test]
fn identity_basis_is_exact_and_invalid_geometry_fails_closed() {
    let basis = LinearTimeBasis::new(10.0, 4.0).expect("basis");
    let intercepts = [0.25, -0.75];
    let slopes = [1.5, -0.5];
    let identity = reexpress_linear_prevalence_time_basis(&intercepts, &slopes, basis, basis)
        .expect("identity transform");
    assert_eq!(identity.intercepts(), intercepts);
    assert_eq!(identity.event_time_slopes(), slopes);

    for invalid_basis in [
        LinearTimeBasis::new(f64::NAN, 1.0),
        LinearTimeBasis::new(0.0, 0.0),
        LinearTimeBasis::new(0.0, -1.0),
        LinearTimeBasis::new(0.0, f64::INFINITY),
    ] {
        assert_eq!(invalid_basis, Err(ValidationError::InvalidInput));
    }

    assert_eq!(
        reexpress_linear_prevalence_time_basis(&[], &[], basis, basis),
        Err(ValidationError::InvalidInput)
    );
    assert_eq!(
        reexpress_linear_prevalence_time_basis(&[0.0], &[0.0, 1.0], basis, basis),
        Err(ValidationError::InvalidInput)
    );
    assert_eq!(
        reexpress_linear_prevalence_time_basis(&[f64::NAN], &[0.0], basis, basis),
        Err(ValidationError::InvalidInput)
    );
    assert_eq!(
        reexpress_linear_prevalence_time_basis(&[0.0], &[f64::INFINITY], basis, basis),
        Err(ValidationError::InvalidInput)
    );
}
