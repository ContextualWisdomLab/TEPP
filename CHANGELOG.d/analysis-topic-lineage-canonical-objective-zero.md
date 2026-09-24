### Fixed

- Topic-lineage artifacts now have one digest-bound zero identity for the final penalized `objective`: canonical `+0.0` remains valid, while IEEE-754 sign-negative zero fails closed in owner serialization and parser admission. This is a release-identity rule only; reference-estimator optimization, convergence, tie-breaking, and scientific acceptance are unchanged. (#638, #661)
