### Fixed

- The unreleased `tepp.trsl_topic_lineage.v2` parser now accepts only the owner serializer's exact canonical JSON bytes. Whitespace, pretty-printing, trailing newlines, and other byte-distinct aliases fail closed instead of being silently reserialized into a different digest identity. This is a release-identity rule only; topic estimation and scientific acceptance are unchanged. (#657)
