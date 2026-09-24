//! In-sample Schwarz criteria must not masquerade as held-out likelihood.

use model_selection::ModelCandidate;

#[test]
fn statistical_candidate_exposes_its_schwarz_selection_score() {
    let candidate = ModelCandidate::statistical(4, -12.5, 8.0).expect("statistical candidate");
    assert_eq!(candidate.schwarz_score(), Some(-12.5));
    assert!(candidate.is_statistically_supported());
}
