# Longitudinal ESEM/DSEM engine composition analysis-run bind

**Review date:** 2026-08-31
**Active slice:** GAP-169 / issue #169 remaining operator-visible composition

Protected main already recovers construct-class, posterior-draw, invariance,
event-time, membership, and within/between gates in scientific crates. This
slice binds those gates to `analysis_engine` as a cutoff-safe analysis-run
output: eligibility against the request knowledge cutoff, digest-bound
`tepp.longitudinal_esem_dsem_composition.v1`, and an explicit refusal to treat
the composition as an estimator or a causal effect.

This is not a new sampler, not a Driver p.16 `std` restore, not persistence,
and not implemented-main.

## Evidence boundary

Exact-head checks, independent review, and protected merge are required before
the profile can be promoted.
