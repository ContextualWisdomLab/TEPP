//! Shard-safe execution and owner-bound assembly for the prospective coverage study.
//!
//! Per-replication scientific composition lives in
//! [`crate::execute_coverage_calibration_replication`]. This module adds the
//! application boundary needed to execute declared ordinal ranges without
//! caller-authored seeds, persist shard provenance without losing prospective
//! validation-design/source/scenario identity, and assemble the complete indexed
//! outcome ledger into schema-owned validation evidence.

use std::fmt;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tepp_simulation::{CoverageCalibrationSimulationDesign, SimulationError};
use validation_core::{
    CoverageCalibrationDesign, CoverageCalibrationEvidenceRecord,
    CoverageCalibrationReplicationOutcome, ValidationError, coverage_calibration_design_sha256,
    parse_commit_head,
};

use crate::{CoverageCalibrationExecutionError, execute_coverage_calibration_replication};

const LOWER_COVERAGE_PERCENTILE: f64 = 0.025;
const UPPER_COVERAGE_PERCENTILE: f64 = 0.975;
const COVERAGE_CALIBRATION_SHARD_SCHEMA_VERSION: u32 = 2;
const COVERAGE_CALIBRATION_SHARD_FINGERPRINT_DOMAIN: &[u8] =
    b"tepp.analysis.coverage-calibration-shard.v2\0";

/// Fail-closed error from prospective coverage-study execution or evidence assembly.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoverageCalibrationStudyError {
    /// The requested half-open shard range was empty or outside the declared scenario.
    InvalidShardRange,
    /// The exact source commit identity was not canonical lowercase forty-hex.
    InvalidSourceIdentity,
    /// A persisted shard record disagreed with the declared design/scenario/source/range binding.
    InvalidShardProvenance,
    /// The supplied shard set did not exactly cover the declared replication schedule.
    IncompleteShardSet,
    /// The versioned simulation and validation owners disagree on the declared design binding.
    InvalidDesignBinding,
    /// A structurally invalid replication aborted scientific execution.
    Execution(CoverageCalibrationExecutionError),
    /// The simulation owner could not reconstruct its versioned scenario fingerprint.
    InvalidScenarioBinding,
    /// Indexed outcomes or source identity could not form canonical validation evidence.
    InvalidEvidence,
}

impl fmt::Display for CoverageCalibrationStudyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidShardRange => {
                formatter.write_str("invalid coverage calibration shard range")
            }
            Self::InvalidSourceIdentity => {
                formatter.write_str("invalid coverage calibration source identity")
            }
            Self::InvalidShardProvenance => {
                formatter.write_str("invalid coverage calibration shard provenance")
            }
            Self::IncompleteShardSet => {
                formatter.write_str("incomplete coverage calibration shard set")
            }
            Self::InvalidDesignBinding => formatter
                .write_str("invalid coverage calibration validation/simulation design binding"),
            Self::Execution(error) => error.fmt(formatter),
            Self::InvalidScenarioBinding => {
                formatter.write_str("invalid coverage calibration scenario binding")
            }
            Self::InvalidEvidence => formatter.write_str("invalid coverage calibration evidence"),
        }
    }
}

impl std::error::Error for CoverageCalibrationStudyError {}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CoverageCalibrationShardWire {
    schema_version: u32,
    start_replication_index: usize,
    end_replication_index_exclusive: usize,
    validation_design_id: String,
    validation_design_fingerprint: String,
    simulation_scenario_id: String,
    simulation_scenario_fingerprint: String,
    source_head: String,
    outcomes: Vec<CoverageCalibrationOutcomeWire>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CoverageCalibrationOutcomeWire {
    replication_index: usize,
    window_coverages: Option<Vec<f64>>,
}

/// Immutable application-level provenance record for one executed calibration shard.
///
/// The record binds a half-open ordinal range to the prospective validation design,
/// the simulation-owner scenario, the exact source commit supplied by the execution
/// environment, and the indexed outcomes actually produced for that range. Its
/// digest is an immutable transfer binding, not authentication of external storage,
/// runners, or the Git build.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CoverageCalibrationShardRecord {
    schema_version: u32,
    start_replication_index: usize,
    end_replication_index_exclusive: usize,
    validation_design_id: String,
    validation_design_fingerprint: String,
    simulation_scenario_id: String,
    simulation_scenario_fingerprint: String,
    source_head: String,
    outcomes: Vec<CoverageCalibrationReplicationOutcome>,
}

impl CoverageCalibrationShardRecord {
    /// Version of the persisted shard-record wire contract.
    #[must_use]
    pub const fn schema_version(&self) -> u32 {
        self.schema_version
    }

    /// First zero-based replication ordinal represented by this shard.
    #[must_use]
    pub const fn start_replication_index(&self) -> usize {
        self.start_replication_index
    }

    /// Exclusive upper replication ordinal represented by this shard.
    #[must_use]
    pub const fn end_replication_index_exclusive(&self) -> usize {
        self.end_replication_index_exclusive
    }

    /// Prospective validation-design identity fixed when this shard executed.
    #[must_use]
    pub fn validation_design_id(&self) -> &str {
        &self.validation_design_id
    }

    /// Canonical validation-design fingerprint fixed when this shard executed.
    #[must_use]
    pub fn validation_design_fingerprint(&self) -> &str {
        &self.validation_design_fingerprint
    }

    /// Simulation-owner scenario identity bound to this shard.
    #[must_use]
    pub fn simulation_scenario_id(&self) -> &str {
        &self.simulation_scenario_id
    }

    /// Simulation-owner scenario fingerprint bound to this shard.
    #[must_use]
    pub fn simulation_scenario_fingerprint(&self) -> &str {
        &self.simulation_scenario_fingerprint
    }

    /// Exact lowercase source commit declared by the execution environment.
    #[must_use]
    pub fn source_head(&self) -> &str {
        &self.source_head
    }

    /// Indexed outcomes in the shard's declared ordinal order.
    #[must_use]
    pub fn outcomes(&self) -> &[CoverageCalibrationReplicationOutcome] {
        &self.outcomes
    }

    /// Serialize the deterministic shard record to struct-order JSON.
    ///
    /// # Errors
    ///
    /// Returns [`CoverageCalibrationStudyError::InvalidEvidence`] if serialization
    /// unexpectedly fails.
    pub fn to_json(&self) -> Result<String, CoverageCalibrationStudyError> {
        serde_json::to_string(self).map_err(|_| CoverageCalibrationStudyError::InvalidEvidence)
    }

    /// Rehydrate one persisted shard through the application owner boundary.
    ///
    /// The parser accepts only the current shard schema and reconstructs indexed
    /// outcomes through their public validation-owned constructors. It validates
    /// self-contained provenance/range geometry here; current-design/current-scenario
    /// equality, full-study tiling, and scientific coverage-value validation remain
    /// with final assembly and `validation_core` owners.
    ///
    /// # Errors
    ///
    /// Returns [`CoverageCalibrationStudyError::InvalidShardProvenance`] when the
    /// JSON shape, schema, design/source/fingerprint identity, half-open range, or
    /// exact contiguous outcome identities are not canonical.
    pub fn from_json(json: &str) -> Result<Self, CoverageCalibrationStudyError> {
        let wire: CoverageCalibrationShardWire = serde_json::from_str(json)
            .map_err(|_| CoverageCalibrationStudyError::InvalidShardProvenance)?;
        if wire.schema_version != COVERAGE_CALIBRATION_SHARD_SCHEMA_VERSION
            || wire.validation_design_id.is_empty()
            || wire.simulation_scenario_id.is_empty()
        {
            return Err(CoverageCalibrationStudyError::InvalidShardProvenance);
        }
        require_canonical_source_head(&wire.source_head)
            .map_err(|_| CoverageCalibrationStudyError::InvalidShardProvenance)?;
        require_canonical_sha256_fingerprint(&wire.validation_design_fingerprint)?;
        require_canonical_sha256_fingerprint(&wire.simulation_scenario_fingerprint)?;
        if wire.start_replication_index >= wire.end_replication_index_exclusive {
            return Err(CoverageCalibrationStudyError::InvalidShardProvenance);
        }

        let outcomes: Vec<_> = wire
            .outcomes
            .into_iter()
            .map(|outcome| match outcome.window_coverages {
                Some(window_coverages) => CoverageCalibrationReplicationOutcome::successful(
                    outcome.replication_index,
                    window_coverages,
                ),
                None => CoverageCalibrationReplicationOutcome::numerical_failure(
                    outcome.replication_index,
                ),
            })
            .collect();
        validate_outcome_range(
            wire.start_replication_index,
            wire.end_replication_index_exclusive,
            &outcomes,
        )?;

        Ok(Self {
            schema_version: wire.schema_version,
            start_replication_index: wire.start_replication_index,
            end_replication_index_exclusive: wire.end_replication_index_exclusive,
            validation_design_id: wire.validation_design_id,
            validation_design_fingerprint: wire.validation_design_fingerprint,
            simulation_scenario_id: wire.simulation_scenario_id,
            simulation_scenario_fingerprint: wire.simulation_scenario_fingerprint,
            source_head: wire.source_head,
            outcomes,
        })
    }

    /// Rehydrate one persisted shard and verify its expected owner digest.
    ///
    /// This composes the strict JSON parser with the shard's canonical SHA-256
    /// arithmetic so callers do not have to recreate transfer-integrity checks at
    /// each persistence/resume boundary. The digest remains an application-level
    /// integrity binding; it does not authenticate external storage, runners, or builds.
    ///
    /// # Errors
    ///
    /// Returns [`CoverageCalibrationStudyError::InvalidShardProvenance`] when the
    /// expected digest is not canonical lowercase SHA-256, JSON rehydration fails,
    /// or the recomputed owner digest differs from the expected value.
    pub fn from_json_with_sha256(
        json: &str,
        expected_sha256: &str,
    ) -> Result<Self, CoverageCalibrationStudyError> {
        require_canonical_sha256_fingerprint(expected_sha256)?;
        let record = Self::from_json(json)?;
        if record.sha256()? != expected_sha256 {
            return Err(CoverageCalibrationStudyError::InvalidShardProvenance);
        }
        Ok(record)
    }

    /// Return a domain-separated SHA-256 binding over exact shard provenance/outcomes.
    ///
    /// Coverage values enter as exact IEEE-754 binary64 bits. The digest therefore
    /// remains independent of JSON float formatting while changing for any prospective
    /// design, source, scenario, range, success/failure identity, or coverage-bit change.
    ///
    /// # Errors
    ///
    /// Returns [`CoverageCalibrationStudyError::InvalidShardProvenance`] if a
    /// platform-sized count cannot be represented in the canonical `u64` geometry.
    pub fn sha256(&self) -> Result<String, CoverageCalibrationStudyError> {
        let mut hasher = Sha256::new();
        hasher.update(COVERAGE_CALIBRATION_SHARD_FINGERPRINT_DOMAIN);
        hasher.update(self.schema_version.to_le_bytes());
        update_len_prefixed(&mut hasher, self.validation_design_id.as_bytes())?;
        update_len_prefixed(&mut hasher, self.validation_design_fingerprint.as_bytes())?;
        update_len_prefixed(&mut hasher, self.simulation_scenario_id.as_bytes())?;
        update_len_prefixed(
            &mut hasher,
            self.simulation_scenario_fingerprint.as_bytes(),
        )?;
        update_len_prefixed(&mut hasher, self.source_head.as_bytes())?;
        hasher.update(to_u64(self.start_replication_index)?.to_le_bytes());
        hasher.update(to_u64(self.end_replication_index_exclusive)?.to_le_bytes());
        hasher.update(to_u64(self.outcomes.len())?.to_le_bytes());
        for outcome in &self.outcomes {
            hasher.update(to_u64(outcome.replication_index())?.to_le_bytes());
            match outcome.window_coverages() {
                Some(window_coverages) => {
                    hasher.update([1_u8]);
                    hasher.update(to_u64(window_coverages.len())?.to_le_bytes());
                    for coverage in window_coverages {
                        hasher.update(coverage.to_bits().to_le_bytes());
                    }
                }
                None => {
                    hasher.update([0_u8]);
                    hasher.update(0_u64.to_le_bytes());
                }
            }
        }
        Ok(format!("{:x}", hasher.finalize()))
    }
}

/// Execute one half-open range of prospectively declared DGP replication ordinals.
///
/// The caller chooses only a bounded ordinal range. Seed and DGP configuration
/// remain owned by [`CoverageCalibrationSimulationDesign`], and every element is
/// produced by the same indexed executor used for unsharded scientific
/// execution. Returned outcomes retain declared replication identity.
///
/// # Errors
///
/// Returns [`CoverageCalibrationStudyError::InvalidShardRange`] for an empty or
/// out-of-range shard. Structural replication invalidity propagates as
/// [`CoverageCalibrationStudyError::Execution`]; owner-admitted numerical fitting
/// inability remains an ordinary indexed failure outcome rather than an error.
pub fn execute_coverage_calibration_shard(
    design: CoverageCalibrationSimulationDesign,
    start_replication_index: usize,
    end_replication_index_exclusive: usize,
) -> Result<Vec<CoverageCalibrationReplicationOutcome>, CoverageCalibrationStudyError> {
    if start_replication_index >= end_replication_index_exclusive
        || end_replication_index_exclusive > design.attempted_replication_count()
    {
        return Err(CoverageCalibrationStudyError::InvalidShardRange);
    }

    (start_replication_index..end_replication_index_exclusive)
        .map(|replication_index| {
            map_execution_result(execute_coverage_calibration_replication(
                design,
                replication_index,
            ))
        })
        .collect()
}

/// Execute and bind one shard to prospective-design/scenario/source provenance.
///
/// Validation-design identity/fingerprint are derived directly from `validation_core`,
/// while simulation identity/fingerprint come from the simulation owner. The caller
/// supplies only the exact source commit because that identity belongs to the execution
/// environment rather than either scientific domain. All provenance is fixed before
/// any expensive scientific replication executes.
///
/// # Errors
///
/// Returns [`CoverageCalibrationStudyError::InvalidSourceIdentity`] for a
/// non-canonical source commit, [`CoverageCalibrationStudyError::InvalidShardRange`]
/// for an invalid range, [`CoverageCalibrationStudyError::InvalidDesignBinding`]
/// when validation-design fingerprinting fails,
/// [`CoverageCalibrationStudyError::InvalidScenarioBinding`] when simulation-owner
/// fingerprinting fails, or the structural execution error produced by a declared
/// replication.
pub fn execute_coverage_calibration_shard_record(
    design: CoverageCalibrationSimulationDesign,
    start_replication_index: usize,
    end_replication_index_exclusive: usize,
    source_head: &str,
) -> Result<CoverageCalibrationShardRecord, CoverageCalibrationStudyError> {
    require_canonical_source_head(source_head)?;
    let validation_design = CoverageCalibrationDesign::tepp_nominal_95_v1();
    require_matching_attempt_counts(
        validation_design.attempted_dgp_count(),
        design.attempted_replication_count(),
    )?;
    let validation_design_fingerprint = coverage_calibration_design_sha256(&validation_design)
        .map_err(|_| CoverageCalibrationStudyError::InvalidDesignBinding)?;
    require_canonical_sha256_fingerprint(&validation_design_fingerprint)
        .map_err(|_| CoverageCalibrationStudyError::InvalidDesignBinding)?;
    let scenario_fingerprint = map_scenario_fingerprint(design.scenario_fingerprint())?;
    require_canonical_sha256_fingerprint(&scenario_fingerprint)
        .map_err(|_| CoverageCalibrationStudyError::InvalidScenarioBinding)?;
    let outcomes = execute_coverage_calibration_shard(
        design,
        start_replication_index,
        end_replication_index_exclusive,
    )?;
    validate_outcome_range(
        start_replication_index,
        end_replication_index_exclusive,
        &outcomes,
    )?;

    Ok(CoverageCalibrationShardRecord {
        schema_version: COVERAGE_CALIBRATION_SHARD_SCHEMA_VERSION,
        start_replication_index,
        end_replication_index_exclusive,
        validation_design_id: validation_design.design_id().to_owned(),
        validation_design_fingerprint,
        simulation_scenario_id: design.scenario_id().to_owned(),
        simulation_scenario_fingerprint: scenario_fingerprint,
        source_head: source_head.to_owned(),
        outcomes,
    })
}

/// Assemble the complete v1 prospective coverage study from provenance-bound shards.
///
/// Shards may arrive in any completion order, but together they must exactly tile
/// `0..attempted_dgp_count` without gaps or overlap. Every shard must bind the same
/// canonical source commit and the current validation-design plus simulation-scenario
/// identities/fingerprints. Only after those application-level provenance checks pass
/// are outcomes flattened into `validation_core`, which remains authoritative for the
/// exact full permutation, denominator, coverage, Monte Carlo uncertainty,
/// percentiles, and schema-v4 evidence arithmetic.
///
/// # Errors
///
/// Returns [`CoverageCalibrationStudyError::IncompleteShardSet`] for an empty,
/// gapped, overlapping, or partial shard set,
/// [`CoverageCalibrationStudyError::InvalidShardProvenance`] for mixed or stale
/// design/scenario/source/range bindings,
/// [`CoverageCalibrationStudyError::InvalidDesignBinding`] if simulation and
/// validation owners drift in attempted-DGP count or validation fingerprinting fails,
/// or the existing scenario and evidence errors when their owners reject reconstruction.
pub fn assemble_coverage_calibration_evidence_v1(
    shards: &[CoverageCalibrationShardRecord],
) -> Result<CoverageCalibrationEvidenceRecord, CoverageCalibrationStudyError> {
    let validation_design = CoverageCalibrationDesign::tepp_nominal_95_v1();
    let simulation_design = CoverageCalibrationSimulationDesign::rolling_origin_coverage_v1();
    let attempted_replication_count = validation_design.attempted_dgp_count();
    require_matching_attempt_counts(
        attempted_replication_count,
        simulation_design.attempted_replication_count(),
    )?;
    let validation_design_fingerprint = coverage_calibration_design_sha256(&validation_design)
        .map_err(|_| CoverageCalibrationStudyError::InvalidDesignBinding)?;
    require_canonical_sha256_fingerprint(&validation_design_fingerprint)
        .map_err(|_| CoverageCalibrationStudyError::InvalidDesignBinding)?;
    let scenario_fingerprint = map_scenario_fingerprint(simulation_design.scenario_fingerprint())?;
    let (source_head, outcomes) = validate_and_flatten_shards(
        shards,
        attempted_replication_count,
        validation_design.design_id(),
        &validation_design_fingerprint,
        simulation_design.scenario_id(),
        &scenario_fingerprint,
    )?;

    map_evidence_result(CoverageCalibrationEvidenceRecord::from_indexed_outcomes(
        &validation_design,
        &outcomes,
        LOWER_COVERAGE_PERCENTILE,
        UPPER_COVERAGE_PERCENTILE,
        simulation_design.scenario_id(),
        &scenario_fingerprint,
        &source_head,
    ))
}

fn validate_and_flatten_shards(
    shards: &[CoverageCalibrationShardRecord],
    attempted_replication_count: usize,
    expected_validation_design_id: &str,
    expected_validation_design_fingerprint: &str,
    expected_scenario_id: &str,
    expected_scenario_fingerprint: &str,
) -> Result<(String, Vec<CoverageCalibrationReplicationOutcome>), CoverageCalibrationStudyError> {
    if shards.is_empty() {
        return Err(CoverageCalibrationStudyError::IncompleteShardSet);
    }

    let mut ordered: Vec<_> = shards.iter().collect();
    ordered.sort_unstable_by_key(|shard| shard.start_replication_index);
    let source_head = ordered[0].source_head.clone();
    require_canonical_source_head(&source_head)
        .map_err(|_| CoverageCalibrationStudyError::InvalidShardProvenance)?;

    let mut next_replication_index = 0_usize;
    let mut outcomes = Vec::with_capacity(attempted_replication_count);
    for shard in ordered {
        if shard.schema_version != COVERAGE_CALIBRATION_SHARD_SCHEMA_VERSION
            || shard.validation_design_id != expected_validation_design_id
            || shard.validation_design_fingerprint != expected_validation_design_fingerprint
            || shard.simulation_scenario_id != expected_scenario_id
            || shard.simulation_scenario_fingerprint != expected_scenario_fingerprint
            || shard.source_head != source_head
        {
            return Err(CoverageCalibrationStudyError::InvalidShardProvenance);
        }
        if shard.start_replication_index != next_replication_index
            || shard.start_replication_index >= shard.end_replication_index_exclusive
            || shard.end_replication_index_exclusive > attempted_replication_count
        {
            return Err(CoverageCalibrationStudyError::IncompleteShardSet);
        }
        validate_outcome_range(
            shard.start_replication_index,
            shard.end_replication_index_exclusive,
            &shard.outcomes,
        )?;
        outcomes.extend(shard.outcomes.iter().cloned());
        next_replication_index = shard.end_replication_index_exclusive;
    }

    if next_replication_index != attempted_replication_count {
        return Err(CoverageCalibrationStudyError::IncompleteShardSet);
    }
    Ok((source_head, outcomes))
}

fn validate_outcome_range(
    start_replication_index: usize,
    end_replication_index_exclusive: usize,
    outcomes: &[CoverageCalibrationReplicationOutcome],
) -> Result<(), CoverageCalibrationStudyError> {
    let expected_len = end_replication_index_exclusive
        .checked_sub(start_replication_index)
        .ok_or(CoverageCalibrationStudyError::InvalidShardProvenance)?;
    if outcomes.len() != expected_len {
        return Err(CoverageCalibrationStudyError::InvalidShardProvenance);
    }
    for (offset, outcome) in outcomes.iter().enumerate() {
        let expected_index = start_replication_index
            .checked_add(offset)
            .ok_or(CoverageCalibrationStudyError::InvalidShardProvenance)?;
        if outcome.replication_index() != expected_index {
            return Err(CoverageCalibrationStudyError::InvalidShardProvenance);
        }
    }
    Ok(())
}

fn require_matching_attempt_counts(
    validation_attempt_count: usize,
    simulation_attempt_count: usize,
) -> Result<(), CoverageCalibrationStudyError> {
    if validation_attempt_count != simulation_attempt_count {
        return Err(CoverageCalibrationStudyError::InvalidDesignBinding);
    }
    Ok(())
}

fn require_canonical_source_head(source_head: &str) -> Result<(), CoverageCalibrationStudyError> {
    let lowercase_hex = source_head.len() == 40
        && source_head
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if !lowercase_hex || parse_commit_head(source_head).is_err() {
        return Err(CoverageCalibrationStudyError::InvalidSourceIdentity);
    }
    Ok(())
}

fn require_canonical_sha256_fingerprint(
    fingerprint: &str,
) -> Result<(), CoverageCalibrationStudyError> {
    let lowercase_hex = fingerprint.len() == 64
        && fingerprint
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if !lowercase_hex {
        return Err(CoverageCalibrationStudyError::InvalidShardProvenance);
    }
    Ok(())
}

fn update_len_prefixed(
    hasher: &mut Sha256,
    bytes: &[u8],
) -> Result<(), CoverageCalibrationStudyError> {
    hasher.update(to_u64(bytes.len())?.to_le_bytes());
    hasher.update(bytes);
    Ok(())
}

fn to_u64(value: usize) -> Result<u64, CoverageCalibrationStudyError> {
    u64::try_from(value).map_err(|_| CoverageCalibrationStudyError::InvalidShardProvenance)
}

fn map_execution_result(
    result: Result<CoverageCalibrationReplicationOutcome, CoverageCalibrationExecutionError>,
) -> Result<CoverageCalibrationReplicationOutcome, CoverageCalibrationStudyError> {
    result.map_err(CoverageCalibrationStudyError::Execution)
}

fn map_scenario_fingerprint(
    result: Result<String, SimulationError>,
) -> Result<String, CoverageCalibrationStudyError> {
    result.map_err(|_| CoverageCalibrationStudyError::InvalidScenarioBinding)
}

fn map_evidence_result(
    result: Result<CoverageCalibrationEvidenceRecord, ValidationError>,
) -> Result<CoverageCalibrationEvidenceRecord, CoverageCalibrationStudyError> {
    result.map_err(|_| CoverageCalibrationStudyError::InvalidEvidence)
}

#[cfg(test)]
mod tests {
    use tepp_simulation::{CoverageCalibrationSimulationDesign, SimulationError};
    use validation_core::{
        CoverageCalibrationDesign, CoverageCalibrationReplicationOutcome, ValidationError,
        coverage_calibration_design_sha256,
    };

    use super::{
        COVERAGE_CALIBRATION_SHARD_SCHEMA_VERSION, CoverageCalibrationShardRecord,
        CoverageCalibrationStudyError, assemble_coverage_calibration_evidence_v1,
        map_evidence_result, map_execution_result, map_scenario_fingerprint,
        require_matching_attempt_counts, validate_and_flatten_shards,
    };
    use crate::CoverageCalibrationExecutionError;

    const SOURCE_HEAD: &str = "0123456789abcdef0123456789abcdef01234567";
    const OTHER_SOURCE_HEAD: &str = "1123456789abcdef0123456789abcdef01234567";

    fn synthetic_shard(
        start: usize,
        end: usize,
        source_head: &str,
    ) -> CoverageCalibrationShardRecord {
        let validation_design = CoverageCalibrationDesign::tepp_nominal_95_v1();
        let validation_design_fingerprint = coverage_calibration_design_sha256(&validation_design)
            .expect("versioned validation design fingerprint");
        let design = CoverageCalibrationSimulationDesign::rolling_origin_coverage_v1();
        let fingerprint = design
            .scenario_fingerprint()
            .expect("versioned scenario fingerprint");
        CoverageCalibrationShardRecord {
            schema_version: COVERAGE_CALIBRATION_SHARD_SCHEMA_VERSION,
            start_replication_index: start,
            end_replication_index_exclusive: end,
            validation_design_id: validation_design.design_id().to_owned(),
            validation_design_fingerprint,
            simulation_scenario_id: design.scenario_id().to_owned(),
            simulation_scenario_fingerprint: fingerprint,
            source_head: source_head.to_owned(),
            outcomes: (start..end)
                .map(|replication_index| {
                    CoverageCalibrationReplicationOutcome::successful(
                        replication_index,
                        vec![0.95, 0.95, 0.95, 0.95, 0.95],
                    )
                })
                .collect(),
        }
    }

    #[test]
    fn owner_binding_and_error_mappers_fail_closed() {
        assert_eq!(require_matching_attempt_counts(10, 10), Ok(()));
        assert_eq!(
            require_matching_attempt_counts(10, 9),
            Err(CoverageCalibrationStudyError::InvalidDesignBinding)
        );
        assert_eq!(
            map_scenario_fingerprint(Err(SimulationError::InvalidConfiguration)),
            Err(CoverageCalibrationStudyError::InvalidScenarioBinding)
        );
        assert_eq!(
            map_execution_result(Err(CoverageCalibrationExecutionError::InvalidSimulationScenario)),
            Err(CoverageCalibrationStudyError::Execution(
                CoverageCalibrationExecutionError::InvalidSimulationScenario
            ))
        );
        assert_eq!(
            map_evidence_result(Err(ValidationError::InvalidInput)),
            Err(CoverageCalibrationStudyError::InvalidEvidence)
        );

        let successful = CoverageCalibrationReplicationOutcome::successful(0, vec![0.95]);
        assert_eq!(map_execution_result(Ok(successful.clone())), Ok(successful));
        assert_eq!(
            map_scenario_fingerprint(Ok("fingerprint".to_owned())),
            Ok("fingerprint".to_owned())
        );
    }

    #[test]
    fn shard_assembly_is_completion_order_independent_and_binds_one_source() {
        let second = synthetic_shard(5_000, 10_000, SOURCE_HEAD);
        let first = synthetic_shard(0, 5_000, SOURCE_HEAD);
        let evidence = assemble_coverage_calibration_evidence_v1(&[second, first])
            .expect("complete provenance-bound shard set");

        assert_eq!(evidence.source_head(), SOURCE_HEAD);
        assert_eq!(evidence.attempted_replication_count(), 10_000);
        assert_eq!(evidence.successful_replication_count(), 10_000);
        assert!(evidence.supports_calibration_claim());
    }

    #[test]
    fn shard_assembly_rejects_mixed_source_and_non_tiling_ranges() {
        let first = synthetic_shard(0, 5_000, SOURCE_HEAD);
        let mixed_source = synthetic_shard(5_000, 10_000, OTHER_SOURCE_HEAD);
        assert_eq!(
            assemble_coverage_calibration_evidence_v1(&[first.clone(), mixed_source]),
            Err(CoverageCalibrationStudyError::InvalidShardProvenance)
        );

        let gap = synthetic_shard(5_001, 10_000, SOURCE_HEAD);
        assert_eq!(
            assemble_coverage_calibration_evidence_v1(&[first.clone(), gap]),
            Err(CoverageCalibrationStudyError::IncompleteShardSet)
        );

        let overlap = synthetic_shard(4_999, 10_000, SOURCE_HEAD);
        assert_eq!(
            assemble_coverage_calibration_evidence_v1(&[first, overlap]),
            Err(CoverageCalibrationStudyError::IncompleteShardSet)
        );
    }

    #[test]
    fn shard_validation_rejects_outcome_identity_drift() {
        let validation_design = CoverageCalibrationDesign::tepp_nominal_95_v1();
        let validation_design_fingerprint = coverage_calibration_design_sha256(&validation_design)
            .expect("versioned validation design fingerprint");
        let design = CoverageCalibrationSimulationDesign::rolling_origin_coverage_v1();
        let fingerprint = design
            .scenario_fingerprint()
            .expect("versioned scenario fingerprint");
        let malformed = CoverageCalibrationShardRecord {
            schema_version: COVERAGE_CALIBRATION_SHARD_SCHEMA_VERSION,
            start_replication_index: 0,
            end_replication_index_exclusive: 1,
            validation_design_id: validation_design.design_id().to_owned(),
            validation_design_fingerprint: validation_design_fingerprint.clone(),
            simulation_scenario_id: design.scenario_id().to_owned(),
            simulation_scenario_fingerprint: fingerprint.clone(),
            source_head: SOURCE_HEAD.to_owned(),
            outcomes: vec![CoverageCalibrationReplicationOutcome::successful(
                1,
                vec![0.95],
            )],
        };
        assert_eq!(
            validate_and_flatten_shards(
                &[malformed],
                1,
                validation_design.design_id(),
                &validation_design_fingerprint,
                design.scenario_id(),
                &fingerprint,
            ),
            Err(CoverageCalibrationStudyError::InvalidShardProvenance)
        );
    }

    #[test]
    fn public_error_messages_are_stable() {
        let messages = [
            (
                CoverageCalibrationStudyError::InvalidShardRange,
                "invalid coverage calibration shard range",
            ),
            (
                CoverageCalibrationStudyError::InvalidSourceIdentity,
                "invalid coverage calibration source identity",
            ),
            (
                CoverageCalibrationStudyError::InvalidShardProvenance,
                "invalid coverage calibration shard provenance",
            ),
            (
                CoverageCalibrationStudyError::IncompleteShardSet,
                "incomplete coverage calibration shard set",
            ),
            (
                CoverageCalibrationStudyError::InvalidDesignBinding,
                "invalid coverage calibration validation/simulation design binding",
            ),
            (
                CoverageCalibrationStudyError::Execution(
                    CoverageCalibrationExecutionError::InvalidSimulationScenario,
                ),
                "invalid coverage calibration simulation scenario",
            ),
            (
                CoverageCalibrationStudyError::InvalidScenarioBinding,
                "invalid coverage calibration scenario binding",
            ),
            (
                CoverageCalibrationStudyError::InvalidEvidence,
                "invalid coverage calibration evidence",
            ),
        ];
        for (error, message) in messages {
            assert_eq!(error.to_string(), message);
        }
    }
}
