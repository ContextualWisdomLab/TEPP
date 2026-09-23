# Held-out document-state recovery

- Add a frozen-training local MAP-style topic-state inference path for evaluation documents. Held-out counts may update only their own ALR coordinate; fitted topic-term probabilities, prevalence coefficients, relation parameters, training document states, and training input remain unchanged.
- Add fail-closed coverage for row/vocabulary/EventTime/Membership/count geometry and numerical non-convergence.
- Add simulator-backed out-of-temporal-sample recovery evidence that joins evaluation states to known truth by document identity, re-expresses them with the global truth-topic alignment, and measures finite simplex RMSE plus a non-cancelling mean-absolute residual without refitting the global model.
- This is not a calibrated posterior interval or final #680 practical acceptance criterion; repeated rolling-origin aggregation and empirical coverage remain separate scientific work.
