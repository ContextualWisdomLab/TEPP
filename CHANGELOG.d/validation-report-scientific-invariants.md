### Fixed

- `validation_core::ValidationReport` now rejects finite but scientifically impossible metric payloads: negative RMSE/standard errors, coverage or temporal-order accuracy outside `[0, 1]`, invalid Wilson endpoints, and Wilson intervals that do not contain the empirical coverage recorded in the same report.
- JSON deserialization now applies the same report invariants as explicit validation and serialization, so invalid Validation Evidence cannot enter through the wire path while egress remains fail closed.
