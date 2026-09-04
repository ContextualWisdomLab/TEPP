# Validation Evidence: versioned Wilson coverage provenance

- Add `WilsonCoverageEvidenceV1`, a schema-tagged durable carrier that retains empirical `sample_count`, `covered_count`, the caller-supplied standard-normal `z`, represented coverage, and canonical Wilson endpoints.
- Recompute coverage and Wilson bounds from stored counts and `z` during artifact admission so denominator, critical-value, projection, or endpoint tampering fails closed.
- Keep one crate-private count-based Wilson numeric authority shared by the existing interval API and the versioned carrier; no estimator target or Longitudinal Modeling semantics change.
- Preserve `critical_value_kind = "standard_normal_z"` explicitly instead of inferring an unrecorded nominal confidence-level claim.
