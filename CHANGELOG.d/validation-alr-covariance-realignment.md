### Scientific validation

- Add truth-reference ALR covariance propagation for known-topic recovery. `validation_core::realign_additive_log_ratio_covariance(...)` applies the same topic/reference contrast as ALR location recovery through `A Σ Aᵀ`, validates exact finite symmetric positive-semidefinite geometry, and fails closed on non-finite factorization or transformed output. This is coordinate propagation only; calibrated interval coverage still requires repeated scientific recovery evidence.
