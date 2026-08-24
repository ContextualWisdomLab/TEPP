# Inferred status is not observed evidence (doctoring)

## Scope

`inferred_status` keeps model, reasoner, and heuristic proposals
inferred. They cannot be treated as observed documentary evidence or as
forward state transitions. Recovery is the computed share of recovered
statuses that match known truth.

This slice does not persist the graph, allocate migration `0008`, or
replace `relation_graph` or `relation_absence`.

## Authority

### Normative TEPP contract

- `docs/adr/0003-relational-event-multiple-membership.md` — observed
  relation evidence, inferred relations, and promoted transition edges
  remain distinct. Untrusted LLM output may propose mentions or
  relations but cannot promote them without deterministic schema,
  evidence, authorization, and scientific validation.

### Supporting literature

Moreau and Missier (2013) distinguish generated/derived activity from
the entity it describes. Inference is a derivation, not an observation
of the source.

Moreau, L., & Missier, P. (Eds.). (2013). *PROV-DM: The PROV data
model*. World Wide Web Consortium. https://www.w3.org/TR/prov-dm/
