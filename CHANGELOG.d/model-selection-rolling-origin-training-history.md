### Fixed

- Multi-window rolling-origin predictive selection now requires cumulative training history across adjacent canonical windows. A later training partition must retain every document from the preceding training and evaluation partitions; otherwise selection fails closed with a distinct `RollingOriginTrainingHistoryMismatch` instead of aggregating scores over a caller-subselected history. Additional newly historical rows remain allowed. This is numerical rolling-origin sequence integrity only and does not mint Evidence, Membership, relation, or release authority. (#691)
