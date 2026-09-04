use validation_core::accept_within_standard_errors;

#[test]
fn equal_nonzero_correction_projection_preserves_exact_rejection() {
    // The direct subtraction and product both round to the same finite value.
    // Their first-order correction projections also round to the same positive
    // subnormal. The represented subtraction correction is exact, while the
    // exact product correction is smaller than that projected subnormal, so the
    // represented-input inequality is strictly false and must not be admitted as
    // a rounded equality.
    let estimate = f64::from_bits(0x0210_2814_5144_3c99);
    let correction = f64::from_bits(0x0000_0000_6398_c737);
    let target = -correction;
    let k = f64::from_bits(0x20d9_5434_7757_68c7);
    let standard_error = f64::from_bits(0x2124_696e_33e2_baaa);

    assert_eq!(estimate - target, k * standard_error);
    assert!(!accept_within_standard_errors(estimate, target, standard_error, k)
        .expect("finite represented inputs"));
}

#[test]
fn equal_nonzero_correction_projection_preserves_exact_acceptance() {
    // Companion boundary: the exact product correction lies just above the
    // same projected correction carried by the represented residual. A repair
    // must discriminate the exact ordering rather than reject every equal
    // nonzero correction projection.
    let estimate = f64::from_bits(0x01c3_f43e_c52b_4312);
    let correction = f64::from_bits(0x0000_0000_003b_9da9);
    let target = -correction;
    let k = f64::from_bits(0x20c8_7ace_8d72_9746);
    let standard_error = f64::from_bits(0x20ea_1585_cc49_24ca);

    assert_eq!(estimate - target, k * standard_error);
    assert!(accept_within_standard_errors(estimate, target, standard_error, k)
        .expect("finite represented inputs"));
}

#[test]
fn equal_negative_correction_projection_preserves_exact_rejection() {
    // The same projection collision also exists below the rounded value. Here
    // both first-order corrections are the same negative subnormal, but the
    // exact product lies slightly farther below the rounded bound than the exact
    // residual does. The strict inequality therefore remains a rejection.
    let estimate = f64::from_bits(0x018b_af20_e855_2bb6);
    let correction = f64::from_bits(0x0000_0000_0005_d636);
    let target = correction;
    let k = f64::from_bits(0x2c6f_de1e_a0d4_de3d);
    let standard_error = f64::from_bits(0x150b_cc8f_a576_9411);

    assert_eq!(estimate - target, k * standard_error);
    assert!(!accept_within_standard_errors(estimate, target, standard_error, k)
        .expect("finite represented inputs"));
}
