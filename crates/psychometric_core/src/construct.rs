//! Construct-class classification and interpretation gates.

use crate::error::PsychometricError;
use crate::latent_mean::{MeanInvarianceStatus, TwoGroupMeasurement};

/// Higher-order construct class before ESEM, composite, or network modeling.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ConstructClass {
    /// Reflective indicators of a common latent factor.
    Reflective,
    /// Formative or composite indicators that define the construct.
    Formative,
    /// Interacting indicators that belong in a network model.
    Network,
    /// Insufficient evidence to classify the construct.
    Unresolved,
}

impl ConstructClass {
    /// Stable wire name for the construct class.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Reflective => "reflective",
            Self::Formative => "formative",
            Self::Network => "network",
            Self::Unresolved => "unresolved",
        }
    }

    /// Return whether reflective ESEM/set-ESEM is admissible.
    #[must_use]
    pub const fn admits_reflective_esem(self) -> bool {
        matches!(self, Self::Reflective)
    }
}

/// Interpret a classified construct as reflective.
///
/// A good global fit statistic is not authority to reinterpret a formative or
/// network structure as reflective (ADR 0005).
///
/// # Errors
///
/// Returns [`PsychometricError::FormativeReinterpretationForbidden`] for
/// formative or network classes and
/// [`PsychometricError::UnresolvedConstruct`] when the class is unresolved.
pub fn interpret_as_reflective(
    classified: ConstructClass,
    global_fit_acceptable: bool,
) -> Result<ConstructClass, PsychometricError> {
    match (classified, global_fit_acceptable) {
        (ConstructClass::Reflective, true | false) => Ok(ConstructClass::Reflective),
        (ConstructClass::Unresolved, true | false) => Err(PsychometricError::UnresolvedConstruct),
        (ConstructClass::Formative | ConstructClass::Network, true | false) => {
            Err(PsychometricError::FormativeReinterpretationForbidden)
        }
    }
}

/// Typed evidence required before a latent-mean or path comparison.
///
/// The evidence replaces the former bare-boolean gate: an invariance
/// classification cannot be collapsed into a passing flag because the
/// [`MeanInvarianceStatus`] travels inside the evidence and
/// [`compare_latent_means`] re-inspects it against the strong/strict
/// requirement. Metric evidence still licenses shared metric meaning only.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LatentMeanComparisonEvidence {
    /// Classified measurement-invariance status backing the comparison.
    pub status: MeanInvarianceStatus,
    /// Description of what is being compared (constructs, groups, paths).
    pub comparison_scope: String,
    /// Version label of the fitted model that produced `status`.
    pub model_version: String,
}

impl LatentMeanComparisonEvidence {
    /// Build typed evidence from a two-group OLS classification result.
    ///
    /// Any classified status is accepted here so the evidence records what
    /// was actually measured; [`compare_latent_means`] fails closed on
    /// insufficient levels.
    ///
    /// # Errors
    ///
    /// Returns [`PsychometricError::MalformedInvarianceEvidence`] when either
    /// label is empty.
    pub fn from_two_group_measurement(
        measurement: &TwoGroupMeasurement,
        comparison_scope: &str,
        model_version: &str,
    ) -> Result<Self, PsychometricError> {
        if comparison_scope.is_empty() || model_version.is_empty() {
            return Err(PsychometricError::MalformedInvarianceEvidence);
        }
        Ok(Self {
            status: measurement.status,
            comparison_scope: String::from(comparison_scope),
            model_version: String::from(model_version),
        })
    }
}

/// Permit a latent-mean or path comparison only when typed invariance
/// evidence carries strong or strict status.
///
/// Configural and metric evidence fail closed; a good global fit or shared
/// metric meaning is not authority to compare latent means.
///
/// # Errors
///
/// Returns [`PsychometricError::StrongInvarianceRequired`] when the carried
/// status does not license latent-mean comparison.
pub fn compare_latent_means(
    evidence: &LatentMeanComparisonEvidence,
) -> Result<(), PsychometricError> {
    if evidence.status.licenses_latent_mean_comparison() {
        Ok(())
    } else {
        Err(PsychometricError::StrongInvarianceRequired)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ConstructClass, LatentMeanComparisonEvidence, compare_latent_means, interpret_as_reflective,
    };
    use crate::error::PsychometricError;
    use crate::indicator::IndicatorKind;
    use crate::latent_mean::{
        GroupIndicatorSeries, MeanInvarianceStatus, TwoGroupMeasurement,
        classify_two_group_ols_invariance,
    };

    fn series(factors: &[f64], intercept: f64, loading: f64) -> GroupIndicatorSeries {
        GroupIndicatorSeries {
            factor_scores: factors.to_vec(),
            indicators: factors
                .iter()
                .map(|score| intercept + loading * score)
                .collect(),
        }
    }

    fn classify(
        reference: &GroupIndicatorSeries,
        comparison: &GroupIndicatorSeries,
    ) -> TwoGroupMeasurement {
        classify_two_group_ols_invariance(
            reference,
            comparison,
            IndicatorKind::AdditiveLogRatio,
            1e-9,
            1e-9,
            1e-9,
        )
        .expect("classification succeeds on finite three-point series")
    }

    fn evidence_for(status: MeanInvarianceStatus) -> LatentMeanComparisonEvidence {
        LatentMeanComparisonEvidence {
            status,
            comparison_scope: String::from("construct mean across two groups"),
            model_version: String::from("construct-tests-v1"),
        }
    }

    #[test]
    fn reflective_only_admits_esem_and_strong_evidence_licenses_means() {
        assert!(ConstructClass::Reflective.admits_reflective_esem());
        assert!(!ConstructClass::Formative.admits_reflective_esem());
        compare_latent_means(&evidence_for(MeanInvarianceStatus::Strong)).expect("strong");
        assert_eq!(
            interpret_as_reflective(ConstructClass::Reflective, true).expect("fit unused"),
            ConstructClass::Reflective
        );
        assert_eq!(
            interpret_as_reflective(ConstructClass::Network, false),
            Err(PsychometricError::FormativeReinterpretationForbidden)
        );
    }

    #[test]
    fn strict_evidence_and_classified_strong_evidence_pass_the_gate() {
        compare_latent_means(&evidence_for(MeanInvarianceStatus::Strict)).expect("strict");
        let reference = series(&[-1.0, 0.0, 1.0], 0.5, 1.2);
        let comparison = series(&[1.0, 2.0, 3.0], 0.5, 1.2);
        let measurement = classify(&reference, &comparison);
        assert_eq!(measurement.status, MeanInvarianceStatus::Strict);
        let evidence = LatentMeanComparisonEvidence::from_two_group_measurement(
            &measurement,
            "two-group OLS latent means",
            "construct-tests-v1",
        )
        .expect("non-empty labels");
        assert_eq!(evidence.status, MeanInvarianceStatus::Strict);
        assert_eq!(
            evidence.comparison_scope,
            String::from("two-group OLS latent means")
        );
        assert_eq!(evidence.model_version, String::from("construct-tests-v1"));
        compare_latent_means(&evidence).expect("classified strict licenses means");
    }

    #[test]
    fn metric_status_evidence_cannot_reduce_to_a_passing_flag() {
        // Hand-assembled metric evidence still fails: no boolean input can
        // bypass the carried status.
        let hand_built = evidence_for(MeanInvarianceStatus::Metric);
        assert_eq!(
            compare_latent_means(&hand_built),
            Err(PsychometricError::StrongInvarianceRequired)
        );

        let reference = series(&[-1.0, 0.0, 1.0], 0.5, 1.2);
        let metric_only = series(&[1.0, 2.0, 3.0], 1.5, 1.2);
        let measurement = classify(&reference, &metric_only);
        assert_eq!(measurement.status, MeanInvarianceStatus::Metric);
        assert!(measurement.status.licenses_shared_metric_meaning());
        let evidence = LatentMeanComparisonEvidence::from_two_group_measurement(
            &measurement,
            "metric-only two-group comparison",
            "construct-tests-v1",
        )
        .expect("non-empty labels");
        assert_eq!(
            compare_latent_means(&evidence),
            Err(PsychometricError::StrongInvarianceRequired)
        );
    }

    #[test]
    fn configural_status_evidence_is_refused() {
        let reference = series(&[-1.0, 0.0, 1.0], 0.5, 1.2);
        let configural = series(&[1.0, 2.0, 3.0], 0.5, 0.4);
        let measurement = classify(&reference, &configural);
        assert_eq!(measurement.status, MeanInvarianceStatus::Configural);
        let evidence = LatentMeanComparisonEvidence::from_two_group_measurement(
            &measurement,
            "configural two-group comparison",
            "construct-tests-v1",
        )
        .expect("non-empty labels");
        assert_eq!(
            compare_latent_means(&evidence),
            Err(PsychometricError::StrongInvarianceRequired)
        );
    }

    #[test]
    fn empty_scope_or_model_version_labels_fail_closed() {
        let reference = series(&[-1.0, 0.0, 1.0], 0.5, 1.2);
        let comparison = series(&[1.0, 2.0, 3.0], 0.5, 1.2);
        let measurement = classify(&reference, &comparison);
        assert_eq!(
            LatentMeanComparisonEvidence::from_two_group_measurement(
                &measurement,
                "",
                "construct-tests-v1",
            ),
            Err(PsychometricError::MalformedInvarianceEvidence)
        );
        assert_eq!(
            LatentMeanComparisonEvidence::from_two_group_measurement(
                &measurement,
                "two-group OLS latent means",
                "",
            ),
            Err(PsychometricError::MalformedInvarianceEvidence)
        );
    }
}
