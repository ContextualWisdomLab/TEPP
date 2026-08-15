//! Shared metric meaning requires an explicit invariance status and identified loading RMSE.

use measurement_invariance::{
    GroupLoading, InvarianceError, InvarianceLevel, loading_root_mean_square_error,
    require_shared_metric_meaning,
};

#[test]
fn configural_status_cannot_claim_shared_metric_meaning() {
    assert_eq!(
        require_shared_metric_meaning(InvarianceLevel::Configural),
        Err(InvarianceError::InvarianceTooWeakForSharedMetricMeaning)
    );
    assert_eq!(
        require_shared_metric_meaning(InvarianceLevel::Metric),
        Ok(())
    );
    assert_eq!(
        require_shared_metric_meaning(InvarianceLevel::Scalar),
        Ok(())
    );
}

#[test]
fn aligned_loadings_have_lower_computed_rmse_than_a_crossed_collapse() {
    let truth = [
        GroupLoading::new(0, 0, 0, 0.80),
        GroupLoading::new(1, 0, 0, 0.80),
        GroupLoading::new(0, 1, 0, 0.40),
        GroupLoading::new(1, 1, 0, 0.40),
    ];
    let aligned = truth;
    let crossed = [
        GroupLoading::new(0, 0, 0, 0.80),
        GroupLoading::new(1, 0, 0, 0.40),
        GroupLoading::new(0, 1, 0, 0.40),
        GroupLoading::new(1, 1, 0, 0.80),
    ];

    let aligned_rmse = loading_root_mean_square_error(&truth, &aligned).expect("aligned");
    let crossed_rmse = loading_root_mean_square_error(&truth, &crossed).expect("crossed");
    let expected = {
        let mut sum_squares = 0.0_f64;
        for (truth_row, decided_row) in truth.iter().zip(aligned.iter()) {
            let residual = decided_row.loading() - truth_row.loading();
            sum_squares += residual * residual;
        }
        (sum_squares / f64::from(u32::try_from(truth.len()).expect("len"))).sqrt()
    };
    assert!((aligned_rmse - expected).abs() < f64::EPSILON);
    assert!(aligned_rmse < crossed_rmse);
}

#[test]
fn mismatched_loading_coordinates_fail_closed() {
    let truth = [GroupLoading::new(0, 0, 0, 0.80)];
    let wrong_indicator = [GroupLoading::new(0, 1, 0, 0.80)];
    let wrong_factor = [GroupLoading::new(0, 0, 1, 0.80)];

    assert_eq!(
        loading_root_mean_square_error(&truth, &wrong_indicator),
        Err(InvarianceError::InvalidLoadingPayload)
    );
    assert_eq!(
        loading_root_mean_square_error(&truth, &wrong_factor),
        Err(InvarianceError::InvalidLoadingPayload)
    );
}

#[test]
fn empty_or_non_finite_loadings_fail_closed() {
    assert_eq!(
        loading_root_mean_square_error(&[], &[]),
        Err(InvarianceError::InvalidLoadingPayload)
    );
}
