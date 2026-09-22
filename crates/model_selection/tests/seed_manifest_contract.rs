//! Fitted model-selection seeds must already satisfy the reference owner domain.

use model_selection::{FittedCandidateKConfig, ModelSelectionError};

#[test]
fn fitted_selection_seed_manifest_is_nonzero_unique_and_order_preserving() {
    let valid = FittedCandidateKConfig::new(vec![2, 3], vec![11, 1], 20, 1e-5)
        .expect("nonzero unique seed manifest");
    assert_eq!(valid.seeds(), &[11, 1]);

    assert_eq!(
        FittedCandidateKConfig::new(vec![2], vec![0, 11], 20, 1e-5),
        Err(ModelSelectionError::InvalidDiagnostic)
    );
    assert_eq!(
        FittedCandidateKConfig::new(vec![2], vec![7, 11, 7], 20, 1e-5),
        Err(ModelSelectionError::InvalidDiagnostic)
    );
}
