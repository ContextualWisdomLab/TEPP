//! Versioned provenance carrier for empirical interval-coverage evidence.

use crate::ValidationError;
use crate::coverage::{interval_covered_count, wilson_coverage_interval_from_counts};
use serde::{Deserialize, Serialize};

const SCHEMA: &str = "tepp.wilson_coverage_evidence.v1";
const CRITICAL_VALUE_KIND: &str = "standard_normal_z";
const INTERVAL_SIDEDNESS: &str = "two_sided";

/// Durable Wilson interval-coverage evidence with denominator and critical-value provenance.
///
/// The carrier stores the retained sample denominator and covered count rather than only the
/// projected empirical proportion, plus the caller-supplied standard-normal critical value used
/// by TEPP's canonical two-sided Wilson producer. Serialization fixes the schema identifier,
/// critical-value scale, and interval sidedness so a numeric `z` cannot later be reinterpreted as
/// a Student-t value or a one-sided confidence claim. Validation recomputes the empirical coverage
/// and both Wilson endpoints from the stored counts and `z`; tampered or internally inconsistent
/// artifacts fail closed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WilsonCoverageEvidenceV1 {
    /// Number of interval/truth triples admitted to the empirical coverage calculation.
    pub sample_count: usize,
    /// Number of admitted triples whose closed interval contains the corresponding truth value.
    pub covered_count: usize,
    /// Caller-supplied standard-normal critical value used by the Wilson score producer.
    pub normal_critical_value: f64,
    /// Empirical coverage projected from `covered_count / sample_count`.
    pub empirical_coverage: f64,
    /// Canonical Wilson score lower endpoint.
    pub wilson_lower: f64,
    /// Canonical Wilson score upper endpoint.
    pub wilson_upper: f64,
}

impl WilsonCoverageEvidenceV1 {
    /// Build versioned coverage evidence from the same interval triples used by the Wilson producer.
    ///
    /// # Errors
    ///
    /// Returns [`ValidationError::InvalidInput`] for empty, mismatched, non-finite, or inverted
    /// interval triples, and [`ValidationError::InvalidConfiguration`] for a non-positive,
    /// non-finite, or square-overflowing normal critical value.
    pub fn from_intervals(
        truth: &[f64],
        lower: &[f64],
        upper: &[f64],
        normal_critical_value: f64,
    ) -> Result<Self, ValidationError> {
        if !normal_critical_value.is_finite() || normal_critical_value <= 0.0 {
            return Err(ValidationError::InvalidConfiguration);
        }
        let covered_count = interval_covered_count(truth, lower, upper)?;
        let sample_count = truth.len();
        let (wilson_lower, wilson_upper) = wilson_coverage_interval_from_counts(
            covered_count,
            sample_count,
            normal_critical_value,
        )?;
        let evidence = Self {
            sample_count,
            covered_count,
            normal_critical_value,
            empirical_coverage: covered_count as f64 / sample_count as f64,
            wilson_lower,
            wilson_upper,
        };
        evidence.validate()?;
        Ok(evidence)
    }

    /// Validate denominator, standard-normal critical-value, and exact recomputation coherence.
    ///
    /// Numeric equality intentionally treats IEEE `-0.0` and `+0.0` as one zero-valued
    /// scientific state. The JSON representation produced by this crate round-trips binary64
    /// values exactly, so canonical evidence must reproduce the stored coverage and endpoints
    /// rather than merely fall within a loose interval-pair tolerance.
    ///
    /// # Errors
    ///
    /// Returns [`ValidationError::InvalidInput`] when counts, numeric domains, or recomputed
    /// empirical/Wilson values disagree with the stored artifact.
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.sample_count == 0 || self.covered_count > self.sample_count {
            return Err(ValidationError::InvalidInput);
        }
        if !self.normal_critical_value.is_finite() || self.normal_critical_value <= 0.0 {
            return Err(ValidationError::InvalidInput);
        }
        for value in [
            self.empirical_coverage,
            self.wilson_lower,
            self.wilson_upper,
        ] {
            if !value.is_finite() || !(0.0..=1.0).contains(&value) {
                return Err(ValidationError::InvalidInput);
            }
        }

        let expected_coverage = self.covered_count as f64 / self.sample_count as f64;
        let (expected_lower, expected_upper) = wilson_coverage_interval_from_counts(
            self.covered_count,
            self.sample_count,
            self.normal_critical_value,
        )
        .map_err(|_| ValidationError::InvalidInput)?;

        if self.empirical_coverage != expected_coverage
            || self.wilson_lower != expected_lower
            || self.wilson_upper != expected_upper
        {
            return Err(ValidationError::InvalidInput);
        }
        Ok(())
    }

    /// Serialize the validated carrier to canonical JSON.
    ///
    /// # Errors
    ///
    /// Returns [`ValidationError::InvalidInput`] when the artifact is inconsistent or JSON
    /// serialization fails.
    pub fn to_json(&self) -> Result<String, ValidationError> {
        self.validate()?;
        serde_json::to_string(self).map_err(|_| ValidationError::InvalidInput)
    }
}

impl Serialize for WilsonCoverageEvidenceV1 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        self.validate().map_err(serde::ser::Error::custom)?;
        let mut state = serializer.serialize_struct("WilsonCoverageEvidenceV1", 9)?;
        state.serialize_field("schema", SCHEMA)?;
        state.serialize_field("sample_count", &self.sample_count)?;
        state.serialize_field("covered_count", &self.covered_count)?;
        state.serialize_field("critical_value_kind", CRITICAL_VALUE_KIND)?;
        state.serialize_field("interval_sidedness", INTERVAL_SIDEDNESS)?;
        state.serialize_field("normal_critical_value", &self.normal_critical_value)?;
        state.serialize_field("empirical_coverage", &self.empirical_coverage)?;
        state.serialize_field("wilson_lower", &self.wilson_lower)?;
        state.serialize_field("wilson_upper", &self.wilson_upper)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for WilsonCoverageEvidenceV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Raw {
            schema: String,
            sample_count: usize,
            covered_count: usize,
            critical_value_kind: String,
            interval_sidedness: String,
            normal_critical_value: f64,
            empirical_coverage: f64,
            wilson_lower: f64,
            wilson_upper: f64,
        }

        let raw = Raw::deserialize(deserializer)?;
        if raw.schema != SCHEMA
            || raw.critical_value_kind != CRITICAL_VALUE_KIND
            || raw.interval_sidedness != INTERVAL_SIDEDNESS
        {
            return Err(serde::de::Error::custom("unsupported Wilson coverage evidence schema"));
        }
        let evidence = Self {
            sample_count: raw.sample_count,
            covered_count: raw.covered_count,
            normal_critical_value: raw.normal_critical_value,
            empirical_coverage: raw.empirical_coverage,
            wilson_lower: raw.wilson_lower,
            wilson_upper: raw.wilson_upper,
        };
        evidence.validate().map_err(serde::de::Error::custom)?;
        Ok(evidence)
    }
}
