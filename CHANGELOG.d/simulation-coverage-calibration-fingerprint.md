## Scientific validation

- Bind `CoverageCalibrationSimulationDesign::rolling_origin_coverage_v1()` to an immutable SHA-256 scenario fingerprint covering its domain tag, versioned scenario identity, declared 10,000-attempt count, and every owner-issued replication configuration in declared order.
- Reuse the exact `SimulationConfig` fingerprint bytes already used by truth-manifest generation so acceptance evidence cannot silently serialize the DGP differently from the generated truth corpus.
- Pin the v1 digest in a contract test; any seed/config schedule change now requires an explicit scenario-version/fingerprint decision rather than allowing the test and implementation to drift together under the same identity.
- This is provenance/reproducibility plumbing only. It does not execute the 10,000-DGP calibration study or promote #680 scientific acceptance.
