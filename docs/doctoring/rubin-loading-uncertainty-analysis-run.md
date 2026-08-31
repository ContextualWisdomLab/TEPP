# Rubin loading-uncertainty analysis-run bind

**Review date:** 2026-08-31
**Active slice:** GAP-006 / issue #169 remaining operator-visible composition

Protected main already averages posterior-draw OLS loadings and combines them
with Rubin (1996) total variance in `psychometric_core`. This slice binds those
helpers jointly to `analysis_engine` as a cutoff-safe analysis-run output:
eligibility against the request knowledge cutoff, digest-bound
`tepp.rubin_loading_uncertainty.v1`, and an explicit refusal to treat the
draws as Mislevy person-level plausible values.

This is not a new estimator, not a Driver p.16 `std` restore, not CWC, not
persistence, and not implemented-main.

## Evidence boundary

Exact-head checks, independent review, and protected merge are required before
the profile can be promoted.
