# Topic-lineage canonical edge order

- Canonically order digest-bound topic-lineage sequence edges by predecessor document UUID, successor document UUID, then topic index before artifact construction.
- Reject otherwise-valid v2 artifacts whose sequence-edge vector is a permutation of the same semantic edge set, preventing input-local vector order from creating multiple valid artifact byte identities and SHA-256 digests.
- Preserve estimator arithmetic, directed edge semantics, association strengths, duplicate-pair refusal, and the unreleased v2 schema shape; this is a deterministic serialization repair for #638/#655.
