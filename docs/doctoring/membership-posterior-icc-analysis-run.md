# Membership-posterior ICC analysis-run composition

**Active slice:** ADR 0045 / `membership_posterior_icc_v1`
**Protected-main status:** not implemented-main

`membership_core` already classifies nested versus multiple-membership versus
cross-classified designs, recovers nested ANOVA ICC only when nested, and
computes Kish ESS of membership weights. `psychometric_core` already averages
posterior-draw point estimates without Rubin pooling. This slice binds those
recoveries to a cutoff-safe analysis-run profile so an operator can request a
digest-bound terminal result.

Multiple membership is classified and preserved. Nested ICC is refused for
multiple-membership and cross-classified designs; Kish ESS is still emitted.
The profile is not ESEM, not DSEM, not Rubin `T`, not CWC, and not an MMMC
sampler.

Exact-head Checks and two independent approvals are required before any
implemented-main claim.
