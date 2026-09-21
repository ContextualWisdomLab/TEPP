### Fixed

- The unreleased `tepp.trsl_topic_lineage.reference_config.v1` wire now rejects deterministic seed `0`. The CPU reference estimator initializes its xorshift state with `seed.max(1)`, so seeds `0` and `1` otherwise execute the same RNG stream while minting different retained configuration bytes, selected-seed labels, and SHA-256 identities. Nonzero seed order remains unchanged; no caller value is normalized or rewritten. (#638, #660)
