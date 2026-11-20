//! Check componentwise binary64 rounding of the smallest positive count.

use super::{PrevalenceFeature, ReferenceTopicInput, expectation};
use temporal_core::EventTime;
use uuid::Uuid;

/// Check both sufficient-statistic arrays on one injected expectation state.
fn assert_counts(count: f64, theta: &[f64], beta: &[f64], expected: &[u64]) {
    let topics = theta.len();
    assert_eq!(beta.len(), topics);
    assert_eq!(expected.len(), topics);
    assert!(theta.iter().all(|part| part.is_finite() && *part >= 0.0));
    assert_eq!(theta.iter().sum::<f64>().to_bits(), 1.0_f64.to_bits());
    assert!(count.is_finite() && count > 0.0);
    let beta: Vec<_> = beta
        .iter()
        .map(|part| {
            assert!((0.0..=1.0).contains(part));
            vec![*part, 1.0 - part]
        })
        .collect();
    let input = ReferenceTopicInput {
        document_ids: vec![Uuid::from_u128(1)],
        event_times: vec![EventTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("time")],
        term_rows: vec![vec![(0, count)]],
        vocabulary_size: 2,
        design: vec![vec![1.0]],
        features: vec![PrevalenceFeature::Intercept],
        transition_pairs: Vec::new(),
    };
    let (document, topic, ll) = expectation(&input, &[theta.to_vec()], &beta, topics)
        .expect("finite injected sufficient statistics");
    assert_eq!(document.len(), 1);
    assert_eq!(document[0].len(), topics);
    assert_eq!(topic.len(), topics);
    assert!(topic.iter().all(|row| row.len() == 2));
    assert!(ll.is_finite());
    assert_eq!(
        document[0]
            .iter()
            .map(|part| part.to_bits())
            .collect::<Vec<_>>(),
        expected,
        "document-topic counts"
    );
    assert_eq!(
        topic.iter().map(|row| row[0].to_bits()).collect::<Vec<_>>(),
        expected,
        "topic-term counts"
    );
    assert!(topic.iter().all(|row| row[1].to_bits() == 0));
}

/// Exact products differ across the half-quantum threshold despite rounded normalization.
#[test]
fn asymmetric_two_topics_round_toward_the_larger_exact_mass() {
    let q = f64::from_bits(1);
    let smaller = f64::from_bits((27.0_f64 / 32.0).to_bits() - 1);
    assert_counts(
        q,
        &[9.0 / 16.0, 7.0 / 16.0],
        &[21.0 / 32.0, smaller],
        &[1, 0],
    );
}

/// The reversed topic order must place the quantum at the corresponding index.
#[test]
fn asymmetric_two_topics_keep_topic_order() {
    let q = f64::from_bits(1);
    let smaller = f64::from_bits((27.0_f64 / 32.0).to_bits() - 1);
    assert_counts(
        q,
        &[7.0 / 16.0, 9.0 / 16.0],
        &[smaller, 21.0 / 32.0],
        &[0, 1],
    );
}

/// An exact half-quantum rounds to zero rather than inventing a winner.
#[test]
fn true_half_ties_round_to_even_zero() {
    assert_counts(
        f64::from_bits(1),
        &[9.0 / 16.0, 7.0 / 16.0],
        &[21.0 / 32.0, 27.0 / 32.0],
        &[0, 0],
    );
}

/// Preserve a quantum with one nonzero component and ordinary count arithmetic.
#[test]
fn one_hot_minimum_and_normal_count_controls() {
    assert_counts(f64::from_bits(1), &[1.0, 0.0], &[1.0, 1.0], &[1, 0]);
    assert_counts(
        1.0,
        &[0.25, 0.75],
        &[1.0, 1.0],
        &[0.25_f64.to_bits(), 0.75_f64.to_bits()],
    );
}

/// Compare exact products even when both round to zero in binary64.
#[test]
fn both_rounded_zero_products_still_have_an_order() {
    let q = f64::from_bits(1);
    assert_counts(q, &[0.375, 0.25, 0.375], &[q, q, 0.0], &[1, 0, 0]);
}

/// A component must beat the total of all others, not only the runner-up.
#[test]
fn three_topics_compare_the_other_mass_in_aggregate() {
    let q = f64::from_bits(1);
    assert_counts(
        q,
        &[0.5, 0.25, 0.25],
        &[1.0, 1.0, f64::from_bits(1.0_f64.to_bits() - 1)],
        &[1, 0, 0],
    );
    assert_counts(q, &[0.375, 0.3125, 0.3125], &[1.0, 1.0, 1.0], &[0, 0, 0]);
}

/// Preserve the winning count or half tie when exact masses begin on a word boundary.
#[test]
fn minimum_count_word_aligned_mass_keeps_the_winner_and_tie() {
    let q = f64::from_bits(1);
    let theta = [0.5, 0.5];
    // For theta = 1/2, beta = 2^-123 gives a product shift of 1920 = 15 * 128.
    let aligned = 900_u64 << 52;
    let lo = f64::from_bits(aligned);
    let hi = f64::from_bits(aligned + 1);
    assert_counts(q, &theta, &[hi, lo], &[1, 0]);
    assert_counts(q, &theta, &[lo, hi], &[0, 1]);
    assert_counts(q, &theta, &[lo, lo], &[0, 0]);
    // Scaling both masses by two moves the shift off the boundary, not the winner.
    assert_counts(q, &theta, &[2.0 * hi, 2.0 * lo], &[1, 0]);
}
