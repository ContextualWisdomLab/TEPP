# Standard-error acceptance both-overflow exact comparison

## Problem and scientific contract

TEPP's Validation bounded context exposes `accept_within_standard_errors(estimate, target, standard_error, k)` as the deterministic CPU `f64` reference for the decision

`|estimate - target| <= k * standard_error`.

The preceding repair made finite represented residuals and finite represented `k × SE` bounds direct-first, but retained scale normalization when both direct operations overflowed. Fresh represented-input review found that the remaining fallback can round both normalized sides to the same binary64 value even when the exact rational values represented by the original four binary64 inputs are strictly ordered.

Public RED `a9a45714eb6266f6b922086112b313060e77c522` uses these exact binary64 payloads:

- `estimate = 0x1.e446cf80dddbcp+1023` (`0x7fee446cf80dddbc`),
- `target = -0x1.2d7e397966af3p+1023` (`0xffe2d7e397966af3`),
- `standard_error = 0x1.20ad22ddb6f38p+823` (`0x73620ad22ddb6f38`), and
- `k = 0x1.5c69ac1c7a9edp+201` (`0x4c85c69ac1c7a9ed`).

Both `estimate - target` and `k * standard_error` overflow to positive infinity in magnitude. The predecessor then normalized by the largest magnitude and obtained the same rounded binary64 value for the residual and bound (`0x1.9f6056b74e5cap+0`), so it accepted. An exact rational comparison of the represented inputs shows the bound is smaller than the residual; the difference is approximately `3.2267077731482595e292`. The immediately adjacent multiplier `0x1.5c69ac1c7a9eep+201` crosses the represented-input boundary and must remain accepted.

## Causal repair and constraints

Causal repair `425e89638d594fdb2f3586d73f021b60a530b456` keeps the existing direct-first cases and changes only the both-overflow branch. It decodes each finite binary64 magnitude into its exact integer significand and power-of-two exponent. The opposite-sign residual becomes an exact sum of two at-most-53-bit significands; `k × SE` becomes an exact at-most-106-bit significand product. The comparison then aligns those integer values by powers of two inside `u128`, avoiding another floating-point normalization step.

This bounded method is possible because direct finite subtraction can overflow only for opposite-sign inputs near the top of binary64 range. Their exact significands therefore require at most one 53-bit-width alignment shift. The product of two binary64 significands needs at most 106 bits, so the complete comparison remains allocation-free and deterministic in Rust.

Always retaining the scale-normalized fallback was rejected because it is the demonstrated cause of the false acceptance. Log-domain comparison was rejected because transcendental rounding is unnecessary for values that already have exact binary decompositions. Arbitrary-precision runtime arithmetic was also rejected: the relevant exact integers fit `u128`, so a heap-allocated big-number dependency would add cost and supply-chain surface without improving this bounded decision.

The nearby adjacent-multiplier contract prevents the repair from turning the both-overflow region into a blanket rejection. CHANGELOG trace: `e62706291ad556c6d9078a2bc50dcf3bca5a6feb`.

## Ownership, validation, and non-claims

This is TEPP Validation decision semantics, not reusable static psychometric estimation. It does not move arithmetic owned by `fast-mlsirm`, does not consume mutable sibling source, and does not depend on an unreleased `contextual-orchestrator` contract.

The RED and adjacent control are public Rust contract tests. An independent exact-rational search was used only to discover and verify the counterexample; it is not production arithmetic, does not replace the Rust decision path, and is not scientific acceptance evidence by itself.

This repair claims exact represented-input comparison only for the existing both-overflow branch of `accept_within_standard_errors`. It does not claim that unrelated Wilson, Monte Carlo summary, strict-interior interval, or other validation formulas are globally correctly rounded.

## Standards trace

IEEE. (2019). *IEEE standard for floating-point arithmetic* (IEEE Std 754-2019). https://standards.ieee.org/ieee/754/6210/

IEEE 754-2019 remains an Active Standard as checked on 2026-09-04. IEEE P754, approved as a PAR on 2024-06-06, is an Active PAR intended to supersede 754-2019 and is not treated here as a published replacement. https://standards.ieee.org/ieee/754/11684/

International Organization for Standardization. (2020). *Information technology—Microprocessor systems—Floating-point arithmetic* (ISO/IEC 60559:2020). https://www.iso.org/standard/80985.html

ISO/IEC 60559:2020 remains Published, stage 60.60, as checked on 2026-09-04.

American Educational Research Association, American Psychological Association, & National Council on Measurement in Education. (2014). *Standards for educational and psychological testing*. American Educational Research Association.

AERA, APA, and NCME announced the Joint Committee charged with revising the 2014 edition on 2024-06-12. As checked on 2026-09-04, that announcement still describes the committee as revising the 2014 edition; TEPP therefore continues to use the 2014 published edition rather than treating an unpublished revision as normative authority. https://www.aera.net/Newsroom/Members-of-the-Joint-Committee-for-the-Revision-of-the-Standards-for-Educational-and-Psychological-Testing-Named
