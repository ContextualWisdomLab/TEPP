- `tepp_api` adds `lineageweave_analysis_run_running_exchange` /
  `lineageweave_analysis_run_terminal_exchange`, Naruon compatibility-listener
  lifecycle POST, and a `tepp-loopback` TCP running proof (ADR 0029). Metric-free
  running and request-bound terminal semantics are unchanged from ADR 0028.
  `NaruonLiveService` stays POST-only and Naruon-only. Not GET status, not
  cancel, not an ADR 0014 claim.
