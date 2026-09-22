# Rolling-origin partition admission

`corpus_split` now issues `RollingOriginPartition` only after deriving a canonical adjacent cutoff window, matching training/evaluation snapshots to those exact knowledge horizons, requiring newly available evaluation identities, and reusing governed connected-group leakage checks across the train/evaluation boundary.

The contract fails closed on invalid window selection, cutoff mismatch, duplicate/overlapping identities, unavailable rows, evaluation rows already present at the training cutoff, or governed revisions/translations/copied variants/shared episodes/canonical-equivalent records that straddle partitions. It is an availability/split-integrity contract only; Evidence authentication, event-valid semantics, Membership provenance, relation activation, and scientific recovery remain separate owner responsibilities.
