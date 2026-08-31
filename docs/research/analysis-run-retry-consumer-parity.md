# Analysis-run retry consumer parity (doctoring)

## Scope

`LineageWeave` and the Naruon compatibility listener must be able to retry a
failed or cancelled analysis run without inventing a second DTO. HTTP method,
path, and `Host` semantics follow current HTTP semantics (Fielding, Nottingham,
& Reschke, 2022). Fail-closed refusal of non-loopback binds, table-access
hosts, review/Copilot/GitHub credential headers, and scientific-authority
promotion is repository contract authority (ADR 0011; ADR 0018; ADR 0032;
ADR 0033), not an RFC inference rule.

This slice does not serve GET status, running/terminal POST, collection GET,
cancel DTO changes, or persistence. `NaruonLiveService` stays POST-only.

## Authority

### External standards (HTTP only)

Fielding, R., Nottingham, M., & Reschke, J. (Eds.). (2022). *HTTP semantics*
(RFC 9110). IETF. https://doi.org/10.17487/RFC9110

### Internal contract evidence

- `docs/adr/0032-analysis-run-retry-http.md` — shared-listener retry
- `docs/adr/0033-analysis-run-retry-consumer-parity.md` — consumer parity
- `docs/adr/0018-consumer-scoped-analysis-run-ingress.md` — closed consumers
- `crates/tepp_api/tests/lineageweave_http_contract.rs` — LineageWeave builder
- `crates/tepp_api/tests/naruon_live_http_contract.rs` — compatibility listener
- `crates/tepp_api/tests/loopback_binary_contract.rs` — `tepp-loopback` TCP

## Verification

- LineageWeave retry exchange sets only the published consumer header;
- NaruonLiveService retries failed/cancelled Naruon runs and refuses LineageWeave;
- retry JSON has no RMSE/bias/coverage/SE-gate/scientific-acceptance keys;
- `tepp-loopback` create-then-cancel-then-retry over TCP returns `202` accepted.

## Non-claims

This slice does not implement GET status, lifecycle POST, persistence,
production TLS, or an ADR 0014 scientific claim.
