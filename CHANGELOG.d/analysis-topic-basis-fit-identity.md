### Added

- `topic_measurement` now derives a versioned, fit-local `FittedTopicBasisIdentity` from the exact fitted topic-term probability rows. Per-topic SHA-256 identities follow row content under label permutation, while the enclosing basis SHA-256 remains order-sensitive so ALR and lineage coordinates retain their fitted local basis.
- The identity hashes exact finite-positive binary64 probability bits and fails closed on malformed dimensions or duplicate topic rows. It is numerical-owner evidence only: it does not authenticate an Evidence snapshot or vocabulary coordinate and is not yet projected into the unreleased `tepp.trsl_topic_lineage.v2` artifact. Source/vocabulary provenance remains the #658/#527 prerequisite before #663 can become released topic-coordinate authority.
