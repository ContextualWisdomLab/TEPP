//! Regression contract for permutation and sign invariance of the three-sample metric.
//!
//! The historical filename references an earlier anchor implementation. The
//! scientific invariant is that reordering or mirroring the represented sample
//! cannot change the correctly rounded standard error.

use validation_core::bias_standard_error;

#[test]
fn three_observation_standard_error_is_permutation_and_sign_invariant() {
    let low = f64::from_bits(0x4194_f788_9184_b980);
    let middle = f64::from_bits(0x420c_409f_fce3_8390);
    let high = f64::from_bits(0x4222_70c4_634c_c6b6);
    let expected_bits = 0x4205_7185_8078_f946;

    for recovered in [
        [low, middle, high],
        [middle, low, high],
        [high, middle, low],
    ] {
        let standard_error = bias_standard_error(&[0.0; 3], &recovered)
            .expect("represented-input bias standard error");
        assert_eq!(standard_error.to_bits(), expected_bits);

        let mirrored = recovered.map(|value| -value);
        let mirrored_standard_error = bias_standard_error(&[0.0; 3], &mirrored)
            .expect("mirrored represented-input bias standard error");
        assert_eq!(mirrored_standard_error.to_bits(), expected_bits);
    }
}
