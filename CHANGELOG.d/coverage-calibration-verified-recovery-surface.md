### Fixed

- Made digest-verified coverage-calibration shard rehydration the only public resume path. The strict JSON-only parser is now private to `analysis_engine`; persisted callers must provide the owner-issued SHA-256 alongside shard JSON, while schema/provenance parsing, final study tiling, and scientific validation remain with their existing owners. The digest is an application-level transfer-integrity binding, not external storage, runner, or build authentication.
