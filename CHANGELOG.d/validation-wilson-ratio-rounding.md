### Fixed

- `validation_core` now rounds durable covered/sample count ratios directly from their exact `u64` provenance instead of first rounding each integer to binary64. This removes one-ULP empirical-coverage errors above binary64's exact-integer range and feeds the Wilson score producer the correctly rounded represented proportion while retaining complement-symmetric evaluation near the all-covered boundary.
