### Added

- `tepp_api` GAP-003A retry-lineage CLI slice (ADR 0048, active-PR, not implemented-main): naruon and `LineageWeave` mint credential-free `GET /v1/analysis-runs/{run_id}/retries` through the published `tepp-retry-lineage` CLI onto spawned `tepp-loopback` TCP. Inspect stays metric-free. Persistence remains GAP-003B.
