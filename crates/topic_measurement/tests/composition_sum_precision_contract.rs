//! Composition validation must not lose tiny positive mass after a dominant part.

use topic_measurement::{TopicMeasurementError, additive_log_ratio};

#[test]
fn compensated_sum_rejects_mass_hidden_by_naive_floating_point_addition() {
    let mut composition = Vec::with_capacity(20_001);
    composition.push(1.0);
    composition.extend(std::iter::repeat_n(1.0e-16, 20_000));

    assert_eq!(
        additive_log_ratio(&composition),
        Err(TopicMeasurementError::InvalidComposition),
        "the true mass exceeds one by 2e-12 even though naive ordered addition rounds to one"
    );
}

#[test]
fn compensated_sum_accepts_a_valid_many_part_composition() {
    let tiny_mass = 1.0e-16;
    let tiny_parts = 10_000_usize;
    let dominant = 1.0 - tiny_mass * tiny_parts as f64;
    let mut composition = Vec::with_capacity(tiny_parts + 1);
    composition.push(dominant);
    composition.extend(std::iter::repeat_n(tiny_mass, tiny_parts));

    let coordinates = additive_log_ratio(&composition).expect("valid unit simplex");
    assert_eq!(coordinates.len(), tiny_parts);
    assert!(coordinates.iter().all(|value| value.is_finite()));
}
