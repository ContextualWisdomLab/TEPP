//! Scientific recovery uncertainty is counted over independent simulation replications, not state rows.

use validation_core::{
    ValidationError, promote_scientific_recovery, rmse_standard_error, root_mean_square_error,
};

const HEAD: &str = "b2a3f879ca61daefa534f122647074666d5604bc";

fn as_slices<const N: usize, const M: usize>(rows: &[[f64; M]; N]) -> Vec<&[f64]> {
    rows.iter().map(|row| row.as_slice()).collect()
}

#[test]
fn repeated_correlated_states_do_not_masquerade_as_independent_monte_carlo_replications() {
    let flat_truth = [0.0; 128];
    let mut flat_recovered = [0.0; 128];
    flat_recovered[64..].fill(0.08);

    let flat_rmse = root_mean_square_error(&flat_truth, &flat_recovered).expect("flat rmse");
    let flat_rmse_se =
        rmse_standard_error(&flat_truth, &flat_recovered).expect("flat pseudo-replication se");
    assert!(
        flat_rmse + 3.0 * flat_rmse_se < 0.08,
        "the predecessor flat residual path must demonstrate the pseudo-replication false promotion"
    );

    let truth_replications = [[0.0; 64], [0.0; 64]];
    let recovered_replications = [[0.0; 64], [0.08; 64]];
    let truth = as_slices(&truth_replications);
    let recovered = as_slices(&recovered_replications);

    assert_eq!(
        promote_scientific_recovery(HEAD, HEAD, &truth, &recovered, 2, 0.08, 3.0),
        Err(ValidationError::ClaimRecoveryRejected),
        "64 perfectly correlated state coordinates inside each of two runs must still count as only two independent Monte Carlo replications"
    );
}

#[test]
fn replication_structure_must_be_complete_before_promotion() {
    let truth_rows = [[0.0], [0.0]];
    let recovered_rows = [[0.0], [0.0]];
    let truth = as_slices(&truth_rows);
    let recovered = as_slices(&recovered_rows);

    assert_eq!(
        promote_scientific_recovery(HEAD, HEAD, &truth, &recovered[..1], 2, 0.01, 3.0),
        Err(ValidationError::InvalidInput),
        "outer replication counts must match the declared design denominator"
    );

    let invalid_truth_rows = [[0.0, 0.0], [0.0, 0.0]];
    let invalid_recovered_rows = [[0.0, 0.0], [0.0, 0.0]];
    let invalid_truth = as_slices(&invalid_truth_rows);
    let mut invalid_recovered = as_slices(&invalid_recovered_rows);
    invalid_recovered[1] = &invalid_recovered_rows[1][..1];
    assert_eq!(
        promote_scientific_recovery(
            HEAD,
            HEAD,
            &invalid_truth,
            &invalid_recovered,
            2,
            0.01,
            3.0,
        ),
        Err(ValidationError::InvalidInput),
        "each replication must contain a valid truth/recovery pair"
    );
}

#[test]
fn planned_replication_denominator_prevents_survivor_only_promotion() {
    let truth_rows = [[0.0], [0.0]];
    let recovered_rows = [[0.0], [0.0]];
    let truth = as_slices(&truth_rows);
    let recovered = as_slices(&recovered_rows);

    assert_eq!(
        promote_scientific_recovery(HEAD, HEAD, &truth, &recovered, 3, 0.01, 3.0),
        Err(ValidationError::InvalidInput),
        "two successful survivors must not satisfy a profile that planned three independent replications"
    );

    let one_truth_rows = [[0.0]];
    let one_recovered_rows = [[0.0]];
    let one_truth = as_slices(&one_truth_rows);
    let one_recovered = as_slices(&one_recovered_rows);
    assert_eq!(
        promote_scientific_recovery(HEAD, HEAD, &one_truth, &one_recovered, 1, 0.01, 3.0),
        Err(ValidationError::InvalidConfiguration),
        "one exact-recovery repetition cannot define Monte Carlo scientific support"
    );
}
