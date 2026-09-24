### Scientific validation

- `model_selection::admit_recovery_replication_result` now turns only numerical fit/selection failures (`RecoveryCandidateFitFailed`, `NoSuccessfulFit`, `InvalidDiagnostic`) into failed scientific recovery replications. Split/identity/chronology/candidate-grid/configuration/training-state and authority errors continue to fail closed instead of being hidden in the Monte Carlo failure denominator. This is #709 failure-semantics plumbing for #680, not recovery acceptance evidence.
