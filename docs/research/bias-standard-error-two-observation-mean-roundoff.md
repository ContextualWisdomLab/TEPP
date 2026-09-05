# Bias standard error: two-observation mean-rounding boundary

## Problem

`validation_core::bias_standard_error` already used the exact two-observation identity `SE(mean) = |r₁-r₂| / 2` when either `recovered - truth` subtraction discarded represented low-order mass. Exact pairwise residuals, however, still fell through to the generic path that first rounded the residual mean and then formed deviations around that rounded mean.

That distinction is scientifically material even when every input subtraction is exact. Let

- `truth = [0, 0]`,
- `recovered = [1, next_down(1)]`,
- `next_down(1) = 0x1.fffffffffffffp-1`.

The represented residuals are therefore exactly `[1, 1 - 2^-53]`. Their exact mean is `1 - 2^-54`, which is the midpoint between the two adjacent binary64 residuals and rounds to `1` under round-to-nearest, ties-to-even. Centering on that rounded mean produces deviations `[0, -2^-53]`; the generic scaled second-moment path consequently returns approximately `2^-53 / sqrt(2)` (`0x3c96_a09e_667f_3bcd`).

For two observations the sample standard error of the mean simplifies algebraically before any mean is needed:

`SE(mean) = |r₁ - r₂| / 2 = 2^-54`, whose binary64 bits are `0x3c90_0000_0000_0000`.

The sign-mirrored residuals have the same dispersion, and equal residuals remain exactly zero.

## Decision

For every two-observation sample, evaluate `SE(mean) = |r₁-r₂| / 2` before the generic rounded-mean path.

- If either pairwise `recovered - truth` subtraction has a nonzero error-free low term, retain the predecessor expanded-input difference `[recovered₁, -truth₁, -recovered₂, truth₂]` so discarded subtraction mass is not lost.
- If both residual subtractions are exact, form the half-difference from the represented residuals `[r₁, -r₂]` through `deterministic_representable_sum_over_count(..., 2)`. This preserves the existing cancellation/overflow-safe denominator handling; in particular, opposite extreme finite residuals can still yield a representable standard error without materializing `r₁-r₂` as an overflowing intermediate.
- Take the absolute value only after the signed half-difference is formed.

This is a bounded algebraic repair, not a claim that the general `n > 2` standard-error path is globally correctly rounded.

## Rejected alternatives

Keeping the generic rounded-mean path for exact residuals was rejected because the counterexample shows that exact input subtraction does not imply an exact residual mean, and a rounded two-point mean can change the dispersion geometry substantially rather than by a negligible reporting-only amount.

Computing `(r₁-r₂).abs()/2` directly was rejected because the raw difference can overflow for opposite-sign extreme finite residuals even when the final halved result is representable. Reusing the repository's deterministic representable sum-over-count boundary keeps the scientific denominator and range policy intact.

Introducing arbitrary-precision arithmetic for all standard errors was rejected as disproportionate to this proven two-observation identity and outside the current Validation Evidence runtime boundary.

## Evidence and traceability

- Public RED: `d3dcd918bece928ae4103dffda8e7dc654927da0`, `crates/validation_core/tests/bias_standard_error_two_observation_mean_roundoff_contract.rs`.
- Minimal causal repair: `02b0a178154ea0ae7da87289756897d7c2f361e3`, `crates/validation_core/src/bias.rs`.
- CHANGELOG: `5603b12d2aedb6dfcc0aa4203d57047a98aee789`.
- API: `validation_core::bias_standard_error`.

IEEE 754 binary64 round-to-nearest, ties-to-even semantics determine the midpoint behavior in the counterexample. The performance-measure interpretation follows the existing TEPP Validation Evidence trace to known-truth simulation evaluation and Monte Carlo uncertainty rather than treating an LLM judgment as numerical authority.

## References

IEEE Computer Society. (2019). *IEEE standard for floating-point arithmetic* (IEEE Std 754-2019). IEEE. https://doi.org/10.1109/IEEESTD.2019.8766229

Morris, T. P., White, I. R., & Crowther, M. J. (2019). Using simulation studies to evaluate statistical methods. *Statistics in Medicine, 38*(11), 2074–2102. https://doi.org/10.1002/sim.8086
