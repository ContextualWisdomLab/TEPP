# ADR 0024 — Independent lineage-pair criterion and posterior Project Journey

**Decision status:** Proposed  
**Implementation maturity:** active-PR — strict wire contracts and Rust CPU independent-criterion estimator  
**Date:** 2026-08-26  
**Supersedes:** None; narrows ADR 0016 and complements ADR 0021.

## Context

Event-Lineage channel weights require an independent pair-level criterion;
channel covariance, a fusion floor, keywords, or one run-level accepted flag do
not identify it. Project Journey cannot treat the earliest stored record or a
fixed sales lifecycle as its start: prior projects, customer requests,
procurement notices, direct or negotiated bids, external sensing, internal
discussions, and leads can be concurrent, uncertain, or branching.

## Decision

TEPP publishes `tepp.lineage_pair_criterion_posterior.v2` with bijective pair
identity, continuous criterion draws, separate record and event-time draws,
TDT/CHRONOS provenance, unique independent anchor alignment, and CPU/GPU
receipts over the same objective, parameters, and draws. On Apple Silicon the
accelerator receipt is `mlx_metal_macos_native`: Rust remains the computation
and validation authority. MLX Metal executes only in a macOS-native Rust-owned
service; Compose reaches it through an authenticated local Unix socket or an
authenticated host-gateway adapter. It never runs inside Colima's Linux VM.
Python never owns or reproduces the arithmetic. Linux container/CI may record
MLX CPU or CUDA only when that backend actually ran; it may never emit Metal.
The CPU f64 path is an explicitly tested portability/reference fallback, not a
silent replacement for a missing MLX execution. The parity bound is
method-derived by the producer; a consumer may not choose or repair it.

TEPP separately publishes `tepp.project_journey_posterior.v1`. It retains every
event in stable identity order and every posterior temporal dependency, branch,
and transition draw. It has no start stage, earliest-row selection, total
ordering, rank, or causal status. Multiple predecessors and exact ties remain
first-class.

For an independently observed binary TDT link criterion, the Rust scientific
core fits the Bernoulli likelihood with Jeffreys' invariant
`Beta(1/2, 1/2)` prior and emits posterior mean, variance, and deterministic
midpoint-quantile quadrature draws. This is not a thresholded channel score.
It does not turn a CHRONOS forecast into a fact, and it carries rather than
invents the temporal model's event-time draws.

## Verification and invariants

- record time never substitutes for event time;
- present transition draws never move backward in event time;
- anchor ties, mixed draw counts, missing evidence, non-finite draws, and
  CPU/MLX digest or parity failures fail closed;
- synthetic tests cover record/event disagreement, multiple predecessors,
  branches, ties, uncertain relations, backward-edge refusal, anchor ambiguity,
  and hardware-receipt divergence.

The Rust CPU independent-criterion estimator has deterministic synthetic
parameter-recovery tests. Full TDT/CHRONOS temporal inference, artifact
assembly with actual MLX execution, and hardware parity remain unavailable
until their owning implementations and receipts pass the same gates.

## Alternatives considered

1. Earliest record as project start — rejected because reporting delay changes
   the journey.
2. One fixed lifecycle — rejected because evidence forms different DAGs.
3. Threshold similarity into criterion truth — rejected as circular.
4. Posterior clocks and graph relations with independent criterion evidence —
   accepted.

## Consequences and trade-offs

Consumers can render branches and uncertainty without local inference.
Artifacts are larger because draws, not point labels, cross the boundary. Until
the scientific producer exists, the capability remains unavailable.

## Failure, recovery, security, and privacy

Malformed, oversized, mixed-provenance, ambiguous, temporally reversed, or
hardware-divergent artifacts return redacted errors. A missing native Metal
receipt on Apple Silicon is unavailable; recovery is replay from
the owning fitted run; no consumer repair is permitted. Only opaque identities
and digests cross the contract.

## Rollback and compatibility

Both schemas are additive. Existing deterministic project history v1 is not
reinterpreted as posterior Journey evidence. Rollback disables the new adapter
and retains immutable artifacts. Estimator-target or temporal-semantics changes
require a superseding ADR and PRD version.

## References

Allan, J. (Ed.). (2002). *Topic detection and tracking: Event-based information
organization*. Kluwer Academic Publishers.

Anagnostopoulos, E., Batsakis, S., & Petrakis, E. G. M. (2013). CHRONOS: A
reasoning engine for qualitative temporal information in OWL. *Procedia
Computer Science, 22*, 70–77. https://doi.org/10.1016/j.procs.2013.09.082

Li, M., Li, S., Wang, Z., Huang, L., Cho, K., Ji, H., Han, J., & Voss, C.
(2021). The future is not one-dimensional: Complex event schema induction by
graph modeling for event prediction. In *Proceedings of EMNLP 2021* (pp.
5203–5215). https://doi.org/10.18653/v1/2021.emnlp-main.422

Jeffreys, H. (1946). An invariant form for the prior probability in estimation
problems. *Proceedings of the Royal Society of London. Series A, Mathematical
and Physical Sciences, 186*(1007), 453–461.
https://doi.org/10.1098/rspa.1946.0056
