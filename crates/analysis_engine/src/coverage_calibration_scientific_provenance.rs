//! Cross-owner provenance for the prospective coverage-calibration experiment.
//!
//! Validation evidence deliberately does not import model-selection authority.
//! This application boundary therefore binds the model-selection numerical fit
//! design to shard execution and final validation evidence without copying either
//! owner's scientific arithmetic.

use model_selection::CoverageCalibrationFitDesign;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tepp_simulation::CoverageCalibrationSimulationDesign;
use validation_core::CoverageCalibrationEvidenceRecord;

use crate::coverage_calibration_study::{
    CoverageCalibrationShardRecord, CoverageCalibrationStudyError,
    assemble_coverage_calibration_evidence_v1, execute_coverage_calibration_shard_record,
};

const SCIENTIFIC_SHARD_SCHEMA_VERSION: u32 = 1;
const SCIENTIFIC_EVIDENCE_SCHEMA_VERSION: u32 = 1;
const FIT_SCHEDULE_FINGERPRINT_DOMAIN: &[u8] =
    b"tepp.analysis.coverage-calibration-fit-schedule.v1\0";
const SCIENTIFIC_SHARD_FINGERPRINT_DOMAIN: &[u8] =
    b"tepp.analysis.coverage-calibration-scientific-shard.v1\0";
const VALIDATION_EVIDENCE_JSON_FINGERPRINT_DOMAIN: &[u8] =
    b"tepp.analysis.coverage-calibration-validation-evidence-json.v1\0";
const SCIENTIFIC_EVIDENCE_FINGERPRINT_DOMAIN: &[u8] =
    b"tepp.analysis.coverage-calibration-scientific-evidence.v1\0";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ScientificShardWire {
    schema_version: u32,
    fit_design_id: String,
    fit_schedule_sha256: String,
    shard_record_sha256: String,
    shard_record_json: String,
}

#[derive(Serialize)]
struct ScientificShardWireRef<'a> {
    schema_version: u32,
    fit_design_id: &'a str,
    fit_schedule_sha256: &'a str,
    shard_record_sha256: &'a str,
    shard_record_json: &'a str,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ScientificEvidenceWire {
    schema_version: u32,
    fit_design_id: String,
    fit_schedule_sha256: String,
    validation_evidence_sha256: String,
    validation_evidence_json: String,
}

#[derive(Serialize)]
struct ScientificEvidenceWireRef<'a> {
    schema_version: u32,
    fit_design_id: &'a str,
    fit_schedule_sha256: &'a str,
    validation_evidence_sha256: &'a str,
    validation_evidence_json: &'a str,
}

/// Provenance-bound shard that fixes the numerical model-selection design used during execution.
///
/// The nested [`CoverageCalibrationShardRecord`] retains validation design,
/// simulation scenario, exact source, range, and indexed outcomes. This wrapper
/// adds the model-selection owner identity and a deterministic schedule digest over
/// the exact truth-`K` fit design applied to every replication ordinal in the shard.
/// The wrapper is an application-level integrity binding, not authentication of the
/// external runner, Git checkout, or persistence service.
#[derive(Clone, Debug, PartialEq)]
pub struct CoverageCalibrationScientificShardRecord {
    schema_version: u32,
    fit_design_id: String,
    fit_schedule_sha256: String,
    shard_record_sha256: String,
    shard_record: CoverageCalibrationShardRecord,
}

impl CoverageCalibrationScientificShardRecord {
    /// Version of the scientific shard wrapper wire contract.
    #[must_use]
    pub const fn schema_version(&self) -> u32 {
        self.schema_version
    }

    /// Versioned model-selection owner identity fixed for this shard.
    #[must_use]
    pub fn fit_design_id(&self) -> &str {
        &self.fit_design_id
    }

    /// Domain-separated digest over the concrete per-replication fit-design schedule.
    #[must_use]
    pub fn fit_schedule_sha256(&self) -> &str {
        &self.fit_schedule_sha256
    }

    /// Underlying validation/scenario/source/outcome shard record.
    #[must_use]
    pub const fn shard_record(&self) -> &CoverageCalibrationShardRecord {
        &self.shard_record
    }

    /// Canonical SHA-256 of the nested shard record.
    #[must_use]
    pub fn shard_record_sha256(&self) -> &str {
        &self.shard_record_sha256
    }

    /// Serialize the strict scientific-shard wire representation.
    ///
    /// The nested shard remains an opaque JSON string so its own strict parser can
    /// reject duplicate fields rather than allowing an outer generic JSON value to
    /// collapse them before owner recovery.
    ///
    /// # Errors
    ///
    /// Returns [`CoverageCalibrationStudyError::InvalidEvidence`] if nested or outer
    /// serialization unexpectedly fails.
    pub fn to_json(&self) -> Result<String, CoverageCalibrationStudyError> {
        let shard_record_json = self.shard_record.to_json()?;
        serde_json::to_string(&ScientificShardWireRef {
            schema_version: self.schema_version,
            fit_design_id: &self.fit_design_id,
            fit_schedule_sha256: &self.fit_schedule_sha256,
            shard_record_sha256: &self.shard_record_sha256,
            shard_record_json: &shard_record_json,
        })
        .map_err(|_| CoverageCalibrationStudyError::InvalidEvidence)
    }

    /// Return the domain-separated digest of this scientific shard wrapper.
    ///
    /// # Errors
    ///
    /// Returns [`CoverageCalibrationStudyError::InvalidShardProvenance`] if any
    /// platform-sized identity cannot be represented in canonical `u64` geometry.
    pub fn sha256(&self) -> Result<String, CoverageCalibrationStudyError> {
        let mut hasher = Sha256::new();
        hasher.update(SCIENTIFIC_SHARD_FINGERPRINT_DOMAIN);
        hasher.update(self.schema_version.to_le_bytes());
        update_len_prefixed(&mut hasher, self.fit_design_id.as_bytes())?;
        update_len_prefixed(&mut hasher, self.fit_schedule_sha256.as_bytes())?;
        update_len_prefixed(&mut hasher, self.shard_record_sha256.as_bytes())?;
        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Rehydrate a persisted scientific shard and verify both wrapper and nested digests.
    ///
    /// Recovery recomputes the expected model-selection fit schedule from the current
    /// versioned simulation and model-selection owners for the nested ordinal range.
    /// Any post-execution fit-design substitution therefore fails closed before the
    /// shard can enter final study assembly.
    ///
    /// # Errors
    ///
    /// Returns [`CoverageCalibrationStudyError::InvalidShardProvenance`] for malformed,
    /// noncanonical, stale, duplicate-field, digest-mismatched, or owner-mismatched input.
    pub fn from_json_with_sha256(
        json: &str,
        expected_sha256: &str,
    ) -> Result<Self, CoverageCalibrationStudyError> {
        require_canonical_sha256(expected_sha256)?;
        let wire: ScientificShardWire = serde_json::from_str(json)
            .map_err(|_| CoverageCalibrationStudyError::InvalidShardProvenance)?;
        if wire.schema_version != SCIENTIFIC_SHARD_SCHEMA_VERSION {
            return Err(CoverageCalibrationStudyError::InvalidShardProvenance);
        }
        require_canonical_sha256(&wire.fit_schedule_sha256)?;
        require_canonical_sha256(&wire.shard_record_sha256)?;
        let shard_record = CoverageCalibrationShardRecord::from_json_with_sha256(
            &wire.shard_record_json,
            &wire.shard_record_sha256,
        )?;
        let record = bind_scientific_shard_record(
            CoverageCalibrationSimulationDesign::rolling_origin_coverage_v1(),
            shard_record,
        )?;
        if record.fit_design_id != wire.fit_design_id
            || record.fit_schedule_sha256 != wire.fit_schedule_sha256
            || record.shard_record_sha256 != wire.shard_record_sha256
            || record.sha256()? != expected_sha256
        {
            return Err(CoverageCalibrationStudyError::InvalidShardProvenance);
        }
        Ok(record)
    }
}

/// Final application-level evidence that persists the model-selection fit design with validation evidence.
///
/// The nested validation record remains authoritative for coverage arithmetic,
/// denominators, Monte Carlo uncertainty, criterion components, and its schema.
/// This wrapper adds only the cross-owner numerical-fit provenance that validation
/// must not own itself.
#[derive(Clone, Debug, PartialEq)]
pub struct CoverageCalibrationScientificEvidenceRecord {
    schema_version: u32,
    fit_design_id: String,
    fit_schedule_sha256: String,
    validation_evidence_sha256: String,
    validation_evidence: CoverageCalibrationEvidenceRecord,
}

impl CoverageCalibrationScientificEvidenceRecord {
    /// Version of the final scientific-evidence wrapper wire contract.
    #[must_use]
    pub const fn schema_version(&self) -> u32 {
        self.schema_version
    }

    /// Versioned model-selection owner identity applied by execution.
    #[must_use]
    pub fn fit_design_id(&self) -> &str {
        &self.fit_design_id
    }

    /// Digest over the concrete fit design for every declared DGP ordinal.
    #[must_use]
    pub fn fit_schedule_sha256(&self) -> &str {
        &self.fit_schedule_sha256
    }

    /// SHA-256 binding over the nested validation evidence JSON.
    #[must_use]
    pub fn validation_evidence_sha256(&self) -> &str {
        &self.validation_evidence_sha256
    }

    /// Validation-owner scientific evidence retained without copied arithmetic.
    #[must_use]
    pub const fn validation_evidence(&self) -> &CoverageCalibrationEvidenceRecord {
        &self.validation_evidence
    }

    /// Serialize the strict final scientific-evidence wire representation.
    ///
    /// The nested validation record is stored as an opaque JSON string so owner
    /// rehydration sees its original member stream and can reject duplicate fields.
    ///
    /// # Errors
    ///
    /// Returns [`CoverageCalibrationStudyError::InvalidEvidence`] when nested or
    /// outer serialization unexpectedly fails.
    pub fn to_json(&self) -> Result<String, CoverageCalibrationStudyError> {
        let validation_evidence_json = self
            .validation_evidence
            .to_json()
            .map_err(|_| CoverageCalibrationStudyError::InvalidEvidence)?;
        serde_json::to_string(&ScientificEvidenceWireRef {
            schema_version: self.schema_version,
            fit_design_id: &self.fit_design_id,
            fit_schedule_sha256: &self.fit_schedule_sha256,
            validation_evidence_sha256: &self.validation_evidence_sha256,
            validation_evidence_json: &validation_evidence_json,
        })
        .map_err(|_| CoverageCalibrationStudyError::InvalidEvidence)
    }

    /// Return the domain-separated digest of the final scientific-evidence wrapper.
    ///
    /// # Errors
    ///
    /// Returns [`CoverageCalibrationStudyError::InvalidShardProvenance`] if canonical
    /// identity lengths cannot be represented as `u64`.
    pub fn sha256(&self) -> Result<String, CoverageCalibrationStudyError> {
        let mut hasher = Sha256::new();
        hasher.update(SCIENTIFIC_EVIDENCE_FINGERPRINT_DOMAIN);
        hasher.update(self.schema_version.to_le_bytes());
        update_len_prefixed(&mut hasher, self.fit_design_id.as_bytes())?;
        update_len_prefixed(&mut hasher, self.fit_schedule_sha256.as_bytes())?;
        update_len_prefixed(&mut hasher, self.validation_evidence_sha256.as_bytes())?;
        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Rehydrate final scientific evidence through both application and validation owners.
    ///
    /// Recovery verifies the nested validation record through
    /// [`CoverageCalibrationEvidenceRecord::from_json`], reconstructs the canonical
    /// simulation scenario, recomputes the full model-selection fit schedule, and
    /// checks both nested and outer digests.
    ///
    /// # Errors
    ///
    /// Returns [`CoverageCalibrationStudyError::InvalidEvidence`] or
    /// [`CoverageCalibrationStudyError::InvalidShardProvenance`] when any wire,
    /// owner, scenario, fit-schedule, or digest condition fails.
    pub fn from_json_with_sha256(
        json: &str,
        expected_sha256: &str,
    ) -> Result<Self, CoverageCalibrationStudyError> {
        require_canonical_sha256(expected_sha256)?;
        let wire: ScientificEvidenceWire = serde_json::from_str(json)
            .map_err(|_| CoverageCalibrationStudyError::InvalidEvidence)?;
        if wire.schema_version != SCIENTIFIC_EVIDENCE_SCHEMA_VERSION {
            return Err(CoverageCalibrationStudyError::InvalidEvidence);
        }
        require_canonical_sha256(&wire.fit_schedule_sha256)
            .map_err(|_| CoverageCalibrationStudyError::InvalidEvidence)?;
        require_canonical_sha256(&wire.validation_evidence_sha256)
            .map_err(|_| CoverageCalibrationStudyError::InvalidEvidence)?;
        let validation_evidence = CoverageCalibrationEvidenceRecord::from_json(
            &wire.validation_evidence_json,
        )
        .map_err(|_| CoverageCalibrationStudyError::InvalidEvidence)?;
        let simulation_design = CoverageCalibrationSimulationDesign::rolling_origin_coverage_v1();
        require_current_scenario(&validation_evidence, simulation_design)?;
        let fit_design = CoverageCalibrationFitDesign::truth_k_v1();
        let expected_fit_schedule = fit_schedule_sha256(
            simulation_design,
            0,
            validation_evidence.attempted_replication_count(),
        )?;
        let expected_validation_evidence_sha256 =
            validation_evidence_json_sha256(&wire.validation_evidence_json);
        if wire.fit_design_id != fit_design.design_id()
            || wire.fit_schedule_sha256 != expected_fit_schedule
            || wire.validation_evidence_sha256 != expected_validation_evidence_sha256
        {
            return Err(CoverageCalibrationStudyError::InvalidEvidence);
        }
        let record = Self {
            schema_version: wire.schema_version,
            fit_design_id: wire.fit_design_id,
            fit_schedule_sha256: wire.fit_schedule_sha256,
            validation_evidence_sha256: wire.validation_evidence_sha256,
            validation_evidence,
        };
        if record.sha256()? != expected_sha256 {
            return Err(CoverageCalibrationStudyError::InvalidEvidence);
        }
        Ok(record)
    }
}

/// Execute one ordinal shard and persist the exact numerical fit schedule with it.
///
/// The simulation owner still mints each DGP and the existing shard executor still
/// owns indexed execution. This function adds the model-selection fit provenance
/// immediately after execution, before the record crosses a persistence/resume boundary.
///
/// # Errors
///
/// Propagates the existing shard execution/provenance errors or fails closed when
/// the current simulation and model-selection owners cannot reconstruct the exact
/// fit schedule for the requested ordinal range.
pub fn execute_coverage_calibration_scientific_shard_record(
    design: CoverageCalibrationSimulationDesign,
    start_replication_index: usize,
    end_replication_index_exclusive: usize,
    source_head: &str,
) -> Result<CoverageCalibrationScientificShardRecord, CoverageCalibrationStudyError> {
    let shard_record = execute_coverage_calibration_shard_record(
        design,
        start_replication_index,
        end_replication_index_exclusive,
        source_head,
    )?;
    bind_scientific_shard_record(design, shard_record)
}

/// Assemble final validation evidence only from fit-bound scientific shards.
///
/// Every shard is rechecked against the current versioned simulation and
/// model-selection owners before the existing validation-owned assembly consumes
/// its nested shard. The returned wrapper persists the full 10,000-ordinal fit
/// schedule separately from validation arithmetic.
///
/// # Errors
///
/// Propagates incomplete/gapped shard-set, validation-evidence, simulation-owner,
/// and numerical-fit provenance failures from the owning boundaries.
pub fn assemble_coverage_calibration_scientific_evidence_v1(
    shards: &[CoverageCalibrationScientificShardRecord],
) -> Result<CoverageCalibrationScientificEvidenceRecord, CoverageCalibrationStudyError> {
    if shards.is_empty() {
        return Err(CoverageCalibrationStudyError::IncompleteShardSet);
    }
    let simulation_design = CoverageCalibrationSimulationDesign::rolling_origin_coverage_v1();
    let mut base_shards = Vec::with_capacity(shards.len());
    for shard in shards {
        validate_scientific_shard_record(simulation_design, shard)?;
        base_shards.push(shard.shard_record.clone());
    }
    let validation_evidence = assemble_coverage_calibration_evidence_v1(&base_shards)?;
    require_current_scenario(&validation_evidence, simulation_design)?;
    let fit_design = CoverageCalibrationFitDesign::truth_k_v1();
    let fit_schedule_sha256 = fit_schedule_sha256(
        simulation_design,
        0,
        validation_evidence.attempted_replication_count(),
    )?;
    let validation_evidence_json = validation_evidence
        .to_json()
        .map_err(|_| CoverageCalibrationStudyError::InvalidEvidence)?;
    let validation_evidence_sha256 =
        validation_evidence_json_sha256(&validation_evidence_json);
    Ok(CoverageCalibrationScientificEvidenceRecord {
        schema_version: SCIENTIFIC_EVIDENCE_SCHEMA_VERSION,
        fit_design_id: fit_design.design_id().to_owned(),
        fit_schedule_sha256,
        validation_evidence_sha256,
        validation_evidence,
    })
}

fn bind_scientific_shard_record(
    design: CoverageCalibrationSimulationDesign,
    shard_record: CoverageCalibrationShardRecord,
) -> Result<CoverageCalibrationScientificShardRecord, CoverageCalibrationStudyError> {
    require_current_shard_scenario(&shard_record, design)?;
    let fit_design = CoverageCalibrationFitDesign::truth_k_v1();
    let fit_schedule_sha256 = fit_schedule_sha256(
        design,
        shard_record.start_replication_index(),
        shard_record.end_replication_index_exclusive(),
    )?;
    let shard_record_sha256 = shard_record.sha256()?;
    Ok(CoverageCalibrationScientificShardRecord {
        schema_version: SCIENTIFIC_SHARD_SCHEMA_VERSION,
        fit_design_id: fit_design.design_id().to_owned(),
        fit_schedule_sha256,
        shard_record_sha256,
        shard_record,
    })
}

fn validate_scientific_shard_record(
    design: CoverageCalibrationSimulationDesign,
    record: &CoverageCalibrationScientificShardRecord,
) -> Result<(), CoverageCalibrationStudyError> {
    if record.schema_version != SCIENTIFIC_SHARD_SCHEMA_VERSION {
        return Err(CoverageCalibrationStudyError::InvalidShardProvenance);
    }
    let fit_design = CoverageCalibrationFitDesign::truth_k_v1();
    let expected_fit_schedule = fit_schedule_sha256(
        design,
        record.shard_record.start_replication_index(),
        record.shard_record.end_replication_index_exclusive(),
    )?;
    require_current_shard_scenario(&record.shard_record, design)?;
    if record.fit_design_id != fit_design.design_id()
        || record.fit_schedule_sha256 != expected_fit_schedule
        || record.shard_record.sha256()? != record.shard_record_sha256
    {
        return Err(CoverageCalibrationStudyError::InvalidShardProvenance);
    }
    Ok(())
}

fn require_current_shard_scenario(
    record: &CoverageCalibrationShardRecord,
    design: CoverageCalibrationSimulationDesign,
) -> Result<(), CoverageCalibrationStudyError> {
    let scenario_fingerprint = design
        .scenario_fingerprint()
        .map_err(|_| CoverageCalibrationStudyError::InvalidScenarioBinding)?;
    if record.simulation_scenario_id() != design.scenario_id()
        || record.simulation_scenario_fingerprint() != scenario_fingerprint
    {
        return Err(CoverageCalibrationStudyError::InvalidShardProvenance);
    }
    Ok(())
}

fn require_current_scenario(
    evidence: &CoverageCalibrationEvidenceRecord,
    design: CoverageCalibrationSimulationDesign,
) -> Result<(), CoverageCalibrationStudyError> {
    let scenario_fingerprint = design
        .scenario_fingerprint()
        .map_err(|_| CoverageCalibrationStudyError::InvalidScenarioBinding)?;
    if evidence.simulation_scenario_id() != design.scenario_id()
        || evidence.simulation_scenario_fingerprint() != scenario_fingerprint
    {
        return Err(CoverageCalibrationStudyError::InvalidEvidence);
    }
    Ok(())
}

fn fit_schedule_sha256(
    design: CoverageCalibrationSimulationDesign,
    start_replication_index: usize,
    end_replication_index_exclusive: usize,
) -> Result<String, CoverageCalibrationStudyError> {
    if start_replication_index >= end_replication_index_exclusive
        || end_replication_index_exclusive > design.attempted_replication_count()
    {
        return Err(CoverageCalibrationStudyError::InvalidShardRange);
    }
    let fit_design = CoverageCalibrationFitDesign::truth_k_v1();
    let mut hasher = Sha256::new();
    hasher.update(FIT_SCHEDULE_FINGERPRINT_DOMAIN);
    update_len_prefixed(&mut hasher, fit_design.design_id().as_bytes())?;
    update_len_prefixed(&mut hasher, design.scenario_id().as_bytes())?;
    hasher.update(to_u64(start_replication_index)?.to_le_bytes());
    hasher.update(to_u64(end_replication_index_exclusive)?.to_le_bytes());
    hasher.update(
        to_u64(
            end_replication_index_exclusive
                .checked_sub(start_replication_index)
                .ok_or(CoverageCalibrationStudyError::InvalidShardRange)?,
        )?
        .to_le_bytes(),
    );
    for replication_index in start_replication_index..end_replication_index_exclusive {
        let config = design
            .config_for_replication(replication_index)
            .map_err(|_| CoverageCalibrationStudyError::InvalidScenarioBinding)?;
        let truth_k = config.topic_dgp().true_topic_count();
        let fingerprint = fit_design
            .fingerprint_for_truth_k(truth_k)
            .map_err(|_| CoverageCalibrationStudyError::InvalidDesignBinding)?;
        require_canonical_sha256(&fingerprint)
            .map_err(|_| CoverageCalibrationStudyError::InvalidDesignBinding)?;
        hasher.update(to_u64(replication_index)?.to_le_bytes());
        update_len_prefixed(&mut hasher, fingerprint.as_bytes())?;
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn validation_evidence_json_sha256(json: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(VALIDATION_EVIDENCE_JSON_FINGERPRINT_DOMAIN);
    hasher.update(json.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn require_canonical_sha256(value: &str) -> Result<(), CoverageCalibrationStudyError> {
    let canonical = value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if !canonical {
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
