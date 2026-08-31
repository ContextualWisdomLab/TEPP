### Added

- `tepp_api` GAP-003A retry CLI slice (ADR 0043, active-PR, not implemented-main): naruon and `LineageWeave` mint credential-free `POST /v1/analysis-runs/{run_id}/retry` through the published `tepp-retry` CLI onto spawned `tepp-loopback` TCP. Child `202 Accepted` stays metric-free. Persistence remains GAP-003B.
