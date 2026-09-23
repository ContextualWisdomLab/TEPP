# Coverage calibration study assembly

- Add an `analysis_engine` study boundary that executes a bounded half-open range of the prospectively declared coverage-calibration replication ordinals. Shards consume only `CoverageCalibrationSimulationDesign`; seed and DGP configuration remain owner-issued, structural invalidity aborts, and owner-admitted numerical fitting failure remains an indexed outcome.
- Add canonical v1 evidence assembly that binds `tepp.coverage.nominal95.v1` to `tepp.simulation.rolling_origin_coverage.v1`, derives the simulation scenario identity and SHA-256 fingerprint from the simulation owner, preserves the established 2.5%/97.5% reporting probabilities, and delegates exact-permutation outcome validation plus schema-v4 evidence arithmetic to `validation_core`.
- Keep the exact Git source head explicit because it is an execution-environment identity rather than a simulation/validation constant. The new boundary records but does not independently authenticate that head.
- This makes the declared study shardable/resumable without caller-authored seed/config/scenario provenance. It does not execute or promote the 10,000-DGP acceptance study, invent a numerical-failure-rate threshold, or satisfy #680 by itself.

Source RED: `94227ecbc4d7ba5141bbb856b59fd946e0bf7137`. Owner implementation: `f08a2b623bcdd0cd9420f5c121e4ca4213dcccbc`. Public export: `3dcae16dde0b808f544b8d82fa4766cb045315e5`. Failure-boundary coverage hardening: `dcbae4e7c3ea3039c8ef75f4a96e6a4ee6732f1e`.
