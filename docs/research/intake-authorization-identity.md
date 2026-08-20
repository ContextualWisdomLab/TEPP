# Untrusted intake requires a grant (doctoring)

## Scope

`intake_authorization` keeps documents, serialized records, checkpoints,
and LLM outputs out of the analysis boundary until a purpose-bound grant
is present. Size, identity, and provenance bounds are not that grant.
Recovery is the computed share of grant-presence flags that match known
truth.

This slice does not persist grants, allocate migration `0008`, or replace
`purpose_authorization` (one grant, one purpose) or `payload_bound`
(identity/provenance/size/depth).

## Authority

### Normative TEPP contract

- `docs/adr/0009-purpose-bound-pii-governance.md` — processing is
  purpose-bound; blanket masking is not authorization.
- `AGENTS.md` — documents, serialized payloads, checkpoints, and LLM
  outputs are untrusted until identity, provenance, size/depth,
  authorization, and scientific semantics validate.

### Supporting literature

Voigt and Von dem Bussche (2017) treat purpose limitation as a
processing precondition, not a post-hoc filter. A size bound is not a
purpose.

Voigt, P., & Von dem Bussche, A. (2017). *The EU General Data Protection
Regulation (GDPR): A practical guide*. Springer.
https://doi.org/10.1007/978-3-319-57959-7
