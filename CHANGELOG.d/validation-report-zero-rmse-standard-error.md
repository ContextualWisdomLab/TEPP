# Validation Evidence: zero RMSE standard-error coherence

`ValidationReport` now rejects an exact-zero point RMSE paired with a positive RMSE standard error. Under TEPP's squared-residual delta-method definition, exact-zero RMSE means every residual is exactly zero, so its RMSE standard error is also exactly zero. Signed zero remains one numerical zero-valued scientific state.
