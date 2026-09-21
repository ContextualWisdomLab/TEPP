### Fixed

- Preserve a representable mean signed bias when individual `recovered - truth` residuals overflow but opposing represented input terms cancel. The fallback keeps the original paired-observation denominator, preserves minimum-subnormal results after extreme cancellation, and still fails closed when the final mean bias itself is unrepresentable.
