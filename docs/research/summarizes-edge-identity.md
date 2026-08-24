# A summary is not a transition or the source document (doctoring)

## Scope

`summarizes_edge` keeps summaries out of the forward state-transition
vocabulary and out of the source-document identity. Recovery is the
computed share of recovered kinds that match known truth.

This slice does not persist the graph, allocate migration `0008`, or
replace `relation_graph` or `citation_edge`.

## Authority

### Normative TEPP contract

- `docs/adr/0003-relational-event-multiple-membership.md` — typed
  relations distinguish transition from provenance. Translation,
  revision, and copy variants keep distinct identities for
  relation-aware splits.

### Supporting literature

Moreau and Missier (2013) treat a derived entity as distinct from the
entity it summarizes. A summary is a derivation, not a state transition
and not a reuse of the source identity.

Moreau, L., & Missier, P. (Eds.). (2013). *PROV-DM: The PROV data
model*. World Wide Web Consortium. https://www.w3.org/TR/prov-dm/
