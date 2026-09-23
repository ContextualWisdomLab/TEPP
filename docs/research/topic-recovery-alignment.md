# Known-topic recovery alignment

## Scope

TEPP recovery studies compare fitted topic content and downstream prevalence/state quantities with a simulator-owned known topic basis. Component indexes are not treated as stable scientific identities. Before row-wise recovery error is computed, `validation_core::align_topic_probability_rows(...)` finds one deterministic one-to-one mapping from truth topics to fitted topics.

This is a validation coordinate map. It does not reorder or mutate the fitted model, assign semantic topic names, authenticate the vocabulary/source population, or create release-level topic identity. Those remain separate owner concerns.

## Metric and assignment rule

For each truth/fitted topic pair, the owner computes squared Hellinger distance over their validated topic-term probability rows:

`H²(p, q) = 0.5 * Σ_v (sqrt(p_v) - sqrt(q_v))²`.

The rows must be finite, nonnegative, normalized probability distributions with the same vocabulary width. Gibbs and Su (2002) review Hellinger among bounded probability metrics and emphasize that the metric used to compare distributions is itself a substantive choice. TEPP therefore records this choice rather than treating label matching as an incidental array permutation.

Pairwise distances form a square cost matrix. The owner solves the global one-to-one minimum-cost assignment using the Hungarian method family rather than greedily choosing the nearest fitted row for each truth row. Kuhn (1955) establishes the assignment formulation and algorithmic method. Exact numerical ties use ascending fitted-topic index as the deterministic traversal order; that rule is reproducibility behavior, not semantic topic identity.

Stephens (2000) documents the label-switching problem created by likelihood symmetry in finite mixture models and motivates explicit relabeling/alignment before component-specific quantities are compared. TEPP uses the known generating topic-term distributions as the recovery reference; it does not impose an artificial semantic ordering on the fitted estimator.

## Recovery use

For a replication whose selected/fitted `K` equals the known generating `K`, the returned `truth_to_fitted` map is applied consistently before topic-content, prevalence-trajectory, or document-state residuals are formed. A failed `K` fit/selection remains a failed replication and is not repaired by alignment. When fitted `K != truth K`, this square alignment contract is inapplicable; selected-K error remains the model-selection recovery quantity.

The first owner contract is exercised by `crates/validation_core/tests/topic_alignment_contract.rs`, including an exact permutation and a case where row-wise greedy nearest-neighbor matching is not the global optimum.

## References

Gibbs, A. L., & Su, F. E. (2002). On choosing and bounding probability metrics. *International Statistical Review, 70*(3), 419–435. https://doi.org/10.1111/j.1751-5823.2002.tb00178.x

Kuhn, H. W. (1955). The Hungarian method for the assignment problem. *Naval Research Logistics Quarterly, 2*(1–2), 83–97. https://doi.org/10.1002/nav.3800020109

Stephens, M. (2000). Dealing with label switching in mixture models. *Journal of the Royal Statistical Society: Series B (Statistical Methodology), 62*(4), 795–809. https://doi.org/10.1111/1467-9868.00265
