//! Preserve correctly rounded mean bias after compensated residual accumulation.
//!
//! Three represented residuals combine one `2^-53` term, one `-2^-52` term, and
//! `-(1 + 2^-52)`. Their compensated numerator must be divided by the exact sample count without
//! first rounding that numerator onto a coarser binary64 value. The public mean bias is fixed at
//! `0xbfd5_5555_5555_5557`, and mirroring every residual must preserve the magnitude while changing
//! only the sign (`0x3fd5_5555_5555_5557`). This regression constrains `mean_bias`; it does not
//! change the `bias_standard_error` exact-proof admission budget.

use validation_core::mean_bias;

#[test]
fn mean_bias_divides_compensated_numerator_without_double_rounding() {
    let two_to_minus_53 = 2.0_f64.powi(-53);
    let two_to_minus_52 = 2.0_f64.powi(-52);
    let truth = [0.0, 0.0, 0.0];
    let recovered = [
        two_to_minus_53,
        -two_to_minus_52,
        -(1.0 + two_to_minus_52),
    ];

    let bias = mean_bias(&truth, &recovered).expect("represented mean bias");
    assert_eq!(bias.to_bits(), 0xbfd5_5555_5555_5557);

    let mirrored: Vec<_> = recovered.iter().map(|value| -*value).collect();
    let mirrored_bias = mean_bias(&truth, &mirrored).expect("mirrored represented mean bias");
    assert_eq!(mirrored_bias.to_bits(), 0x3fd5_5555_5555_5557);
}
