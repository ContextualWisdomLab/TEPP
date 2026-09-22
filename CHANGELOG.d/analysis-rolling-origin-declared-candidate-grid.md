# Rolling-origin recovery binds the declared candidate-K grid

Scientific rolling-origin recovery now has a dedicated `RollingOriginRecoveryEvaluation` / `select_declared_rolling_origin_recovery_candidate_k(...)` path. Every evaluation window must expose exactly the candidate topic counts predeclared by `FittedCandidateKConfig` before predictive aggregation can select a winner.

A candidate that fails fitting and disappears from every window therefore cannot silently shrink the recovery design. Missing or undeclared fitted dimensions fail closed with `ModelSelectionError::PredictiveCandidateGridMismatch`; the recovery harness can count that replication as failed in `SelectedKRecoverySummary` instead of conditioning the scientific design on optimizer survival.

The existing generic multi-window predictive selector keeps its operational survivor semantics. This change does not manufacture failed fits or mint Evidence, Membership, relation, cutoff, or release authority. See #680 and #693.
