# Wilson coverage evidence provenance

## Problem

`ValidationReport` historically retained only the empirical coverage proportion and two Wilson score endpoints. That compact projection is useful for presentation but is not sufficient provenance for a durable validation artifact: the empirical denominator and the caller-supplied Wilson critical value are lost. Endpoint-pair algebra can reject many impossible combinations, but it cannot reconstruct the exact finite-sample calculation and becomes deliberately non-identifying at exact boundary coverage.

For a coverage study with `k` covered intervals among `n` admitted interval/truth triples, TEPP's canonical producer uses `p = k / n` and a caller-supplied standard-normal critical value `z`. Two reports can therefore show the same represented `p` while having different denominators, and the same `p` can produce different Wilson bounds under different `z` values. A durable evidence record that omits `n`, `k`, or the critical-value semantics cannot independently recompute the interval it claims to preserve.

## Versioned carrier and envelope

`validation_core::WilsonCoverageEvidenceV1` is the versioned provenance carrier for the calculation. Its JSON contract emits `schema = "tepp.wilson_coverage_evidence.v1"`, fixed-width `u64` `sample_count` and `covered_count`, `critical_value_kind = "standard_normal_z"`, `interval_sidedness = "two_sided"`, the caller-supplied `normal_critical_value`, represented empirical coverage, and the canonical Wilson lower and upper endpoints.

The type does not infer or invent a nominal confidence-level label. The current producer accepts a numeric standard-normal critical value directly and evaluates the symmetric lower/upper Wilson roots, so v1 records that scientific input, its scale, and two-sided interpretation. A downstream product may display `z = 1.96` as a nominal two-sided 95% convention only when that claim is supplied by its own validated contract; the Validation Evidence carrier does not reverse-engineer a confidence label that the producer was never given.

`ValidationEvidenceV1` is the durable outer envelope. It retains the existing `ValidationReport` as a backward-compatible compact projection and nests `WilsonCoverageEvidenceV1` as its recomputable coverage provenance. Admission validates both artifacts and requires the report's empirical coverage, Wilson lower endpoint, and Wilson upper endpoint to equal the nested carrier exactly. Existing callers can continue using `ValidationReport`; durable v1 evidence can no longer pair a valid-looking projection with a different denominator or critical value.

## Canonical recomputation

The coverage implementation has one crate-private count-based Wilson authority. `wilson_coverage_interval` still accepts interval/truth triples, but it counts covered observations once and delegates the numeric interval calculation to `wilson_coverage_interval_from_counts`. `WilsonCoverageEvidenceV1` uses the same helper for construction and validation. This avoids a second copy of Wilson arithmetic and makes the durable carrier recomputable from the evidence it stores.

Artifact admission is exact for the represented binary64 contract. The carrier recomputes represented empirical coverage and both Wilson endpoints with the canonical producer and requires numeric equality. TEPP's JSON serializer emits round-trip-safe binary64 decimals, so a serialized artifact produced by this crate decodes to the same represented values. A changed denominator, covered count, critical value, projected coverage, or endpoint fails closed. Unknown fields, an unsupported schema version, a critical-value kind other than `standard_normal_z`, or sidedness other than `two_sided` also fail deserialization rather than being silently reinterpreted.

This exact recomputation contract is intentionally stronger than `ValidationReport`'s legacy endpoint-pair admission. The legacy report lacks enough provenance to reproduce the original interval and therefore uses necessary algebraic support checks; the versioned carrier has the missing denominator and critical value, so it can call the actual canonical producer instead of relying on a loose identity.

## Large-count binary64 boundary

Durable integer provenance creates a numerical obligation that the in-memory interval API normally cannot reach in practice. Binary64 has 53 bits of significand precision, so every integer is exactly representable only through `2^53`. At `n = 2^53 + 1` and `k = 2^53`, converting `k` and `n` independently to binary64 rounds both to `2^53`; the naive expression `(k as f64) / (n as f64)` therefore becomes exact `1.0` even though one admitted interval is uncovered. The same collapse also steers a count-based Wilson implementation onto an all-covered numerical path and materially changes the representable lower endpoint.

The corrected count authority preserves the smaller side of the binomial partition. For `k > n-k`, represented empirical coverage is formed as `1 - (n-k)/n`, and Wilson endpoints are evaluated on the uncovered proportion and reflected by the score interval's complement symmetry. At `n = 9,007,199,254,740,993`, `k = 9,007,199,254,740,992`, and `z = 1.96`, the durable contract records represented coverage `0x3fefffffffffffff` (`0.9999999999999999`) and Wilson lower endpoint `0x3feffffffffffffa` (`0.9999999999999993`), rather than falsely treating the count state as all-covered. The upper endpoint rounds to `1.0`, which is representable and does not erase the retained uncovered count because the integer provenance remains authoritative.

The v1 JSON count fields are `u64`, not `usize`. A durable schema must not change its numeric domain with the pointer width of the Rust process that reads it. The in-memory slice constructor safely widens its `usize` lengths/counts to `u64`; deserialization can therefore preserve the same versioned count artifact on 32-bit and 64-bit consumers even when the count could not be materialized as one process-resident slice.

IEEE 754-2019 remains the published active IEEE floating-point standard, and IEEE/ISO/IEC 60559-2020 is its active international adoption. IEEE P754 is an active revision project intended to supersede IEEE 754-2019, not a published replacement. This repair therefore grounds binary64 representation claims in the published 2019/2020 standards and does not treat the active P754 project as current normative authority.

## RED → repair trace

The first test draft `9fc96345f67ee0d6e6e8b62903b9994f13932a1d` contained a bad fixture and placeholder assertion and is not scientific evidence. Non-force correction `6f6e06d2446cc459cc29879c3e4bc34a2fff8e82` is the first valid RED for denominator/covered-count retention, standard-normal critical-value semantics, JSON round-trip, and fail-closed tampering.

The initial implementation lineage is:

- `ca517ed3755a11b4574f8909acd6965273cf69e9`: factor interval hit counting and count-based Wilson recomputation into one crate-private numeric authority while preserving public interval APIs;
- `31e1ab2bbaf9ce40cf74bed2110a310116e7a80a`: add `WilsonCoverageEvidenceV1` with validated construction, exact recomputation, schema-tagged manual serde, and fail-closed unknown-field behavior;
- `b6714d2365bbdbad0f127631edd8463f1829f0e2`: export the coverage carrier from `validation_core`;
- `02c8763fd0d921189b51fc437016d04505182e1b`: expand edge contracts for tampered counts/endpoints/probabilities, serializer refusal, invalid inputs, and square-overflowing `z`;
- `e9c6392604140b819a192bc2969636f62e770a0b` and `fdd24a1a9d3c9d22802f6a8ec6379e687009afe7`: bind and test `two_sided` interval semantics rather than leaving one-sided reinterpretation implicit;
- `07766cb0b475da3d5610b42577dd9a5e91038bc9`: public RED requiring a versioned durable envelope to bind `ValidationReport` coverage fields to the provenance carrier;
- `a16f22e674a0493fc0d49589e66e7ad66ea1e543`: add `ValidationEvidenceV1` with nested validation, exact projection identity, schema-tagged serde, and unknown-field refusal;
- export edit `4e2381f31a43d753cd8c37bb153aac701876c1ef` accidentally changed unrelated claim-authority wording and is not accepted repair evidence; non-force correction `57d8bf57687c5157c7c06c941a113071e22c5430` restores that wording while retaining only the intended module/export delta;
- `6e5467428b9de851f075cb251f9dc38a4ec6a728`: update release-facing change documentation for the carrier/envelope contract.

The large-count repair lineage is:

- `29d710a53d4988b46a37667b4ff03352d520b3c7`: public RED showing that a durable one-uncovered count state above `2^53` must decode to a non-all-covered represented coverage/Wilson artifact;
- `29968c807368fe3fe19bef3013af9d577d5f6025`: make empirical coverage complement-aware and make count-based Wilson evaluation reflect the smaller uncovered proportion near the all-covered boundary;
- `91d9a3bb54b47605377e5159763a55f3386efcf9`: make v1 durable counts fixed-width `u64` and route carrier construction/validation through the same count-preserving authority;
- `d908ce2a37685bdf915cc513be9f3f9dd36aae19`: add release-facing documentation for the count-precision and schema-portability repair.

Every later source or documentation commit on PR #488 invalidates predecessor exact-head workflow evidence; only the current surviving head's hosted gates and independent review count for landing.

## DDD and owner boundary

This is Validation Evidence provenance and projection policy. It does not define a new psychometric estimator, change the Wilson score estimand, move longitudinal/time-varying composition into `validation_core`, or copy mutable arithmetic from fast-mlsirm. The count-based helper is private to the existing TEPP coverage producer so there remains one numeric authority inside this bounded context.

The carrier and envelope do not involve semantic LLM execution. contextual-orchestrator remains the owner of model routing; no unreleased orchestrator source, direct provider route, or provider credential is introduced here.

## Standards and primary sources

Wilson's original score-interval paper remains the primary statistical source for the interval family used by this producer. The current published AERA/APA/NCME testing standards remain the 2014 edition; AERA, APA, and NCME have convened a Joint Committee to revise that edition, and AERA's Task Force roster was current as of August 31, 2026. An unpublished revision is not treated as present normative authority.

American Educational Research Association, American Psychological Association, & National Council on Measurement in Education. (2014). *Standards for educational and psychological testing*. American Educational Research Association.

Institute of Electrical and Electronics Engineers. (2019). *IEEE standard for floating-point arithmetic* (IEEE Std 754-2019). IEEE. https://standards.ieee.org/ieee/754/6210/

International Organization for Standardization, & International Electrotechnical Commission. (2020). *Information technology—Microprocessor systems—Floating-point arithmetic* (ISO/IEC 60559:2020). ISO. https://www.iso.org/standard/80985.html

Wilson, E. B. (1927). Probable inference, the law of succession, and statistical inference. *Journal of the American Statistical Association, 22*(158), 209–212. https://doi.org/10.1080/01621459.1927.10502953

## Verification contract

`crates/validation_core/tests/wilson_coverage_evidence_v1_contract.rs` covers the provenance carrier and `crates/validation_core/tests/wilson_coverage_count_precision_contract.rs` covers the large-count representation boundary. `crates/validation_core/tests/validation_evidence_v1_coverage_provenance_contract.rs` covers the durable report/provenance envelope. Exact-head Rust tests, rustdoc/docstring checks, owned line/branch coverage, documentation validation, security/SAST/supply-chain checks, and qualifying independent review remain required before the Draft landing vehicle can be promoted or merged.
