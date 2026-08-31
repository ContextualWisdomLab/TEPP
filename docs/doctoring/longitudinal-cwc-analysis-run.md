# Longitudinal CWC analysis-run bind

**Review date:** 2026-08-31
**Active slice:** GAP-006 / issue #169 remaining operator-visible composition

Protected main already recovers Enders and Tofighi (2007) CWC within/between
OLS in `psychometric_core`. This slice binds that recovery to
`analysis_engine` as a cutoff-safe analysis-run output: eligibility against
the request knowledge cutoff, digest-bound `tepp.longitudinal_cwc.v1`, and an
explicit refusal to treat the slopes as a causal effect.

This is not a new estimator, not a Driver p.16 `std` restore, not persistence,
and not implemented-main.

## Evidence boundary

Exact-head checks, independent review, and protected merge are required before
the profile can be promoted.
