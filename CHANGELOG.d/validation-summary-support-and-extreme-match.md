### Fixed

- `validation_core::MonteCarloSummary` now rejects zero-sample-spread artifacts whose empirical percentile support is not degenerate at the represented mean. Zero spread means every retained replication is identical, so a durable summary cannot simultaneously claim `SD = 0` and percentile support away from its mean; signed zero remains one numeric zero-valued state.
- `validation_core::match_count` now decides finite absolute-tolerance matches without requiring an unrepresentable residual magnitude. Opposite-sign finite extremes whose subtraction overflows are deterministically mismatches for every finite tolerance, while `absolute_residuals` continues to fail closed when the residual value itself is requested.
