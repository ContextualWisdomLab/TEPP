//! Contract tests for standardised manifest-variance boundary error messages.

use psychometric_core::PsychometricError;

#[test]
fn standardised_manifest_variance_boundary_messages_are_stable() {
    assert_eq!(
        PsychometricError::StandardisedManifestVarianceRequiresPositiveManifestVariance.to_string(),
        "standardised measurement-error variance requires strictly positive measurement-error variance"
    );
    assert_eq!(
        PsychometricError::UnstandardisedManifestVarianceIsNotStandardisedManifestVariance
            .to_string(),
        "unstandardised measurement-error variance is not standardised measurement-error variance"
    );
    assert_eq!(
        PsychometricError::StandardisedManifestTraitVarianceIsNotStandardisedManifestVariance
            .to_string(),
        "standardised manifest-trait variance is not standardised measurement-error variance"
    );
    assert_eq!(
        PsychometricError::ObservedVarianceIsNotStandardisedManifestVariance.to_string(),
        "observed-indicator variance is not standardised measurement-error variance"
    );
}
