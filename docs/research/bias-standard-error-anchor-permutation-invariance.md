# Bias standard error exact-anchor permutation invariance

## Problem

GAP-105 fixed exact rational-scale projection at the normal/subnormal boundary, but the larger exact-translated residual path still selected `diffs[0]` as its translation anchor. Translation does not change a sample standard error, so the numerical admission decision must not depend on transport order. The predecessor did: if the first represented residual could not be subtracted exactly from every other represented residual, it abandoned the exact translated path even when another represented observation was a valid exact anchor.

The public counterexample uses three exactly represented residuals with `truth = [0, 0, 0]`:

- `low = f64::from_bits(0x4194_f788_9184_b980) = 46106761431411 / 524288`,
- `middle = f64::from_bits(0x420c_409f_fce3_8390) = 497022202165305 / 32768`,
- `high = f64::from_bits(0x4222_70c4_634c_c6b6) = 2595269181334363 / 65536`.

For these represented inputs, the exact squared standard error of the mean is

`327877142843256291246417577647793 / 2473901162496`,

whose correctly rounded binary64 square root is `0x4205_7185_8078_f946`.

With `middle` as the first residual, both `low - middle` and `high - middle` are exact binary64 differences, so the predecessor admits the translated second moment and returns `0x4205_7185_8078_f946`. With `low` first, `high - low` has a nonzero error-free subtraction tail of `2^-19`; with `high` first, `low - high` has the mirrored `-2^-19` tail. Those orderings reject the translated path and fall back to rounded-mean dispersion, returning adjacent lower bits `0x4205_7185_8078_f945`.

The estimand and represented multiset are identical in every ordering. A one-ULP result change caused only by which observation is first is therefore a deterministic arithmetic defect, not Monte Carlo uncertainty.

## Constraints

- TEPP owns Validation Evidence performance-measure arithmetic. Reusable static psychometric estimators remain in `fast-mlsirm`.
- The repair must preserve the existing exact `high + low` translated-residual admission. It must not declare an inexact delta exact or weaken fail-closed behavior.
- Observation order is not scientific evidence and cannot select a different numerical estimator path.
- Production arithmetic remains deterministic Rust binary64; no arbitrary-precision runtime dependency is introduced.
- Existing two-level algebraic shortcuts, normal/subnormal projection policy, and fallback semantics remain intact.

## Decision

Exact translated-residual admission now considers every represented observation as a possible anchor, but candidate anchors are examined in canonical `(high, low)` binary64 total order. For each candidate, the implementation requires all three existing proofs to remain true for every observation:

1. `high - anchor_high` is finite and error-free,
2. `low - anchor_low` is finite and error-free,
3. recombining the exact high and low deltas is finite and error-free.

The first candidate in canonical order satisfying all three conditions supplies the translated residual vector. If no candidate satisfies them, the predecessor rounded-residual fallback remains authoritative.

Canonical candidate ordering matters. Merely scanning observations in incoming order would fix the specific low-first payload only when a viable anchor happens to occur early, and multiple viable anchors could still make path selection transport-order dependent. Sorting the represented `(high, low)` candidate keys before admission makes the anchor choice a function of the represented multiset rather than array order.

## Alternatives rejected

1. **Keep `diffs[0]` as the anchor.** Rejected because the public contract demonstrates a one-ULP permutation violation for the same represented sample.
2. **Always choose the median high residual.** Rejected because subtraction low terms are part of the exact residual representation; a high-part median is not guaranteed to be an exact anchor for both high and low deltas.
3. **Scan anchors in incoming order and stop at the first exact candidate.** Rejected because the chosen exact anchor would still depend on transport order when multiple candidates are viable.
4. **Sort the whole observation sample before all Validation Evidence arithmetic.** Rejected as broader than the defect and capable of changing unrelated pairing/provenance assumptions.
5. **Use arbitrary-precision arithmetic for every standard error.** Rejected as substantially broader than this bounded exact-admission defect.

## Evidence and traceability

- Public RED: `5cb45a4f204ab4fbcd5581c4d4504e82f0339a30`
- Causal source repair: `159659a9510a7ced437ad872d02e26619abc8236`
- CHANGELOG fragment: `7e57b930daebb01b3583b4a5108ce3c1a89a06a6`
- Module/API: `crates/validation_core/src/bias.rs` → `bias_standard_error` → `exact_translated_residual_standard_error`
- Public contract: `crates/validation_core/tests/bias_standard_error_anchor_permutation_contract.rs`
- Correct represented result: `0x4205_7185_8078_f946`
- Predecessor low/high-anchor result: `0x4205_7185_8078_f945`
- Middle-anchor predecessor result: `0x4205_7185_8078_f946`
- Sign-mirrored permutations must produce the same positive standard error.

The contract is intentionally a represented-input arithmetic acceptance test. It does not claim that every `n > 2` standard error is globally correctly rounded, and it does not expand the set of deltas treated as exact. Hosted exact-head CI remains the authority for GREEN after the surviving branch head is known.

## Standards and methodological basis

IEEE 754-2019 defines the binary floating-point arithmetic model used by the Rust `f64` reference path. ISO/IEC 60559:2020 adopts that floating-point model internationally. The relevant engineering requirement here is not a new statistical estimator: it is deterministic use of the same represented input multiset without making an incidental array position numerically authoritative.

Morris, White, and Crowther (2019) distinguish deterministic simulation performance measures from Monte Carlo uncertainty. Bias and empirical standard-error evidence should therefore not acquire extra variation from observation permutation inside the implementation.

The currently published AERA/APA/NCME *Standards for Educational and Psychological Testing* remains the 2014 edition while the announced revision is in progress. The published edition is retained as the normative testing reference until a replacement is actually issued.

## References

American Educational Research Association, American Psychological Association, & National Council on Measurement in Education. (2014). *Standards for educational and psychological testing*. American Educational Research Association. https://www.testingstandards.net/open-access-files.html

Institute of Electrical and Electronics Engineers. (2019). *IEEE standard for floating-point arithmetic* (IEEE Std 754-2019). IEEE Standards Association. https://standards.ieee.org/ieee/754/6210/

International Organization for Standardization, & International Electrotechnical Commission. (2020). *Information technology—Microprocessor systems—Floating-point arithmetic* (ISO/IEC 60559:2020). https://www.iso.org/standard/80985.html

Morris, T. P., White, I. R., & Crowther, M. J. (2019). Using simulation studies to evaluate statistical methods. *Statistics in Medicine, 38*(11), 2074–2102. https://doi.org/10.1002/sim.8086
