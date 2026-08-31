### Added

- `analysis_engine` GAP-003A execute consumer-exchange slice (ADR 0034, active-PR, not implemented-main): naruon and `LineageWeave` mint credential-free `POST /v1/analysis-runs/{run_id}/execute` through typed exchanges so a `scientific_acceptance_v1` run produces `tepp.scientific_acceptance.v1` without hand-rolled HTTP. Persistence remains GAP-003B.
