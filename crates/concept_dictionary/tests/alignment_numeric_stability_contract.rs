//! Concept-coordinate RMSE must stay finite when the true result is representable.

use concept_dictionary::{ConceptError, concept_coordinate_rmse};

#[test]
fn scaled_rmse_handles_large_finite_coordinates_without_square_overflow() {
    let magnitude = f64::MAX / 2.0;
    let error = concept_coordinate_rmse(&[magnitude, -magnitude], &[0.0, 0.0])
        .expect("representable RMSE must remain finite");
    assert!(error.is_finite());
    assert!((error / magnitude - 1.0).abs() < 1.0e-15);
}

#[test]
fn unrepresentable_residual_fails_closed() {
    assert_eq!(
        concept_coordinate_rmse(&[f64::MAX], &[-f64::MAX]),
        Err(ConceptError::InvalidNumericInput)
    );
}
