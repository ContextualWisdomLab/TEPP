# Untrusted payload identity, provenance, size, and depth (doctoring)

## Scope

`payload_bound` keeps documents, serialized records, model checkpoints,
and LLM outputs untrusted until identity, provenance, size, and nesting
depth validate. Recovery is the computed share of accept/reject flags
that match known truth.

This slice does not persist payloads, allocate migration `0008`, or
replace `evidence_core` span/digest contracts.

## Authority

### Normative TEPP contract

- `AGENTS.md` — documents, external metadata, serialized payloads, model
  checkpoints, and LLM outputs are untrusted until the owning boundary
  validates identity, provenance, size/depth, authorization, and
  scientific semantics.
- `docs/adr/0008-immutable-evidence-identities-digests-and-spans.md` —
  wire records reconstruct only through domain validation.

### Supporting literature

Bray (2017) treats JSON as untrusted interchange. Moreau and Missier
(2013) require provenance for derived entities. Size and depth bounds
are the fail-closed intake gate before those reconstructions run.

Bray, T. (Ed.). (2017). *The JavaScript Object Notation (JSON) data
interchange format* (RFC 8259). RFC Editor.
https://doi.org/10.17487/RFC8259

Moreau, L., & Missier, P. (Eds.). (2013). *PROV-DM: The PROV data
model*. World Wide Web Consortium. https://www.w3.org/TR/prov-dm/
