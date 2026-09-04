use validation_core::bias_standard_error;

#[test]
fn bias_standard_error_rounds_the_final_normalized_se_once() {
    let unit = 2.0_f64.powi(-55);
    let truth = [0.0; 3];
    let recovered = [-unit, 0.0, unit];

    let standard_error = bias_standard_error(&truth, &recovered).expect("represented standard error");
    assert_eq!(standard_error.to_bits(), 0x3c72_79a7_4590_331c);

    let mirrored_recovered = recovered.map(|value| -value);
    let mirrored_standard_error =
        bias_standard_error(&truth, &mirrored_recovered).expect("mirrored standard error");
    assert_eq!(mirrored_standard_error.to_bits(), 0x3c72_79a7_4590_331c);

    assert_eq!(bias_standard_error(&[0.0, 0.0], &[1.0, -1.0]), Ok(1.0));
}
