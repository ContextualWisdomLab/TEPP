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

This is a necessary, not sufficient, provenance check. The current report still does not retain the coverage denominator or the critical value `z`; therefore this repair does not claim full recomputation or denominator provenance. That remains a separate schema-level Validation Evidence gap rather than being inferred from unavailable data.

## Executable trace

- Public RED: `a839c606fb2329ce1b339eb235c79e02abf40e16`, `crates/validation_core/tests/validation_report_wilson_pair_coherence_contract.rs`.
- Causal source repair: `38c5b8e83fe2433167afb6ece13e72b6608ceb03`, `crates/validation_core/src/report.rs`.
- Complementary-identity coverage: `c1cb16a78499648c10d6d5a8dad5e212a267064a`.
- Changelog trace: `6af0821bc2fca3e9f101cfa4ad36048ecfaa6ddd`.
- Owner: TEPP Validation Evidence. No reusable static psychometric estimator is introduced; fast-mlsirm remains untouched. No semantic LLM path is involved.

## Methodological trace

Wilson's score interval is the primary statistical source for the interval family used by `validation_core::wilson_coverage_interval`. The AERA/APA/NCME *Standards for Educational and Psychological Testing* remain the current published testing standards while the sponsoring organizations revise the 2014 edition; this repair follows the evidence-integrity principle by refusing a durable summary whose reported components are mutually inconsistent.

### References

American Educational Research Association, American Psychological Association, & National Council on Measurement in Education. (2014). *Standards for educational and psychological testing*. American Educational Research Association.

Wilson, E. B. (1927). Probable inference, the law of succession, and statistical inference. *Journal of the American Statistical Association, 22*(158), 209–212. https://doi.org/10.1080/01621459.1927.10502953
