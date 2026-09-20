//! Extreme binary64 contract for scientific recovery's conservative RMSE bound.

use validation_core::{
    ClaimEvidence, ClaimEvidenceKind, ScientificRecoveryFailurePolicyV1,
    ScientificRecoveryProfileV1, ValidationError, promote_scientific_recovery,
    rmse_standard_error, root_mean_square_error,
};

const HEAD: &str = "b2a3f879ca61daefa534f122647074666d5604bc";
const DGP: &str = "1111111111111111111111111111111111111111111111111111111111111111";
const SEEDS: &str = "2222222222222222222222222222222222222222222222222222222222222222";
const ESTIMAND: &str = "3333333333333333333333333333333333333333333333333333333333333333";
const STATE: &str = "4444444444444444444444444444444444444444444444444444444444444444";

fn as_slices<const N: usize, const M: usize>(rows: &[[f64; M]; N]) -> Vec<&[f64]> {
    rows.iter().map(|row| row.as_slice()).collect()
}

fn profile(max_rmse: f64) -> ScientificRecoveryProfileV1 {
    ScientificRecoveryProfileV1::new(
        2,
        max_rmse,
        3.0,
        DGP,
        SEEDS,
        ESTIMAND,
        STATE,
        ScientificRecoveryFailurePolicyV1::RequireAllPlannedRecovered,
    )
    .expect("valid profile")
}

fn exact_head_evidence() -> [ClaimEvidence; 1] {
    [ClaimEvidence::new(ClaimEvidenceKind::ExactHeadTests, true)]
}

#[test]
fn scaled_projection_must_not_erase_positive_uncertainty_above_the_target() {
    let previous_max = f64::from_bits(f64::MAX.to_bits() - 1);
    let truth_flat = [0.0, 0.0];
    let recovered_flat = [f64::MAX, previous_max];
    let rmse = root_mean_square_error(&truth_flat, &recovered_flat).expect("finite extreme rmse");
    let rmse_se =
        rmse_standard_error(&truth_flat, &recovered_flat).expect("finite extreme rmse se");

    assert!(rmse < f64::MAX);
    assert!(rmse_se > 0.0);

    let scale = f64::MAX;
    let projected_upper = (rmse / scale) + 3.0 * (rmse_se / scale);
    assert_eq!(
        projected_upper.to_bits(),
        1.0_f64.to_bits(),
        "the predecessor scale projection loses the positive uncertainty contribution"
    );

    let truth_rows = [[0.0], [0.0]];
    let recovered_rows = [[f64::MAX], [previous_max]];
    let truth = as_slices(&truth_rows);
    let recovered = as_slices(&recovered_rows);
    let profile = profile(f64::MAX);
    assert_eq!(
        promote_scientific_recovery(
            HEAD,
            HEAD,
            &truth,
            &recovered,
            &profile,
            &exact_head_evidence(),
        ),
        Err(ValidationError::ClaimRecoveryRejected),
        "scientific authority must not be minted when positive RMSE uncertainty exceeds the remaining practical margin"
    );
}

#[test]
fn practical_target_boundary_is_not_promoted_as_scientific_support() {
    let truth_flat = [0.0, 0.0];
    let recovered_flat = [0.05, -0.05];
    let rmse = root_mean_square_error(&truth_flat, &recovered_flat).expect("rmse");
    let rmse_se = rmse_standard_error(&truth_flat, &recovered_flat).expect("rmse se");

    assert_eq!(rmse.to_bits(), 0.05_f64.to_bits());
    assert_eq!(rmse_se.to_bits(), 0.0_f64.to_bits());

    let truth_rows = [[0.0], [0.0]];
    let recovered_rows = [[0.05], [-0.05]];
    let truth = as_slices(&truth_rows);
    let recovered = as_slices(&recovered_rows);
    let profile = profile(0.05);
    assert_eq!(
        promote_scientific_recovery(
            HEAD,
            HEAD,
            &truth,
            &recovered,
            &profile,
            &exact_head_evidence(),
        ),
        Err(ValidationError::ClaimRecoveryRejected),
        "promotion requires the conservative RMSE bound to remain strictly inside the caller-owned target"
    );
}
