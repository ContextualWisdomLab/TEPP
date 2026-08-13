//! Method sources are modeled explicitly and cannot become inferential weights.

use method_effects::{
    MethodEffectsError, MethodSourceKind, refuse_method_source_as_inferential_weight,
    source_recovery_rate,
};

#[test]
fn method_sources_cannot_become_inferential_weights() {
    for kind in [
        MethodSourceKind::Template,
        MethodSourceKind::Section,
        MethodSourceKind::CopiedText,
        MethodSourceKind::Style,
        MethodSourceKind::Modality,
        MethodSourceKind::CorpusBackground,
    ] {
        assert_eq!(
            refuse_method_source_as_inferential_weight(kind),
            Err(MethodEffectsError::MethodSourceIsNotInferentialWeight)
        );
    }
}

#[test]
fn recovered_sources_match_known_truth_better_than_a_single_label() {
    let truth = [
        MethodSourceKind::Template,
        MethodSourceKind::CopiedText,
        MethodSourceKind::Section,
    ];
    let recovered = [
        MethodSourceKind::Template,
        MethodSourceKind::CopiedText,
        MethodSourceKind::Section,
    ];
    let collapsed = [
        MethodSourceKind::Template,
        MethodSourceKind::Template,
        MethodSourceKind::Template,
    ];

    let recovered_rate = source_recovery_rate(&truth, &recovered).expect("recovered");
    let collapsed_rate = source_recovery_rate(&truth, &collapsed).expect("collapsed");
    let expected = {
        let mut matches = 0_u32;
        for (truth_kind, decided_kind) in truth.iter().zip(recovered.iter()) {
            if truth_kind == decided_kind {
                matches += 1;
            }
        }
        f64::from(matches) / f64::from(u32::try_from(truth.len()).expect("len"))
    };
    assert!((recovered_rate - expected).abs() < f64::EPSILON);
    assert!(recovered_rate > collapsed_rate);
}

#[test]
fn empty_source_payloads_fail_closed() {
    assert_eq!(
        source_recovery_rate(&[], &[]),
        Err(MethodEffectsError::InvalidSourcePayload)
    );
}
