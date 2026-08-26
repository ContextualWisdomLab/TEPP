# ADR 0024 — Posterior topic-context producer contract

**Date:** 2026-08-25  
**Decision status:** Accepted  
**Implementation maturity:** active-PR — contract validation and deterministic
CPU joint Laplace plausible values; no complete producer artifact yet
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

The standalone CPU reference-fit result retains only per-document diagonal
Laplace variances. Its Rust API therefore reports `DiagonalLaplace` and refuses
requests for a joint coordinate precision with `JointPosteriorUnavailable`.
Those diagonal entries cannot be used as independent plausible-value draws:
the fitted relational penalty and shared structural coefficients induce
dependence that the retained diagonal does not identify. The separate
fit-bound input API recomputes and validates the full identified joint
precision while the exact admitted documents, event instants, relations, and
stable topic basis remain available.

The CPU prerequisite builds an identified document-major joint precision
at a converged MAP fit. Its within-document block is the conditional
multinomial information used by generalized EM plus Gaussian prior precision;
each admitted relation adds the positive-semidefinite generalized-Gauss-Newton
blocks from the softmax Jacobians of the harmonic network residual. The Rust
boundary validates stable topic order, document order, finiteness, symmetry,
and positive-definiteness. It is the only precision accepted by the governed
deterministic sampler below; the standalone diagonal fit output remains
ineligible.

The CPU draw boundary now uses Philox4x32-10 counters keyed by an explicit
`u64` seed, a versioned Box-Muller transform, and an upper-triangular Cholesky
solve. It never substitutes the older diagonal variances. A SHA-256 draw-set
identity binds the algorithm version, seed, draw count, ordered document and
topic identities, MAP coordinates, joint precision, and emitted values. The
counter layout assigns each `(draw_index, normal_block)` independently of
execution order so a later GPU implementation can test the same stream
contract (Salmon et al., 2011). Empirical covariance tests use probability
bounds derived from Gaussian second moments rather than an arbitrary tolerance.
The CPU implementation returns exactly the requested coordinate count for both
odd and even dimensions; unused Box-Muller partners are not exposed as part of
the versioned CPU/GPU stream contract.

Every accepted lineage, document-relation, and membership provenance assertion
is bound by length-prefixed SHA-256 input to its complete asserted identity,
including event time or validity window, evidence resource and digest, source
snapshot, and membership weight where applicable. Reusing one assertion ID for
records that differ only in time therefore fails closed rather than silently
collapsing distinct temporal claims.

This still is not a complete `tepp.topic_context_posterior.v1` producer. The
topic fit binds each admitted event instant but does not own the declared event
clock identity, source snapshot/run/cutoff identity, activity intervals,
qualified document-lineage evidence, or time-valid membership provenance. The
analysis layer must bind those exact records before assembling an artifact;
absence of any required record remains unavailable.

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

Box, G. E. P., & Muller, M. E. (1958). A note on the generation of random
normal deviates. *The Annals of Mathematical Statistics, 29*(2), 610–611.
https://doi.org/10.1214/aoms/1177706645

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

Salmon, J. K., Moraes, M. A., Dror, R. O., & Shaw, D. E. (2011). Parallel
random numbers: As easy as 1, 2, 3. In *Proceedings of 2011 International
Conference for High Performance Computing, Networking, Storage and Analysis*
(Article 16). Association for Computing Machinery.
https://doi.org/10.1145/2063384.2063405

## Rollback and supersession

Disable production or consumption of this schema while retaining source
artifacts and provenance. Never replace missing posterior output with a local
score or hard label. Any change to estimand, draw semantics, temporal identity,
membership meaning, or ownership requires a versioned schema and superseding
ADR.
