### Fixed

- `validation_core::accept_within_standard_errors` now compares the exact represented binary64 residual magnitude with the exact represented `k × SE` product when both direct operations overflow. This prevents independently rounded scale normalization from turning a strict rejection into a false acceptance at the full-range boundary while preserving the adjacent admissible multiplier.
