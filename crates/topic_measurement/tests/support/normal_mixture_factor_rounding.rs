//! Injected expectation-state controls for unrounded subnormal factors.

use super::{PrevalenceFeature, ReferenceTopicInput, expectation};
use temporal_core::EventTime;
use uuid::Uuid;

/// Build a one-cell private expectation fixture, not an optimizer-produced fit.
fn injected_input(count: f64) -> ReferenceTopicInput {
    ReferenceTopicInput {
        document_ids: vec![Uuid::from_u128(1)],
        event_times: vec![EventTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("time")],
        term_rows: vec![vec![(0, count)]],
        vocabulary_size: 2,
        design: vec![vec![1.0]],
        features: vec![PrevalenceFeature::Intercept],
        transition_pairs: Vec::new(),
    }
}

/// Check finite shapes and the focal count against an independently derived ratio.
fn assert_focal(theta: [f64; 2], beta: [[f64; 2]; 2], expected: f64) {
    let count = f64::from_bits(2023_u64 << 52); // Exactly 2^1000.
    let probability = theta[0] * beta[0][0] + theta[1] * beta[1][0];
    assert!(probability.is_normal());
    let input = injected_input(count);
    let (document, topic, ll) =
        expectation(&input, &[theta.to_vec()], &beta.map(|row| row.to_vec()), 2)
            .expect("finite injected sufficient statistics");
    assert_eq!(document.len(), 1);
    assert_eq!(document[0].len(), 2);
    assert_eq!(topic.len(), 2);
    assert!(topic.iter().all(|row| row.len() == 2));
    assert!(
        document
            .iter()
            .flatten()
            .chain(topic.iter().flatten())
            .all(|v| v.is_finite())
    );
    assert!(ll.is_finite());
    let actual = topic[0][0];
    println!(
        "actual_bits={} expected_bits={} actual={actual:e} expected={expected:e}",
        actual.to_bits(),
        expected.to_bits()
    );
    if expected == 0.0 {
        assert_eq!(actual.to_bits(), 0);
    } else {
        assert!(expected.is_normal());
        assert!(
            (actual / expected - 1.0).abs() <= 8.0 * f64::EPSILON,
            "focal sufficient statistic loses relative precision: actual={actual:e} expected={expected:e}"
        );
    }
}

/// A minimum-subnormal beta divided first by0.75 loses one quarter of the count.
#[test]
fn subnormal_beta_retains_normal_expected_count() {
    let q = f64::from_bits(1);
    assert_focal([0.25, 0.75], [[q, 1.0], [1.0, q]], 2.0_f64.powi(-74) / 3.0);
}

/// The symmetric theta-factor placement must retain the same representable count.
#[test]
fn subnormal_theta_retains_normal_expected_count() {
    let q = f64::from_bits(1);
    assert_focal(
        [q, 1.0],
        [[0.25, 0.75], [0.75, 0.25]],
        2.0_f64.powi(-74) / 3.0,
    );
}

/// Ordinary normal factors preserve the existing fast-path count.
#[test]
fn normal_factor_control_preserves_expected_count() {
    assert_focal([0.25, 0.75], [[1.0, 0.0], [1.0, 0.0]], 2.0_f64.powi(998));
}

/// Exact zeros stay zero while a rounded-zero positive product recovers its count.
#[test]
fn zero_and_rounded_zero_factors_preserve_outcomes() {
    assert_focal([0.0, 1.0], [[0.25, 0.75], [0.75, 0.25]], 0.0);
    assert_focal([0.25, 0.75], [[0.0, 1.0], [1.0, 0.0]], 0.0);
    let small = f64::from_bits(423_u64 << 52); // Exactly 2^-600.
    assert_eq!((small * small).to_bits(), 0);
    assert_focal(
        [small, 1.0],
        [[small, 1.0], [0.75, 0.25]],
        2.0_f64.powi(-198) / 3.0,
    );
}
