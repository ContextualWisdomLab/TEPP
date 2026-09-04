use validation_core::bias_standard_error;

#[test]
fn bias_standard_error_preserves_multi_observation_subtraction_roundoff_spread() {
    let truth = [2.0_f64.powi(-54), 2.0_f64.powi(-55), 0.0];
    let recovered = [1.0; 3];

    let standard_error =
        bias_standard_error(&truth, &recovered).expect("represented standard error");
    assert_eq!(standard_error.to_bits(), 0x3c72_79a7_4590_331d);

    let mirrored_truth = truth.map(|value| -value);
    let mirrored_recovered = recovered.map(|value| -value);
    let mirrored_standard_error = bias_standard_error(&mirrored_truth, &mirrored_recovered)
        .expect("mirrored represented standard error");
    assert_eq!(mirrored_standard_error.to_bits(), 0x3c72_79a7_4590_331d);

    assert_eq!(
        bias_standard_error(&[2.0_f64.powi(-54); 3], &[1.0; 3]),
        Ok(0.0)
    );
}
