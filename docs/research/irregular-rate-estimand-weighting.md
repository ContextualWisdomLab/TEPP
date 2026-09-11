# Irregular residual log-rate weighting and estimand identity

## Status and scope

This note records a Longitudinal Modeling estimand boundary exposed by TEPP PR #310 and issue #495. The branch now contains a typed first-release candidate contract, but it remains Draft and is not protected-main or release authority.

Scientific finding head: `df0b4d6d3e0622de3c988b840114fbdb41e5d1b0`. Typed-contract implementation lineage starts at `c7f55acb347ceac38675d9566eff767372739dba`; public contract tests start at `f45e83ba5223ad9d89022482e759ad9e705b229c` and refusal-class denominators are completed by `a3065f6d7589d5a310e54f85d6447b27414955ba`.

`center_within_unit_event_lags` forms one consecutive event-time lag pair for each admitted adjacent occasion inside a unit. `LaggedWithinResidual` carries the earlier residual, later residual, and typed event interval but not the originating unit identity. The existing scalar recovery therefore computes a lag-pair-average estimand: every admissible pair enters one common rate vector before averaging.

The Draft candidate now names that target explicitly as `tepp.irregular_rate.lag_pair_average.v1`. `tepp.irregular_rate.unit_average.v1` is also a typed name, but it fails closed until an immutable released reusable finite-mean contract can support the second aggregation step without adding another TEPP-local generic summation kernel.

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

The current pair-average summary attributes candidate/contributing-unit denominators using the same deterministic `BTreeMap<u32, ...>` unit order and consecutive-pair counts used by `center_within_unit_event_lags`. This is sufficient for denominator evidence for the existing pair-weighted target. It is deliberately not treated as the future equal-unit computation boundary: activating `unit_average.v1` must preserve unit identity through within-unit numerical aggregation instead of depending on flattened-pair reconstruction.

## Failure denominators are part of the estimand

The Draft `IrregularRateSummary` reports:

- candidate units with at least two admitted event-time occasions;
- units contributing at least one admitted scalar rate;
- candidate consecutive pairs;
- admitted pairs;
- zero/opposite-sign refusals;
- same-sign pairs whose represented scalar log-rate is not admissible.

`refused_pairs()` is the sum of the two pair-level refusal classes. Non-finite input rows remain a payload-level admission failure before a scientific summary is constructed; they are not silently converted into missing pair observations.

If no pair is numerically admissible after otherwise valid CWC/event-time admission, the summary returns `estimate = None` while retaining the complete unit/pair denominators. The legacy scalar recovery remains fail-closed for that case. This separation lets evidence reporting preserve its failure population without changing legacy scalar semantics.

Dropping a refused pair changes `k_i` and can change both the pair-average weight and whether a unit contributes to a unit-average target. A unit with no admitted pairs must therefore not disappear silently from a denominator whose interpretation says otherwise.

## Public contract evidence

`crates/longitudinal_core/tests/irregular_rate_estimand_contract.rs` fixes a deterministic unequal-pair-count fixture. One unit contributes three candidate pairs and another contributes two; three rates are admitted and two are refused by the sign/zero rule. The test establishes that:

- `LagPairAverageV1` bit-matches the legacy pair-average scalar recovery;
- the pair-average differs from the independently computed equal-unit comparison when admitted pair counts differ;
- row permutation leaves the typed summary unchanged;
- a two-unit fixture with no admissible rates retains `candidate_units = 2`, `candidate_pairs = 2`, `admitted_pairs = 0`, `refused_pairs = 2`, and `estimate = None`;
- `UnitAverageV1` has a stable external name but fails closed while the owner mean release is unavailable.

This evidence resolves the naming/denominator ambiguity for the pair-weighted first-release candidate. It does not claim equal-unit scientific acceptance and does not repair #310's independent mixed-sign binary64 mean RED.

## Relation to unequal and informative cluster size

This is not a cluster-randomized treatment-effect model. However, the statistical identification issue is analogous: unequal numbers of observations inside a higher-level unit can induce different weighting targets, and informative cluster or subcluster size can make those targets materially different. The estimator has to match the declared estimand rather than inheriting weights accidentally from record multiplicity.

Wang, Kong, and Datta (2011) study clustered longitudinal data and show that informative cluster size can invalidate ordinary marginal inference when cluster size is related to the outcome distribution. Huang (2011) further shows that the appropriate weights depend on the population of interest and on within-cluster covariate structure. Kahan et al. (2023) give a clear modern estimand distinction between equal participant weighting and equal cluster weighting. TEPP does not import their treatment-effect estimands; it imports the narrower methodological requirement that aggregation weights are part of estimand identity.

## Decision and documentation boundary

Issue #495 remains open because a branch-local typed API is not the complete scientific acceptance package. Before buyer-facing promotion:

1. The first released target must be named consistently in PRD/TRD/TRACEABILITY and a superseding scientific-estimand ADR. The current ADR directory already contains historical number collisions, so this Draft does not mint another potentially colliding ADR identifier; canonical documentation ownership must allocate and repair that identity before acceptance.
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
