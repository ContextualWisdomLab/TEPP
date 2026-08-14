//! Shared meaning requires an explicit invariance status and computed loading RMSE.

use measurement_invariance::{
    GroupLoading, InvarianceError, InvarianceLevel, loading_root_mean_square_error,
    refuse_noninvariant_as_shared_meaning,
};

#[test]
fn configural_status_cannot_claim_shared_metric_meaning() {
    assert_eq!(
        refuse_noninvariant_as_shared_meaning(InvarianceLevel::Configural),
        Err(InvarianceError::InvarianceTooWeakForSharedMeaning)
    );
    assert_eq!(
        refuse_noninvariant_as_shared_meaning(InvarianceLevel::Metric),
        Ok(())
    );
    assert_eq!(
        refuse_noninvariant_as_shared_meaning(InvarianceLevel::Scalar),
        Ok(())
    );
}

#[test]
fn aligned_loadings_have_lower_computed_rmse_than_a_crossed_collapse() {
    let truth = [
        GroupLoading::new(0, 0.80),
        GroupLoading::new(1, 0.80),
        GroupLoading::new(0, 0.40),
        GroupLoading::new(1, 0.40),
    ];
    let aligned = truth;
    let crossed = [
        GroupLoading::new(0, 0.80),
        GroupLoading::new(1, 0.40),
        GroupLoading::new(0, 0.40),
        GroupLoading::new(1, 0.80),
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
fn empty_or_non_finite_loadings_fail_closed() {
    assert_eq!(
        loading_root_mean_square_error(&[], &[]),
        Err(InvarianceError::InvalidLoadingPayload)
    );
}
