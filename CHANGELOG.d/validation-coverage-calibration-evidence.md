## Scientific validation

- Add `CoverageCalibrationEvidenceRecord` as a validation-owned, machine-readable record for prospective interval-calibration runs. The record binds validation-design identity, opaque simulation scenario identity and SHA-256 fingerprint, and one exact lowercase Git source head without importing simulation-domain source into `validation_core`.
- Preserve the unconditional attempted/success/failure DGP denominator, failure rate and Bernoulli Monte Carlo standard error, conditional coverage mean and coverage MCSE, prospective practical-band/precision decisions, and the narrower conditional calibration claim in deterministic JSON.
- Fail closed when scenario identity/fingerprint/source-head shape is non-canonical or the observed attempt count differs from the prospectively declared validation design.
- This record is persistence/provenance evidence only. It does not execute the 10,000-DGP study, define a numerical-failure-rate cutoff, or grant scientific/release promotion authority.
