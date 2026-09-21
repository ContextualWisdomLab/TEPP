### Validation

- `validation_core::promote_scientific_recovery` now requires the conservative recovery bound to remain **strictly inside** the caller-owned practical RMSE target. The gate reuses TEPP's exact represented residual-vs-`k × SE` comparator instead of adding normalized RMSE and uncertainty terms after common-scale projection, preventing a positive uncertainty contribution near `f64::MAX` from being rounded away into a false scientific promotion. The crate still defines no universal RMSE cutoff.
