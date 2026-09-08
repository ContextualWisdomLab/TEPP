# ADR 0026 — Independent lineage-pair criterion and posterior Project Journey

**Decision status:** Proposed
**Implementation maturity:** active-PR — strict wire contracts and Rust CPU independent-criterion estimator
**Date:** 2026-08-26
**Supersedes:** The pre-normalization colliding identity `0024-lineage-pair-criterion-and-project-journey-posterior.md`; ADR 0024 remains the earlier posterior topic-context producer decision. This ADR narrows ADR 0016 and complements ADR 0021.

## Context

Event-Lineage channel weights require an independent pair-level criterion; channel covariance, a fusion floor, keywords, or one run-level accepted flag do not identify it. Project Journey cannot treat the earliest stored record or a fixed sales lifecycle as its start: prior projects, customer requests, procurement notices, direct or negotiated bids, external sensing, internal discussions, and leads can be concurrent, uncertain, or branching.

The previous branch-local numbering collided with the already-existing ADR 0024 posterior topic-context producer contract. Architecture decision identity is repository-wide, so this genuinely distinct later decision is renumbered to ADR 0026 rather than sharing a numeric identity.

## Decision

TEPP publishes `tepp.lineage_pair_criterion_posterior.v2` with bijective pair identity, continuous criterion draws, separate record and event-time draws, TDT/CHRONOS provenance, unique independent anchor alignment, and CPU/GPU receipts over the same objective, parameters, and draws. On Apple Silicon the accelerator receipt is `mlx_metal_macos_native`: Rust remains the computation and validation authority. MLX Metal executes only in a macOS-native Rust-owned service; Compose reaches it through an authenticated local Unix socket or an authenticated host-gateway adapter. It never runs inside Colima's Linux VM. Python never owns or reproduces the arithmetic. Linux container/CI may record MLX CPU or CUDA only when that backend actually ran; it may never emit Metal. The CPU f64 path is an explicitly tested portability/reference fallback, not a silent replacement for a missing MLX execution. The parity bound is method-derived by the producer; a consumer may not choose or repair it.

TEPP separately publishes `tepp.project_journey_posterior.v1`. It retains every event in deterministic, non-semantic identity serialization order and every posterior temporal dependency, branch, and transition draw. That serialization order exists only so identical artifacts produce identical bytes; consumers must not interpret it as temporal order, causal order, rank, priority, or evidence strength. The artifact has no start stage, earliest-row selection, total ordering, rank, or causal status. Multiple predecessors and exact ties remain first-class.

The Rust CHRONOS relation slice compares common event-time draws directly and publishes the complete `before`/`simultaneous`/`after` draw sequence and its posterior frequencies. It estimates no timestamp, uses no tolerance or nearest date, preserves exact ties, and cannot promote a predicted event into fact. An owning estimator may hand the Rust core an identified discrete event-time posterior as unique atoms with integer multiplicities. The core canonicalizes and materializes that mass exactly; zero mass and duplicate atoms fail closed. This operation does not estimate atom locations or posterior mass.

For an independently observed binary TDT link criterion, the Rust scientific core fits the Bernoulli likelihood with Jeffreys' invariant `Beta(1/2, 1/2)` prior and emits posterior mean, variance, and deterministic midpoint-quantile quadrature draws. This is not a thresholded channel score. It does not turn a CHRONOS forecast into a fact, and it carries rather than invents the temporal model's event-time draws.

## Verification and invariants

- record time never substitutes for event time;
- present transition draws never move backward in event time;
- anchor ties, mixed draw counts, missing evidence, non-finite draws, and CPU/MLX digest or parity failures fail closed;
- synthetic tests cover record/event disagreement, multiple predecessors, branches, ties, uncertain relations, backward-edge refusal, anchor ambiguity, and hardware-receipt divergence.

The Rust CPU independent-criterion estimator, exact event-time posterior materializer, and qualitative temporal-relation posterior have deterministic synthetic exact-recovery tests. Estimation of event-time atom locations and mass, artifact assembly with estimator-bound MLX execution, and hardware parity remain unavailable until their owning implementations and receipts pass the same gates.

`analysis_engine::fit_exhaustive_case_deletion` is the normative producer orchestration prerequisite: it invokes one scientific fitter on `D` and every actual `D \ {i}` retained set with mutually domain-separated randomness identities, preserving deleted and retained document identities. It performs no posterior reweighting or approximation. This runner does not make the topic artifact available until a topic likelihood fitter, anchor alignment, incident relation/membership removal receipts, and estimator-bound CPU/accelerator receipts are connected.

## Alternatives considered

1. Earliest record as project start — rejected because reporting delay changes the journey.
2. One fixed lifecycle — rejected because evidence forms different DAGs.
3. Threshold similarity into criterion truth — rejected as circular.
4. Posterior clocks and graph relations with independent criterion evidence — accepted.
5. Keep the colliding ADR 0024 number — rejected because repository-wide architecture authority requires one immutable identity per decision.

## Consequences and trade-offs

Consumers can render branches and uncertainty without local inference. Artifacts are larger because draws, not point labels, cross the boundary. Until the scientific producer exists, the capability remains unavailable.

Historic branches or discussions that refer to `0024-lineage-pair-criterion-and-project-journey-posterior.md` remain provenance only; new canonical references use ADR 0026. The unrelated ADR 0024 posterior topic-context producer identity is unchanged.

## Failure, recovery, security, and privacy

Malformed, oversized, mixed-provenance, ambiguous, temporally reversed, or hardware-divergent artifacts return redacted errors. A missing native Metal receipt on Apple Silicon is unavailable; recovery is replay from the owning fitted run; no consumer repair is permitted. Identity-bearing references and error responses expose only opaque identifiers or digests. The versioned scientific payload still carries the schema-authorized posterior draws, relation sequences, frequencies, uncertainty, and execution receipts required by this decision; the opaque-identity rule does not erase those measurement fields or convert the wire contract into an identity-only envelope.

## Rollback and compatibility

Both schemas are additive. Existing deterministic project history v1 is not reinterpreted as posterior Journey evidence. Rollback disables the new adapter and retains immutable artifacts. Estimator-target or temporal-semantics changes require a superseding ADR and PRD version.

Renumbering rollback does not reuse ADR 0024 for this decision; if ADR 0026 is rejected or superseded, its identity remains historical and a new repository-wide ADR records the successor.

## References

Allan, J. (Ed.). (2002). *Topic detection and tracking: Event-based information organization*. Kluwer Academic Publishers.

Anagnostopoulos, E., Batsakis, S., & Petrakis, E. G. M. (2013). CHRONOS: A reasoning engine for qualitative temporal information in OWL. *Procedia Computer Science, 22*, 70–77. https://doi.org/10.1016/j.procs.2013.09.082

Li, M., Li, S., Wang, Z., Huang, L., Cho, K., Ji, H., Han, J., & Voss, C. (2021). The future is not one-dimensional: Complex event schema induction by graph modeling for event prediction. In *Proceedings of EMNLP 2021* (pp. 5203–5215). https://doi.org/10.18653/v1/2021.emnlp-main.422

Jeffreys, H. (1946). An invariant form for the prior probability in estimation problems. *Proceedings of the Royal Society of London. Series A, Mathematical and Physical Sciences, 186*(1007), 453–461. https://doi.org/10.1098/rspa.1946.0056
