### Added

- `analysis_engine` GAP-003A engine-on-loopback slice (ADR 0032, active-PR, not implemented-main): `ScientificAcceptanceLoopbackService` serves `POST /v1/analysis-runs/{run_id}/execute` so a `scientific_acceptance_v1` run produces `tepp.scientific_acceptance.v1` without a caller-supplied artifact. Persistence remains GAP-003B.
