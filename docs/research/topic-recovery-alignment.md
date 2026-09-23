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

## Additive-log-ratio basis transformation

A topic permutation cannot be applied to a stored additive-log-ratio vector as though the vector contained `K` exchangeable coordinates. TEPP stores `K - 1` ALR coordinates and fixes fitted topic `K - 1` as the local reference. If the truth reference topic maps to another fitted topic, a plain permutation would compare ratios with different denominators.

Let `m(t)` be the fitted-topic index assigned to truth topic `t`. Restore the implicit fitted-reference log score as `s_f(K - 1) = 0`. `validation_core::realign_additive_log_ratio(...)` then emits

`z_t = s_f(m(t)) - s_f(m(K - 1))`, for `t = 0, …, K - 2`.

This is the exact change of ALR reference induced by the topic permutation. It can be applied to document latent ALR coordinates and row-wise to prevalence coefficients expressed in the same fitted ALR basis. Aitchison (1982) establishes log-ratio treatment of compositional data; Egozcue et al. (2003) make explicit that log-ratio coordinates are basis dependent. The recovery transform therefore changes validation coordinates, not the fitted composition itself.

The current contract does **not** transform covariance matrices. Any covariance recovery under a changed ALR basis requires the corresponding linear map `A Σ Aᵀ` as its own validated matrix contract rather than applying this vector helper element-wise.

## Recovery use

For a replication whose selected/fitted `K` equals the known generating `K`, the returned `truth_to_fitted` map is applied consistently before topic-content residuals are formed. Fitted ALR document-state or prevalence quantities are additionally transformed into the truth reference basis before RMSE/bias/coverage. A failed `K` fit/selection remains a failed replication and is not repaired by alignment. When fitted `K != truth K`, this square alignment contract is inapplicable; selected-K error remains the model-selection recovery quantity.

The owner contract is exercised by `crates/validation_core/tests/topic_alignment_contract.rs`, including an exact topic permutation, a case where row-wise greedy nearest-neighbor matching is not the global optimum, and a three-topic case where the truth ALR reference maps to a non-reference fitted topic.

## References

Aitchison, J. (1982). The statistical analysis of compositional data. *Journal of the Royal Statistical Society: Series B (Methodological), 44*(2), 139–160. https://doi.org/10.1111/j.2517-6161.1982.tb01195.x

Egozcue, J. J., Pawlowsky-Glahn, V., Mateu-Figueras, G., & Barceló-Vidal, C. (2003). Isometric logratio transformations for compositional data analysis. *Mathematical Geology, 35*(3), 279–300. https://doi.org/10.1023/A:1023818214614

Gibbs, A. L., & Su, F. E. (2002). On choosing and bounding probability metrics. *International Statistical Review, 70*(3), 419–435. https://doi.org/10.1111/j.1751-5823.2002.tb00178.x

Kuhn, H. W. (1955). The Hungarian method for the assignment problem. *Naval Research Logistics Quarterly, 2*(1–2), 83–97. https://doi.org/10.1002/nav.3800020109

Stephens, M. (2000). Dealing with label switching in mixture models. *Journal of the Royal Statistical Society: Series B (Statistical Methodology), 62*(4), 795–809. https://doi.org/10.1111/1467-9868.00265
