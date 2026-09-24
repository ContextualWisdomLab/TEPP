### Fixed

- Topic-lineage reference configuration now has one digest-bound zero identity for the zero-capable `relation_strength` and `ridge` ablations: canonical `+0.0` remains valid, while IEEE-754 sign-negative zero fails closed before estimator configuration reconstruction. This changes only the released configuration serialization boundary; numerical estimator arithmetic, tolerances, and positive hyperparameter domains are unchanged. (#638, #656)
