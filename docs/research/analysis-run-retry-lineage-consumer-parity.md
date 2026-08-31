# Analysis-run retry-lineage consumer parity (doctoring)

## Scope

`LineageWeave` and the Naruon compatibility listener must be able to inspect
the metric-free direct retry children of a listed analysis run without inventing
a second DTO. HTTP method, path, and `Host` semantics follow current HTTP
semantics (Fielding, Nottingham, & Reschke, 2022). Fail-closed refusal of
non-loopback binds, table-access hosts, review/Copilot/GitHub credential
headers, and scientific-authority promotion is repository contract authority
(ADR 0011; ADR 0018; ADR 0035; ADR 0045), not an RFC inference rule.

This slice does not serve GET status, running/terminal POST, collection GET,
retry POST, retry-parent, or persistence. `NaruonLiveService` does not retry;
accepted creates return an empty `retries` array.

## Authority

### External standards (HTTP only)

Fielding, R., Nottingham, M., & Reschke, J. (Eds.). (2022). *HTTP semantics*
(RFC 9110). IETF. https://doi.org/10.17487/RFC9110

### Internal contract evidence

- `docs/adr/0035-analysis-run-retry-lineage-get.md` — shared-listener inspect
- `docs/adr/0045-analysis-run-retry-lineage-consumer-parity.md` — consumer parity
- `docs/adr/0018-consumer-scoped-analysis-run-ingress.md` — closed consumers
- `crates/tepp_api/tests/lineageweave_http_contract.rs` — LineageWeave builder
- `crates/tepp_api/tests/naruon_live_http_contract.rs` — compatibility listener
- `crates/tepp_api/tests/loopback_binary_contract.rs` — `tepp-loopback` TCP

## Verification

- LineageWeave retry-lineage exchange sets only the published consumer header;
- NaruonLiveService inspects accepted Naruon runs as an empty `retries` array and refuses LineageWeave;
- retry-lineage JSON has no RMSE/bias/coverage/SE-gate/scientific-acceptance keys;
- `tepp-loopback` create-cancel-retry-inspect over TCP returns `200` with a non-empty child list.

## Non-claims

This slice does not implement GET status, lifecycle POST, persistence,
production TLS, Naruon-listener retry, or an ADR 0014 scientific claim.
