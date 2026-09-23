use validation_core::{
    ValidationError, align_topic_probability_rows, realign_topic_probability_rows,
    realign_topic_probability_vector,
};

fn alignment() -> validation_core::TopicAlignment {
    let truth = vec![
        vec![0.80, 0.15, 0.05],
        vec![0.10, 0.80, 0.10],
        vec![0.05, 0.15, 0.80],
    ];
    let fitted = vec![truth[2].clone(), truth[0].clone(), truth[1].clone()];
    align_topic_probability_rows(&truth, &fitted).expect("topic alignment")
}

#[test]
fn fitted_document_state_is_reexpressed_in_truth_topic_order() {
    let alignment = alignment();
    let fitted = vec![0.20, 0.55, 0.25];

    let aligned =
        realign_topic_probability_vector(&alignment, &fitted).expect("aligned topic state");

    assert_eq!(aligned, vec![0.55, 0.25, 0.20]);
    assert!((aligned.iter().sum::<f64>() - 1.0).abs() < 1.0e-12);
}

#[test]
fn document_state_rows_preserve_row_identity_while_reordering_topics() {
    let alignment = alignment();
    let fitted_rows = vec![vec![0.20, 0.55, 0.25], vec![0.50, 0.10, 0.40]];

    let aligned =
        realign_topic_probability_rows(&alignment, &fitted_rows).expect("aligned topic states");

    assert_eq!(aligned[0], vec![0.55, 0.25, 0.20]);
    assert_eq!(aligned[1], vec![0.10, 0.40, 0.50]);
}

#[test]
fn document_state_alignment_fails_closed_on_invalid_simplex_geometry() {
    let alignment = alignment();

    for invalid in [
        vec![0.5, 0.5],
        vec![0.5, 0.5, 0.5],
        vec![0.5, -0.1, 0.6],
        vec![0.5, f64::NAN, 0.5],
    ] {
        assert_eq!(
            realign_topic_probability_vector(&alignment, &invalid),
            Err(ValidationError::InvalidInput)
        );
    }

    assert_eq!(
        realign_topic_probability_rows(&alignment, &[]),
        Err(ValidationError::InvalidInput)
    );
}
