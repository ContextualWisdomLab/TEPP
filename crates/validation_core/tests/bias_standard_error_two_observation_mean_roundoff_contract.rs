use validation_core::bias_standard_error;

#[test]
fn bias_standard_error_uses_exact_two_observation_identity_when_mean_rounds() {
    let upper = 1.0_f64;
    let lower = f64::from_bits(upper.to_bits() - 1);
    let truth = [0.0, 0.0];
    let recovered = [upper, lower];

    let standard_error =
        bias_standard_error(&truth, &recovered).expect("represented standard error");
    assert_eq!(standard_error.to_bits(), 2.0_f64.powi(-54).to_bits());

    let mirrored_recovered = recovered.map(|value| -value);
    let mirrored_standard_error = bias_standard_error(&truth, &mirrored_recovered)
        .expect("mirrored represented standard error");
    assert_eq!(
        mirrored_standard_error.to_bits(),
        2.0_f64.powi(-54).to_bits()
    );

    assert_eq!(bias_standard_error(&truth, &[upper, upper]), Ok(0.0));
}
