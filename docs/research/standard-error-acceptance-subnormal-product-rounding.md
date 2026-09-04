# Standard-error acceptance below correction resolution

## Problem

TEPP Validation evaluates `|estimate - target| <= k * standard_error` from represented binary64 inputs. GAP-081 and GAP-082 repaired finite rounded ties when multiplication or subtraction retained a nonzero error term. A narrower correction-resolution boundary remains: the exact product error can fall below half of the minimum subnormal, so the FMA correction projects to signed zero even though the rounded finite bound has crossed the scientific decision boundary.

Public RED `b55c5473c8ee7a70ec2508d14d0755c2aeb38191` uses an exact residual equal to the minimum positive subnormal, `estimate = 0x0.0000000000001p-1022`, `target = 0`, `k = 0x1.8p-538`, and `SE = 0x1p-537`. The exact dyadic product of the represented factors is `3/4` of the minimum subnormal, so the scientific inequality is false. Binary64 multiplication rounds that product up to the minimum subnormal, while `fma(k, SE, -rounded_bound)` rounds the `-1/4`-subnormal correction to signed zero. The predecessor therefore observed equal rounded residual/bound and equal zero correction projections and falsely accepted. The exact-equality control uses `k = 1` and `SE = minimum_subnormal`.

Initial repair `210cebc4980c861ddfd6d098bf1c9d66c8449e72` correctly closed that RED but restricted exact dyadic comparison to a *subnormal rounded bound*. A concurrent follow-up RED `c77ac440971abc649e6ede35874e5a7439e96797` demonstrated that the restriction was too narrow. At `estimate = f64::MIN_POSITIVE`, `target = 0`, `SE = (1 - 2^-27) * 2^-511`, and `k = (1 + 2^-27) * 2^-511`, the exact product is `(1 - 2^-54) * 2^-1022`: one quarter of a minimum-subnormal ULP below the minimum normal. Multiplication rounds to `f64::MIN_POSITIVE` and the negative quarter-subnormal FMA correction again projects to signed zero. The exact represented residual is the minimum normal, so the scientific inequality is still false even though the rounded bound itself is normal. The adjacent exact minimum-normal equality remains an acceptance control.

## Causal repair

Corrected causal repair `772ad8ed3be8d105975bf776e9f9d898774a4a23` stays inside the existing `validation_core` decision writer and removes the inappropriate rounded-bound-class condition. On a nonzero finite rounded tie, nonzero correction projections continue to use the GAP-081/GAP-082 low-term ordering. If both subtraction and product correction projections are zero, the zero subtraction correction means the represented residual is exact at that rounded value, while the product correction may have fallen below binary64 resolution. TEPP therefore decodes the exact integer significands and powers of two already used by the both-overflow comparator and compares that exact represented residual magnitude directly with the exact dyadic product of represented `k` and `SE`.

This covers both minimum-subnormal and minimum-normal boundary REDs without claiming a global exact comparator for finite ties with equal nonzero projected corrections. Ordinary finite non-ties, one-sided overflow handling, and the both-overflow exact comparator remain unchanged. No arbitrary-precision dependency, scale normalization, or alternate estimator is introduced.

Alternatives rejected: keeping the subnormal-bound guard fails the minimum-normal RED; blanket rejection of zero-projection ties breaks exact equality controls; relying on FMA alone cannot distinguish exact product equality from product error below binary64 correction resolution; broad arbitrary precision would widen the production surface beyond the demonstrated boundary.

CHANGELOG correction: `ceb68825f2759637034c2aaea935e50771aa6afc` supersedes the narrower wording introduced at `338c270c65c10d305432029c455b9ac37c28c0f1`.

## Ownership and standards

This is TEPP Validation Evidence decision semantics, not reusable static psychometric estimation. It does not move work into or copy source from fast-mlsirm and does not consume mutable contextual-orchestrator behavior.

Institute of Electrical and Electronics Engineers. (2019). *IEEE standard for floating-point arithmetic* (IEEE Std 754-2019). IEEE.

International Organization for Standardization. (2020). *Information technology—Microprocessor systems—Floating-point arithmetic* (ISO/IEC 60559:2020). ISO.

American Educational Research Association, American Psychological Association, & National Council on Measurement in Education. (2014). *Standards for educational and psychological testing*. American Educational Research Association.
