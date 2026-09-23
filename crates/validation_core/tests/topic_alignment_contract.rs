//! Known-topic recovery must align fitted topic labels before parameter error is scored.

use validation_core::{ValidationError, align_topic_probability_rows};

#[test]
fn permutation_equivalent_topics_align_with_zero_content_distance() {
    let truth = vec![
        vec![0.8, 0.1, 0.1],
        vec![0.1, 0.8, 0.1],
        vec![0.1, 0.1, 0.8],
    ];
    let fitted = vec![truth[2].clone(), truth[0].clone(), truth[1].clone()];

    let alignment = align_topic_probability_rows(&truth, &fitted).expect("topic alignment");

    assert_eq!(alignment.truth_to_fitted(), &[1, 2, 0]);
    assert!(alignment.total_squared_hellinger_distance() <= f64::EPSILON);
}

#[test]
fn alignment_solves_the_global_assignment_instead_of_greedy_row_matching() {
    let truth = vec![
        vec![0.248282060982837, 0.3135482053042615, 0.43816973371290163],
        vec![0.39942717362058733, 0.05384571875969858, 0.5467271076197141],
    ];
    let fitted = vec![
        vec![0.470282522814068, 0.21030927527541535, 0.31940820191051655],
        vec![0.47459909500752845, 0.4008933115592991, 0.12450759343317243],
    ];

    let alignment = align_topic_probability_rows(&truth, &fitted).expect("global alignment");

    // Truth row 0 is individually closer to fitted row 0, but consuming that row
    // forces a much worse match for truth row 1. The global one-to-one optimum is
    // therefore the crossed assignment.
    assert_eq!(alignment.truth_to_fitted(), &[1, 0]);
    assert!((alignment.total_squared_hellinger_distance() - 0.11090633121459383).abs() < 1.0e-12);
}

#[test]
fn malformed_probability_bases_fail_closed() {
    let valid = vec![vec![0.8, 0.2], vec![0.2, 0.8]];

    for invalid in [
        Vec::<Vec<f64>>::new(),
        vec![vec![0.5, 0.5]],
        vec![vec![0.5, 0.5], vec![0.2]],
        vec![vec![0.5, 0.5], vec![f64::NAN, 1.0]],
        vec![vec![0.5, 0.5], vec![-0.1, 1.1]],
        vec![vec![0.5, 0.5], vec![0.0, 0.0]],
        vec![vec![0.5, 0.5], vec![0.4, 0.4]],
        vec![vec![0.5, 0.5], vec![f64::MAX, f64::MAX]],
    ] {
        assert_eq!(
            align_topic_probability_rows(&valid, &invalid),
            Err(ValidationError::InvalidInput)
        );
    }

    assert_eq!(
        align_topic_probability_rows(&[vec![0.7, 0.3]], &[vec![0.7, 0.3]]),
        Err(ValidationError::InvalidInput)
    );
    assert_eq!(
        align_topic_probability_rows(
            &valid,
            &[vec![0.7, 0.2, 0.1], vec![0.1, 0.2, 0.7]],
        ),
        Err(ValidationError::InvalidInput)
    );
    assert_eq!(
        align_topic_probability_rows(
            &valid,
            &[vec![0.8, 0.2], vec![0.2, 0.8], vec![0.5, 0.5]],
        ),
        Err(ValidationError::InvalidInput)
    );
}
