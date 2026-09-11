# Bias-SE binary64 seed-distance bound

## Status and scope

This note turns the bounded-rounder review finding in #488 into repository-owned numerical derivation evidence. It analyzes only the seed used by `correctly_rounded_scaled_sqrt_ratio` in `crates/validation_core/src/bias_se.rs` at parent exact head `a063bd238ce838ea28e40e14151aeed8ea05cc2a`:

```text
((numerator as f64) / (denominator as f64)).sqrt() * 2^unit_exponent
```

The exact dyadic-square and midpoint comparisons remain the acceptance authority. This derivation does not authorize replacing them with the seed, returning an unchecked candidate, weakening the fail-closed exits, or claiming production coverage GREEN. Its purpose is narrower: establish a conservative upper bound on how far a finite positive seed can start from the correctly rounded finite binary64 result under the function's existing admission preconditions.

## Preconditions owned by the current implementation

`correctly_rounded_scaled_sqrt_ratio` admits only:

- integer numerator `N` with `1 <= N <= 2^128 - 1`;
- integer denominator `D` with `1 <= D <= 2^53`;
- an exactly represented power-of-two unit `U = 2^k` for `-1074 <= k <= 1023`;
- a finite positive initial candidate after the final multiplication by `U`.

`D as f64` is exact because every integer through `2^53` is exactly representable in binary64. `N as f64` is a Rust integer-to-float conversion and therefore produces the closest binary64 value, using roundTiesToEven when rounding is necessary. Rust's `f64::sqrt` is guaranteed to return the rounded infinite-precision square root. IEEE 754-2019 / ISO/IEC/IEEE 60559:2020 define the binary floating-point arithmetic model used by these operations; Rust documents the relevant conversion and square-root guarantees directly.

Let binary64 precision be `p = 53` and let `u = 2^-53` denote the round-to-nearest relative error bound for a finite normal result.

## The unscaled seed is within three adjacent steps

Define the exact positive ratio and root

`q = N / D`,

`r = sqrt(q)`.

The admitted integer ranges imply

`2^-53 <= q < 2^128`,

so both `q` and `r` are far inside the normal binary64 exponent range. The `N -> f64` conversion, division, and square root can therefore each be bounded by one normal round-to-nearest factor:

`fl(N) = N(1 + delta_0)`,

`fl(fl(N) / D) = q(1 + delta_0)(1 + delta_1)`,

`z = fl(sqrt(fl(fl(N) / D))) = r sqrt((1 + delta_0)(1 + delta_1))(1 + delta_2)`,

with `|delta_i| <= u`.

The extremal multiplicative envelope is then

`(1 - u)^2 <= z / r <= (1 + u)^2`.

Hence

`|z / r - 1| <= alpha = 2u + u^2`.

For any positive normal binary64 neighborhood, the smaller adjacent spacing is at least `u` times the real magnitude. Therefore the unscaled seed differs from the exact root by strictly less than

`alpha / u = 2 + u < 3`

local adjacent spacings. This is deliberately conservative; it is sufficient for the bounded-control-flow argument and does not depend on a statistical or empirical search.

## Power-of-two restoration and the normal/subnormal boundary

Let the exact scaled target be `t = r U` and let the real scaled seed before its final binary64 rounding be `s = z U`.

If `s` stays normal and finite, multiplication by a power of two changes only the exponent and is exact. The same `< 3`-adjacent-step bound therefore carries through unchanged.

The remaining finite case is the normal/subnormal transition, where the final multiplication may round. Let `eta = 2^-1074` be the binary64 subnormal quantum and `m = 2^52 eta = 2^-1022` the minimum normal value. The pre-restoration bound gives

`|s - t| <= alpha t`.

If either the real scaled seed or the exact target is subnormal while the other is at the boundary, then `t < m / (1 - alpha)`. Since `alpha = 2^-52 + 2^-106`, this is conservatively below `(2^52 + 2) eta`. Thus

`|s - t| < alpha (2^52 + 2) eta < 1.000000000000002 eta`.

The final round-to-nearest multiplication can add at most `eta/2`, and the correctly rounded finite target is at most `eta/2` from `t`. Consequently the produced positive candidate is still strictly fewer than three subnormal grid steps from the correctly rounded finite result. The same conclusion holds when both `s` and `t` are subnormal.

The endpoint refusals stay separate from this theorem. If restoration produces represented zero, the implementation refuses the non-positive seed. If movement from the largest finite value would require an infinite neighbor, the implementation refuses that neighbor. Neither case can reach the terminal four-pass fallthrough.

## Consequence for the current correction loop

Each loop pass compares the exact represented candidate square with the exact rational target. If unequal, it chooses the unique adjacent finite neighbor in the target direction and compares the exact midpoint square. The pass either:

1. returns the candidate because it is already on the correct side of the midpoint;
2. returns the neighbor for a roundTiesToEven midpoint tie; or
3. advances by exactly one adjacent binary64 value toward the correctly rounded result.

The seed-distance result above rules out an initial separation of three or more representable neighbors from the correctly rounded finite result. Therefore at most two neighbor-advance passes are required; the following pass performs the final exact-square/midpoint decision. The existing four-pass loop is conservative by at least one pass on the admitted finite-result domain.

This establishes a repository-owned derivation for the prior independent-review claim. It does **not** by itself remove `bias_se.rs`'s final fail-closed `None`. Source simplification is a separate product decision: before removing or restructuring that exit, TEPP must encode the theorem's assumptions as executable contract evidence, rerun exact-head line/branch coverage, and obtain independent review of the implementation change. Until then, the terminal refusal remains future-contract defense rather than a coverage exclusion.

## Alternatives considered

Increasing the iteration count is rejected because it weakens neither the proof obligation nor the coverage obligation and would make a bounded algorithm less precise about its actual arithmetic envelope. An unbounded correction loop is rejected because a numerical proof path must preserve a finite resource bound. Returning the last candidate after four passes is rejected because it would convert proof exhaustion into unchecked numerical authority. Replacing the exact midpoint logic with an LLM or empirical search is outside the Validation Evidence contract.

A direct integer/rational square-root rounding implementation could eliminate dependence on a floating seed altogether, but it is a larger arithmetic change. The present derivation is the minimal evidence needed to decide whether the existing exact-comparison design can be simplified without changing its scientific target.

## Traceability

| Item | Evidence |
|---|---|
| Domain owner | TEPP Validation Evidence |
| Systemic issue | #491 |
| Landing vehicle | #488 |
| Parent exact source | `a063bd238ce838ea28e40e14151aeed8ea05cc2a` |
| Production function | `crates/validation_core/src/bias_se.rs::correctly_rounded_scaled_sqrt_ratio` |
| Exact comparison | `compare_scaled_ratio_to_dyadic_square`, `adjacent_midpoint_dyadic`, `Wide256` |
| Existing bounded loop | four exact candidate/midpoint passes followed by fail-closed `None` |
| Required follow-up | executable theorem-precondition contract, exact-head coverage, independent review |

## References

IEEE. (2019). *IEEE Standard for Floating-Point Arithmetic* (IEEE Std 754-2019). https://doi.org/10.1109/IEEESTD.2019.8766229

ISO/IEC/IEEE. (2020). *ISO/IEC/IEEE 60559:2020: Floating-point arithmetic*. https://standards.ieee.org/ieee/60559/10226/

The Rust Project Developers. (n.d.). *Operator expressions: Numeric casts*. The Rust Reference. Retrieved September 11, 2026, from https://doc.rust-lang.org/reference/expressions/operator-expr.html#numeric-cast

The Rust Project Developers. (n.d.). *Primitive type f64: sqrt*. Rust standard library documentation. Retrieved September 11, 2026, from https://doc.rust-lang.org/std/primitive.f64.html#method.sqrt
