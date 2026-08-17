# Untrusted payloads require scientific semantics (doctoring)

## Scope

`payload_semantics` keeps documents, external metadata, serialized
records, and LLM outputs out of estimator and posterior authority.
An LLM output is interpretation, not source evidence. Identity, size,
and authorization bounds are not that scientific-role gate. Recovery is
the computed share of recovered roles that match known truth.

This slice does not persist payloads, allocate migration `0008`, or
replace `payload_bound` (identity/provenance/size/depth),
`intake_authorization` (grant presence), or `checkpoint_authority`
(checkpoint versus estimator).

## Authority

### Normative TEPP contract

- `docs/adr/0008-immutable-evidence-identities-digests-and-spans.md` —
  untrusted wire and source payloads reconstruct only through validated
  domain constructors.
- `docs/adr/0014-scientific-claim-promotion-and-release-evidence.md` —
  an LLM output cannot promote a scientific claim or replace the CPU
  `f64` estimator.
- `AGENTS.md` — documents, external metadata, serialized payloads,
  checkpoints, and LLM outputs are untrusted until identity, provenance,
  size/depth, authorization, and scientific semantics validate.

### Supporting literature

The *Standards for Educational and Psychological Testing* treat score
meaning as an interpretive argument that requires validity evidence.
A narrative, metadata record, or serialized buffer is not that score.

American Educational Research Association, American Psychological
Association, & National Council on Measurement in Education. (2014).
*Standards for educational and psychological testing*. American
Educational Research Association.
