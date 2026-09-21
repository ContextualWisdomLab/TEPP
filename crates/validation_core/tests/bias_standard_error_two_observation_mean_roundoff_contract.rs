//! Preserve the exact two-observation standard-error identity when the residual mean rounds.
//!
//! Adjacent binary64 residuals around `1.0` have an exact separation of `2^-53`; for two
//! observations the sample standard error is half that separation, `2^-54`, even though their
//! arithmetic mean is not itself exactly representable. Mirroring both residuals must preserve the
//! same result, while equal residuals must remain exactly zero. This regression constrains public
//! behavior below the `n=4..=16` production exact-proof admission range and does not enlarge it.

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
