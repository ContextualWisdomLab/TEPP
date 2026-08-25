# ADR 0024 — Posterior topic-context producer contract

**Date:** 2026-08-25  
**Decision status:** Accepted  
**Implementation maturity:** active-PR — contract validation and an explicit
joint-posterior unavailable boundary; no estimator emits it
**Supersedes:** None; composes ADR 0012 and Event Lineage contracts.

## Context

LineageWeave needs posterior topic coordinates and time-valid organizational
memberships without reimplementing TEPP measurement or turning an uncalibrated
coordinate into business importance. The downstream influence estimator also
needs an exact, provenance-bound input rather than hard topic labels.

## Decision

TEPP owns `tepp.topic_context_posterior.v1`: the exact run, source snapshot
digest, knowledge cutoff, model contract, declared event clock, posterior draw
set, opaque stable global topic identities and activity intervals, explicit
topic-lineage events, admitted Event Lineage document relations, per-post
posterior logistic-normal plausible values with event time, and provenance-
bound time-valid BU/PU/team/person multiple memberships. The ordered
`topic_ids` array alone defines artifact-local coordinate order; consumers join
activity and lineage by stable topic identity. Lineage, document-relation, and
membership evidence carries a content digest, immutable evidence-resource
identifier, and provenance-assertion identifier so a consumer can materialize
normalized qualified provenance rather than treating a digest as provenance.
All record collections are canonically sorted before serialization so
equivalent input permutations produce identical JSON and SHA-256. Version 1
admits only `event_lineage_precedes`, with source event time no later than the
target event time; topic event target cardinality and activity transitions fail
closed.

The artifact never hard-labels a post by thresholding a topic coordinate and
never calls topic prevalence importance. Missing draws, non-finite coordinates,
missing membership dimensions or weights, invalid intervals, duplicate rows,
foreign schemas, and mixed identities fail closed.

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

The CPU reference fit currently retains only per-document diagonal Laplace
variances. Its Rust API reports `DiagonalLaplace` and refuses requests for a
joint coordinate precision with `JointPosteriorUnavailable`. Those diagonal
entries cannot be used as independent plausible-value draws: the fitted
relational penalty and shared structural coefficients induce dependence that
the retained diagonal does not identify. A producer can become available only
after the estimator retains and validates the full identified joint precision,
then binds document order, stable topic order, run, source snapshot, and cutoff
identities before sampling.

The next CPU prerequisite builds an identified document-major joint precision
at a converged MAP fit. Its within-document block is the conditional
multinomial information used by generalized EM plus Gaussian prior precision;
each admitted relation adds the positive-semidefinite generalized-Gauss-Newton
blocks from the softmax Jacobians of the harmonic network residual. The Rust
boundary validates stable topic order, document order, finiteness, symmetry,
and positive-definiteness. This matrix is not yet a covariance or draw set;
inversion and deterministic sampling remain unavailable until separately
governed and recovery-tested.

## Verification

TEPP release evidence requires realistic synthetic known-truth parameter
recovery, RMSE, bias, interval coverage, temporal ordering, graph recovery,
invariance, posterior diagnostics, leakage-safe splits, CPU worker determinism,
and actual CPU/GPU parameter and objective parity. A skipped, ignored,
emulated, or unavailable-device GPU test is not GPU evidence. The current
CPU-only TRSL-TM estimator does not emit this artifact or satisfy the GPU
requirement; this contract-only ADR does not claim otherwise.

## References

Blei, D. M., & Lafferty, J. D. (2006). Dynamic topic models. In *Proceedings of
the 23rd International Conference on Machine Learning* (pp. 113–120). ACM.
https://doi.org/10.1145/1143844.1143859

Chang, J., & Blei, D. M. (2009). Relational topic models for document networks.
In *Proceedings of Machine Learning Research, 5*, 81–88.
https://proceedings.mlr.press/v5/chang09a.html

Zhang, D. C., & Lauw, H. W. (2022). Dynamic topic models for temporal document
networks. In *Proceedings of Machine Learning Research, 162*, 26281–26292.
https://proceedings.mlr.press/v162/zhang22n.html

Rue, H., Martino, S., & Chopin, N. (2009). Approximate Bayesian inference for
latent Gaussian models by using integrated nested Laplace approximations.
*Journal of the Royal Statistical Society: Series B (Statistical Methodology),
71*(2), 319–392. https://doi.org/10.1111/j.1467-9868.2008.00700.x

Schraudolph, N. N. (2002). Fast curvature matrix-vector products for
second-order gradient descent. *Neural Computation, 14*(7), 1723–1738.
https://doi.org/10.1162/08997660260028683

## Rollback and supersession

Disable production or consumption of this schema while retaining source
artifacts and provenance. Never replace missing posterior output with a local
score or hard label. Any change to estimand, draw semantics, temporal identity,
membership meaning, or ownership requires a versioned schema and superseding
ADR.
