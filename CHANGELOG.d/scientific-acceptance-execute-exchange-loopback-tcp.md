### Added

- `analysis_engine` GAP-003A execute-exchange loopback TCP slice (ADR 0037, active-PR, not implemented-main): typed naruon and `LineageWeave` execute exchanges render onto the spawned `tepp-loopback` TCP listener so a `scientific_acceptance_v1` run produces `tepp.scientific_acceptance.v1` without hand-rolled HTTP. Public bind hosts and `localhost` fail closed. Persistence remains GAP-003B.
