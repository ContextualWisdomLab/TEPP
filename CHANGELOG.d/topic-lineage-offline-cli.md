### Added

- Add the standalone offline `tepp-topic-lineage` binary to `analysis_engine`.
  It reads one bounded `tepp.topic_lineage_cli_input.v1` JSON file and prints
  one `tepp.topic_lineage_cli_output.v1` JSON line that holds the digest-bound
  `tepp.trsl_topic_lineage.v1` artifact, its request-bound terminal result,
  the SHA-256 digest of the exact input bytes, and exclusion counts.
- Documents, memberships, and relations whose `available_time` is later than
  the request `knowledge_cutoff` are excluded and counted before estimation.
  Memberships and relations that touch an excluded document are also
  excluded. Relation kinds `causes` and `intervenes_on` are refused because
  the association estimator does not identify causation.
- Estimation is the unchanged ADR 0012 CPU `f64` reference path
  (`execute_topic_lineage_run`) with a caller-declared topic count. The binary
  does not select `K`, use a GPU, persist a run, or access the network.
- The accepted receipt is minted locally from the input `run_id`; it is not a
  durable, service-registered analysis run.

### Fixed

- Reject nonregular input paths before reading with redacted `io`/exit `5`.
  Unix nonblocking opens and descriptor type revalidation also prevent a
  pathname-to-FIFO replacement from waiting for a writer. Regular-file
  symlinks remain supported; bounded regular-file reads are unchanged.
- Correct the reference-model and lineage-edge documentation: proportions
  are MAP-fitted point estimates, and association strength is their minimum
  for a shared dominant topic, not a posterior mean or uncertainty probability.
  Numeric estimates, field names, schema versions, and posterior-aware targets
  are unchanged.
