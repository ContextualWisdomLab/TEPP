#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Leakage-safe temporal monitoring for governed rater parameters.
//!
//! This crate owns monitoring-run and monitoring-artifact lifecycle. It keeps
//! invocation noise, gradual drift, configuration change, and measurement
//! invariance as distinct domain concepts. It does not generate observations,
//! estimate rater parameters, rewrite published scores, or apply product
//! decisions.

use std::error::Error;
use std::fmt::{Display, Formatter};

/// Fail-closed errors returned by rater-monitoring aggregates.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum MonitoringError {
    /// An opaque identity or evidence reference was not exact and safe.
    InvalidReference,
    /// An input artifact became available after the monitoring knowledge cutoff.
    EvidenceAfterKnowledgeCutoff,
    /// The same parameter snapshot was included more than once.
    DuplicateParameterSnapshot,
    /// A monitoring run cannot be sealed without parameter observations.
    EmptyMonitoringRun,
    /// A monitoring run lifecycle transition is invalid.
    InvalidRunTransition,
    /// A monitoring artifact requires one or more source snapshots.
    EmptyArtifactEvidence,
    /// A monitoring artifact repeats a source snapshot.
    DuplicateArtifactEvidence,
    /// A monitoring artifact references a snapshot outside its sealed run.
    ArtifactEvidenceOutsideRun,
}

impl Display for MonitoringError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::InvalidReference => "monitoring references must be exact opaque values",
            Self::EvidenceAfterKnowledgeCutoff => {
                "monitoring evidence must be available by the knowledge cutoff"
            }
            Self::DuplicateParameterSnapshot => {
                "parameter snapshot references must be unique within a monitoring run"
            }
            Self::EmptyMonitoringRun => {
                "a monitoring run requires at least one parameter observation"
            }
            Self::InvalidRunTransition => "the requested monitoring-run transition is invalid",
            Self::EmptyArtifactEvidence => {
                "a monitoring artifact requires at least one source snapshot"
            }
            Self::DuplicateArtifactEvidence => {
                "monitoring artifact source snapshot references must be unique"
            }
            Self::ArtifactEvidenceOutsideRun => {
                "monitoring artifact evidence must belong to the sealed source run"
            }
        })
    }
}

impl Error for MonitoringError {}

fn exact_reference(reference: &str) -> Result<String, MonitoringError> {
    let normalized = reference.trim();
    let numeric_like = normalized.chars().any(char::is_numeric)
        && normalized.chars().all(|character| {
            character.is_numeric()
                || matches!(character, '+' | '-' | '.' | ',' | 'e' | 'E')
        });
    if normalized.is_empty()
        || normalized != reference
        || numeric_like
        || normalized.chars().any(char::is_control)
    {
        return Err(MonitoringError::InvalidReference);
    }
    Ok(reference.to_owned())
}

/// One immutable observation of a versioned rater-parameter snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RaterParameterObservation {
    rater_configuration_ref: String,
    parameter_snapshot_ref: String,
    effective_at_unix_ms: i64,
    available_at_unix_ms: i64,
    recorded_at_unix_ms: i64,
}

impl RaterParameterObservation {
    /// Create one temporal parameter observation without collapsing its clocks.
    ///
    /// `effective_at_unix_ms` describes when the parameter applied,
    /// `available_at_unix_ms` describes when an analysis could use it, and
    /// `recorded_at_unix_ms` describes when the monitoring system recorded it.
    /// No total ordering between those clocks is manufactured here.
    ///
    /// # Errors
    ///
    /// Returns [`MonitoringError::InvalidReference`] when either identity is not
    /// an exact opaque reference.
    pub fn new(
        rater_configuration_ref: &str,
        parameter_snapshot_ref: &str,
        effective_at_unix_ms: i64,
        available_at_unix_ms: i64,
        recorded_at_unix_ms: i64,
    ) -> Result<Self, MonitoringError> {
        Ok(Self {
            rater_configuration_ref: exact_reference(rater_configuration_ref)?,
            parameter_snapshot_ref: exact_reference(parameter_snapshot_ref)?,
            effective_at_unix_ms,
            available_at_unix_ms,
            recorded_at_unix_ms,
        })
    }

    /// Return the exact rater-configuration identity.
    #[must_use]
    pub fn rater_configuration_ref(&self) -> &str {
        &self.rater_configuration_ref
    }

    /// Return the immutable numerical parameter-snapshot identity.
    #[must_use]
    pub fn parameter_snapshot_ref(&self) -> &str {
        &self.parameter_snapshot_ref
    }

    /// Return when the parameter state applied in the measured domain.
    #[must_use]
    pub const fn effective_at_unix_ms(&self) -> i64 {
        self.effective_at_unix_ms
    }

    /// Return when the parameter artifact became available to analysis.
    #[must_use]
    pub const fn available_at_unix_ms(&self) -> i64 {
        self.available_at_unix_ms
    }

    /// Return when TEPP recorded the parameter artifact.
    #[must_use]
    pub const fn recorded_at_unix_ms(&self) -> i64 {
        self.recorded_at_unix_ms
    }
}

/// Lifecycle state of one monitoring run.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum MonitoringRunState {
    /// Parameter observations may still be added.
    Draft,
    /// Inputs are frozen and monitoring artifacts may reference the run.
    Sealed,
}

/// Aggregate root for one leakage-safe rater-monitoring run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RaterMonitoringRun {
    run_ref: String,
    knowledge_cutoff_unix_ms: i64,
    state: MonitoringRunState,
    parameter_observations: Vec<RaterParameterObservation>,
}

impl RaterMonitoringRun {
    /// Create an empty draft run with a distinct knowledge cutoff.
    ///
    /// # Errors
    ///
    /// Returns [`MonitoringError::InvalidReference`] when `run_ref` is not an
    /// exact opaque reference.
    pub fn new(run_ref: &str, knowledge_cutoff_unix_ms: i64) -> Result<Self, MonitoringError> {
        Ok(Self {
            run_ref: exact_reference(run_ref)?,
            knowledge_cutoff_unix_ms,
            state: MonitoringRunState::Draft,
            parameter_observations: Vec::new(),
        })
    }

    /// Add one parameter observation that was available by the cutoff.
    ///
    /// # Errors
    ///
    /// Returns [`MonitoringError::InvalidRunTransition`] after sealing,
    /// [`MonitoringError::EvidenceAfterKnowledgeCutoff`] when the artifact was
    /// unavailable at the analysis cutoff, or
    /// [`MonitoringError::DuplicateParameterSnapshot`] for a repeated snapshot.
    pub fn add_parameter_observation(
        &mut self,
        observation: RaterParameterObservation,
    ) -> Result<(), MonitoringError> {
        if self.state != MonitoringRunState::Draft {
            return Err(MonitoringError::InvalidRunTransition);
        }
        if observation.available_at_unix_ms > self.knowledge_cutoff_unix_ms {
            return Err(MonitoringError::EvidenceAfterKnowledgeCutoff);
        }
        if self
            .parameter_observations
            .iter()
            .any(|existing| existing.parameter_snapshot_ref == observation.parameter_snapshot_ref)
        {
            return Err(MonitoringError::DuplicateParameterSnapshot);
        }
        self.parameter_observations.push(observation);
        Ok(())
    }

    /// Freeze the input set for reproducible monitoring.
    ///
    /// # Errors
    ///
    /// Returns [`MonitoringError::InvalidRunTransition`] unless the run is
    /// draft, or [`MonitoringError::EmptyMonitoringRun`] when it has no input.
    pub fn seal(&mut self) -> Result<(), MonitoringError> {
        if self.state != MonitoringRunState::Draft {
            return Err(MonitoringError::InvalidRunTransition);
        }
        if self.parameter_observations.is_empty() {
            return Err(MonitoringError::EmptyMonitoringRun);
        }
        self.state = MonitoringRunState::Sealed;
        Ok(())
    }

    /// Return the monitoring-run identity.
    #[must_use]
    pub fn run_ref(&self) -> &str {
        &self.run_ref
    }

    /// Return the latest availability time admitted by this analysis.
    #[must_use]
    pub const fn knowledge_cutoff_unix_ms(&self) -> i64 {
        self.knowledge_cutoff_unix_ms
    }

    /// Return the monitoring-run lifecycle state.
    #[must_use]
    pub const fn state(&self) -> MonitoringRunState {
        self.state
    }

    /// Return frozen parameter observations in insertion order.
    #[must_use]
    pub fn parameter_observations(&self) -> &[RaterParameterObservation] {
        &self.parameter_observations
    }
}

/// Mutually distinct phenomenon represented by one monitoring artifact.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum MonitoringArtifactKind {
    /// Repeated-invocation variation within one exact rater configuration.
    InvocationNoise,
    /// Continuous parameter movement without asserting a configuration change.
    GradualDrift,
    /// Discontinuous change associated with a new configuration identity.
    ConfigurationChange,
    /// Evidence about comparability of the measurement structure over time.
    MeasurementInvariance,
}

/// Immutable artifact derived from one sealed monitoring run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RaterMonitoringArtifact {
    artifact_ref: String,
    source_run_ref: String,
    kind: MonitoringArtifactKind,
    source_snapshot_refs: Vec<String>,
    conclusion_ref: String,
}

impl RaterMonitoringArtifact {
    /// Create a typed artifact over evidence contained by one sealed run.
    ///
    /// # Errors
    ///
    /// Returns [`MonitoringError::InvalidRunTransition`] for a draft run,
    /// [`MonitoringError::EmptyArtifactEvidence`] for an empty source set,
    /// [`MonitoringError::DuplicateArtifactEvidence`] for repeated sources,
    /// [`MonitoringError::ArtifactEvidenceOutsideRun`] for evidence absent from
    /// the run, or [`MonitoringError::InvalidReference`] for an unsafe
    /// artifact, source, or conclusion reference.
    pub fn new(
        artifact_ref: &str,
        source_run: &RaterMonitoringRun,
        kind: MonitoringArtifactKind,
        source_snapshot_refs: &[&str],
        conclusion_ref: &str,
    ) -> Result<Self, MonitoringError> {
        if source_run.state != MonitoringRunState::Sealed {
            return Err(MonitoringError::InvalidRunTransition);
        }
        if source_snapshot_refs.is_empty() {
            return Err(MonitoringError::EmptyArtifactEvidence);
        }
        let mut accepted = Vec::with_capacity(source_snapshot_refs.len());
        for source_ref in source_snapshot_refs {
            let source_ref = exact_reference(source_ref)?;
            if accepted.iter().any(|existing| existing == &source_ref) {
                return Err(MonitoringError::DuplicateArtifactEvidence);
            }
            if !source_run
                .parameter_observations
                .iter()
                .any(|observation| observation.parameter_snapshot_ref == source_ref)
            {
                return Err(MonitoringError::ArtifactEvidenceOutsideRun);
            }
            accepted.push(source_ref);
        }
        Ok(Self {
            artifact_ref: exact_reference(artifact_ref)?,
            source_run_ref: source_run.run_ref.clone(),
            kind,
            source_snapshot_refs: accepted,
            conclusion_ref: exact_reference(conclusion_ref)?,
        })
    }

    /// Return the monitoring-artifact identity.
    #[must_use]
    pub fn artifact_ref(&self) -> &str {
        &self.artifact_ref
    }

    /// Return the sealed source-run identity.
    #[must_use]
    pub fn source_run_ref(&self) -> &str {
        &self.source_run_ref
    }

    /// Return the phenomenon this artifact estimates.
    #[must_use]
    pub const fn kind(&self) -> MonitoringArtifactKind {
        self.kind
    }

    /// Return the immutable source parameter-snapshot identities.
    #[must_use]
    pub fn source_snapshot_refs(&self) -> &[String] {
        &self.source_snapshot_refs
    }

    /// Return the versioned conclusion identity.
    #[must_use]
    pub fn conclusion_ref(&self) -> &str {
        &self.conclusion_ref
    }
}

#[cfg(test)]
mod tests {
    use super::{
        MonitoringArtifactKind, MonitoringError, MonitoringRunState,
        RaterMonitoringArtifact, RaterMonitoringRun, RaterParameterObservation,
    };

    fn observation(
        configuration_ref: &str,
        snapshot_ref: &str,
        effective_at_unix_ms: i64,
        available_at_unix_ms: i64,
    ) -> RaterParameterObservation {
        RaterParameterObservation::new(
            configuration_ref,
            snapshot_ref,
            effective_at_unix_ms,
            available_at_unix_ms,
            available_at_unix_ms + 1,
        )
        .expect("valid parameter observation")
    }

    fn sealed_run() -> RaterMonitoringRun {
        let mut run = RaterMonitoringRun::new("monitoring_run_alpha", 1_000)
            .expect("valid monitoring run");
        run.add_parameter_observation(observation(
            "configuration_alpha",
            "snapshot_alpha",
            100,
            500,
        ))
        .expect("first observation");
        run.add_parameter_observation(observation(
            "configuration_beta",
            "snapshot_beta",
            800,
            900,
        ))
        .expect("second observation");
        run.seal().expect("seal monitoring run");
        run
    }

    #[test]
    fn parameter_observation_preserves_distinct_clocks() {
        let observation = RaterParameterObservation::new(
            "configuration_alpha",
            "snapshot_alpha",
            300,
            200,
            100,
        )
        .expect("valid observation");
        assert_eq!(
            observation.rater_configuration_ref(),
            "configuration_alpha"
        );
        assert_eq!(observation.parameter_snapshot_ref(), "snapshot_alpha");
        assert_eq!(observation.effective_at_unix_ms(), 300);
        assert_eq!(observation.available_at_unix_ms(), 200);
        assert_eq!(observation.recorded_at_unix_ms(), 100);
        assert_eq!(
            RaterParameterObservation::new(" configuration ", "snapshot", 0, 0, 0),
            Err(MonitoringError::InvalidReference)
        );
    }

    #[test]
    fn monitoring_run_enforces_cutoff_uniqueness_and_sealing() {
        let mut run =
            RaterMonitoringRun::new("monitoring_run", 500).expect("valid monitoring run");
        assert_eq!(run.run_ref(), "monitoring_run");
        assert_eq!(run.knowledge_cutoff_unix_ms(), 500);
        assert_eq!(run.state(), MonitoringRunState::Draft);
        assert_eq!(run.seal(), Err(MonitoringError::EmptyMonitoringRun));

        let admissible = observation("configuration", "snapshot", 100, 500);
        run.add_parameter_observation(admissible.clone())
            .expect("admissible evidence");
        assert_eq!(
            run.add_parameter_observation(admissible),
            Err(MonitoringError::DuplicateParameterSnapshot)
        );
        assert_eq!(
            run.add_parameter_observation(observation(
                "configuration_late",
                "snapshot_late",
                100,
                501,
            )),
            Err(MonitoringError::EvidenceAfterKnowledgeCutoff)
        );
        assert_eq!(run.parameter_observations().len(), 1);
        run.seal().expect("seal run");
        assert_eq!(run.state(), MonitoringRunState::Sealed);
        assert_eq!(run.seal(), Err(MonitoringError::InvalidRunTransition));
        assert_eq!(
            run.add_parameter_observation(observation(
                "configuration_other",
                "snapshot_other",
                100,
                400,
            )),
            Err(MonitoringError::InvalidRunTransition)
        );
    }

    #[test]
    fn typed_monitoring_artifacts_do_not_collapse_distinct_phenomena() {
        let run = sealed_run();
        let kinds = [
            MonitoringArtifactKind::InvocationNoise,
            MonitoringArtifactKind::GradualDrift,
            MonitoringArtifactKind::ConfigurationChange,
            MonitoringArtifactKind::MeasurementInvariance,
        ];
        for (index, kind) in kinds.into_iter().enumerate() {
            let artifact = RaterMonitoringArtifact::new(
                &format!("artifact_{index}"),
                &run,
                kind,
                &["snapshot_alpha", "snapshot_beta"],
                "conclusion_reviewed",
            )
            .expect("valid monitoring artifact");
            assert_eq!(artifact.source_run_ref(), "monitoring_run_alpha");
            assert_eq!(artifact.kind(), kind);
            assert_eq!(
                artifact.source_snapshot_refs(),
                ["snapshot_alpha", "snapshot_beta"]
            );
            assert_eq!(artifact.conclusion_ref(), "conclusion_reviewed");
            assert!(!artifact.artifact_ref().is_empty());
        }
    }

    #[test]
    fn artifact_requires_a_sealed_run_and_bounded_in_run_evidence() {
        let draft = RaterMonitoringRun::new("draft_run", 1_000).expect("valid draft");
        assert_eq!(
            RaterMonitoringArtifact::new(
                "artifact",
                &draft,
                MonitoringArtifactKind::GradualDrift,
                &["snapshot"],
                "conclusion",
            ),
            Err(MonitoringError::InvalidRunTransition)
        );

        let run = sealed_run();
        assert_eq!(
            RaterMonitoringArtifact::new(
                "artifact",
                &run,
                MonitoringArtifactKind::GradualDrift,
                &[],
                "conclusion",
            ),
            Err(MonitoringError::EmptyArtifactEvidence)
        );
        assert_eq!(
            RaterMonitoringArtifact::new(
                "artifact",
                &run,
                MonitoringArtifactKind::GradualDrift,
                &["snapshot_alpha", "snapshot_alpha"],
                "conclusion",
            ),
            Err(MonitoringError::DuplicateArtifactEvidence)
        );
        assert_eq!(
            RaterMonitoringArtifact::new(
                "artifact",
                &run,
                MonitoringArtifactKind::GradualDrift,
                &["snapshot_outside"],
                "conclusion",
            ),
            Err(MonitoringError::ArtifactEvidenceOutsideRun)
        );
        assert_eq!(
            RaterMonitoringArtifact::new(
                " artifact ",
                &run,
                MonitoringArtifactKind::GradualDrift,
                &["snapshot_alpha"],
                "conclusion",
            ),
            Err(MonitoringError::InvalidReference)
        );
        assert_eq!(
            RaterMonitoringArtifact::new(
                "artifact",
                &run,
                MonitoringArtifactKind::GradualDrift,
                &[" snapshot_alpha "],
                "conclusion",
            ),
            Err(MonitoringError::InvalidReference)
        );
        assert_eq!(
            RaterMonitoringArtifact::new(
                "artifact",
                &run,
                MonitoringArtifactKind::GradualDrift,
                &["snapshot_alpha"],
                " conclusion ",
            ),
            Err(MonitoringError::InvalidReference)
        );
    }

    #[test]
    fn invalid_references_and_error_messages_fail_closed() {
        assert_eq!(
            RaterMonitoringRun::new("123", 0),
            Err(MonitoringError::InvalidReference)
        );
        assert_eq!(
            RaterMonitoringRun::new("bad\nreference", 0),
            Err(MonitoringError::InvalidReference)
        );
        let errors = [
            MonitoringError::InvalidReference,
            MonitoringError::EvidenceAfterKnowledgeCutoff,
            MonitoringError::DuplicateParameterSnapshot,
            MonitoringError::EmptyMonitoringRun,
            MonitoringError::InvalidRunTransition,
            MonitoringError::EmptyArtifactEvidence,
            MonitoringError::DuplicateArtifactEvidence,
            MonitoringError::ArtifactEvidenceOutsideRun,
        ];
        for error in errors {
            assert!(!error.to_string().is_empty());
        }
    }
}
