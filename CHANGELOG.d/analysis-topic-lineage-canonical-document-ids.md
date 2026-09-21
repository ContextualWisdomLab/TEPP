# Topic-lineage canonical document identity

- Require every digest-bound topic-lineage predecessor/successor document UUID to use the single lowercase hyphenated spelling emitted by `Uuid::to_string()`.
- Reject alternate textual aliases for the same UUID instead of allowing semantically identical document coordinates to mint different valid artifact bytes and SHA-256 identities.
- Keep the existing topic-lineage v2 schema, estimator arithmetic, edge semantics, and UUID identity unchanged; this is a fail-closed canonical-serialization repair for #638/#654.
