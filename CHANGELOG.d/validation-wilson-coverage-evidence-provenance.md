# Validation Evidence: versioned Wilson coverage provenance

- Add `WilsonCoverageEvidenceV1`, a schema-tagged durable carrier that retains empirical `sample_count`, `covered_count`, the caller-supplied standard-normal `z`, represented coverage, and canonical Wilson endpoints.
- Fix the interval interpretation as `critical_value_kind = "standard_normal_z"` and `interval_sidedness = "two_sided"`; v1 records the producer input and sidedness rather than inventing a nominal confidence-level label that the API never received.
- Recompute coverage and Wilson bounds from stored counts and `z` during artifact admission so denominator, critical-value, projection, or endpoint tampering fails closed.
- Add `ValidationEvidenceV1` as the versioned durable envelope that binds the existing `ValidationReport` projection to the recomputable Wilson coverage provenance; legacy report callers remain source-compatible.
- Keep one crate-private count-based Wilson numeric authority shared by the existing interval API and the versioned carrier; no estimator target, Longitudinal Modeling semantics, fast-mlsirm ownership, or contextual-orchestrator routing contract changes.
