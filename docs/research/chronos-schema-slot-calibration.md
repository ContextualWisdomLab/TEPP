# CHRONOS schema-slot calibration

## Scope

This note doctors the `event_core` gate that keeps CHRONOS schema-slot prediction distinct from event-instance promotion:

1. a filled versus empty slot label is prediction evidence, not a promoted instance or state transition;
2. slot precision and recall are computed from known-truth `(role, argument)` fills;
3. calibrated occupancy probabilities recover the binary fill target with lower RMSE than an always-fill predictor.

No database migration is allocated. A later CHRONOS reasoner may consume these scores as hypothetical schema evidence only.

## Authoritative sources

Anagnostopoulos, E., Batsakis, S., & Petrakis, E. G. M. (2013). CHRONOS: A reasoning engine for qualitative temporal information in OWL. *Procedia Computer Science, 22*, 70–77. https://doi.org/10.1016/j.procs.2013.09.082

Chambers, N., & Jurafsky, D. (2009). Unsupervised learning of narrative schemas and their participants. In *Proceedings of the Joint Conference of the 47th Annual Meeting of the ACL and the 4th International Joint Conference on Natural Language Processing of the AFNLP* (pp. 602–610). Association for Computational Linguistics.

Doddington, G., Mitchell, A., Przybocki, M., Ramshaw, L., Strassel, S., & Weischedel, R. (2004). The Automatic Content Extraction (ACE) program—Tasks, data, and evaluation. In *Proceedings of the Fourth International Conference on Language Resources and Evaluation (LREC’04)* (pp. 837–840). European Language Resources Association.

## Application

Anagnostopoulos et al. (2013) keep CHRONOS completions in a qualitative reasoning layer rather than treating them as observed chronology. Chambers and Jurafsky (2009) evaluate narrative schemas by recovered participant slots, and Doddington et al. (2004) score argument fills with precision and recall against known truth. TEPP therefore refuses to cast a schema prediction as an event instance or transition and requires computed slot precision, recall, and RMSE against known truth (Anagnostopoulos et al., 2013; Chambers & Jurafsky, 2009; Doddington et al., 2004).

## Verification

- `refuse_schema_prediction_as_instance` always returns `SchemaPredictionIsNotEventInstance`;
- `refuse_schema_prediction_as_transition` always returns `SchemaPredictionIsNotStateTransition`;
- `decide_schema_slot` uses an inclusive probability threshold;
- `schema_slot_precision` and `schema_slot_recall` fail closed on empty or duplicate fill sets;
- computed RMSE of known occupancy targets is lower under calibrated probabilities than under an always-fill predictor.
