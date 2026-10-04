//! Private regression tests for finite expectation sufficient statistics.

use super::{PrevalenceFeature, ReferenceTopicInput, expectation};
use crate::TopicMeasurementError;
use temporal_core::EventTime;
use uuid::Uuid;

/// Construct the same two-document private shape used by reference unit tests.
fn input(term_rows: Vec<Vec<(usize, f64)>>) -> ReferenceTopicInput {
    ReferenceTopicInput {
        document_ids: vec![Uuid::from_u128(1), Uuid::from_u128(2)],
        event_times: vec![
            EventTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("first event time"),
            EventTime::parse_rfc3339("2026-01-02T00:00:00Z").expect("second event time"),
        ],
        term_rows,
        vocabulary_size: 2,
        design: vec![vec![1.0], vec![1.0]],
        features: vec![PrevalenceFeature::Intercept],
        transition_pairs: vec![(0, 1)],
    }
}

/// Finite likelihood and per-document counts must not admit an infinite topic count.
#[test]
fn topic_count_overflow_is_rejected_with_finite_likelihood() {
    let input = input(vec![vec![(0, f64::MAX)], vec![(0, f64::MAX)]]);
    let theta = vec![vec![0.75, 0.25]; 2];
    let beta = vec![vec![1.0 - 1e-12, 1e-12]; 2];
    let probability = 0.75 * beta[0][0] + 0.25 * beta[1][0];
    let expected = f64::MAX * 0.75 * beta[0][0] / probability;
    let likelihood = 2.0 * (f64::MAX * probability.ln());
    assert!(expected.is_finite());
    assert!(!(expected + expected).is_finite());
    assert!(likelihood.is_finite());
    assert_eq!(
        expectation(&input, &theta, &beta, 2),
        Err(TopicMeasurementError::NonFiniteEstimate),
        "finite likelihood cannot certify finite sufficient statistics"
    );
}

/// Finite term-level contributions must not overflow a document-level sum.
#[test]
fn document_count_overflow_is_rejected_with_finite_likelihood() {
    let input = input(vec![
        vec![(0, f64::MAX * 0.7), (1, f64::MAX * 0.7)],
        vec![(0, 1.0)],
    ]);
    let theta = vec![vec![0.9, 0.1]; 2];
    let beta = vec![vec![0.5, 0.5]; 2];
    let expected = f64::MAX * 0.7 * 0.9 * 0.5 / 0.5;
    assert!(expected.is_finite());
    assert!(!(expected + expected).is_finite());
    assert!((2.0 * (f64::MAX * 0.7 * 0.5_f64.ln())).is_finite());
    assert_eq!(
        expectation(&input, &theta, &beta, 2),
        Err(TopicMeasurementError::NonFiniteEstimate),
        "finite term-level contributions cannot certify a finite document sum"
    );
}

/// Ordinary representable counts retain every original responsibility and shape.
#[test]
fn ordinary_counts_preserve_finite_sufficient_statistics() {
    let input = input(vec![vec![(0, 8.0)], vec![(1, 4.0)]]);
    let theta = vec![vec![0.75, 0.25]; 2];
    let beta = vec![vec![0.75, 0.25], vec![0.25, 0.75]];
    let (document_counts, topic_counts, likelihood) =
        expectation(&input, &theta, &beta, 2).expect("finite expectation");
    assert_eq!(document_counts.len(), 2);
    assert_eq!(topic_counts.len(), 2);
    assert!(document_counts.iter().all(|row| row.len() == 2));
    assert!(topic_counts.iter().all(|row| row.len() == 2));
    assert!(
        document_counts
            .iter()
            .flatten()
            .all(|value| value.is_finite())
    );
    assert!(topic_counts.iter().flatten().all(|value| value.is_finite()));
    assert!(likelihood.is_finite());
    assert_eq!(document_counts, vec![vec![7.2, 0.8], vec![2.0, 2.0]]);
    assert_eq!(topic_counts, vec![vec![7.2, 2.0], vec![0.8, 2.0]]);
    assert_eq!(
        likelihood.to_bits(),
        (8.0 * 0.625_f64.ln() + 4.0 * 0.375_f64.ln()).to_bits()
    );
    println!("finite values: document=4/4 topic=4/4 likelihood=1/1");
}
