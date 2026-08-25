# ADR 0012 — Temporal Relational Shared-Latent Topic Measurement

**Date:** 2026-08-24
**Decision status:** Accepted
**Implementation maturity:** coordinates and the CPU `f64` reference estimator are implemented-main; fitted candidate-`K` scoring is this PR; method effects and GPU remain accepted-target.
**Supersedes:** None; refines ADR 0004 and ADR 0005 without replacing their multilingual and psychometric authorities.

## Context

TEPP needs a stable scientific contract for the text-measurement layer itself. ADR 0004 establishes one multilingual latent space and ADR 0005 governs posterior-aware downstream psychometrics, but neither by itself defines the topic estimator, temporal identity of topics, backend substitutability, method/background effects, or model-selection authority.

Without a dedicated decision, implementations could silently drift toward independent monolingual clustering, raw embedding clusters, keyword matching, TF-IDF/BM25-weighted pseudo-inference, per-time-bin unrelated topic identities, or point-estimate outputs that downstream ESEM/DSEM cannot interpret consistently.

## Decision

TEPP adopts **Temporal Relational Shared-Latent Topic Measurement (TRSL-TM)** as the product model-family contract. TRSL-TM is the accepted-target estimator identity; it is not a claim that a production topic backend is already shipped on protected `main`.

The reference family is a temporal/relational shared-latent STM-style estimator with logistic-normal document coordinates and posterior uncertainty (Roberts et al., 2014, 2019; Chang & Blei, 2009). Temporal topic identity over a modeled period follows the dynamic topic-model family (Blei & Lafferty, 2006): one global topic identity set may change in prevalence without silently becoming a new topic merely because a time slice was fitted independently. Alternative polylingual or neural/contextual backends are allowed only when they satisfy the same versioned evidence, temporal, relational, posterior, invariance, and interoperability contracts.

For the first production line:

- one global topic identity set is selected across the modeled period;
- topics may be active, dormant, or reactivated over time without losing identity;
- topic birth/split/merge/retirement is a later explicit lineage extension, not an implicit consequence of fitting unrelated time slices;
- multilingual semantic/concept evidence shares topic identities, while native lexical/morphological channels remain language-specific (Mimno et al., 2009);
- repeated template, section, copied-text, style, modality, source, and corpus-background effects are modeled explicitly as method/background structure;
- stopword deletion is not the default preprocessing rule;
- TF-IDF and BM25 are not inferential weights for the statistical topic estimator;
- topic proportions are compositional and downstream network/psychometric analysis uses logistic-normal coordinates or valid orthonormal log-ratio coordinates;
- ALR is a reference-dependent full-rank logistic-normal map, not an Aitchison-distance isometry; distance-based Euclidean Aitchison geometry uses an orthonormal ILR basis;
- model selection uses statistical/recovery/stability/alignment/fairness gates and a Pareto-style comparison before any blinded LLM review;
- the LLM may recommend among statistically admissible candidates but never defines the numerical optimum or bypasses diagnostics.

### CPU `f64` reference estimand and inference

The bounded reference estimator uses sparse document-by-term counts `C`, one
global `K`-topic word matrix `β`, and document logistic-normal coordinates
`η_d`, with `θ_d = softmax([η_d, 0])`. Its prevalence mean is the PRD-owned
structural equation

\[
m_d = x_d\Gamma + \sum_{g \in G_d} w_{dg}u_g,
\qquad
\eta_d \sim N(m_d, \sigma^2 I),
\]

where `x_d` includes an intercept, standardized event time, and admitted
prevalence covariates, while the second term retains every active weighted
cross-classified/multiple-membership assignment. This is the logistic-normal
prevalence boundary of correlated/structural topic models, not a raw-simplex
regression (Blei & Lafferty, 2007; Roberts et al., 2019).

For explicit observed predecessor/successor relations only, the reference
objective adds the harmonic network penalty

\[
R(\Theta,G)=\frac{1}{2}\sum_{(d,e)\in E}a_{de}
\lVert\theta_d-\theta_e\rVert_2^2,
\]

so absent relations remain unobserved rather than negative. This follows the
document-network regularization estimand of Mei et al. (2008); it is not a
causal edge, an event-identity promotion, or an RTM link-probability claim.
The full bounded MAP objective is

\[
\sum_{d,v} C_{dv}\log\!\left(\sum_k\theta_{dk}\beta_{kv}\right)
-\frac{1}{2\sigma^2}\sum_d\lVert\eta_d-m_d\rVert_2^2
-\lambda R(\Theta,G)-\frac{\rho}{2}\lVert\Gamma,u\rVert_2^2.
\]

Production inference uses deterministic generalized EM: normalized latent
term-topic responsibilities, smoothed multinomial `β` updates, bounded
gradient updates for `η` and structural coefficients, and a diagonal Laplace
curvature approximation for document-coordinate uncertainty. Multiple seeded
initializations retain the best finite converged objective. A non-finite
intermediate, invalid sparse matrix, missing cutoff-safe document, reverse
transition, or exhausted iteration budget returns a typed failure; it never
emits a partial topic artifact. Recovery gates remain caller-owned promotion
criteria over completed validation evidence.

## Non-goals

This ADR does not select one neural architecture forever, claim a unique true topic count for every corpus, or authorize topic labels as causal constructs. It does not allow a fitted backend to redefine TEPP's temporal/event/membership or evidence semantics.

## Alternatives considered

1. **Translate everything into one language and fit ordinary STM** — rejected as the primary architecture because translation error and cultural/lexical differences become uncontrolled measurement error.
2. **Embedding clustering as the production topic model** — rejected because it does not preserve the required prevalence/content covariates, posterior uncertainty, temporal identity, and psychometric interfaces by default.
3. **Independent per-language or per-time topic models followed by matching** — rejected because topic identity and longitudinal invariance become post-hoc and underidentified.
4. **Shared-latent temporal/relational model family with swappable compliant backends** — accepted.

## Consequences

- every backend publishes the same semantic model contract and comparable validation evidence;
- semantic, lexical, temporal, relational, and method-effect channels remain distinguishable;
- per-time prevalence change can be separated from semantic, lexical, and measurement drift;
- candidate topic counts and backend choices become versioned model artifacts rather than ad-hoc notebook settings;
- naruon or another consumer cannot substitute keyword matching for a TEPP inference result while claiming equivalence.

## Failure and recovery

A model that fails convergence, recovery, language alignment, invariance, relation-aware leakage checks, posterior diagnostics, or configured resource limits is not promoted. A backend can fall back to a validated CPU reference or an earlier approved backend/version without changing the estimand. If no candidate meets the scientific gate, the system returns unresolved/abstain rather than fabricating topics.

## Security, privacy, and governance impact

Concept evidence and topic artifacts inherit source-data sensitivity. Provider/model calls are evidence-minimized and purpose-bound. Topic labels and LLM interpretations cannot alter numeric parameters, source evidence, membership structure, or release gates.

## Compatibility and migration

Backend changes require a versioned model contract, migration notes, reproducibility manifest, comparison against the previous backend, and explicit compatibility evidence for downstream network/ESEM/DSEM consumers. A change that alters latent-variable meaning requires a superseding ADR and PRD version update.

## Verification

Required evidence includes known-truth topic/covariate/covariance recovery, bias/RMSE/interval coverage, held-out predictive evidence, seed/bootstrap stability, relation-aware split integrity, language alignment/invariance, method-effect recovery, known-K/acceptable-set behavior, posterior calibration, and downstream coordinate compatibility. CPU/GPU implementations additionally require parity under ADR 0001/0006.

## Rollback and supersession

Rollback selects the last validated model/backend contract and immutable model artifact. Supersede only with evidence that the new model family preserves or explicitly and deliberately changes the estimand, with corresponding PRD/ADR and migration updates.

## References

Blei, D. M., & Lafferty, J. D. (2006). Dynamic topic models. In *Proceedings of the 23rd International Conference on Machine Learning* (pp. 113–120). ACM. https://doi.org/10.1145/1143844.1143859

Chang, J., & Blei, D. M. (2009). Relational topic models for document networks. In *Proceedings of the 12th International Conference on Artificial Intelligence and Statistics* (pp. 81–88). PMLR.

Mimno, D., Wallach, H. M., Naradowsky, J., Smith, D. A., & McCallum, A. (2009). Polylingual topic models. In *Proceedings of the 2009 Conference on Empirical Methods in Natural Language Processing* (pp. 880–889). Association for Computational Linguistics.

Roberts, M. E., Stewart, B. M., & Tingley, D. (2019). stm: An R package for structural topic models. *Journal of Statistical Software, 91*(2), 1–40. https://doi.org/10.18637/jss.v091.i02

Roberts, M. E., Stewart, B. M., Tingley, D., Lucas, C., Leder-Luis, J., Gadarian, S. K., Albertson, B., & Rand, D. G. (2014). Structural topic models for open-ended survey responses. *American Journal of Political Science, 58*(4), 1064–1082. https://doi.org/10.1111/ajps.12103

Blei, D. M., & Lafferty, J. D. (2007). A correlated topic model of Science.
*The Annals of Applied Statistics, 1*(1), 17–35.
https://doi.org/10.1214/07-AOAS114

Mei, Q., Cai, D., Zhang, D., & Zhai, C. (2008). Topic modeling with network
regularization. In *Proceedings of the 17th International Conference on World
Wide Web* (pp. 101–110). Association for Computing Machinery.
https://doi.org/10.1145/1367497.1367512
