# JSON-LD node semantic identifier

## Decision

The `tepp_api::JsonLdExport` Rust source contract names the primary node identity `node_id` rather than the underspecified bare field `id`. The existing version-1 JSON wire remains byte-key compatible at the field-name level: Serde maps the Rust `node_id` field to the serialized/deserialized key `"id"`.

This is a source-contract naming repair, not a v1 wire-format migration. Organization search found no ContextualWisdomLab source consumer accessing `JsonLdExport.id`; the published consumer boundary is the versioned serialized artifact. The crate is also marked `publish = false`, so the repository remains the source owner for this API surface.

## DDD boundary

**Bounded context:** Analytical Artifact Export.

**Aggregate:** `JsonLdExport` is a versioned export envelope owned by `tepp_api`.

**Value object:** `node_id` identifies the primary node represented by the envelope. `type_name` describes the node type and `artifact_digest_sha256` binds the envelope to the artifact payload.

**Invariant:** internal Rust names must identify the domain role without relying on a generic `id`; the existing v1 serialized key remains stable so naruon and other modular consumers do not require a coordinated wire migration.

## JSON-LD compatibility

JSON-LD 1.1 defines node identifiers through the `@id` keyword and permits terms to be defined through a context. TEPP v1 already has a deployed envelope field named `id`; this change does not claim that bare `id` is itself the normative JSON-LD keyword and does not silently rewrite the wire to `@id`. The existing contract is preserved until a separately versioned wire decision is made.

The new regression test proves both sides of the anti-corruption seam: Rust code reads `node_id`, while serialized v1 JSON contains `"id"` and not `"node_id"`; deserializing the same v1 payload restores `node_id`.

## Security, privacy, and scientific scope

No estimator, psychometric arithmetic, temporal semantics, evidence cutoff, PII authority, database schema, network endpoint, or export authorization rule changes. The node identifier remains an opaque string and passes the same non-empty validation. `deny_unknown_fields` remains enabled.

## Verification

The RED commit introduced an external crate test that accessed `JsonLdExport.node_id` while production still exposed only `id`, producing a compile-time contract failure. The production repair then renamed the Rust field and constructor parameter and added `#[serde(rename = "id")]` to preserve the wire contract. Fresh exact-head repository and central checks remain the merge authority.

## References

Feitelson, D. G., Mizrahi, A., Noy, N., Ben Shabat, A., Eliyahu, O., & Sheffer, R. (2022). How developers choose names. *IEEE Transactions on Software Engineering, 48*(1), 37–52. https://doi.org/10.1109/TSE.2020.2976920

Schankin, A., Berger, A., Holt, D. V., Hofmeister, J. C., Riedel, T., & Beigl, M. (2018). Descriptive compound identifier names improve source code comprehension. In *Proceedings of the 26th Conference on Program Comprehension* (pp. 31–40). Association for Computing Machinery. https://doi.org/10.1145/3196321.3196332

World Wide Web Consortium. (2020, July 16). *JSON-LD 1.1: A JSON-based serialization for linked data* (W3C Recommendation). https://www.w3.org/TR/json-ld11/
