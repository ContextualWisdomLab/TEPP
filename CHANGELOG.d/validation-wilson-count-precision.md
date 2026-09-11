### Fixed

- `validation_core` now preserves distinct covered/sample counts when Wilson coverage evidence exceeds binary64's exact-integer range. Near the all-covered boundary it forms empirical coverage from the smaller uncovered complement and evaluates the Wilson interval by complement symmetry, so one retained miss cannot be rounded away into an all-covered numerical path.
- `WilsonCoverageEvidenceV1` stores `sample_count` and `covered_count` as fixed-width `u64` values, making the versioned JSON count contract independent of Rust pointer width while retaining exact integer provenance.
