//! Exact-recovery contract for a positive acceptance bound that projects to zero.

use validation_core::accept_within_standard_errors;

#[test]
fn exact_recovery_survives_positive_bound_projection_to_zero() {
    let standard_error = f64::from_bits(1);
    let multiplier = 0.5;

    assert!(standard_error > 0.0);
    assert!(multiplier > 0.0);
    assert_eq!((multiplier * standard_error).to_bits(), 0);
    assert!(
        accept_within_standard_errors(1.0, 1.0, standard_error, multiplier)
            .expect("finite positive acceptance inputs")
    );
}
