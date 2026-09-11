# Bias standard-error translated-dispersion positivity bound

## Decision scope

This note narrows the remaining `validation_core::bias_standard_error` proof obligation in PR #488 / issue #491. It applies only to the general translated path in `crates/validation_core/src/bias.rs` after the exact two-level path refuses and, for three observations, after the exact three-level path also refuses, after canonical anchor translation, after power-of-two normalization, and after the nonzero-to-zero normalization preflight.

It does **not** claim that the current floating implementation of `dispersion_numerator` is always positive. The production fail-closed `dispersion_numerator <= 0.0` guard remains required until an implementation-matched floating-point forward-error proof or a compact caller-valid counterexample closes that separate question.

## Exact-real lower bound

Let the represented normalized binary64 values entering the moment sums be

\[
y_1,\ldots,y_n, \qquad n \ge 3,
\]

interpreted as exact real numbers for this section only. Canonical translation supplies at least one exact zero. Choose that value as `0`. Let `x` be a normalized value with maximum magnitude. `exact_power_of_two_scale` chooses the leading-binade power of the maximum translated magnitude, so

\[
1 \le |x| < 2.
\]

Define the exact-real dispersion numerator

\[
D = n\sum_i y_i^2 - \left(\sum_i y_i\right)^2.
\]

The standard pairwise identity gives

\[
D = \sum_{i<j}(y_i-y_j)^2.
\]

The pair `(0,x)` contributes `x²`. For every remaining value `z`, use the two distinct pairs `(0,z)` and `(x,z)`:

\[
z^2 + (z-x)^2
= 2\left(z-\frac{x}{2}\right)^2 + \frac{x^2}{2}
\ge \frac{x^2}{2}.
\]

These selected pair terms are disjoint. All unselected pair-square terms are nonnegative. Therefore

\[
D \ge x^2 + (n-2)\frac{x^2}{2}
= \frac{n x^2}{2}
\ge \frac{n}{2}.
\]

The derivation also covers the `n=3` case that can reach the general path after `exact_three_level_standard_error` declines admission. This is stronger than the earlier `D >= 1` narrowing. For the represented normalized values, exact real dispersion is separated from zero by a margin that grows linearly with sample count.

## Why this does not remove the production guard

The implementation does not evaluate the exact-real expression above in one exact operation. It currently performs all of the following before the final sign test:

1. rounds each `y_i * y_i` to binary64;
2. computes the first moment and the rounded-square second moment through separate sorted compensated sums;
3. converts `n` to binary64 and rounds `n * Q`;
4. forms `(-S).mul_add(S, nQ)`, so only the final multiply-subtract is fused.

Consequently, the remaining proof obligation is not positivity of the mathematical variance identity. It is whether the combined absolute error of those implemented stages can reach or exceed the exact-real margin `n/2` anywhere in the admitted caller domain.

The repository's `deterministic_compensated_sum` is a specific sorted Neumaier-style implementation whose correction accumulator is itself updated in working precision and whose final `sum + correction` is rounded. A generic statement that “compensated summation is accurate” is not sufficient acceptance evidence. Any bound used for source removal must match this implementation, its canonical ordering, binary64 round-to-nearest-ties-to-even behavior, subnormal handling, the separately rounded square path, and the final FMA.

## Research anchors

Higham (1993) gives classical forward-error analysis for floating-point summation, including compensated variants. Ogita, Rump, and Oishi (2005) develop accurate summation and dot-product algorithms with verified error analysis, and Rump, Ogita, and Oishi (2008) develop faithful-rounding summation. These works are appropriate proof tools and comparison points, but none is imported here as a theorem about TEPP's implementation without an explicit algorithm-to-algorithm derivation.

Higham, N. J. (1993). The accuracy of floating point summation. *SIAM Journal on Scientific Computing, 14*(4), 783–799. https://doi.org/10.1137/0914050

Ogita, T., Rump, S. M., & Oishi, S. (2005). Accurate sum and dot product. *SIAM Journal on Scientific Computing, 26*(6), 1955–1988. https://doi.org/10.1137/030601818

Rump, S. M., Ogita, T., & Oishi, S. (2008). Accurate floating-point summation part I: Faithful rounding. *SIAM Journal on Scientific Computing, 31*(1), 189–224. https://doi.org/10.1137/050645671

## Acceptance routes

The `dispersion_numerator <= 0.0` branch may be changed only after one of these routes is completed with repository-owned evidence:

- derive an implementation-matched absolute forward-error bound strictly below `n/2` over the admitted domain, covering square rounding, both compensated sums, `usize -> f64` sample-count conversion, `n * Q`, and the final FMA; or
- produce a compact, caller-valid represented-input counterexample that reaches the nonpositive floating result and preserve it as a realistic deterministic regression.

If the proof requires a bounded product/resource admission domain, that bound must be justified as an operational/scientific contract and enforced consistently at the public boundary. A coverage-only sample-count cutoff, giant synthetic allocation, skip/xfail, branch suppression, source rewriting, or a copied production implementation used as its own oracle is not acceptable.

## Traceability

- owner bounded context: Validation Evidence
- landing vehicle: TEPP PR #488
- scientific/resource issue: TEPP #491
- production module: `crates/validation_core/src/bias.rs`
- numerical primitive under analysis: `crates/validation_core/src/numeric.rs::deterministic_compensated_sum`
- current fail-closed state: general translated `dispersion_numerator <= 0.0`
- exact-real theorem in this note: `D >= n x² / 2 >= n / 2` for every admitted general-path `n >= 3`
- theorem effect: narrows the floating proof obligation; does not authorize source removal or claim 100% branch coverage
