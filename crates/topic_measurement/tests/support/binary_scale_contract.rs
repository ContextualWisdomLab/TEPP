//! Direct numeric boundary controls for the shared binary64 scale helpers.

use super::{positive_binary_parts, scale_binary, scaled_expected_count};
use crate::TopicMeasurementError;

/// Zero, overflow, normal, subnormal, and below-half-ulp scaling keep typed outcomes.
#[test]
fn binary_scale_covers_representable_and_refused_boundaries() {
    assert_eq!(scale_binary(0.0, 0), Ok(0.0));
    assert_eq!(scale_binary(1.0, 0), Ok(1.0));
    assert_eq!(scale_binary(1.0, 1023), Ok(2.0_f64.powi(1023)));
    assert_eq!(
        scale_binary(1.0, 1024),
        Err(TopicMeasurementError::NonFiniteEstimate)
    );
    assert_eq!(scale_binary(1.0, -1022), Ok(f64::MIN_POSITIVE));
    assert_eq!(scale_binary(1.0, -1074), Ok(f64::from_bits(1)));
    assert_eq!(scale_binary(1.0, -1075), Ok(0.0));
    assert_eq!(scale_binary(1.5, -1075), Ok(f64::from_bits(1)));
    assert_eq!(scale_binary(1.0, -1076), Ok(0.0));
    assert_eq!(
        scale_binary(-1.0, 0),
        Err(TopicMeasurementError::NonFiniteEstimate)
    );
}

/// Invalid count or factor inputs must not bypass direct helper finite admission.
#[test]
fn binary_parts_and_scaled_counts_reject_invalid_inputs() {
    for value in [0.0, -1.0, f64::INFINITY, f64::NAN] {
        assert_eq!(
            positive_binary_parts(value),
            Err(TopicMeasurementError::NonFiniteEstimate)
        );
    }
    assert_eq!(positive_binary_parts(f64::from_bits(1)), Ok((1.0, -1074)));
    assert_eq!(positive_binary_parts(f64::MIN_POSITIVE), Ok((1.0, -1022)));
    assert_eq!(positive_binary_parts(1.5), Ok((1.5, 0)));
    assert_eq!(scaled_expected_count(0.0, 1.0, 1.0, 0, 1.0), Ok(0.0));
    assert_eq!(scaled_expected_count(2.0, 0.5, 0.5, -2, 1.0), Ok(2.0));
    for (count, theta, beta) in [(-1.0, 1.0, 1.0), (1.0, 0.0, 1.0), (1.0, 1.0, 0.0)] {
        assert_eq!(
            scaled_expected_count(count, theta, beta, 0, 1.0),
            Err(TopicMeasurementError::NonFiniteEstimate)
        );
    }
}
