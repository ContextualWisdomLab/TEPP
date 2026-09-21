# Validation: preserve independent simulation replications in scientific recovery

`validation_core::promote_scientific_recovery` now requires recovery values grouped by independent simulation replication. Correlated temporal, clustered, cross-classified, multiple-membership, or other within-replication coordinates are reduced to one replication-level RMSE before Monte Carlo uncertainty is evaluated, so row/state duplication cannot inflate `n_sim` or create a false `ScientificallySupported` promotion.

The practical RMSE target, exact-head binding, strict conservative uncertainty boundary, and binary64 comparison guarantees remain unchanged. Validation profiles remain responsible for defining scientifically appropriate within-replication state composition and for ensuring outer repetitions are independent.
