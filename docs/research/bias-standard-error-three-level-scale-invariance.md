# GAP-109 — exact three-level bias-SE proof must be scale invariant

## Problem

GAP-108 added a bounded exact path for three represented observations whose translated offsets satisfy

`SE(mean)^2 = (x^2 + y^2 - xy) / 9`

and whose dispersion numerator is itself an exactly represented square. The proof was nevertheless evaluated on the raw offsets. That made admission depend on magnitude rather than only on represented geometry: multiplying a valid dyadic sample by an exact power of two can overflow `x^2`, `y^2`, or `xy` even when the final standard error is finite and representable.

The public counterexample is the exact power-of-two scaling of GAP-108 by `2^600`:

- `truth = [0, 0, 0]`
- `recovered = [0, 5 * 2^590, 21 * 2^590]`
- represented bits: `0x0000000000000000`, `0x64f4000000000000`, `0x6515000000000000`
- minimax exact translation: `[-5 * 2^590, 0, 16 * 2^590]`

For the unscaled geometry, `x^2 + y^2 - xy = 361 / 2^20 = (19 / 1024)^2`; scaling by `2^600` therefore gives the exact target

`SE(mean) = (19 / 3072) * 2^600 = 0x1.9555555555555p+592`

with binary64 bits `0x64f9555555555555`.

Before GAP-109, the raw square/cross-product proof overflowed and the implementation fell back to the general translated normalized-moment path. That path evaluates the same normalized geometry `[-5/16, 0, 1]` through moment reconstruction and `sqrt`, returning adjacent upper `0x64f9555555555556`. The one-ULP shift is deterministic arithmetic error, not Monte Carlo uncertainty.

## Constraints

The repair must not weaken the GAP-108 error-free admission, make arbitrary-precision arithmetic a production dependency, special-case the payload, or move reusable static psychometric estimation out of `fast-mlsirm`. It must preserve permutation/sign symmetry and fail closed whenever the represented geometry cannot be proved exactly reversible.

## Decision

When any of the three raw products overflows, `validation_core::bias` now retries the same GAP-108 proof after dividing both offsets by the exact power-of-two binade scale of their maximum magnitude. Admission requires:

1. both normalized offsets to remain finite and nonzero when their sources are nonzero;
2. multiplying each normalized offset by the scale to reconstruct the original represented offset exactly;
3. normalized squares and cross-product to be finite and FMA-proven error-free;
4. square addition and cross-product subtraction to be error-free;
5. the normalized dispersion numerator to have an exactly represented square root.

Only then is the normalized root divided by three through the existing deterministic representable-denominator primitive and restored by the same exact power-of-two scale. Raw finite-product cases keep the GAP-108 path unchanged, including its subnormal exact-rational projection. If normalization or any proof fails, the predecessor translated path remains authoritative.

This is intentionally narrower than a claim that all `n > 2` standard errors are globally correctly rounded.

## Alternatives rejected

Payload-specific branching on `5/16/19` would encode the fixture rather than the invariant. Unconditionally normalizing every three-level proof would broaden behavior for low-scale/subnormal cases already covered by GAP-105/GAP-108. Replacing the bounded binary64 proof with arbitrary precision would duplicate numerical ownership and enlarge production cost without evidence that the broader dependency is required. Accepting overflow as proof failure is rejected because exact power-of-two scaling is representation preserving here and would make scientific output depend on unit magnitude.

## Standards and methodological trace

IEEE P754 is an active revision PAR approved 2024-06-06 and identifies IEEE 754-2019 as the standard it is intended to supersede. ISO/IEC 60559:2020 remains a published International Standard at stage 60.60, and IEEE/ISO/IEC 60559-2020 is listed as an active adoption of IEEE 754-2019. These authorities support treating the sequence and destination format of binary64 operations as part of the reproducible numerical contract rather than assuming algebraically equivalent floating-point expressions are interchangeable.

Morris, White, and Crowther (2019, *Statistics in Medicine*, https://doi.org/10.1002/sim.8086) distinguish deterministic performance-measure computation from Monte Carlo error due to a finite number of simulation repetitions. GAP-109 concerns the former: the represented sample is fixed and only the arithmetic path changes.

AERA, APA, and NCME continue to publish the 2014 *Standards for Educational and Psychological Testing* as the current public edition. This repair is therefore recorded as validity-evidence computation infrastructure; it does not substitute numerical implementation detail for the broader evidentiary requirements governing score interpretation and use.

## Traceability

- Public RED: `67fa485248a8673f90bb71a43c2a58865a764383`
- Contract: `crates/validation_core/tests/bias_standard_error_three_level_scale_invariance_contract.rs`
- Causal Rust repair: `dcaf25b37d9860e7956de5429d3ef5894b129b49`
- Production API/module: `validation_core::bias_standard_error` / `crates/validation_core/src/bias.rs`
- CHANGELOG: `31b1aff811703a41e3524e804d095be32621b004`
- Protected base observed before repair: `main@a243f18da4a4ca8a8d068c39922537f1f8ed6ad0`

## Remaining risk

The exact three-observation shortcut still applies only when its rational-square identity can be proved in represented binary64 arithmetic. Other three-level and larger-sample geometries remain on the translated second-moment path and may justify separate findings only when a concrete represented-input counterexample demonstrates a scientifically material discrepancy.
