# Same-sign remainder compensation at the mean-bias division boundary

## Decision status

Proposed evidence for the Validation bounded context. This note records the represented-input numerical contract implemented on PR #488; it does not claim globally correctly rounded binary64 summation or division.

## Problem

`deterministic_representable_sum_over_count` cancels opposite signs before scale reduction. GAP-088 repaired the path where cancellation itself produced retained low terms, but exact opposite-sign cancellation can leave a same-sign remainder and no cancellation roundoff term. That path still called `same_sign_mean_over_total`, which formed the compensated numerator as `sum + correction` before dividing by the scientific count.

The public RED commit `84476aad7ef2918c174c1ef986cbec2851cac656` fixes the represented residual payload to:

- `0x1.0000000000004p-3`
- `0x1.ffffffffffffcp-4`
- `-0x1.ffffffffffffdp-4`

Their exact represented-input numerator is `9007199254740999 / 72057594037927936`; dividing by three gives `3002399751580333 / 72057594037927936`. The correctly rounded binary64 mean is bits `0x3fa555555555555a`. The predecessor returned `0x3fa555555555555b` because the Neumaier high part and correction were rounded back together before the count division. The sign-mirrored payload has the symmetric expected bits `0xbfa555555555555a`.

## Constraints

- Keep `mean_bias` as TEPP Validation Evidence semantics; do not move this decision rule into fast-mlsirm.
- Preserve the original paired-observation denominator even when cancellation rewrites the represented numerator.
- Do not add arbitrary-precision arithmetic to the production hot path for a boundary that binary64 FMA and retained compensation can resolve.
- Do not change `bias_standard_error`; this counterexample concerns only the represented mean-bias numerator/division path.
- Do not infer a general correctly-rounded summation guarantee from this repair.

## Alternatives considered

Forming `sum + correction` before division was rejected because it reproduces the observed one-ULP error. Always switching to arbitrary-precision rational arithmetic was rejected because the public metric is a deterministic binary64 reference and the failure is caused by one avoidable intermediate rounding boundary. Replacing the cancellation algorithm was also rejected because the counterexample shows that the cancellation result is already correct; the defect is downstream in the same-sign remainder division.

## Decision

Causal repair `5f0d40b838e9b9867ec40281f1e4ab6db96a12cb` changes only `same_sign_mean_over_total`. It keeps the canonical Neumaier high part and correction separate, divides the high part by the original scientific denominator, recovers the division residual with binary64 FMA, and then carries that residual and the retained correction through the same denominator before restoring the exact power-of-two scale. This mirrors the already-established GAP-088 division boundary without changing the cancellation owner or public estimand.

## Expected effect and risk

The RED and its sign mirror now select the represented-input result rather than the predecessor's one-ULP neighbor. Existing same-sign and exact-cancellation paths retain their scientific denominator and scale policy. Remaining risk is intentionally bounded: `division_residual + correction` and the final correction addition are still binary64 operations, so unrelated payloads may require separate evidence before any stronger rounding claim is made.

## Traceability

- PR: #488, `fix/validation-bias-overflow-safe-mean`
- Public RED: `84476aad7ef2918c174c1ef986cbec2851cac656`
- Production repair: `5f0d40b838e9b9867ec40281f1e4ab6db96a12cb`
- Changelog: `5763cde0292b05fda4ebc07436e31e1309e97572`
- Production module: `crates/validation_core/src/numeric.rs`
- Public contract: `crates/validation_core/tests/bias_same_sign_remainder_division_contract.rs`

## Standards and methodological authority

IEEE 754-2019 remains the active IEEE floating-point standard as verified on 2026-09-05; IEEE P754 is an active revision PAR and not a published replacement. ISO/IEC 60559:2020 remains a published International Standard. The AERA/APA/NCME published testing standards remain the 2014 edition while the Joint Committee revision proceeds. Morris, White, and Crowther (2019) remains the methodological authority used here for defining simulation performance measures against known truth; it does not prescribe this implementation detail.

## References

American Educational Research Association, American Psychological Association, & National Council on Measurement in Education. (2014). *Standards for educational and psychological testing*. American Educational Research Association.

IEEE. (2019). *IEEE standard for floating-point arithmetic* (IEEE Std 754-2019). https://doi.org/10.1109/IEEESTD.2019.8766229

International Organization for Standardization & International Electrotechnical Commission. (2020). *Information technology—Microprocessor systems—Floating-point arithmetic* (ISO/IEC 60559:2020).

Morris, T. P., White, I. R., & Crowther, M. J. (2019). Using simulation studies to evaluate statistical methods. *Statistics in Medicine, 38*(11), 2074–2102. https://doi.org/10.1002/sim.8086
