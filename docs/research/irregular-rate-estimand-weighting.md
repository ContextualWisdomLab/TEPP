# Irregular residual log-rate weighting and estimand identity

## Status and scope

This note records a Longitudinal Modeling estimand boundary exposed by TEPP PR #310. It does not change numerical arithmetic and does not promote an unmerged branch to protected-main authority.

Exact source reviewed before this note: `df0b4d6d3e0622de3c988b840114fbdb41e5d1b0`.

`center_within_unit_event_lags` forms one consecutive event-time lag pair for each admitted adjacent occasion inside a unit. `LaggedWithinResidual` then carries the earlier residual, later residual, and typed event interval but not the originating unit identity. `pairwise_same_sign_log_rate` computes an admitted scalar log-rate for each pair and passes the resulting rate vector to the Longitudinal-local mean boundary.

Consequently, the existing aggregate is a lag-pair-average estimand. It must not be described as an equal-unit average unless the design makes those two estimands coincide.

## Two distinct estimands

Let unit `i` contribute `k_i >= 1` admitted consecutive lag pairs after all temporal and residual admission rules. Let `r_ij` be the scalar event-time log-rate for admitted pair `j` of unit `i`.

The current pair-average target is

\[
\theta_{pair}
= \frac{\sum_i \sum_{j=1}^{k_i} r_{ij}}
       {\sum_i k_i}.
\]

Every admitted lag pair receives equal weight. A unit with more admitted occasions can therefore receive more total weight through more consecutive pairs.

A different, equal-unit target is

\[
\theta_{unit}
= \frac{1}{I}\sum_{i=1}^{I}
  \left(\frac{1}{k_i}\sum_{j=1}^{k_i} r_{ij}\right).
\]

Every contributing unit receives equal final weight after its admitted pair rates are summarized within unit. The targets are not algebraically interchangeable when `k_i` varies. They may also target different populations when follow-up duration, observation count, missingness, or pair admissibility is associated with the longitudinal process.

The existing TEPP API has enough information to compute `theta_pair` after pairs are formed, but it does not retain unit identity in `LaggedWithinResidual`; it therefore cannot reconstruct `theta_unit` from that vector without changing the typed contract or preserving a unit-keyed aggregation boundary.

## Failure denominators are part of the estimand

Zero, opposite-sign, non-finite, or otherwise inadmissible residual pairs are not ordinary numerical observations. A longitudinal summary must report at least:

- candidate units;
- units contributing one or more admitted lag pairs;
- candidate consecutive pairs;
- admitted pairs;
- refused pairs by typed reason.

Dropping a refused pair changes `k_i` and can change both the pair-average weight and whether a unit contributes to a unit-average target. A unit with no admitted pairs must not disappear silently from a denominator whose interpretation says otherwise. The public contract therefore needs a declared estimand and failure-denominator policy before a result can be promoted as scientific evidence.

## Relation to unequal and informative cluster size

This is not a cluster-randomized treatment-effect model. However, the statistical identification issue is analogous: unequal numbers of observations inside a higher-level unit can induce different weighting targets, and informative cluster or subcluster size can make those targets materially different. The estimator has to match the declared estimand rather than inheriting weights accidentally from record multiplicity.

Wang, Kong, and Datta (2011) study clustered longitudinal data and show that informative cluster size can invalidate ordinary marginal inference when cluster size is related to the outcome distribution. Huang (2011) further shows that the appropriate weights depend on the population of interest and on within-cluster covariate structure. Kahan et al. (2023) give a clear modern estimand distinction between equal participant weighting and equal cluster weighting. TEPP does not import their treatment-effect estimands; it imports the narrower methodological requirement that aggregation weights are part of estimand identity.

## Required product contract

Issue #495 owns the follow-up gap. Before any equal-unit result is implemented or any current pair-average result is described as an average unit/person effect:

1. PRD/TRD/ADR/TRACEABILITY must name the target (`lag_pair_average`, `unit_average`, or a separately justified design-weighted target).
2. A unit-average path must preserve unit identity until within-unit rates are summarized; occasion count cannot stand in for an externally defined design or membership weight.
3. Known-truth acceptance must include balanced and highly unbalanced occasion counts, informative missing/follow-up patterns, irregular intervals, row permutation, worker-count determinism, and explicit failure denominators.
4. Cross-classified and multiple-membership extensions must retain their declared membership structure and must not collapse to a primary group merely to obtain one scalar weight.
5. Neither target is automatically a raw-process autoregressive/DSEM effect. Existing refusal boundaries around CWC and occasion-mean residuals remain in force.
6. Reusable finite binary64 sum/mean arithmetic remains owned by `ContextualWisdomLab/fast-mlsirm`. TEPP may consume it only from an immutable released contract; this note does not authorize another generic summation implementation in `longitudinal_core`.

The current mixed-sign binary64 mean RED in #310 remains independent. Clarifying the estimand does not make that numerical RED pass and does not authorize a merge.

## References

Huang, Y. (2011). Informative cluster sizes for subcluster-level covariates and weighted generalized estimating equations. *Biometrics, 67*(3), 843–851. https://doi.org/10.1111/j.1541-0420.2010.01542.x

Kahan, B. C., Li, F., Blette, B., Jairath, V., Copas, A., & Harhay, M. O. (2023). Informative cluster size in cluster-randomised trials: A case study from the TRIGGER trial. *Clinical Trials*. https://doi.org/10.1177/17407745231186094

Wang, M., Kong, M., & Datta, S. (2011). Inference for marginal linear models for clustered longitudinal data with potentially informative cluster sizes. *Statistical Methods in Medical Research, 20*(4), 347–367. https://doi.org/10.1177/0962280209347043
