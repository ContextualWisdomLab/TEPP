# Wilson coverage evidence provenance

## Problem

`ValidationReport` historically retained only the empirical coverage proportion and two Wilson score endpoints. That projection is useful for presentation but is not sufficient provenance for a durable validation artifact: the empirical denominator and the caller-supplied Wilson critical value are lost. Endpoint-pair algebra can reject many impossible combinations, but it cannot reconstruct the exact finite-sample calculation and becomes deliberately non-identifying at exact boundary coverage.

For a coverage study with `k` covered intervals among `n` admitted interval/truth triples, TEPP's canonical producer uses `p = k / n` and a caller-supplied standard-normal critical value `z`. Two reports can therefore show the same represented `p` while having materially different denominators, and the same `p` can produce different Wilson bounds under different `z` values. A durable evidence record that omits `n`, `k`, or the critical-value semantics cannot independently recompute the interval it claims to preserve.

## Versioned carrier

`validation_core::WilsonCoverageEvidenceV1` is the first versioned provenance carrier for this calculation. Its JSON contract emits:

- `schema = "tepp.wilson_coverage_evidence.v1"`;
- `sample_count` and `covered_count`;
- `critical_value_kind = "standard_normal"` is not used; the exact contract is `critical_value_kind = "standard_normal_z"`;
- the caller-supplied `normal_critical_value`;
- the represented empirical coverage;
- the canonical Wilson lower and upper endpoints.

The type does not infer or invent a nominal confidence-level label. The current producer accepts a numeric standard-normal critical value directly, so v1 records that scientific input and its scale exactly. A UI or downstream report may describe `z = 1.96` as a nominal two-sided 95% convention only when that interpretation is supplied by its own validated contract; the Validation Evidence carrier does not reverse-engineer a confidence claim that the producer was never given.

## Canonical recomputation

The coverage implementation now has one crate-private count-based Wilson authority. `wilson_coverage_interval` still accepts interval/truth triples, but it counts covered observations once and delegates the numeric interval calculation to `wilson_coverage_interval_from_counts`. `WilsonCoverageEvidenceV1` uses the same helper for construction and validation. This avoids a second copy of Wilson arithmetic and makes the durable carrier recomputable from the evidence it stores.

Artifact admission is exact for the represented binary64 contract. The carrier recomputes `covered_count / sample_count` and both Wilson endpoints with the canonical producer and requires numeric equality. TEPP's own JSON serializer emits round-trip-safe binary64 decimals, so a serialized artifact produced by this crate decodes to the same represented values. A changed denominator, covered count, critical value, projected coverage, or endpoint fails closed. Unknown schema fields, an unsupported schema version, and a critical-value kind other than `standard_normal_z` also fail deserialization rather than being silently reinterpreted.

This exact recomputation contract is intentionally stronger than `ValidationReport`'s legacy endpoint-pair admission. The legacy report has insufficient provenance to reproduce the original interval and therefore uses necessary algebraic support checks; the versioned carrier has the missing denominator and critical value, so it can use the actual canonical producer instead of a loose identity.

## RED → repair trace

The first test draft `9fc96345f67ee0d6e6e8b62903b9994f13932a1d` contained a bad fixture and placeholder assertion and is not scientific evidence. Non-force correction `6f6e06d2446cc459cc29879c3e4bc34a2fff8e82` is the valid RED: the public contract requires denominator/covered-count retention, standard-normal critical-value semantics, canonical JSON round-trip, and fail-closed tampering.

The implementation lineage is:

- `ca517ed3755a11b4574f8909acd6965273cf69e9`: factor interval hit counting and count-based Wilson recomputation into one crate-private numeric authority while preserving the public interval APIs;
- `31e1ab2bbaf9ce40cf74bed2110a310116e7a80a`: add `WilsonCoverageEvidenceV1` with validated construction, exact recomputation, schema-tagged manual serde, and fail-closed unknown-field behavior;
- `b6714d2365bbdbad0f127631edd8463f1829f0e2`: export the versioned carrier from `validation_core`;
- `02c8763fd0d921189b51fc437016d04505182e1b`: expand public edge contracts for tampered counts, endpoints, probabilities, schema/kind mismatch, serializer refusal, invalid input triples, and square-overflowing `z`.

Every later source or documentation commit on PR #488 invalidates predecessor exact-head workflow evidence; only the current head's hosted gates and independent review count for landing.

## DDD and owner boundary

This is Validation Evidence provenance and projection policy. It does not define a new psychometric estimator, change the Wilson score estimand, move longitudinal/time-varying composition into `validation_core`, or copy mutable arithmetic from fast-mlsirm. The count-based helper is private to the existing TEPP coverage producer so there remains one numeric authority inside this bounded context.

The carrier also does not involve semantic LLM execution. contextual-orchestrator remains the owner of model routing, and no unreleased orchestrator source or provider credential is introduced here.

## Standards and primary sources

Wilson's original score-interval paper remains the primary statistical source for the interval family used by this producer. The current published AERA/APA/NCME testing standards remain the 2014 edition; AERA, APA, and NCME have convened a Joint Committee to revise that edition, and the AERA Task Force roster was current as of August 31, 2026. An unpublished revision is not treated as present normative authority.

American Educational Research Association, American Psychological Association, & National Council on Measurement in Education. (2014). *Standards for educational and psychological testing*. American Educational Research Association.

Wilson, E. B. (1927). Probable inference, the law of succession, and statistical inference. *Journal of the American Statistical Association, 22*(158), 209–212. https://doi.org/10.1080/01621459.1927.10502953

## Verification contract

`crates/validation_core/tests/wilson_coverage_evidence_v1_contract.rs` is the public regression surface. Exact-head Rust tests, rustdoc/docstring checks, line/branch coverage, documentation validation, security/SAST/supply-chain checks, and qualifying independent review remain required before the Draft landing vehicle can be promoted or merged.
