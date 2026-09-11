# Exact-count Wilson residual midpoint selection

## Problem

For an all-covered sample whose retained count is exactly representable in binary64, the Wilson lower endpoint reduces algebraically to

\[
L = \frac{n}{n + z^2}.
\]

`coverage.rs` already preserves the TwoSum residual of `n + z²` and an FMA quotient residual. The previous repair converted that residual into an additive binary64 quotient correction. That is not sufficient for correct final rounding: when the exact represented-input quotient differs from the direct hardware quotient by less than the midpoint to the adjacent binary64 value, rounding the correction separately can still produce a one-ULP step.

Public RED `d076d344cb3cd3a768e7a5c2d8d7bd9039c657e9` fixes a concrete finite case: `n = 3`, represented `z = 0x1.6a09e667f3bcdp+492`, and represented `z² = 0x1.0000000000001p+985`. The rounded denominator is exactly `z²`, so the count contribution is present only in the TwoSum residual. The direct quotient is `0x1.7ffffffffffffp-984`; the exact rational formed from the already represented inputs `3 / (3 + z²)` rounds to that same binary64 value. The predecessor additive correction instead returned `0x1.7fffffffffffep-984`, one ULP too low.

## Constraints and alternatives

The repair stays in TEPP Validation Evidence because it governs representation of Wilson evidence emitted by the canonical `validation_core::coverage` producer. It does not introduce reusable static psychometric estimation and does not copy fast-mlsirm source.

Always applying the additive correction was rejected because the RED demonstrates a double-rounding failure at the quotient scale. Always keeping the direct quotient was rejected because earlier exact-count cases demonstrate the opposite state: denominator rounding can move the exact represented-input endpoint across an adjacent binary64 midpoint. Replacing the whole Wilson implementation with a second arbitrary-precision writer was rejected because it would violate the single-writer boundary and widen the causal surface.

## Selected repair

Causal repair `32314239754204158f228ec67a0771abf4d39b45` keeps the rounded denominator, its TwoSum residual, and the FMA quotient residual. When the denominator residual is nonzero, it uses the residual sign only to identify the adjacent candidate and compares residual magnitude with the exact-denominator midpoint distance represented as

\[
\tfrac12\,\operatorname{ulp}_{direction}(q)\,(D + \delta D).
\]

The dominant `ulp * D` product is evaluated through FMA with the residual term. The quotient changes by one adjacent binary64 value only when the represented-input rational lies beyond that midpoint; an exact midpoint follows ties-to-even from the direct quotient significand. The existing `direct_lower == 1.0` complementary-miss boundary remains separate because the adjacent uncertainty can be lost before a usable direct quotient exists.

Dedicated CHANGELOG evidence is `a75940c955633d5eb92f227d79e64acd0ef46ea8`.

## Scope and risk

This closes the demonstrated exact-count all-covered residual-overcorrection state. It does not claim globally correctly rounded Wilson endpoints for strict-interior coverage, inexact durable counts, or algebraically different Wilson forms. Those paths require their own represented-input counterexamples before any gate is strengthened.

The main remaining risk is midpoint comparison at binary64 spacing transitions and tie states. Existing predecessor tests cover a large-`z²` case where one-ULP movement is required, a power-of-two large-`z` case where it is not, near-one exact-count recovery, and partial denominator rounding. The new RED covers the opposite extreme-scale state where a real denominator residual is below final quotient resolution.

## Methodological trace

Wilson, E. B. (1927). Probable inference, the law of succession, and statistical inference. *Journal of the American Statistical Association, 22*(158), 209–212. https://doi.org/10.1080/01621459.1927.10502953

IEEE Std 754-2019 and ISO/IEC 60559:2020 remain the published floating-point authorities used for binary64 round-to-nearest, ties-to-even reasoning. The scientific acceptance policy remains anchored to the published AERA/APA/NCME *Standards for Educational and Psychological Testing* while the successor revision is still under development.
