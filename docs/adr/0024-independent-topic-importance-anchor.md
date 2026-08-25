# ADR 0024 — Posterior topic-context producer contract

**Date:** 2026-08-25  
**Decision status:** Accepted  
**Implementation maturity:** contract-only active PR; no estimator emits it  
**Supersedes:** None; composes ADR 0012 and Event Lineage contracts.

## Decision

TEPP owns `tepp.topic_context_posterior.v1`: the exact run, source snapshot
digest, knowledge cutoff, model contract, posterior draw set, global topic
identity and activity intervals, explicit topic-lineage events, per-post
posterior logistic-normal plausible values with event time, and provenance-
bound time-valid BU/PU/team/person multiple memberships.

The artifact never hard-labels a post by thresholding a topic coordinate and
never calls topic prevalence importance. Missing draws, non-finite coordinates,
missing membership dimensions or weights, invalid intervals, duplicate rows,
foreign schemas, and mixed identities fail closed.

fast-mlsirm owns posterior-aware likelihood, observed information, deletion
refits, and the LineageWeave ADR-0210 case-deletion influence diagnostic.
LineageWeave owns authorization, exact provenance persistence, and Dashboard
projection only. Neither consumer performs local mathematical estimation.

## Verification

TEPP release evidence requires known-truth topic, temporal prevalence,
relation, dormancy/reactivation, and membership-effect recovery with bias,
RMSE, interval coverage, posterior diagnostics, leakage-safe splits, CPU worker
determinism, and actual GPU parity. The current CPU-only TRSL-TM estimator does
not satisfy the GPU requirement; this ADR does not claim otherwise.

## References

Blei, D. M., & Lafferty, J. D. (2006). Dynamic topic models. In *Proceedings of
the 23rd International Conference on Machine Learning* (pp. 113–120). ACM.
https://doi.org/10.1145/1143844.1143859

Chang, J., & Blei, D. M. (2009). Relational topic models for document networks.
In *Proceedings of Machine Learning Research, 5*, 81–88.
https://proceedings.mlr.press/v5/chang09a.html

Zhang, D. C., & Lauw, H. (2022). Dynamic topic models for temporal document
networks. In *Proceedings of Machine Learning Research, 162*, 26281–26292.
https://proceedings.mlr.press/v162/zhang22n.html
