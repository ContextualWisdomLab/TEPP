### Added

- `analysis_engine` GAP-003A execute CLI slice (ADR 0041, active-PR, not implemented-main): published `tepp-execute` POSTs typed naruon and `LineageWeave` execute exchanges onto spawned `tepp-loopback` TCP so operators obtain `tepp.scientific_acceptance.v1` without writing HTTP. Public bind hosts and `localhost` fail closed. Persistence remains GAP-003B.
