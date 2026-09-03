//! Exact-zero within residuals have one public encoding.

use longitudinal_core::{ComponentLevel, OccasionObservation, decompose_within_between};

#[test]
fn exact_zero_within_residuals_use_one_canonical_public_encoding() {
    let recovered = decompose_within_between(&[
        OccasionObservation::new(0, 0, -0.0),
        OccasionObservation::new(0, 1, 0.0),
        OccasionObservation::new(1, 0, 1.0),
        OccasionObservation::new(1, 1, 1.0),
    ])
    .expect("finite repeated observations admit within/between decomposition");

    let unit_zero_within: Vec<_> = recovered
        .iter()
        .filter(|component| {
            component.unit_index() == 0 && component.level() == ComponentLevel::Within
        })
        .collect();

    assert_eq!(unit_zero_within.len(), 2);
    for component in unit_zero_within {
        assert_eq!(component.value().to_bits(), 0.0_f64.to_bits());
    }
}
