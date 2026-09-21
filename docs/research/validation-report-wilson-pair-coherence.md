# ValidationReport Wilson endpoint-pair coherence

## Problem

`ValidationReport` previously required `coverage_wilson_lower <= interval_coverage <= coverage_wilson_upper`, but containment alone does not establish that the two stored endpoints came from one Wilson score interval for the same empirical coverage. For example, `p = 0.5`, `L = 0.2`, `U = 0.9` is ordered and contains `p`, yet no Wilson score interval can produce that pair: at `p = 0.5` the Wilson roots are symmetric about `0.5` for every finite positive `z² / n`.

This matters at the Validation Evidence boundary because a durable artifact must not combine independently plausible numbers into a scientifically unrealizable interval. It does not change the Wilson estimator or psychometric model.

## Algebraic invariant

Let `p` be the empirical coverage and `a = z² / n > 0`. The Wilson roots satisfy

`L * U = p² / (1 + a)`

and

`L + U = 1 + (2p - 1) / (1 + a)`.

Eliminating `a` gives the necessary pair identity

`p² * (L + U - 1) = (2p - 1) * L * U`.

For `p < 0.5`, TEPP evaluates the equivalent identity on the uncovered proportion `q = 1 - p`:

`q² * (1 - L - U) = (1 - 2p) * (1 - L) * (1 - U)`.

Using the complementary form avoids needlessly squaring a tiny `p`. Every term is probability-scaled, so the admission comparison cannot overflow finite binary64 inputs. A `64 * EPSILON` absolute tolerance admits normal endpoint-rounding error while rejecting materially incoherent pairs such as `[0.2, 0.9]` at `p = 0.5`.

This is a necessary, not sufficient, provenance check. The current report still does not retain the coverage denominator or the critical value `z`; therefore this repair does not claim full recomputation or denominator provenance. The existing producer can also admit a positive `z` whose squared binary64 representation underflows to zero, so this artifact-level identity is deliberately not strengthened into a non-degeneracy rule that the producer itself does not yet guarantee. Those are separate producer/schema questions rather than facts inferred from unavailable evidence.

## Dependent-fixture review

Adding a cross-field invariant changed the validity of old branch fixtures that used arbitrary Wilson-looking probabilities while testing unrelated RMSE/Monte Carlo behavior. Self-review therefore replaced those incidental values with the exactly coherent `p = 0.5`, `[0.2, 0.8]` pair, which corresponds to a positive finite `z² / n = 0.5625`. The intended RED condition in each test remains unchanged.

The same review rechecked typed RMSE fixtures against the already-landed generic `MonteCarloSummary` moment contracts. Where an older typed test had become generic-invalid, its sample statistics were repaired so the generic carrier is valid and the typed RMSE boundary remains the sole reason for refusal. In particular, the nonnegative percentile RED now uses `n = 4`, `mean = 1`, `SD = 2`, `SE = 1`, and `upper = 4.1`: it fits the generic individual/joint moment support but exceeds the typed nonnegative RMSE sample-sum bound `n * mean = 4`.

These fixture changes are test doctoring, not evidence-gate weakening. They remove confounding failure causes introduced by stronger predecessor contracts.

## Executable trace

- Public RED: `a839c606fb2329ce1b339eb235c79e02abf40e16`, `crates/validation_core/tests/validation_report_wilson_pair_coherence_contract.rs`.
- Causal source repair: `38c5b8e83fe2433167afb6ece13e72b6608ceb03`, `crates/validation_core/src/report.rs`.
- Complementary-identity coverage: `c1cb16a78499648c10d6d5a8dad5e212a267064a`.
- Changelog trace: `6af0821bc2fca3e9f101cfa4ad36048ecfaa6ddd`.
- Dependent fixture doctoring includes `acf573526f955a7700ebd753b83e0baad120628d`, `ccd30ede3b6e90e0ddff0c0c9ef764f320c2cec1`, `fee458966a9dfb64da2aba01f242fe9e2e613540`, `b9eb9465b155b0fd3f44d0ff88b429485f038c8e`, and `59e40b4f0ebbb3b991c84724ded157ffe974abcd`.
- Owner: TEPP Validation Evidence. No reusable static psychometric estimator is introduced; fast-mlsirm remains untouched. No semantic LLM path is involved.

## Methodological trace

Wilson's score interval is the primary statistical source for the interval family used by `validation_core::wilson_coverage_interval`. The AERA/APA/NCME *Standards for Educational and Psychological Testing* remain the current published testing standards while the sponsoring organizations revise the 2014 edition; this repair follows the evidence-integrity principle by refusing a durable summary whose reported components are mutually inconsistent.

### References

American Educational Research Association, American Psychological Association, & National Council on Measurement in Education. (2014). *Standards for educational and psychological testing*. American Educational Research Association.

Wilson, E. B. (1927). Probable inference, the law of succession, and statistical inference. *Journal of the American Statistical Association, 22*(158), 209–212. https://doi.org/10.1080/01621459.1927.10502953
