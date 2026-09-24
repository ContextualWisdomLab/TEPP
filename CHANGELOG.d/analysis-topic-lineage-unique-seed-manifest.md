### Fixed

- The unreleased `tepp.trsl_topic_lineage.reference_config.v1` contract now rejects repeated deterministic initialization seeds. A duplicate seed cannot expand the initialization search space but previously minted different canonical configuration bytes and SHA-256 identity while repeating the same deterministic fit attempt. Caller order for distinct seeds is preserved because exact-objective ties remain first-wins; the parser does not sort or silently deduplicate the manifest. (#638, #659)
