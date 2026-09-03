# Validation report scientific-invariant boundary

## Finding

`validation_core::ValidationReport` previously treated finiteness as sufficient payload validity. That allowed finite but impossible Validation Evidence to be serialized or deserialized: negative RMSE or standard errors, empirical coverage and temporal-order accuracy outside `[0, 1]`, Wilson endpoints outside the probability domain, inverted Wilson intervals, or an empirical coverage value outside the Wilson interval stored beside it.

The same boundary review found that `MonteCarloSummary` already rejected impossible values on explicit validation and serde ingress, but its direct serde serializer wrote public fields without applying that contract. A caller could therefore serialize a negative standard deviation or otherwise invalid summary directly even though the same payload would be rejected on re-ingress.

These defects are evidence-admission/projection problems rather than new estimators. The report and Monte Carlo summary are durable boundaries over already-computed recovery metrics, so their responsibility is to preserve the metric domains and cross-field relationships established by the producing functions. Mean signed bias is intentionally not sign-restricted.

## Decision

The report boundary now enforces the following invariants on explicit validation, canonical JSON egress, direct serde serialization, and JSON ingress:

- `rmse >= 0`, `rmse_standard_error >= 0`, and `bias_standard_error >= 0`;
- empirical coverage, Wilson endpoints, and temporal-order accuracy are each in `[0, 1]`;
- `coverage_wilson_lower <= coverage_wilson_upper`;
- `coverage_wilson_lower <= interval_coverage <= coverage_wilson_upper`;
- all numeric fields remain finite and an embedded Monte Carlo summary must satisfy its existing validation contract.

Custom `Serialize` and `Deserialize` implementations validate `ValidationReport` at both wire boundaries. `MonteCarloSummary` direct serialization now also invokes its existing validator before writing fields. This prevents callers from bypassing durable evidence contracts by choosing a different serde path instead of the explicit validation helpers.

## RED -> repair trace

- Public RED `e9414e6f7b824c2dc508f335d355b97b7e399b9`: impossible finite metric domains and incoherent Wilson evidence must fail `ValidationReport::validate()` / `to_json()`.
- First causal repair `28924b0d82bc2d4663f5ba1317cc0c3d94a4833b`: enforce metric-domain and Wilson coherence invariants on the report object.
- Ingress RED `6d190cd783a00f8ce917e37b63ae87053a929ae1`: serde deserialization must not bypass the same scientific contract.
- Ingress repair `f70a6fc0e58c2ae419c3b3bac322db5f35efe538`: custom `Deserialize` constructs then validates the report before admission.
- Direct-report-egress RED `fb28f959297a50cf50e26ea14dc9bfb5ee10ea89`: `serde_json::to_string(&report)` must not bypass report validation.
- Direct-report-egress repair `f7e58ccdb1864ce775ef6797f7fd88596dff1269`: custom `Serialize` validates before writing any report field.
- Direct-Monte-Carlo-egress RED `b5a8ae1fbd1344b09a9fbb3c65a3c6d0cdc38c28`: an invalid `MonteCarloSummary` must not serialize directly despite its public fields.
- Direct-Monte-Carlo-egress repair `18374c35061fb0b98bfe09079d4967baef7697b4`: the existing summary validator is applied before serde writes fields.
- Changelog trace `0d03efa1474e6ce605db32980c3f65e97e0dc66b`.

Owned module/API/tests:

- `crates/validation_core/src/report.rs`
- `validation_core::ValidationReport::validate`
- `validation_core::ValidationReport::to_json`
- `serde::Serialize` / `serde::Deserialize` for `ValidationReport`
- `serde::Serialize` / existing `serde::Deserialize` for `MonteCarloSummary`
- `crates/validation_core/tests/validation_report_scientific_invariants_contract.rs`

## Methodological basis

The 2014 *Standards for Educational and Psychological Testing* remain the current published AERA/APA/NCME edition as of 2026-09-04; the sponsoring organizations announced a revision process in 2024 rather than a replacement published edition. The Standards' validity framework makes interpretation and use of reported scores/evidence contingent on appropriate evidence and coherent reporting, which supports fail-closed handling of impossible metric artifacts rather than treating mere machine representability as scientific validity.

Morris, White, and Crowther (2019) frame simulation performance measures as quantities tied to explicit estimands/targets, emphasize unambiguous definitions, coverage, bias, and Monte Carlo uncertainty, and recommend checks during coding and execution. TEPP therefore treats metric-domain constraints as part of the Validation Evidence contract, not cosmetic post-processing.

ISO/IEC 25012:2008 defines a data-quality model for structured data and was last reviewed and confirmed in 2025, so it remains current as of 2026-09-04. TEPP uses that standard only as a data-quality support for enforcing validity/consistency requirements at the artifact boundary; it does not replace psychometric validity theory.

## References

American Educational Research Association, American Psychological Association, & National Council on Measurement in Education. (2014). *Standards for educational and psychological testing*. American Educational Research Association. https://www.testingstandards.net/open-access-files.html

American Educational Research Association. (2024, June 12). *Members of the Joint Committee for the Revision of the Standards for Educational and Psychological Testing named*. https://www.aera.net/Newsroom/Members-of-the-Joint-Committee-for-the-Revision-of-the-Standards-for-Educational-and-Psychological-Testing-Named

International Organization for Standardization. (2008). *ISO/IEC 25012:2008 Software engineering—Software product Quality Requirements and Evaluation (SQuaRE)—Data quality model*. https://www.iso.org/standard/35736.html

Morris, T. P., White, I. R., & Crowther, M. J. (2019). Using simulation studies to evaluate statistical methods. *Statistics in Medicine, 38*(11), 2074–2102. https://doi.org/10.1002/sim.8086
