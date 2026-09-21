//! Preserve SE-aware acceptance when residual and bound multiplication cross binary64 range.
//!
//! Overflow of a represented subtraction or of `k * SE` is not itself a scientific verdict.
//! Finite residuals remain covered by a positive bound that overflowed, finite bounds cannot cover
//! an overflowing residual, and the both-overflow case must compare the exact represented dyadic
//! inputs rather than independently rounded infinities.

use validation_core::accept_within_standard_errors;

#[test]
fn finite_residual_is_accepted_when_positive_bound_overflows() {
    assert_eq!(
        accept_within_standard_errors(1.0, 0.0, f64::MAX, 2.0),
        Ok(true)
    );
}

#[test]
fn overflowing_residual_is_rejected_by_finite_bound() {
    assert_eq!(
        accept_within_standard_errors(f64::MAX, -f64::MAX, 1.0, 1.0),
        Ok(false)
    );
}

#[test]
fn both_overflow_paths_compare_exact_represented_inputs() {
    assert_eq!(
        accept_within_standard_errors(f64::MAX, -f64::MAX, f64::MAX, 2.0),
        Ok(true)
    );
    assert_eq!(
        accept_within_standard_errors(f64::MAX, -f64::MAX, f64::MAX, 1.5),
        Ok(false)
    );
}
