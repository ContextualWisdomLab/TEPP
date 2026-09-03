### Fixed

- `validation_core::ValidationReport` now rejects finite but scientifically impossible metric payloads: negative RMSE/standard errors, coverage or temporal-order accuracy outside `[0, 1]`, invalid Wilson endpoints, and Wilson intervals that do not contain the empirical coverage recorded in the same report.
- Explicit validation, canonical JSON helpers, direct serde serialization, and serde deserialization now enforce the same report invariants, so neither wire ingress nor an alternate serialization call can bypass the durable Validation Evidence contract.
- Direct serde serialization of `MonteCarloSummary` now applies its existing count/finiteness/nonnegative-uncertainty/percentile-order invariants before writing fields, matching its fail-closed deserialization contract.
