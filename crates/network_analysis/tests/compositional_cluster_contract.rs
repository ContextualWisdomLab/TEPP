//! Raw simplex coordinates are not Euclidean; cluster pairs recover known truth.

use network_analysis::{
    ClusterLabel, NetworkError, cluster_pair_precision, cluster_pair_recall,
    refuse_raw_simplex_as_euclidean,
};

#[test]
fn raw_simplex_proportions_are_not_euclidean_coordinates() {
    assert_eq!(
        refuse_raw_simplex_as_euclidean(&[0.25, 0.25, 0.5]),
        Err(NetworkError::RawSimplexIsNotEuclidean)
    );
    assert_eq!(
        refuse_raw_simplex_as_euclidean(&[f64::NAN, 0.5]),
        Err(NetworkError::InvalidCoordinate)
    );
}

#[test]
fn pair_metrics_recover_known_clusters_and_are_label_invariant() {
    let truth = [
        ClusterLabel::new(0),
        ClusterLabel::new(0),
        ClusterLabel::new(1),
        ClusterLabel::new(1),
    ];
    let recovered = [
        ClusterLabel::new(7),
        ClusterLabel::new(7),
        ClusterLabel::new(3),
        ClusterLabel::new(3),
    ];
    let scrambled = [
        ClusterLabel::new(0),
        ClusterLabel::new(1),
        ClusterLabel::new(0),
        ClusterLabel::new(1),
    ];

    let precision = cluster_pair_precision(&truth, &recovered).expect("precision");
    let recall = cluster_pair_recall(&truth, &recovered).expect("recall");
    let scrambled_precision = cluster_pair_precision(&truth, &scrambled).expect("scrambled");

    let expected_precision = {
        let mut decided_same = 0_u32;
        let mut decided_and_truth_same = 0_u32;
        for left in 0..truth.len() {
            for right in (left + 1)..truth.len() {
                if recovered[left] == recovered[right] {
                    decided_same += 1;
                    if truth[left] == truth[right] {
                        decided_and_truth_same += 1;
                    }
                }
            }
        }
        f64::from(decided_and_truth_same) / f64::from(decided_same)
    };
    assert!((precision - expected_precision).abs() < f64::EPSILON);
    assert!((recall - 1.0).abs() < f64::EPSILON);
    assert!(precision > scrambled_precision);
}

#[test]
fn empty_or_mismatched_cluster_payloads_fail_closed() {
    assert_eq!(
        cluster_pair_precision(&[], &[]),
        Err(NetworkError::InvalidClusterPayload)
    );
}
