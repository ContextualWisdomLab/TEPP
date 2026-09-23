# Bind prospective coverage criteria to persisted evidence

- `validation_core::coverage_calibration_design_sha256` now derives a domain-separated SHA-256 fingerprint from the versioned validation design identity, attempted independent-DGP count, nominal coverage, practical lower/upper coverage bounds, and maximum Monte Carlo standard error using explicit little-endian integer lengths/counts and exact IEEE-754 binary64 bits.
- `CoverageCalibrationEvidenceRecord` schema v4 persists that fingerprint beside `validation_design_id`, preventing a stable design name from hiding criterion drift while keeping the full source head, simulation scenario fingerprint, indexed outcome ledger, and outcome digest as separate provenance dimensions.
- A pinned `tepp.coverage.nominal95.v1` wire fixture fixes the current criterion fingerprint at `e48b4a504318ee18fcd32eb598232bb8bc7b63944e4285bb207ef173ca4a1286` and verifies deterministic JSON exposure.
- This change does not execute the prospective 10,000-DGP study, define an acceptable numerical-failure rate, promote a scientific claim, or create a release.
