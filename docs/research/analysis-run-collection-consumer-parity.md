# Analysis-run collection consumer parity (doctoring)

## Scope

`LineageWeave` and the Naruon compatibility listener must be able to enumerate
accepted analysis runs without inventing a second DTO. HTTP method, path, and
`Host` semantics follow current HTTP semantics (Fielding, Nottingham, &
Reschke, 2022). Fail-closed refusal of non-loopback binds, table-access hosts,
review/Copilot/GitHub credential headers, and scientific-authority promotion is
repository contract authority (ADR 0011; ADR 0018; ADR 0031; ADR 0042), not an
RFC inference rule.

This slice does not serve GET status, running/terminal POST, stored-request
GET, retry, retry-lineage, or persistence.

## Authority

### External standards (HTTP only)

Fielding, R., Nottingham, M., & Reschke, J. (Eds.). (2022). *HTTP semantics*
(RFC 9110). IETF. https://doi.org/10.17487/RFC9110

### Internal contract evidence

- `docs/adr/0031-analysis-run-collection-get.md` — shared-listener collection
- `docs/adr/0042-analysis-run-collection-consumer-parity.md` — consumer parity
- `docs/adr/0018-consumer-scoped-analysis-run-ingress.md` — closed consumers
- `crates/tepp_api/tests/lineageweave_http_contract.rs` — LineageWeave builder
- `crates/tepp_api/tests/naruon_live_http_contract.rs` — compatibility listener
- `crates/tepp_api/tests/loopback_binary_contract.rs` — `tepp-loopback` TCP

## Verification

- LineageWeave collection exchange sets only the published consumer header;
- NaruonLiveService lists accepted Naruon runs and refuses LineageWeave;
- collection JSON has no RMSE/bias/coverage/SE-gate/scientific-acceptance keys;
- `tepp-loopback` create-then-list over TCP returns `200` with metric-free rows.

## Non-claims

This slice does not implement GET status, lifecycle POST, persistence,
production TLS, or an ADR 0014 scientific claim.
