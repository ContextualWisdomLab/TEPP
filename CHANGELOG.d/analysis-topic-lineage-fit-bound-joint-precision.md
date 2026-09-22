### Changed

- `topic_measurement::ReferenceTopicFit` now exposes the joint generalized-Gauss-Newton precision through the owner-issued fit aggregate, so callers no longer need to pass a detached input/model/config triple for the supported fit-owned path. Caller-supplied topic UUIDs remain provisional fit-local coordinates and are not promoted to Evidence-authenticated or semantic topic identity.
