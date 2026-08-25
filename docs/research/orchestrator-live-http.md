# Orchestrator live HTTP listener (doctoring)

## Scope

`orchestrator_live` serves `POST /v1/interpretation-runs` on a loopback-only
HTTP/1.1 listener. HTTP method, path, and header semantics follow HTTP/1.1
(Fielding & Reschke, 2014). Fail-closed refusal of non-loopback binds,
table-access hosts, review/Copilot/GitHub credential headers,
`COPILOT_GITHUB_TOKEN`, and scientific-authority promotion is repository
contract authority (ADR 0010; ADR 0011), not an RFC inference rule.

Accepted responses always carry `claim_status = hypothetical` and
`scientific_authority = false`. The listener does not terminate TLS, call a
model provider, or replace deterministic/statistical gates.

## Authority

### External standards (HTTP only)

Fielding, R. T., & Reschke, J. (Eds.). (2014). *Hypertext Transfer Protocol
(HTTP/1.1): Semantics and content* (RFC 7231). IETF.
https://doi.org/10.17487/RFC7231

### Orchestration research (mode vocabulary only)

Xu, J., Sun, Q., Schwendeman, P., Nielsen, S., Cetin, E., & Tang, Y. (2026).
TRINITY: An evolved LLM coordinator. In *International Conference on Learning
Representations (ICLR 2026)*. https://arxiv.org/abs/2512.04695

Nielsen, S., Cetin, E., Schwendeman, P., Sun, Q., Xu, J., & Tang, Y. (2026).
Learning to orchestrate agents in natural language with the Conductor. In
*International Conference on Learning Representations (ICLR 2026)*.
https://arxiv.org/abs/2512.04388

Tang, Y., Cetin, E., Xu, J., Sun, Q., Nielsen, S., Richard, V., Goda, H.,
Tymchenko, I., Nguyen, N., Lee, H., Ashiga, M., Kotyan, S., Kuroki, S., &
Clanuwat, T. (2026). *Sakana Fugu technical report* [Preprint]. arXiv.
https://arxiv.org/abs/2606.21228

These sources motivate recording `direct`, `verify`, `committee`, `conductor`,
and `abstain` as first-class modes. They do not authorize treating listener
output as measurement truth.

### Internal contract evidence

- `docs/adr/0010-adaptive-llm-orchestration.md` — mode vocabulary and
  scientific-authority separation
- `docs/adr/0011-standalone-modular-msa-boundary.md` — no cross-service table
  access; standalone loopback operation
- `docs/connectors/contextual-orchestrator-interpretation-port.md` — port
  boundary and credential separation
- `crates/orchestrator_live/tests/live_http_contract.rs` — loopback live
  HTTP/1.1 proofs

## Verification

- loopback bind accepts `127.0.0.1:0` and refuses non-loopback IPs;
- valid `POST /v1/interpretation-runs` returns 202 with hypothetical status;
- idempotent retries replay the same accepted body;
- `scientific_authority: true` and unknown `source_text` fail closed;
- review, Copilot, GitHub, and `NVIDIA_NIM_API_KEY` headers are
  `AuthorizationDenied`;
- `postgres` / `jdbc` / `/sql` / `/tables/` hosts fail closed.

## Non-claims

This slice does not implement adaptive routing, comparable-budget ablation,
provider execution, production TLS, or a scientific claim-promotion package.
