# Standard-error acceptance at the subnormal product boundary

## Problem

TEPP Validation evaluates `|estimate - target| <= k * standard_error` from represented binary64 inputs. GAP-081 and GAP-082 repaired finite rounded ties when multiplication or subtraction retained a nonzero error term. A narrower boundary remains when the rounded bound is subnormal: the exact product error itself can lie below half of the minimum subnormal and the FMA correction can therefore project to signed zero.

Public RED `b55c5473c8ee7a70ec2508d14d0755c2aeb38191` uses an exact residual equal to the minimum positive subnormal, `estimate = 0x0.0000000000001p-1022`, `target = 0`, `k = 0x1.8p-538`, and `SE = 0x1p-537`. The exact dyadic product of the represented factors is `3/4` of the minimum subnormal, so the scientific inequality is false. Binary64 multiplication rounds that product up to the minimum subnormal, while `fma(k, SE, -rounded_bound)` rounds the `-1/4`-subnormal correction to signed zero. The predecessor therefore observed equal rounded residual/bound and equal zero correction projections and falsely accepted.

The control uses `k = 1` and `SE = minimum_subnormal`; its exact product equals the residual and must remain accepted.

## Causal repair

Causal repair `210cebc4980c861ddfd6d098bf1c9d66c8449e72` stays inside the existing `validation_core` decision writer. Only when a nonzero finite rounded tie has zero subtraction/product correction projections and the rounded bound is subnormal, TEPP decodes the exact integer significands and powers of two already used by the both-overflow comparator. Because the subtraction correction is zero, the represented residual itself is exact for this boundary; TEPP compares that represented magnitude directly with the exact dyadic `k * SE` product. No arbitrary-precision dependency, scale normalization, or alternate estimator is introduced.

The repair does not claim that every finite rounded tie is globally exact. Nonzero low-term projections keep the GAP-081/GAP-082 discriminator; ordinary finite non-ties keep direct comparison; one-sided overflow and both-overflow handling are unchanged.

Alternatives rejected: blanket rejection of subnormal ties would reject exact minimum-subnormal equality; relying on FMA alone cannot distinguish exact product equality from an exact product error below binary64 correction resolution; broad arbitrary-precision comparison would widen the production surface beyond the demonstrated boundary.

CHANGELOG trace: `338c270c65c10d305432029c455b9ac37c28c0f1`.

## Ownership and standards

This is TEPP Validation Evidence decision semantics, not reusable static psychometric estimation. It does not move work into or copy source from fast-mlsirm and does not consume mutable contextual-orchestrator behavior.

Institute of Electrical and Electronics Engineers. (2019). *IEEE standard for floating-point arithmetic* (IEEE Std 754-2019). IEEE.

International Organization for Standardization. (2020). *Information technology—Microprocessor systems—Floating-point arithmetic* (ISO/IEC 60559:2020). ISO.

American Educational Research Association, American Psychological Association, & National Council on Measurement in Education. (2014). *Standards for educational and psychological testing*. American Educational Research Association.
