//! Extreme binary64 contract for scientific recovery's conservative RMSE bound.

use validation_core::{
    ValidationError, promote_scientific_recovery, rmse_standard_error, root_mean_square_error,
};

const HEAD: &str = "b2a3f879ca61daefa534f122647074666d5604bc";

#[test]
fn scaled_projection_must_not_erase_positive_uncertainty_above_the_target() {
    let previous_max = f64::from_bits(f64::MAX.to_bits() - 1);
    let truth = [0.0, 0.0];
    let recovered = [f64::MAX, previous_max];
    let rmse = root_mean_square_error(&truth, &recovered).expect("finite extreme rmse");
    let rmse_se = rmse_standard_error(&truth, &recovered).expect("finite extreme rmse se");

    assert!(rmse < f64::MAX);
    assert!(rmse_se > 0.0);

    let scale = f64::MAX;
    let projected_upper = (rmse / scale) + 3.0 * (rmse_se / scale);
    assert_eq!(
        projected_upper.to_bits(),
        1.0_f64.to_bits(),
        "the predecessor scale projection loses the positive uncertainty contribution"
    );

    assert_eq!(
        promote_scientific_recovery(HEAD, HEAD, &truth, &recovered, f64::MAX, 3.0),
        Err(ValidationError::ClaimRecoveryRejected),
        "scientific authority must not be minted when positive RMSE uncertainty exceeds the remaining practical margin"
    );
}

#[test]
fn practical_target_boundary_is_not_promoted_as_scientific_support() {
    let truth = [0.0, 0.0];
    let recovered = [0.05, -0.05];
    let rmse = root_mean_square_error(&truth, &recovered).expect("rmse");
    let rmse_se = rmse_standard_error(&truth, &recovered).expect("rmse se");

    assert_eq!(rmse.to_bits(), 0.05_f64.to_bits());
    assert_eq!(rmse_se.to_bits(), 0.0_f64.to_bits());
    assert_eq!(
        promote_scientific_recovery(HEAD, HEAD, &truth, &recovered, 0.05, 3.0),
        Err(ValidationError::ClaimRecoveryRejected),
        "promotion requires the conservative RMSE bound to remain strictly inside the caller-owned target"
    );
}
