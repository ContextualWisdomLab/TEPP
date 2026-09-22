# Scientific recovery binds candidate fits to the declared numerical design

`select_declared_rolling_origin_recovery_candidate_k` now rejects dimension-compatible `ReferenceTopicTrainingFit` values whose retained `ReferenceTopicModelConfig` does not exactly match the candidate-specific configuration derived from the predeclared `FittedCandidateKConfig`. Seeds, convergence controls, and estimator hyperparameters therefore cannot be substituted after the recovery design is declared.

`FittedCandidateKConfig` is now the single crate-owned construction path for candidate-specific reference configurations used by both ordinary fitted selection and scientific recovery verification. A configuration mismatch is a typed failed-recovery outcome and must remain in the unconditional replication denominator; it is not evidence, Membership, relation, source/event-time, or release authority.
