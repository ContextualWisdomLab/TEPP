//! Realistic Korean/English span identity is independent of language tags.

use evidence_core::{DocumentRecord, SourceArtifact, SourceSpan};
use semantic_core::{LanguageProfile, SemanticError, SemanticIdentity, SemanticUnit};

fn document(text: &str) -> DocumentRecord {
    let artifact = SourceArtifact::from_bytes(b"corpus").expect("artifact");
    DocumentRecord::from_text(artifact.id(), text).expect("document")
}

fn span(document: &DocumentRecord, start: usize, end: usize) -> SourceSpan {
    let text = document.text();
    let scalar_start = text[..start].chars().count();
    let scalar_end = scalar_start + text[start..end].chars().count();
    SourceSpan::new(document, start, end, scalar_start, scalar_end, None).expect("span")
}

#[test]
fn korean_and_english_surfaces_are_distinct_units() {
    let korean_text = "측정 오차는 RMSE로 보고한다.";
    let english_text = "Measurement error is reported as RMSE.";
    let korean_doc = document(korean_text);
    let english_doc = document(english_text);
    let korean = SemanticUnit::bind(
        span(&korean_doc, 0, "측정".len()),
        LanguageProfile::parse_bcp47("ko").expect("ko"),
    );
    let english = SemanticUnit::bind(
        span(&english_doc, 0, "Measurement".len()),
        LanguageProfile::parse_bcp47("en").expect("en"),
    );
    assert_ne!(korean.identity(), english.identity());
    assert_ne!(korean, english);
    assert_eq!(korean.language().as_str(), "ko");
    assert_eq!(english.language().as_str(), "en");
}

#[test]
fn missing_language_does_not_retokenize_or_steal_identity() {
    let text = "측정 오차는 RMSE로 보고한다.";
    let document = document(text);
    let exact = span(&document, 0, "측정".len());
    let unresolved = SemanticUnit::bind(exact, LanguageProfile::unresolved());
    let tagged = unresolved
        .clone()
        .with_language(LanguageProfile::parse_bcp47("ko").expect("ko"));
    assert_eq!(unresolved.identity(), tagged.identity());
    assert_eq!(unresolved.span().byte_start(), tagged.span().byte_start());
    assert_eq!(unresolved.span().byte_end(), tagged.span().byte_end());
    assert_eq!(
        SemanticIdentity::from_language_tag("ko"),
        Err(SemanticError::LanguageIsNotIdentity)
    );
}
