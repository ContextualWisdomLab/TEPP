//! Reject a nonzero Monte Carlo sample spread when binary64 cannot represent its standard deviation.
//!
//! A symmetric retained sample can have an exactly representable zero mean while its positive sample
//! standard deviation lies below the minimum binary64 subnormal. Durable Validation Evidence must
//! fail closed instead of projecting that nonzero uncertainty to an exact zero spread.

use validation_core::{ValidationError, summarize_replications};

#[test]
fn nonzero_sample_standard_deviation_cannot_project_to_zero() {
    let mut samples = [0.0; 16];
    samples[0] = -f64::from_bits(1);
    samples[15] = f64::from_bits(1);

    assert_eq!(
        summarize_replications(&samples, 0.05, 0.95),
        Err(ValidationError::InvalidInput)
    );
}
