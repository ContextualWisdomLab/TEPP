//! Canonical provenance binding for prospective coverage-calibration designs.

use sha2::{Digest, Sha256};

use crate::{CoverageCalibrationDesign, ValidationError};

const COVERAGE_CALIBRATION_DESIGN_FINGERPRINT_DOMAIN: &[u8] =
    b"tepp.validation.coverage-calibration-design.v4\0";

/// Compute the canonical SHA-256 fingerprint of a prospective coverage-calibration design.
///
/// The digest binds the versioned design identity, the exact covered-population/
/// aggregation estimand, attempted independent-DGP count, nominal coverage, exact
/// normal critical value used by the declared marginal interval rule, practical
/// lower and upper coverage bounds, maximum accepted Monte Carlo standard error,
/// and the lower/upper empirical percentile probabilities reported in durable
/// calibration evidence. Floating-point criteria are encoded from their exact
/// IEEE-754 binary64 bit patterns; integer/string lengths use explicit little-endian
/// `u64` wire geometry. The domain tag prevents reuse as another TEPP evidence digest.
///
/// This fingerprint is provenance only. It does not execute a simulation, assess
/// observed coverage, define numerical-failure acceptability, or promote a claim.
///
/// # Errors
///
/// Returns [`ValidationError::InvalidInput`] if the platform cannot represent the
/// owner-issued design/estimand identity lengths or attempted-replication count as
/// canonical `u64` wire values.
pub fn coverage_calibration_design_sha256(
    design: &CoverageCalibrationDesign,
) -> Result<String, ValidationError> {
    let design_id = design.design_id().as_bytes();
    let design_id_len = u64::try_from(design_id.len()).map_err(|_| ValidationError::InvalidInput)?;
    let estimand_id = design.estimand().estimand_id().as_bytes();
    let estimand_id_len =
        u64::try_from(estimand_id.len()).map_err(|_| ValidationError::InvalidInput)?;
    let attempted_dgp_count = u64::try_from(design.attempted_dgp_count())
        .map_err(|_| ValidationError::InvalidInput)?;

    let mut hasher = Sha256::new();
    hasher.update(COVERAGE_CALIBRATION_DESIGN_FINGERPRINT_DOMAIN);
    hasher.update(design_id_len.to_le_bytes());
    hasher.update(design_id);
    hasher.update(estimand_id_len.to_le_bytes());
    hasher.update(estimand_id);
    hasher.update(attempted_dgp_count.to_le_bytes());
    hasher.update(design.nominal_coverage().to_bits().to_le_bytes());
    hasher.update(design.normal_critical_value().to_bits().to_le_bytes());
    hasher.update(design.practical_lower_coverage().to_bits().to_le_bytes());
    hasher.update(design.practical_upper_coverage().to_bits().to_le_bytes());
    hasher.update(
        design
            .maximum_monte_carlo_standard_error()
            .to_bits()
            .to_le_bytes(),
    );
    hasher.update(
        design
            .coverage_percentile_lower_probability()
            .to_bits()
            .to_le_bytes(),
    );
    hasher.update(
        design
            .coverage_percentile_upper_probability()
            .to_bits()
            .to_le_bytes(),
    );

    let digest = hasher.finalize();
    Ok(format!("{digest:x}"))
}
