# Analysis-run stored-request consumer parity (doctoring)

## Scope

`LineageWeave` and the Naruon compatibility listener must be able to inspect
stored analysis-run create fields without inventing a second DTO. HTTP method,
path, and `Host` semantics follow current HTTP semantics (Fielding, Nottingham,
& Reschke, 2022). Fail-closed refusal of non-loopback binds, table-access
hosts, review/Copilot/GitHub credential headers, and scientific-authority
promotion is repository contract authority (ADR 0011; ADR 0018; ADR 0034;
ADR 0040), not an RFC inference rule.

This slice does not serve GET status, running/terminal POST, collection GET,
retry, retry-lineage, or persistence.

## Authority

### External standards (HTTP only)

Fielding, R., Nottingham, M., & Reschke, J. (Eds.). (2022). *HTTP semantics*
(RFC 9110). IETF. https://doi.org/10.17487/RFC9110

### Internal contract evidence

- `docs/adr/0034-analysis-run-stored-request-get.md` — shared-listener inspect
- `docs/adr/0040-analysis-run-stored-request-consumer-parity.md` — consumer parity
- `docs/adr/0018-consumer-scoped-analysis-run-ingress.md` — closed consumers
- `crates/tepp_api/tests/lineageweave_http_contract.rs` — LineageWeave builder
- `crates/tepp_api/tests/naruon_live_http_contract.rs` — compatibility listener
- `crates/tepp_api/tests/loopback_binary_contract.rs` — `tepp-loopback` TCP

## Verification

- LineageWeave stored-request exchange sets only the published consumer header;
- NaruonLiveService inspects accepted Naruon runs and refuses LineageWeave;
- stored-request JSON has no RMSE/bias/coverage/SE-gate/scientific-acceptance keys;
- `tepp-loopback` create-then-inspect over TCP returns `200` with snapshot and profile.

## Non-claims

This slice does not implement GET status, lifecycle POST, persistence,
production TLS, or an ADR 0014 scientific claim.
