//! Preserve one final binary64 rounding after standard-error normalization.
//!
//! A represented three-residual geometry at `2^-55` exercises the boundary where rounding an
//! intermediate square/root path can move the final standard error by one bit. The public result
//! must be sign invariant and round once to `0x3c72_79a7_4590_331c`; the two-observation unit case
//! remains exactly `1.0`. This regression constrains public numerical behavior only and does not
//! widen the production exact-proof admission budget beyond `n=4..=16`.

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
