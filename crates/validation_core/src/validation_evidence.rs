//! Versioned durable envelope for validation projections and their scientific provenance.

use crate::{ValidationError, ValidationReport, WilsonCoverageEvidenceV1};
use serde::{Deserialize, Serialize};

const SCHEMA: &str = "tepp.validation_evidence.v1";

/// Durable validation evidence envelope with a report projection and recomputable coverage proof.
///
/// [`ValidationReport`] remains the compact projection used by existing callers. This versioned
/// envelope binds that projection to [`WilsonCoverageEvidenceV1`], which retains the empirical
/// denominator, covered count, standard-normal critical value, and two-sided interval semantics.
/// Admission requires the report's coverage proportion and Wilson endpoints to equal the values
/// recomputed by the nested provenance carrier, preventing a durable artifact from pairing a
/// valid report with evidence produced from a different finite sample or critical value.
#[derive(Clone, Debug, PartialEq)]
pub struct ValidationEvidenceV1 {
    /// Existing compact validation projection.
    pub report: ValidationReport,
    /// Versioned, recomputable Wilson coverage provenance for the report's coverage fields.
    pub coverage: WilsonCoverageEvidenceV1,
}

impl ValidationEvidenceV1 {
    /// Construct a validated durable evidence envelope.
    ///
    /// # Errors
    ///
    /// Returns [`ValidationError::InvalidInput`] when either nested artifact is invalid or the
    /// report's empirical coverage/Wilson projection does not exactly match its provenance.
    pub fn new(
        report: ValidationReport,
        coverage: WilsonCoverageEvidenceV1,
    ) -> Result<Self, ValidationError> {
        let evidence = Self { report, coverage };
        evidence.validate()?;
        Ok(evidence)
    }

    /// Validate nested artifacts and their cross-artifact coverage projection identity.
    ///
    /// # Errors
    ///
    /// Returns [`ValidationError::InvalidInput`] when nested validation fails or the report's
    /// coverage proportion/lower/upper endpoints differ from the recomputed versioned evidence.
    pub fn validate(&self) -> Result<(), ValidationError> {
        self.report.validate()?;
        self.coverage.validate()?;
        if self.report.interval_coverage != self.coverage.empirical_coverage
            || self.report.coverage_wilson_lower != self.coverage.wilson_lower
            || self.report.coverage_wilson_upper != self.coverage.wilson_upper
        {
            return Err(ValidationError::InvalidInput);
        }
        Ok(())
    }

    /// Serialize the validated envelope to canonical JSON.
    ///
    /// # Errors
    ///
    /// Returns [`ValidationError::InvalidInput`] when validation or serialization fails.
    pub fn to_json(&self) -> Result<String, ValidationError> {
        self.validate()?;
        serde_json::to_string(self).map_err(|_| ValidationError::InvalidInput)
    }
}

impl Serialize for ValidationEvidenceV1 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        self.validate().map_err(serde::ser::Error::custom)?;
        let mut state = serializer.serialize_struct("ValidationEvidenceV1", 3)?;
        state.serialize_field("schema", SCHEMA)?;
        state.serialize_field("report", &self.report)?;
        state.serialize_field("coverage", &self.coverage)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for ValidationEvidenceV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Raw {
            schema: String,
            report: ValidationReport,
            coverage: WilsonCoverageEvidenceV1,
        }

        let raw = Raw::deserialize(deserializer)?;
        if raw.schema != SCHEMA {
            return Err(serde::de::Error::custom("unsupported validation evidence schema"));
        }
        Self::new(raw.report, raw.coverage).map_err(serde::de::Error::custom)
    }
}
