//! True-parameter alignment recovery and fail-closed ADR 0004 gates.
#![allow(clippy::cast_precision_loss)]

use concept_dictionary::{
    ConceptError, ConceptId, InferentialWeightKind, InvarianceLevel, LanguageProfile, LanguageTag,
    ProfileStatus, admit_inferential_weight, apply_default_stopword_deletion, bind_semantic_unit,
    claim_comparative_interpretation, compare_cross_language_means, concept_coordinate_rmse,
    force_unknown_into_known, share_concept, source_span, treat_lexical_form_as_concept_identity,
    treat_translation_as_equivalence,
};

fn rmse(truth: &[f64], recovered: &[f64]) -> f64 {
    let n = truth.len() as f64;
    let sum_sq: f64 = truth
        .iter()
        .zip(recovered)
        .map(|(left, right)| {
            let residual = left - right;
            residual * residual
        })
        .sum();
    (sum_sq / n).sqrt()
}

#[test]
fn shared_concept_coordinates_recover_across_languages_with_computed_rmse() {
    let truth = [0.25_f64, -0.5, 0.75];
    let korean = truth;
    let english = truth;

    let korean_error = concept_coordinate_rmse(&truth, &korean).expect("korean recovery");
    let english_error = concept_coordinate_rmse(&truth, &english).expect("english recovery");
    let cross_error = concept_coordinate_rmse(&korean, &english).expect("shared-space alignment");

    assert!(
        korean_error < 1e-12,
        "Korean RMSE {korean_error} exceeded machine-scale bound"
    );
    assert!(
        english_error < 1e-12,
        "English RMSE {english_error} exceeded machine-scale bound"
    );
    assert!(
        cross_error < 1e-12,
        "cross-language RMSE {cross_error} exceeded machine-scale bound"
    );
    let independently_computed = rmse(&truth, &korean);
    assert!(
        (korean_error - independently_computed).abs() < 1e-12,
        "crate RMSE {korean_error} drifted from independently computed RMSE {independently_computed}"
    );
}

#[test]
fn translation_offset_is_detected_by_computed_rmse() {
    let truth = [0.25_f64, -0.5, 0.75];
    let translated = [0.35_f64, -0.4, 0.85];
    let error = concept_coordinate_rmse(&truth, &translated).expect("offset RMSE");
    let identity = concept_coordinate_rmse(&truth, &truth).expect("identity RMSE");
    assert!(
        error > identity,
        "translation offset RMSE {error} should exceed identity RMSE {identity}"
    );
}

#[test]
fn architecture_support_is_not_validated_interpretation() {
    for tag in [
        LanguageTag::Eng,
        LanguageTag::Kor,
        LanguageTag::Jpn,
        LanguageTag::Zho,
        LanguageTag::Vie,
        LanguageTag::Ind,
        LanguageTag::Fra,
        LanguageTag::Deu,
        LanguageTag::Tur,
    ] {
        let unresolved = LanguageProfile::new(tag, ProfileStatus::Unresolved, true);
        assert_eq!(
            claim_comparative_interpretation(&unresolved),
            Err(ConceptError::ProfileNotValidated)
        );
        let provisional = LanguageProfile::new(tag, ProfileStatus::Provisional, true);
        assert_eq!(
            claim_comparative_interpretation(&provisional),
            Err(ConceptError::ProfileNotValidated)
        );
        let validated = LanguageProfile::new(tag, ProfileStatus::Validated, false);
        claim_comparative_interpretation(&validated).expect("validated without architecture flag");
        let calibrated = LanguageProfile::new(tag, ProfileStatus::Calibrated, false);
        claim_comparative_interpretation(&calibrated)
            .expect("calibrated without architecture flag");
    }
}

#[test]
fn equivalent_meanings_share_one_concept_identity() {
    let concept = ConceptId::from_bytes([7; 16]);
    let shared = share_concept(LanguageTag::Kor, LanguageTag::Eng, concept);
    assert_eq!(shared.concept(), concept);
    assert_eq!(shared.left_language(), LanguageTag::Kor);
    assert_eq!(shared.right_language(), LanguageTag::Eng);
    assert_eq!(concept.as_bytes(), [7; 16]);
}

#[test]
fn translation_lexical_stopword_and_tfidf_paths_fail_closed() {
    assert_eq!(
        treat_translation_as_equivalence(),
        Err(ConceptError::TranslationNotEquivalence)
    );
    assert_eq!(
        treat_lexical_form_as_concept_identity(),
        Err(ConceptError::LexicalFormNotConcept)
    );
    assert_eq!(
        apply_default_stopword_deletion(),
        Err(ConceptError::StopwordDeletionForbidden)
    );
    assert_eq!(
        admit_inferential_weight(InferentialWeightKind::TfIdf),
        Err(ConceptError::InferentialWeightForbidden)
    );
    assert_eq!(
        admit_inferential_weight(InferentialWeightKind::Bm25),
        Err(ConceptError::InferentialWeightForbidden)
    );
    admit_inferential_weight(InferentialWeightKind::StatisticalPosterior)
        .expect("statistical posterior remains admissible");
}

#[test]
fn semantic_units_require_exact_spans_and_keep_unknown_unresolved() {
    assert_eq!(source_span(4, 4), Err(ConceptError::InvalidSourceSpan));
    assert_eq!(source_span(5, 2), Err(ConceptError::InvalidSourceSpan));
    assert_eq!(
        bind_semantic_unit(LanguageTag::Kor, None, None),
        Err(ConceptError::MissingSourceSpan)
    );

    let span = source_span(0, 4).expect("half-open span");
    assert_eq!(span.start_scalar(), 0);
    assert_eq!(span.end_scalar(), 4);

    let unknown = bind_semantic_unit(LanguageTag::Kor, Some(span), None).expect("unresolved");
    assert_eq!(unknown.language(), LanguageTag::Kor);
    assert_eq!(unknown.span(), span);
    assert_eq!(unknown.concept(), None);

    let known = ConceptId::from_bytes([1; 16]);
    assert_eq!(
        force_unknown_into_known(&unknown, known),
        Err(ConceptError::ForcedConceptAssignment)
    );

    let bound = bind_semantic_unit(LanguageTag::Eng, Some(span), Some(known)).expect("bound");
    assert_eq!(bound.concept(), Some(known));
    force_unknown_into_known(&bound, known).expect("already-known concept is not forced");
}

#[test]
fn cross_language_means_require_invariance_evidence() {
    assert_eq!(
        compare_cross_language_means(InvarianceLevel::None),
        Err(ConceptError::InvarianceRequired)
    );
    assert_eq!(
        compare_cross_language_means(InvarianceLevel::Configural),
        Err(ConceptError::InvarianceRequired)
    );
    assert_eq!(
        compare_cross_language_means(InvarianceLevel::Metric),
        Err(ConceptError::InvarianceRequired)
    );
    compare_cross_language_means(InvarianceLevel::Scalar).expect("scalar");
    compare_cross_language_means(InvarianceLevel::Partial).expect("partial");
}

#[test]
fn invalid_alignment_inputs_fail_closed() {
    assert_eq!(
        concept_coordinate_rmse(&[], &[1.0]),
        Err(ConceptError::InvalidNumericInput)
    );
    assert_eq!(
        concept_coordinate_rmse(&[1.0], &[1.0, 2.0]),
        Err(ConceptError::InvalidNumericInput)
    );
    assert_eq!(
        concept_coordinate_rmse(&[f64::NAN], &[0.0]),
        Err(ConceptError::InvalidNumericInput)
    );
    assert_eq!(
        concept_coordinate_rmse(&[0.0], &[f64::INFINITY]),
        Err(ConceptError::InvalidNumericInput)
    );
}
