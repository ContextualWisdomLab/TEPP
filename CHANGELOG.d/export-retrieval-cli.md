### Added

- `tepp_api` GAP-003A export-retrieval CLI slice (ADR 0055, active-PR, not implemented-main): naruon mints credential-free `GET /v1/exports/{export_id}` through the published `tepp-export-get` CLI onto spawned `tepp-loopback` TCP. Retrieval stays metric-free. LineageWeave is refused. `NaruonLiveService` stays POST-only. Persistence remains GAP-003B.
