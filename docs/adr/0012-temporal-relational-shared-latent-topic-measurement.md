# ADR 0012 — Temporal Relational Shared-Latent Topic Measurement

**Decision status:** Accepted  
**Implementation maturity:** accepted-target — prompt-versus-unique-content identity in `prompt_source` on the active PR; estimator-side method model remains accepted-target  
**Date:** 2026-08-12  
**Supersedes:** None; refines ADR 0004 and ADR 0005 without replacing their multilingual and psychometric authorities.

## Context

TEPP needs a stable scientific contract for the text-measurement layer itself. ADR 0004 establishes one multilingual latent space and ADR 0005 governs posterior-aware downstream psychometrics, but neither by itself defines the topic estimator, temporal identity of topics, backend substitutability, method/background effects, or model-selection authority.

Without a dedicated decision, implementations could silently drift toward independent monolingual clustering, raw embedding clusters, keyword matching, TF-IDF/BM25-weighted pseudo-inference, per-time-bin unrelated topic identities, or point-estimate outputs that downstream ESEM/DSEM cannot interpret consistently.

## Decision

TEPP adopts **Temporal Relational Shared-Latent Topic Measurement (TRSL-TM)** as the model-family contract. The initial reference backend is a temporal/relational shared-latent STM-style estimator with logistic-normal document coordinates and posterior uncertainty. Alternative polylingual or neural/contextual backends are allowed only when they satisfy the same versioned evidence, temporal, relational, posterior, invariance, and interoperability contracts.

For the first production line:

- one global topic identity set is selected across the modeled period;
- topics may be active, dormant, or reactivated over time without losing identity;
- topic birth/split/merge/retirement is a later explicit lineage extension, not an implicit consequence of fitting unrelated time slices;
- multilingual semantic/concept evidence shares topic identities, while native lexical/morphological channels remain language-specific;
- repeated template, section, copied-text, style, modality, source, and corpus-background effects are modeled explicitly as method/background structure;
- stopword deletion is not the default preprocessing rule;
- TF-IDF and BM25 are not inferential weights for the statistical topic estimator;
- topic proportions are compositional and downstream network/psychometric analysis uses logistic-normal coordinates or valid orthonormal log-ratio coordinates;
- model selection uses statistical/recovery/stability/alignment/fairness gates and a Pareto-style comparison before any blinded LLM review;
- the LLM may recommend among statistically admissible candidates but never defines the numerical optimum or bypasses diagnostics.

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
