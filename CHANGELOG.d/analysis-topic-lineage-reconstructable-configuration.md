# Analysis engine

- Bump the unreleased TRSL topic-lineage output profile and artifact together to `trsl_topic_lineage_v2` / `tepp.trsl_topic_lineage.v2` and bind each successful artifact to the exact canonical `tepp.trsl_topic_lineage.reference_config.v1` estimator configuration, its SHA-256 digest, the `trsl_tm_cpu_f64_v1` model contract, and the `cpu_f64_reference` backend.
- Build the reference estimator from the canonical configuration bytes that are retained in the artifact. Unknown configuration schemas, non-canonical JSON, detached configuration digests, topic-count mismatches, selected seeds outside the declared ordered seed set, and iteration counts above the declared budget fail closed.
- Keep the previous `trsl_topic_lineage_v1` / `tepp.trsl_topic_lineage.v1` pair unreleased; consumers must not treat the Draft PR head or historical profile/schema strings as immutable release authority.
