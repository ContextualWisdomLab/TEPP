### Fixed

- `validation_core::wilson_coverage_interval` now carries an exact-count all-covered regression for the positive TwoSum/FMA denominator-residual direction. The fixture proves that when the direct lower endpoint is one ULP too small, exact represented-input residual comparison selects the adjacent larger binary64 value without changing the Wilson formula, admission domain, or coverage policy.
