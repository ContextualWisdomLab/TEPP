# Predicted Temporal Assertions Versus Observed Evidence

## Purpose

This note traces the `temporal_core` predicted-assertion gate to Allen-style interval networks and the ADR 0016 separation of CHRONOS/TDT prediction from observed event evidence. References use APA 7th style.

The increment does not implement TDT detection, CHRONOS schema-slot filling, or promoted state-transition edges. It only keeps hypothetical qualitative relations from rewriting an observed constraint network.

## Contract

`TemporalReasoner::assert_relation` remains the observed assertion path. `assert_predicted_relation` accepts a nonempty Allen relation set as a hypothesis:

- a predicted pair is not observed;
- a prediction that is compatible with an already-observed pair is accepted without tightening the observed relation set;
- a prediction that empties an observed pair (or its inverse) is rejected as `PredictedRelationRejected` and leaves the observed cells unchanged;
- two incompatible predictions with no observed support remain an ordinary network `Contradiction`.

Path-consistency closure still does not certify unrestricted Allen satisfiability (Vilain & Kautz, 1986). Rejecting a prediction is not evidence that the observed chronology is complete.

## Sources

Allen, J. F. (1983). Maintaining knowledge about temporal intervals. *Communications of the ACM, 26*(11), 832–843. https://doi.org/10.1145/182.358434

Anagnostopoulos, E., Batsakis, S., & Petrakis, E. G. M. (2013). CHRONOS: A reasoning engine for qualitative temporal information in OWL. *Procedia Computer Science, 22*, 70–77. https://doi.org/10.1016/j.procs.2013.09.082

Allan, J. (2002). *Topic detection and tracking: Event-based information organization*. Springer. https://doi.org/10.1007/978-1-4615-0933-2

Vilain, M. B., & Kautz, H. A. (1986). Constraint propagation algorithms for temporal reasoning. In *Proceedings of the Fifth National Conference on Artificial Intelligence* (pp. 377–382). American Association for Artificial Intelligence. https://www.aaai.org/Papers/AAAI/1986/AAAI86-063.pdf
