//! Dimension-only resource planning never constructs oversized dense arrays.

use topic_measurement::{
    TopicMeasurementError, check_reference_design_limits, check_reference_fit_limits,
};

#[test]
fn design_payload_boundary_and_overflow_are_checked_without_allocation() {
    assert_eq!(check_reference_design_limits(512, 4096), Ok(()));
    for (documents, features) in [(512, 4097), (2, usize::MAX), (usize::MAX, usize::MAX)] {
        assert_eq!(
            check_reference_design_limits(documents, features),
            Err(TopicMeasurementError::ResourceLimitExceeded)
        );
    }
    assert_eq!(check_reference_design_limits(6, 5), Ok(()));
}

#[test]
fn fit_payload_boundary_and_all_dense_axes_are_checked_without_allocation() {
    // 8 * (3*64*87375 + 6*2*64 + 2*3*63 + 2*3 + 64) = 128 MiB.
    assert_eq!(check_reference_fit_limits(2, 87_375, 64, 3), Ok(()));
    for (documents, vocabulary, topics, features) in [
        (2, 87_376, 64, 3),
        (3, 87_375, 64, 3),
        (2, 87_375, 65, 3),
        (2, 87_375, 64, 4),
        (2, 100_000, 20_000, 3),
        (10_000, 10_000, 10_000, 3),
        (2, 100_000, 40_000, 10_002),
        (2, 2, 2, 10_000_000),
        (usize::MAX, usize::MAX, usize::MAX, usize::MAX),
    ] {
        assert_eq!(
            check_reference_fit_limits(documents, vocabulary, topics, features),
            Err(TopicMeasurementError::ResourceLimitExceeded)
        );
    }
    assert_eq!(check_reference_fit_limits(6, 4, 2, 5), Ok(()));
}
