# Analysis-run status consumer parity (doctoring)

## Scope

`LineageWeave` must be able to poll an accepted analysis run without inventing
a second DTO or minting a Naruon-labelled GET. HTTP method, path, and `Host`
semantics follow current HTTP semantics (Fielding, Nottingham, & Reschke,
2022). Fail-closed refusal of non-loopback binds, table-access hosts,
review/Copilot/GitHub credential headers, and scientific-authority promotion
is repository contract authority (ADR 0011; ADR 0018; ADR 0027; ADR 0028),
not an RFC inference rule.

This slice does not serve GET on `NaruonLiveService`, running/terminal POST,
collection GET, retry, or persistence.

## Authority

### External standards (HTTP only)

Fielding, R., Nottingham, M., & Reschke, J. (Eds.). (2022). *HTTP semantics*
(RFC 9110). IETF. https://doi.org/10.17487/RFC9110

### Internal contract evidence

- `docs/adr/0027-scientific-acceptance-http-status.md` — shared-listener GET
- `docs/adr/0028-analysis-run-status-consumer-parity.md` — consumer parity
- `docs/adr/0018-consumer-scoped-analysis-run-ingress.md` — closed consumers
- `crates/tepp_api/tests/lineageweave_http_contract.rs` — LineageWeave builder
- `crates/tepp_api/tests/loopback_binary_contract.rs` — `tepp-loopback` TCP

## Verification

- LineageWeave status exchange sets only the published consumer header;
- LineageWeave GET of its own accepted run is metric-free `200`;
- cancelled JSON is out of scope; accepted GET has no RMSE/scientific-acceptance keys;
- `tepp-loopback` create-then-GET over TCP returns `200` accepted.

## Non-claims

This slice does not implement GET on `NaruonLiveService`, lifecycle POST,
persistence, production TLS, or an ADR 0014 scientific claim.
