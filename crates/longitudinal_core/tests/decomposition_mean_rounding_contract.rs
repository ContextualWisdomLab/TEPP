//! Unit-mean decomposition uses the shared compensated-mean authority.

use longitudinal_core::{ComponentLevel, OccasionObservation, decompose_within_between};

#[test]
fn decomposition_unit_mean_preserves_subnormal_ties_to_even() {
    let ulp = f64::from_bits(1);
    let recovered = decompose_within_between(&[
        OccasionObservation::new(0, 0, ulp),
        OccasionObservation::new(0, 1, f64::from_bits(2)),
        OccasionObservation::new(1, 0, 0.0),
        OccasionObservation::new(1, 1, 0.0),
    ])
    .expect("the representable unit mean must survive binary64 halfway rounding");

    assert_eq!(recovered[0].level(), ComponentLevel::Between);
    assert_eq!(recovered[0].value().to_bits(), f64::from_bits(2).to_bits());
    assert_eq!(recovered[1].level(), ComponentLevel::Within);
    assert_eq!(recovered[1].value().to_bits(), (-ulp).to_bits());
    assert_eq!(recovered[2].level(), ComponentLevel::Within);
    assert_eq!(recovered[2].value().to_bits(), 0.0_f64.to_bits());
}
