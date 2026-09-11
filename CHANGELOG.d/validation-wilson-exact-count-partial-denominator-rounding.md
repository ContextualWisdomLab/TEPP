### Fixed

- `validation_core::wilson_coverage_interval` now compensates the exact TwoSum residual whenever an exactly representable all-covered sample count forms an inexact binary64 denominator `n + z²`. This preserves the represented-input Wilson lower endpoint when ordinary partial denominator rounding moves the direct quotient by one ULP, while exact denominator sums and the existing near-one/inexact-`u64` boundary contracts remain unchanged.
