use validation_core::{ValidationError, bias_standard_error};

fn two_level_sample(gap: f64) -> Vec<f64> {
    let mut recovered = vec![0.0; 6];
    recovered.extend(std::iter::repeat_n(gap, 27));
    recovered
}

#[test]
fn bias_standard_error_rational_scale_rounds_subnormal_result_once() {
    let gap = f64::from_bits(0x004a_2c74_6ac3_028e);
    let recovered = two_level_sample(gap);

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
}

#[test]
fn bias_standard_error_rational_scale_uses_ties_to_even_at_subnormal_units() {
    // With the same exact 3/44 count scale, 22 minimum-subnormal gap units map
    // to 1.5 result units and therefore round to the even value 2.
    let odd_lower = two_level_sample(f64::from_bits(22));
    assert_eq!(
        bias_standard_error(&[0.0; 33], &odd_lower)
            .expect("odd lower midpoint")
            .to_bits(),
        2
    );

    // 66 gap units map to 4.5 result units and remain at the even lower value 4.
    let even_lower = two_level_sample(f64::from_bits(66));
    assert_eq!(
        bias_standard_error(&[0.0; 33], &even_lower)
            .expect("even lower midpoint")
            .to_bits(),
        4
    );
}

#[test]
fn bias_standard_error_rational_scale_preserves_range_boundary_and_refuses_false_zero() {
    let rounds_to_minimum_normal = two_level_sample(f64::from_bits(0x004d_5555_5555_5555));
    assert_eq!(
        bias_standard_error(&[0.0; 33], &rounds_to_minimum_normal)
            .expect("minimum-normal boundary")
            .to_bits(),
        f64::MIN_POSITIVE.to_bits()
    );

    let minimum_subnormal = f64::from_bits(1);
    assert_eq!(
        bias_standard_error(&[0.0; 33], &two_level_sample(minimum_subnormal)),
        Err(ValidationError::InvalidInput)
    );
}
