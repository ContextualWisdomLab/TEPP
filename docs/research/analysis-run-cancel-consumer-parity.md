# Analysis-run cancel consumer parity (doctoring)

## Scope

`LineageWeave` and the Naruon compatibility listener must be able to cancel an
accepted or running analysis run without inventing a second DTO. HTTP method,
path, and `Host` semantics follow current HTTP semantics (Fielding, Nottingham,
& Reschke, 2022). Fail-closed refusal of non-loopback binds, table-access
hosts, review/Copilot/GitHub credential headers, and scientific-authority
promotion is repository contract authority (ADR 0011; ADR 0018; ADR 0029;
ADR 0030), not an RFC inference rule.

This slice does not serve GET status, running/terminal POST, collection GET,
retry, or persistence.

## Authority

### External standards (HTTP only)

Fielding, R., Nottingham, M., & Reschke, J. (Eds.). (2022). *HTTP semantics*
(RFC 9110). IETF. https://doi.org/10.17487/RFC9110

### Internal contract evidence

- `docs/adr/0029-analysis-run-cancel-http.md` — shared-listener cancel
- `docs/adr/0030-analysis-run-cancel-consumer-parity.md` — consumer parity
- `docs/adr/0018-consumer-scoped-analysis-run-ingress.md` — closed consumers
- `crates/tepp_api/tests/lineageweave_http_contract.rs` — LineageWeave builder
- `crates/tepp_api/tests/naruon_live_http_contract.rs` — compatibility listener
- `crates/tepp_api/tests/loopback_binary_contract.rs` — `tepp-loopback` TCP

## Verification

- LineageWeave cancel exchange sets only the published consumer header;
- NaruonLiveService cancels accepted Naruon runs and refuses LineageWeave;
- cancelled JSON has no RMSE/bias/coverage/SE-gate/scientific-acceptance keys;
- `tepp-loopback` create-then-cancel over TCP returns `200` cancelled.

## Non-claims

This slice does not implement GET status, lifecycle POST, persistence,
production TLS, or an ADR 0014 scientific claim.
