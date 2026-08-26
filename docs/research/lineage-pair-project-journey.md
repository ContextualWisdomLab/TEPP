# Event Lineage criterion and Project Journey research basis

## Scope

This note supports ADR 0023's evidence-state and temporal-graph contract. It
does not establish an implemented TDT/CHRONOS estimator, causal identification,
or a universal project lifecycle.

## Synthesis

Allan's TDT task family separates segmentation, link detection, first-story
detection, detection, and tracking. A relatedness channel therefore cannot be
its own independent criterion, and one thresholded similarity response cannot
stand in for posterior event identity.

CHRONOS reasons over qualitative temporal relations and consistency rather than
reducing uncertain evidence to the date on which a record was stored. Li et al.
model complex event schemas as graphs and explicitly reject a one-dimensional
next-event view. Together these sources support a posterior partial-order DAG
with multiple predecessors, branches, hypothetical relations, and distinct
record/event clocks; they do not license causal interpretation.

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

Anagnostopoulos, E., Batsakis, S., & Petrakis, E. G. M. (2013). CHRONOS: A
reasoning engine for qualitative temporal information in OWL. *Procedia
Computer Science, 22*, 70–77. https://doi.org/10.1016/j.procs.2013.09.082

Li, M., Li, S., Wang, Z., Huang, L., Cho, K., Ji, H., Han, J., & Voss, C.
(2021). The future is not one-dimensional: Complex event schema induction by
graph modeling for event prediction. In *Proceedings of the 2021 Conference on
Empirical Methods in Natural Language Processing* (pp. 5203–5215).
https://doi.org/10.18653/v1/2021.emnlp-main.422
