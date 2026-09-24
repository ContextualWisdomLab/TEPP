# Coverage calibration shard provenance

- Added `CoverageCalibrationShardRecord` so each declared calibration shard carries its half-open replication range, simulation scenario identity/fingerprint, exact lowercase source head, and indexed outcomes before any persistence or resume boundary.
- Added a domain-separated SHA-256 shard binding over exact binary64 coverage bits and deterministic JSON serialization; the digest is an application-level provenance binding, not external-storage or build authentication.
- Added `execute_coverage_calibration_shard_record(...)`, which validates source identity before expensive execution and derives scenario identity/fingerprint from `CoverageCalibrationSimulationDesign` rather than caller strings.
- Changed v1 coverage-evidence assembly to consume provenance-bound shard records, require one source/scenario binding, reject gaps/overlap/stale shard metadata, and only then delegate the complete indexed ledger to `validation_core` for denominator, calibration, Monte Carlo, percentile, and schema-v4 evidence arithmetic.
- This change does not execute or promote the declared 10,000-DGP acceptance study and does not define a post-hoc numerical-failure threshold.
