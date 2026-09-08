### Changed

- Simplified exact-count all-covered Wilson midpoint selection by removing the redundant `exact_residual != 0` wrapper. A zero FMA quotient residual cannot cross a positive adjacent-float midpoint, so the existing midpoint comparison already preserves the direct endpoint without a separate branch. This keeps represented-input Wilson results unchanged while removing a structurally non-causal coverage arm.
