//! Versioned simulation scenario for prospective interval-calibration evidence.

use crate::{SimulationConfig, SimulationError};

const COVERAGE_CALIBRATION_FIRST_SEED: u64 = 0x4341_4c49_4252_0000;
const COVERAGE_CALIBRATION_REPLICATION_COUNT: usize = 10_000;
const COVERAGE_CALIBRATION_FINGERPRINT_DOMAIN: &[u8] =
    b"tepp.coverage-calibration-scenario-fingerprint.v1\0";

/// Owner-issued DGP scenario for the first prospective rolling-origin coverage study.
///
/// This type fixes the data-generating process and seed schedule independently
/// from `validation_core`'s practical coverage criterion. A later scientific
/// run may therefore prove which DGP was executed without allowing the validation
/// threshold owner to redefine simulation truth after outcomes are visible.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CoverageCalibrationSimulationDesign;

impl CoverageCalibrationSimulationDesign {
    /// Construct TEPP's first versioned rolling-origin coverage simulation design.
    #[must_use]
    pub const fn rolling_origin_coverage_v1() -> Self {
        Self
    }

    /// Stable scenario identity for persisted acceptance evidence.
    #[must_use]
    pub const fn scenario_id(self) -> &'static str {
        "tepp.simulation.rolling_origin_coverage.v1"
    }

    /// Number of independently seeded DGP replications declared by this scenario.
    #[must_use]
    pub const fn attempted_replication_count(self) -> usize {
        COVERAGE_CALIBRATION_REPLICATION_COUNT
    }

    /// Return the exact domain-separated seed for one zero-based replication ordinal.
    ///
    /// # Errors
    ///
    /// Returns [`SimulationError::InvalidConfiguration`] when `replication_index`
    /// lies outside the prospectively declared scenario.
    pub fn seed_for_replication(self, replication_index: usize) -> Result<u64, SimulationError> {
        if replication_index >= COVERAGE_CALIBRATION_REPLICATION_COUNT {
            return Err(SimulationError::InvalidConfiguration);
        }
        let offset = u64::try_from(replication_index)
            .map_err(|_| SimulationError::InvalidConfiguration)?;
        COVERAGE_CALIBRATION_FIRST_SEED
            .checked_add(offset)
            .ok_or(SimulationError::InvalidConfiguration)
    }

    /// Construct the exact DGP configuration for one declared replication.
    ///
    /// The scenario matches the realistic rolling-origin scientific fixture:
    /// nine latent events, two originals per event, three Membership targets,
    /// bounded report/availability delays, no EventTime missingness, nonzero
    /// observed relation false positives, derivative document method effects,
    /// and the canonical known-topic DGP owned by [`SimulationConfig`].
    ///
    /// # Errors
    ///
    /// Returns [`SimulationError::InvalidConfiguration`] when the replication
    /// ordinal is outside the declared design or the owned configuration becomes
    /// invalid under a future incompatible change.
    pub fn config_for_replication(
        self,
        replication_index: usize,
    ) -> Result<SimulationConfig, SimulationError> {
        SimulationConfig::new(
            self.seed_for_replication(replication_index)?,
            9,
            2,
            3,
            12,
            6,
            0,
            0,
            500,
            3_000,
            3_000,
            3_000,
        )
    }

    /// Digest the complete declared scenario into one immutable SHA-256 fingerprint.
    ///
    /// The digest covers a domain separator, the versioned scenario identity,
    /// the declared attempt count, and the canonical configuration fingerprint
    /// of every replication in ordinal order. Configuration bytes are produced
    /// by the same crate-owned fingerprint used to bind generated truth manifests,
    /// so persisted acceptance evidence cannot silently switch to a second config
    /// serialization.
    ///
    /// # Errors
    ///
    /// Returns [`SimulationError::InvalidConfiguration`] if the declared attempt
    /// count cannot be represented as `u64` or any owner-issued replication
    /// configuration cannot be reconstructed.
    pub fn scenario_fingerprint(self) -> Result<String, SimulationError> {
        let mut bytes = Vec::with_capacity(
            COVERAGE_CALIBRATION_FINGERPRINT_DOMAIN.len()
                + self.scenario_id().len()
                + 1
                + 8
                + (COVERAGE_CALIBRATION_REPLICATION_COUNT * 160),
        );
        bytes.extend_from_slice(COVERAGE_CALIBRATION_FINGERPRINT_DOMAIN);
        bytes.extend_from_slice(self.scenario_id().as_bytes());
        bytes.push(0);
        let attempted_replication_count = u64::try_from(self.attempted_replication_count())
            .map_err(|_| SimulationError::InvalidConfiguration)?;
        bytes.extend_from_slice(&attempted_replication_count.to_le_bytes());
        for replication_index in 0..COVERAGE_CALIBRATION_REPLICATION_COUNT {
            let config = self.config_for_replication(replication_index)?;
            bytes.extend_from_slice(&crate::config_fingerprint(config));
        }
        Ok(crate::digest_bytes(&bytes))
    }
}
