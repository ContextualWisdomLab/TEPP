//! Reject finite Monte Carlo samples whose represented sample spread exceeds binary64 range.
//!
//! The three-point witness forces the direct deviation path to overflow for the negative sample,
//! so summary construction must normalize the finite samples before computing spread. The
//! normalized sample standard deviation is finite and nonzero, but projecting it back through the
//! outer `f64::MAX` scale is not representable. TEPP must fail closed rather than publish an
//! infinite or clipped uncertainty estimate.

use validation_core::{ValidationError, summarize_replications};

#[test]
fn scaled_spread_projection_overflow_is_rejected() {
    assert_eq!(
        summarize_replications(&[f64::MAX, f64::MAX, -f64::MAX], 0.0, 1.0),
        Err(ValidationError::InvalidInput)
    );
}
