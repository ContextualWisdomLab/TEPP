//! Shard-safe execution and owner-bound assembly for the prospective coverage study.
//!
//! Per-replication scientific composition lives in
//! [`crate::execute_coverage_calibration_replication`]. This module adds the
//! application boundary needed to execute declared ordinal ranges without
//! caller-authored seeds and to assemble the complete indexed outcome ledger
//! into schema-owned validation evidence without caller-authored scenario
//! identity or fingerprint values.

use std::fmt;

use tepp_simulation::{CoverageCalibrationSimulationDesign, SimulationError};
use validation_core::{
    CoverageCalibrationDesign, CoverageCalibrationEvidenceRecord,
    CoverageCalibrationReplicationOutcome, ValidationError,
};

use crate::{CoverageCalibrationExecutionError, execute_coverage_calibration_replication};

const LOWER_COVERAGE_PERCENTILE: f64 = 0.025;
const UPPER_COVERAGE_PERCENTILE: f64 = 0.975;

/// Fail-closed error from prospective coverage-study execution or evidence assembly.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoverageCalibrationStudyError {
    /// The requested half-open shard range was empty or outside the declared scenario.
    InvalidShardRange,
    /// The versioned simulation and validation owners disagree on the declared attempt count.
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
            Self::InvalidDesignBinding => formatter
                .write_str("coverage calibration design/scenario attempt counts differ"),
            Self::Execution(error) => error.fmt(formatter),
            Self::InvalidScenarioBinding => {
                formatter.write_str("invalid coverage calibration scenario binding")
            }
            Self::InvalidEvidence => formatter.write_str("invalid coverage calibration evidence"),
        }
    }
}

impl std::error::Error for CoverageCalibrationStudyError {}

/// Execute one half-open range of prospectively declared DGP replication ordinals.
///
/// The caller chooses only a bounded ordinal range. Seed and DGP configuration
/// remain owned by [`CoverageCalibrationSimulationDesign`], and every element is
/// produced by the same indexed executor used for unsharded scientific
/// execution. Returned outcomes therefore retain declared replication identity
/// and may be concatenated in any completion order before canonical evidence
/// assembly.
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

/// Assemble the complete v1 prospective coverage study into canonical evidence.
///
/// This boundary fixes the validation/scenario owner pair and the previously
/// established 2.5%/97.5% empirical reporting probabilities. The simulation
/// scenario identity and fingerprint are derived directly from the simulation
/// owner rather than accepted as caller strings. `outcomes` may arrive in shard
/// completion order; `validation_core` requires an exact permutation of every
/// declared ordinal and canonicalizes it before persistence.
///
/// `source_head` is the exact lowercase forty-hex Git commit that produced the
/// supplied outcomes. This function records that identity but does not claim to
/// authenticate the surrounding build or storage system.
///
/// # Errors
///
/// Returns [`CoverageCalibrationStudyError::InvalidDesignBinding`] if the two
/// versioned owners drift in attempted-DGP count,
/// [`CoverageCalibrationStudyError::InvalidScenarioBinding`] if the simulation
/// fingerprint cannot be reconstructed, or
/// [`CoverageCalibrationStudyError::InvalidEvidence`] when outcome identities,
/// coverage values, source identity, or evidence serialization inputs violate
/// the validation owner contract.
pub fn assemble_coverage_calibration_evidence_v1(
    outcomes: &[CoverageCalibrationReplicationOutcome],
    source_head: &str,
) -> Result<CoverageCalibrationEvidenceRecord, CoverageCalibrationStudyError> {
    let validation_design = CoverageCalibrationDesign::tepp_nominal_95_v1();
    let simulation_design = CoverageCalibrationSimulationDesign::rolling_origin_coverage_v1();
    require_matching_attempt_counts(
        validation_design.attempted_dgp_count(),
        simulation_design.attempted_replication_count(),
    )?;
    let scenario_fingerprint = map_scenario_fingerprint(simulation_design.scenario_fingerprint())?;

    map_evidence_result(CoverageCalibrationEvidenceRecord::from_indexed_outcomes(
        &validation_design,
        outcomes,
        LOWER_COVERAGE_PERCENTILE,
        UPPER_COVERAGE_PERCENTILE,
        simulation_design.scenario_id(),
        &scenario_fingerprint,
        source_head,
    ))
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
    use tepp_simulation::SimulationError;
    use validation_core::{CoverageCalibrationReplicationOutcome, ValidationError};

    use super::{
        CoverageCalibrationStudyError, map_evidence_result, map_execution_result,
        map_scenario_fingerprint, require_matching_attempt_counts,
    };
    use crate::CoverageCalibrationExecutionError;

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
    fn public_error_messages_are_stable() {
        let messages = [
            (
                CoverageCalibrationStudyError::InvalidShardRange,
                "invalid coverage calibration shard range",
            ),
            (
                CoverageCalibrationStudyError::InvalidDesignBinding,
                "coverage calibration design/scenario attempt counts differ",
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
