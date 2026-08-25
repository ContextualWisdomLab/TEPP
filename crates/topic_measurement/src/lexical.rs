//! Refusal of lexical heuristics as inferential topic coordinates.

use crate::error::TopicMeasurementError;

/// Refuse TF-IDF, BM25, and keyword scores as topic-estimator coordinates.
///
/// ADR 0012 forbids treating lexical retrieval weights as inferential topic
/// coordinates. A recognized statistical method name is accepted so callers
/// can share one vocabulary gate.
///
/// # Errors
///
/// Returns [`TopicMeasurementError::LexicalWeightForbidden`] for empty labels
/// and for `tfidf`, `bm25`, and `keyword` after alphanumeric folding.
pub fn refuse_lexical_inferential_weight(method: &str) -> Result<(), TopicMeasurementError> {
    let folded: String = method
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .flat_map(char::to_lowercase)
        .collect();
    if folded.is_empty() || matches!(folded.as_str(), "tfidf" | "bm25" | "keyword") {
        return Err(TopicMeasurementError::LexicalWeightForbidden);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::refuse_lexical_inferential_weight;

    #[test]
    fn statistical_method_names_are_allowed() {
        refuse_lexical_inferential_weight("tepp_topic_measurement").expect("allowed");
        refuse_lexical_inferential_weight("logistic_normal").expect("allowed");
    }
}
