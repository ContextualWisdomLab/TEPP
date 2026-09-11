### Fixed

- `validation_core::bias_standard_error` now forms the normalized standard error as `sqrt(sum(d²) / (n * (n - 1)))` instead of separately rounding `sqrt(sample_variance)` and `sqrt(n)` before division. This removes an avoidable one-ULP shift in representable Validation Evidence uncertainty while retaining the existing scale-before-square overflow protection and fail-closed range checks.
- The corrected public oracle for the three-observation subtraction-collapse boundary is `0x3c72_79a7_4590_331c`; the predecessor `...331d` value came from the discarded double-rounded square-root path.
