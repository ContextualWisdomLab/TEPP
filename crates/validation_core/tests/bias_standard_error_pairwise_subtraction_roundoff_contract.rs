use validation_core::bias_standard_error;

#[test]
fn bias_standard_error_preserves_pairwise_subtraction_roundoff_spread() {
    let truth = [2.0_f64.powi(-54), 2.0_f64.powi(-55)];
    let recovered = [1.0, 1.0];

    let standard_error =
        bias_standard_error(&truth, &recovered).expect("represented standard error");
    assert_eq!(standard_error.to_bits(), 2.0_f64.powi(-56).to_bits());

    let mirrored_truth = truth.map(|value| -value);
    let mirrored_recovered = recovered.map(|value| -value);
    let mirrored_standard_error = bias_standard_error(&mirrored_truth, &mirrored_recovered)
        .expect("mirrored represented standard error");
    assert_eq!(
        mirrored_standard_error.to_bits(),
        2.0_f64.powi(-56).to_bits()
    );

    assert_eq!(
        bias_standard_error(&[2.0_f64.powi(-54); 2], &[1.0; 2]),
        Ok(0.0)
    );
}
