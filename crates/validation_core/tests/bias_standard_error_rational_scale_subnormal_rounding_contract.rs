use validation_core::{ValidationError, bias_standard_error};

#[test]
fn bias_standard_error_rational_scale_rounds_subnormal_result_once() {
    let gap = f64::from_bits(0x004a_2c74_6ac3_028e);
    let mut recovered = vec![0.0; 6];
    recovered.extend(std::iter::repeat(gap).take(27));

    let standard_error = bias_standard_error(&[0.0; 33], &recovered)
        .expect("represented-input subnormal standard error");
    // For counts 6 and 27 of n=33,
    // m(n-m)/(n^2(n-1)) = 162/34848 = 9/1936 = (3/44)^2.
    // The represented-input target is therefore exactly 3*|gap|/44. The
    // predecessor rational-square path normalizes the numerator before division
    // and then restores a power-of-two scale into the subnormal range, producing
    // an avoidable second rounding one ULP below the correctly rounded result.
    assert_eq!(standard_error.to_bits(), 0x000e_46cb_22f6_0165);

    let mut permuted = recovered.clone();
    permuted.rotate_left(11);
    let permuted_standard_error = bias_standard_error(&[0.0; 33], &permuted)
        .expect("permuted represented-input subnormal standard error");
    assert_eq!(permuted_standard_error.to_bits(), 0x000e_46cb_22f6_0165);

    let mirrored: Vec<_> = recovered.iter().map(|value| -*value).collect();
    let mirrored_standard_error = bias_standard_error(&[0.0; 33], &mirrored)
        .expect("mirrored represented-input subnormal standard error");
    assert_eq!(mirrored_standard_error.to_bits(), 0x000e_46cb_22f6_0165);

    let minimum_subnormal = f64::from_bits(1);
    let mut underflowing = vec![0.0; 6];
    underflowing.extend(std::iter::repeat(minimum_subnormal).take(27));
    assert_eq!(
        bias_standard_error(&[0.0; 33], &underflowing),
        Err(ValidationError::InvalidInput)
    );
}
