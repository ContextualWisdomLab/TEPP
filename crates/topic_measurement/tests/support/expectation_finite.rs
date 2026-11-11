//! Private regression tests for finite expectation sufficient statistics.

use super::{PrevalenceFeature, ReferenceTopicInput, expectation, normalize};
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

/// The shared beta normalizer never returns a zero, negative, or infinite part.
#[test]
fn normalized_positive_parts_remain_representable() {
    let mut ordinary = [300.0, f64::MIN_POSITIVE];
    normalize(&mut ordinary).expect("representable positive parts");
    assert!(ordinary.iter().all(|x| x.is_finite() && *x > 0.0));
    let mut smallest = [1.0, f64::from_bits(1)];
    normalize(&mut smallest).expect("smallest part remains representable at denominator one");
    assert_eq!(smallest[1].to_bits(), 1);
    // These are helper-only injected states; the public underflow is tested separately.
    for mut values in [[300.0, f64::from_bits(1)], [1.0, 0.0], [2.0, -1.0]] {
        assert_eq!(
            normalize(&mut values),
            Err(TopicMeasurementError::NonFiniteEstimate)
        );
    }
    let mut division_overflow = [f64::MAX, -f64::MAX, f64::MIN_POSITIVE];
    assert_eq!(
        normalize(&mut division_overflow),
        Err(TopicMeasurementError::NonFiniteEstimate)
    );
}

/// Preserve a tiny topic contribution even when its unscaled mixture mass underflows.
#[test]
fn tiny_topic_mass_preserves_representable_scaled_counts() {
    let tiny = 2.0_f64.powi(-600);
    for (count, other_mass, expected) in [
        (2.0_f64.powi(1000), 0.5, 2.0_f64.powi(-199)),
        (1.0, tiny, tiny),
        (0.5, tiny, tiny * 0.5),
        (0.0, tiny, 0.0),
    ] {
        let input = input(vec![vec![(0, count)], vec![(1, 1.0)]]);
        let theta = vec![vec![tiny, 1.0], vec![0.5, 0.5]];
        let beta = vec![vec![tiny, 1.0], vec![other_mass, 1.0 - other_mass]];
        assert_eq!(theta[0].iter().sum::<f64>().to_bits(), 1.0_f64.to_bits());
        assert_eq!(beta[0].iter().sum::<f64>().to_bits(), 1.0_f64.to_bits());
        assert_eq!(beta[1].iter().sum::<f64>().to_bits(), 1.0_f64.to_bits());
        let (document_counts, topic_counts, likelihood) =
            expectation(&input, &theta, &beta, 2).expect("finite sufficient statistics");
        assert!(likelihood.is_finite());
        assert_eq!(document_counts[0][0].to_bits(), expected.to_bits());
        assert_eq!(topic_counts[0][0].to_bits(), expected.to_bits());
    }
}

/// A subnormal mixture must still allocate exactly the observed count across topics.
#[test]
fn subnormal_mixture_preserves_posterior_count_conservation() {
    let q = f64::from_bits(1);
    let input = input(vec![vec![(0, 1.0)], vec![(1, 1.0)]]);
    let theta = vec![vec![0.625, 0.375], vec![0.5, 0.5]];
    let beta = vec![vec![2.0 * q, 1.0], vec![q, 1.0]];
    for row in theta.iter().chain(&beta) {
        assert!(row.iter().all(|x| x.is_finite() && *x > 0.0));
        assert_eq!(row.iter().sum::<f64>().to_bits(), 1.0_f64.to_bits());
    }
    let (documents, topics, likelihood) =
        expectation(&input, &theta, &beta, 2).expect("finite normalized state");
    let actual = documents[0].iter().sum::<f64>();
    println!(
        "NUM-02 observed=1 actual={actual} document={:?} topic0={:?} likelihood={likelihood}",
        documents[0], topics[0]
    );
    assert!(likelihood.is_finite());
    assert!(
        documents[0]
            .iter()
            .all(|x| x.is_finite() && *x >= 0.0 && *x <= 1.0)
    );
    assert!((actual - 1.0).abs() <= 1e-14);
    assert!((documents[0][0] - 10.0 / 13.0).abs() <= 1e-14);
    assert!((documents[0][1] - 3.0 / 13.0).abs() <= 1e-14);
}

/// Common scaling preserves posterior normalization, likelihood, and large/small count units.
#[test]
fn subnormal_mixture_uses_consistent_likelihood_and_count_scale() {
    let q = f64::from_bits(1);
    for count in [0.0, 0.5, 1.0, 2.0_f64.powi(900)] {
        let input = input(vec![vec![(0, count)], vec![(1, 1.0)]]);
        let theta = vec![vec![0.625, 0.375], vec![0.5, 0.5]];
        let beta = vec![vec![2.0 * q, 1.0], vec![q, 1.0]];
        let (documents, topics, likelihood) =
            expectation(&input, &theta, &beta, 2).expect("finite subnormal mixture");
        let log_probability = 1.625_f64.ln() - 1074.0 * std::f64::consts::LN_2;
        assert!(likelihood.is_finite());
        assert!(
            (likelihood - count * log_probability).abs()
                <= 1e-14 * (1.0 + (count * log_probability).abs())
        );
        if count == 0.0 {
            assert_eq!(documents[0], vec![0.0, 0.0]);
        } else {
            assert!((documents[0].iter().sum::<f64>() / count - 1.0).abs() < 1e-14);
            for (topic, fraction) in [10.0 / 13.0, 3.0 / 13.0].into_iter().enumerate() {
                assert!((documents[0][topic] / count - fraction).abs() < 1e-14);
                assert!((topics[topic][0] / count - fraction).abs() < 1e-14);
            }
        }
    }
}

/// Two individually rounded-zero products can still have positive representable mixture mass.
#[test]
fn all_rounded_zero_masses_keep_a_positive_mixture() {
    let q = f64::from_bits(1);
    let input = input(vec![vec![(0, 1.0)], vec![(1, 1.0)]]);
    let theta = vec![vec![0.5, 0.5]; 2];
    let beta = vec![vec![q, 1.0]; 2];
    let (documents, topics, likelihood) = expectation(&input, &theta, &beta, 2)
        .expect("positive exact mixture despite rounded products");
    assert_eq!(documents[0], vec![0.5, 0.5]);
    assert_eq!(topics[0][0].to_bits(), 0.5_f64.to_bits());
    assert_eq!(topics[1][0].to_bits(), 0.5_f64.to_bits());
    assert!((likelihood - q.ln()).abs() < 1e-12);
}

/// Construct a public result DTO at a private normalized state; this is not a fit trajectory.
fn model_at_state(theta: Vec<Vec<f64>>, beta: Vec<Vec<f64>>) -> super::ReferenceTopicModel {
    super::ReferenceTopicModel {
        seed: 7,
        iterations: 4,
        objective: 0.0,
        topic_term_probabilities: beta,
        document_topic_proportions: theta,
        document_coordinate_variances: vec![vec![1.0]; 2],
        prevalence_coefficients: vec![vec![0.0]],
        prevalence_features: vec![PrevalenceFeature::Intercept],
        sequence_edges: vec![],
        connected_post_count: 0,
        lineage_count: 0,
    }
}

/// Public likelihood must share the expectation mixture and log normalization.
#[test]
fn num03_public_subnormal_likelihood_matches_expectation() {
    let q = f64::from_bits(1);
    let input = input(vec![vec![(0, 1.0)], vec![(1, 1.0)]]);
    let model = model_at_state(
        vec![vec![0.625, 0.375], vec![0.5, 0.5]],
        vec![vec![2.0 * q, 1.0], vec![q, 1.0]],
    );
    let (_, _, expected) = expectation(
        &input,
        &model.document_topic_proportions,
        &model.topic_term_probabilities,
        2,
    )
    .expect("finite expectation");
    let public = input
        .in_sample_log_likelihood(&model)
        .expect("finite public likelihood");
    println!(
        "NUM-03 expected={expected} public={public} difference={}",
        expected - public
    );
    assert!((expected - public).abs() < 1e-12);
}

/// Rounded-zero topic products must not disagree about positive mixture admission.
#[test]
fn num03_public_rounded_zero_mixture_matches_expectation() {
    let q = f64::from_bits(1);
    let input = input(vec![vec![(0, 1.0)], vec![(1, 1.0)]]);
    let model = model_at_state(vec![vec![0.5, 0.5]; 2], vec![vec![q, 1.0]; 2]);
    let (_, _, expected) = expectation(
        &input,
        &model.document_topic_proportions,
        &model.topic_term_probabilities,
        2,
    )
    .expect("positive exact mixture");
    let public = input.in_sample_log_likelihood(&model);
    println!("NUM-03 rounded-zero expectation={expected} public={public:?}");
    assert!(public.is_ok_and(|x| (x - expected).abs() < 1e-12));
}

/// Exact-zero topic factors preserve zero responsibility beside positive subnormal mass.
#[test]
fn zero_factor_subnormal_expectation_preserves_counts() {
    let tiny = f64::from_bits(1_u64 << 44);
    for count in [0.0, 0.5, 1.0] {
        for (first_theta, first_beta, fraction) in [
            (vec![1.0, 0.0], vec![tiny, tiny], 1.0),
            (vec![0.5, 0.5], vec![tiny, 0.0], 1.0),
            (vec![0.0, 1.0], vec![tiny, tiny], 0.0),
            (vec![0.5, 0.5], vec![-0.0, tiny], 0.0),
        ] {
            let input = input(vec![vec![(0, count)], vec![(1, 1.0)]]);
            let theta = vec![first_theta.clone(), vec![0.5, 0.5]];
            let beta = vec![vec![first_beta[0], 1.0], vec![first_beta[1], 1.0]];
            let probability = first_theta[0] * first_beta[0] + first_theta[1] * first_beta[1];
            assert!(probability > 0.0 && probability.is_subnormal());
            let (documents, topics, likelihood) =
                expectation(&input, &theta, &beta, 2).expect("zero factors are zero mass");
            assert_eq!(
                documents[0],
                vec![count * fraction, count * (1.0 - fraction)]
            );
            assert_eq!(topics[0][0].to_bits(), (count * fraction).to_bits());
            assert_eq!(topics[1][0].to_bits(), (count * (1.0 - fraction)).to_bits());
            assert!((likelihood - count * probability.ln()).abs() <= 1e-12);
        }
    }
}

/// A zero mixture and nonfinite or negative partners remain invalid scaled inputs.
#[test]
fn zero_factor_subnormal_expectation_preserves_refusals() {
    let tiny = f64::from_bits(1_u64 << 44);
    let input = input(vec![vec![(0, 1.0)], vec![(1, 1.0)]]);
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -1.0] {
        for (first_theta, first_beta) in [
            (vec![0.5, 0.0], vec![tiny, bad]),
            (vec![0.5, bad], vec![tiny, 0.0]),
        ] {
            let theta = vec![first_theta, vec![0.5, 0.5]];
            let beta = vec![vec![first_beta[0], 1.0], vec![first_beta[1], 1.0]];
            assert_eq!(
                expectation(&input, &theta, &beta, 2),
                Err(TopicMeasurementError::NonFiniteEstimate)
            );
        }
    }
    let theta = vec![vec![1.0, 0.0], vec![0.5, 0.5]];
    let beta = vec![vec![0.0, 1.0], vec![tiny, 1.0]];
    assert_eq!(
        expectation(&input, &theta, &beta, 2),
        Err(TopicMeasurementError::NonFiniteEstimate)
    );
}
