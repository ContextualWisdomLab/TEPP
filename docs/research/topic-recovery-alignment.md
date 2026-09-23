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

The coordinate map is linear. If `z = A x`, uncertainty expressed as covariance in the fitted ALR basis must be carried into the same truth-reference basis as

`Σ_z = A Σ_x Aᵀ`.

`validation_core::realign_additive_log_ratio_covariance(...)` constructs the same topic/reference contrast used by the vector transform and applies that matrix propagation. It accepts only exact `(K - 1) × (K - 1)` finite symmetric positive-semidefinite input, with local scale-relative tolerances limited to binary64 roundoff. Dimension mismatch, ragged matrices, material asymmetry, negative/indefinite covariance, non-finite factorization intermediates, or non-finite transformed output fail closed. A singular covariance is valid when it is positive semidefinite.

This matrix operation is still validation-coordinate arithmetic. It does not turn the fitted approximation into a calibrated posterior, establish interval coverage, mutate estimator state, authenticate Evidence vocabulary/source provenance, or create semantic/release topic identity. Coverage remains an empirical repeated-recovery claim and must be tested after location and covariance have been expressed in the same truth basis.

## EventTime feature-basis transformation

Topic/ALR alignment does not by itself make prevalence coefficients comparable. The reference estimator learns the EventTime feature on the training window's frozen affine coordinate, while the known-topic simulator declares its prevalence intercepts and slopes against its own full-horizon time coordinate. An identical physical trajectory therefore has different intercept and slope values when the center or scale changes.

Write the source coordinate as `x_s = (t - c_s) / s_s` and the target coordinate as `x_t = (t - c_t) / s_t`, with both centers measured in seconds from the same physical origin and both scales strictly positive. For a source trajectory `a_s + b_s x_s`, `validation_core::reexpress_linear_prevalence_time_basis(...)` uses the exact affine reparameterization

`a_t = a_s + b_s (c_t - c_s) / s_s`

and

`b_t = b_s s_t / s_s`.

The transformed coefficients produce the same trajectory at every physical time; only the feature coordinate changes. This operation is separate from `realign_additive_log_ratio(...)`: the EventTime transform acts on the prevalence feature axis, whereas topic alignment acts on the ALR/topic axis. A recovery harness that compares fitted and generating prevalence coefficients must establish both bases against one common physical origin and apply both transformations as required. Directly comparing window-standardized fitted coefficients with full-horizon simulator coefficients would manufacture intercept/slope bias even when the fitted physical trajectory is correct.

`LinearTimeBasis` deliberately accepts already-derived center/scale values rather than reaching into simulation, temporal, or fitted-input owners. It does not authenticate EventTime, infer availability, or mint a cutoff receipt. The #680 acceptance composition must derive the source and target basis geometry from the owner-issued simulation truth and frozen `PrevalenceDesignBasis` before using this arithmetic.

## Recovery use

For a replication whose selected/fitted `K` equals the known generating `K`, the returned `truth_to_fitted` map is applied consistently before topic-content residuals are formed. Fitted ALR document-state or prevalence locations are transformed into the truth reference basis before RMSE/bias. Prevalence coefficient recovery additionally requires the fitted and generating EventTime features to be expressed in one affine time basis before intercept/slope residuals are formed. When interval or covariance recovery is evaluated, the corresponding fitted ALR covariance is transformed with `realign_additive_log_ratio_covariance(...)` before interval construction or coverage comparison. A failed `K` fit/selection remains a failed replication and is not repaired by alignment. When fitted `K != truth K`, this square alignment contract is inapplicable; selected-K error remains the model-selection recovery quantity.

The owner contracts are exercised by `crates/validation_core/tests/topic_alignment_contract.rs`, `crates/validation_core/tests/alr_covariance_alignment_edges.rs`, and `crates/validation_core/tests/prevalence_time_basis_contract.rs`, including an exact topic permutation, a case where row-wise greedy nearest-neighbor matching is not the global optimum, a three-topic case where the truth ALR reference maps to a non-reference fitted topic, identity/singular covariance preservation, the corresponding `A Σ Aᵀ` covariance result, non-finite factorization failure paths, affine EventTime trajectory invariance, identity time-basis preservation, and invalid/overflowing time-basis geometry.

## References

Aitchison, J. (1982). The statistical analysis of compositional data. *Journal of the Royal Statistical Society: Series B (Methodological), 44*(2), 139–160. https://doi.org/10.1111/j.2517-6161.1982.tb01195.x

Egozcue, J. J., Pawlowsky-Glahn, V., Mateu-Figueras, G., & Barceló-Vidal, C. (2003). Isometric logratio transformations for compositional data analysis. *Mathematical Geology, 35*(3), 279–300. https://doi.org/10.1023/A:1023818214614

Gibbs, A. L., & Su, F. E. (2002). On choosing and bounding probability metrics. *International Statistical Review, 70*(3), 419–435. https://doi.org/10.1111/j.1751-5823.2002.tb00178.x

Kuhn, H. W. (1955). The Hungarian method for the assignment problem. *Naval Research Logistics Quarterly, 2*(1–2), 83–97. https://doi.org/10.1002/nav.3800020109

Stephens, M. (2000). Dealing with label switching in mixture models. *Journal of the Royal Statistical Society: Series B (Statistical Methodology), 62*(4), 795–809. https://doi.org/10.1111/1467-9868.00265
