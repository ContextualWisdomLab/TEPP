//! Regression coverage for public psychometric manifest-variance error messages.

use psychometric_core::PsychometricError;

#[test]
fn standardised_manifest_variance_error_messages_are_stable() {
    let cases = [
        (
            PsychometricError::StandardisedManifestVarianceRequiresPositiveManifestVariance,
            "standardised measurement-error variance requires strictly positive measurement-error variance",
        ),
        (
            PsychometricError::UnstandardisedManifestVarianceIsNotStandardisedManifestVariance,
            "unstandardised measurement-error variance is not standardised measurement-error variance",
        ),
        (
            PsychometricError::StandardisedManifestTraitVarianceIsNotStandardisedManifestVariance,
            "standardised manifest-trait variance is not standardised measurement-error variance",
        ),
        (
            PsychometricError::ObservedVarianceIsNotStandardisedManifestVariance,
            "observed-indicator variance is not standardised measurement-error variance",
        ),
    ];

    for (error, expected) in cases {
        assert_eq!(error.to_string(), expected);
    }
}
