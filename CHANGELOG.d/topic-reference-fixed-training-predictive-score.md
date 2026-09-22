# Fixed-training prevalence-mean predictive scoring

`ReferenceTopicTrainingFit` now scores evaluation document counts under the prevalence mean implied by the exact frozen training prevalence basis, fitted prevalence coefficients, and fitted training topic-term probabilities. Evaluation EventTime, covariates, and Membership are projected onto training coordinates without recomputing evaluation-batch statistics, and evaluation counts cannot update fitted global parameters.

The API returns one predictive log likelihood per evaluation document and fails closed on incompatible row/vocabulary/count geometry or frozen-feature dimensions. This is a numerical fixed-training predictive primitive only: it is not STM document-completion likelihood, not the in-sample Schwarz score, and not proof of rolling-origin cutoff/source admission. `corpus_split`/Evidence/Temporal Semantics remain responsible for leakage-safe evaluation admission.
