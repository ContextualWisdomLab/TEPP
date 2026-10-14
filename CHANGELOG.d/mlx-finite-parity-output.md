# Native MLX CPU parity receipt finite-output guard

- Refuse NaN, either infinity, and incomplete or oversized backend output before parity reduction and output digest construction. A single NaN must not collapse through `f64::max` into an exact-zero difference receipt.
- Retain the fixed native MLX CPU matrix objective and byte-identical finite exact-result receipt. Add permanent finite/mismatch/shape/non-finite controls and continue to execute the real native backend probe.
- Exercise the private native-call boundary with device/matmul/evaluation/null-data failure controls and actual native-handle frees; production dispatch stays bound to the pinned native functions. These are injected status controls, not observed native hardware faults.
- This is a local candidate correction of ADR0025's existing probe, not a new estimator/GPU parity claim or protected-main release. Research provenance is in `docs/research/mlx-finite-parity-output.md`; current full coverage and independent review remain separately required.
