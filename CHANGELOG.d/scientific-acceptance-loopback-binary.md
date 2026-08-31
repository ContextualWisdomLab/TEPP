### Added

- `analysis_engine` GAP-003A published-binary slice (ADR 0033, active-PR, not implemented-main): the `tepp-loopback` binary now binds `ScientificAcceptanceLoopbackService` so `POST /v1/analysis-runs/{run_id}/execute` is reachable on the packaged loopback listener without embedding the library. Persistence remains GAP-003B.
