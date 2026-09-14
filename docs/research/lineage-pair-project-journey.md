# Event Lineage criterion and Project Journey research basis

## Scope

This note supports ADR 0026's evidence-state and temporal-graph contract. It
also supports the implemented independent binary TDT-link criterion posterior
and exact materialization of an already identified discrete event-time
posterior, but does not establish completed CHRONOS atom/mass inference, causal
identification, or a universal project lifecycle.

## Synthesis

Allan (2002) separates the TDT task family into segmentation, link detection,
first-story detection, detection, and tracking. From that separation, TEPP
decides that a relatedness channel cannot be its own independent criterion and
that one thresholded similarity response cannot stand in for posterior event
identity; those are ADR 0026 design constraints, not claims made by Allan.

Anagnostopoulos et al. (2013) describe CHRONOS reasoning over qualitative
temporal relations and consistency. Li et al. (2021) model complex event
schemas as graphs rather than a one-dimensional next-event sequence. TEPP
infers from those sources that its product contract should preserve a
posterior partial-order DAG, multiple predecessors, branches, hypothetical
relations, and distinct record/event clocks. Those structures are TEPP's ADR
0026 design, not direct findings of either source, and they do not license a
causal interpretation.

For independently observed binary link outcomes, Jeffreys' invariant prior for
the Bernoulli parameter yields the identified `Beta(s + 1/2, n - s + 1/2)`
posterior. TEPP transports posterior uncertainty using deterministic
midpoint-quantile quadrature draws; these are neither channel-derived weights
nor binary decisions (Jeffreys, 1946).

## Contract implications

- preserve complete criterion and event-time draws;
- preserve exact ties and branching relations instead of date ranking;
- keep observed, inferred, and predicted relations distinct;
- require independent anchor alignment and cutoff provenance;
- publish actual CPU/accelerator execution receipts and method-derived parity;
- fail closed while the scientific estimator or native MLX evidence is absent.

## References (APA 7th)

Allan, J. (Ed.). (2002). *Topic detection and tracking: Event-based information
organization*. Kluwer Academic Publishers.

Jeffreys, H. (1946). An invariant form for the prior probability in estimation
problems. *Proceedings of the Royal Society of London. Series A, Mathematical
and Physical Sciences, 186*(1007), 453–461.
https://doi.org/10.1098/rspa.1946.0056

Anagnostopoulos, E., Batsakis, S., & Petrakis, E. G. M. (2013). CHRONOS: A
reasoning engine for qualitative temporal information in OWL. *Procedia
Computer Science, 22*, 70–77. https://doi.org/10.1016/j.procs.2013.09.082

Li, M., Li, S., Wang, Z., Huang, L., Cho, K., Ji, H., Han, J., & Voss, C.
(2021). The future is not one-dimensional: Complex event schema induction by
graph modeling for event prediction. In *Proceedings of the 2021 Conference on
Empirical Methods in Natural Language Processing* (pp. 5203–5215).
https://doi.org/10.18653/v1/2021.emnlp-main.422
