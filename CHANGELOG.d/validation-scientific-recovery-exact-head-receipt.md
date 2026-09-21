### Fixed

- Scientific recovery promotion no longer accepts a detached `ExactHeadTests=true` boolean as sufficient exact-head evidence. The specialized Validation Evidence boundary now requires an exact-head test receipt bound to the tested Git commit, a canonical immutable receipt SHA-256, and an explicit passed/failed/queued/skipped state.
- `ScientificRecoveryPromotionV1` retains the exact-head receipt SHA-256 beside the recovery-profile SHA-256. Predecessor, failed, queued, skipped, and malformed receipt evidence fails closed before final ADR 0014 authority is minted.
- Receipt contents remain trusted-adapter evidence: the SHA-256 binds immutable receipt identity but does not by itself prove CI truth. Recovery RMSE/MCSE arithmetic, practical targets, independent-replication grouping, planned denominator, and profile semantics are unchanged.
