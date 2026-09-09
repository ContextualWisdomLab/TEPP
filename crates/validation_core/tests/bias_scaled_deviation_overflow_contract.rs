//! Public contract for overflow-safe residual-deviation scaling.

use validation_core::bias_standard_error;

#[test]
fn finite_bias_standard_error_survives_raw_deviation_overflow() {
    let maximum = f64::MAX;
    let truth = [0.0; 3];
    let recovered = [-maximum, maximum, maximum];

    // The represented residual mean is +maximum/3. Subtracting that finite mean
    // from -maximum overflows even though the requested standard error is finite.
    // Exact translated-residual admission also refuses because the two-level gap
    // 2*maximum is not representable. The generic path must therefore normalize
    // the sample before forming deviations rather than treating an intermediate
    // overflow as an unrepresentable scientific result.
    //
    // For residuals [-M, M, M], SE(mean) = 2M/3. With M=f64::MAX, the correctly
    // rounded binary64 result is 0x7fe5555555555555.
    let standard_error = bias_standard_error(&truth, &recovered)
        .expect("a finite standard error must survive raw deviation overflow");
    assert_eq!(standard_error.to_bits(), 0x7fe5_5555_5555_5555);

    let permuted_recovered = [maximum, -maximum, maximum];
    let permuted_standard_error = bias_standard_error(&truth, &permuted_recovered)
        .expect("permutation preserves the represented residual geometry");
    assert_eq!(permuted_standard_error.to_bits(), standard_error.to_bits());
}
