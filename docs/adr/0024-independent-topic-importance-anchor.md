# ADR 0024 — Posterior topic-context producer contract

**Date:** 2026-08-25  
**Decision status:** Accepted  
**Implementation maturity:** active-PR — contract validation plus identity-bound
Laplace moment export; no posterior sampler or GPU estimator emits it
**Supersedes:** None; composes ADR 0012 and Event Lineage contracts.

## Context

LineageWeave needs posterior topic coordinates and time-valid organizational
memberships without reimplementing TEPP measurement or turning an uncalibrated
coordinate into business importance. The downstream influence estimator also
needs an exact, provenance-bound input rather than hard topic labels.

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

The existing CPU reference fit exposes diagonal Laplace variances, not a joint
posterior draw law. TEPP may therefore export each document identity with its
fitted ALR mean and diagonal Laplace variance as an intermediate moment set,
but must not relabel those moments as plausible values. Emitting this artifact
still requires a versioned posterior sampling algorithm (including reproducible
random-stream semantics) and an executing GPU backend; the current GPU module
plans memory and recovery but does not execute topic kernels.

fast-mlsirm owns posterior-aware likelihood, observed information, deletion
refits, and the LineageWeave ADR-0210 case-deletion influence diagnostic.
LineageWeave owns authorization, exact provenance persistence, and Dashboard
projection only. Neither consumer performs local mathematical estimation.

## Alternatives considered

1. **Hard topic labels or local thresholds** — rejected because they discard
   posterior uncertainty and invent a decision rule.
2. **A LineageWeave-owned estimator** — rejected because it violates the TEPP
   measurement boundary.
3. **The bounded posterior contract** — accepted because it preserves draws,
   time, lineage, memberships, and provenance without claiming an estimator.

## Consequences

Consumers can validate exact posterior inputs independently, but no importance
result exists until fast-mlsirm implements and validates the governed
posterior-aware estimator. Contract acceptance therefore remains distinct from
model availability.

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

Zhang, D. C. W., & Lauw, H. W. (2022). Dynamic topic models for temporal document
networks. In *Proceedings of Machine Learning Research, 162*, 26281–26292.
https://proceedings.mlr.press/v162/zhang22n.html

## Rollback and supersession

Disable production or consumption of this schema while retaining source
artifacts and provenance. Never replace missing posterior output with a local
score or hard label. Any change to estimand, draw semantics, temporal identity,
membership meaning, or ownership requires a versioned schema and superseding
ADR.
