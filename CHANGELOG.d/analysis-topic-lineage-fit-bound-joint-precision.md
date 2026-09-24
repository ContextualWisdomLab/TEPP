### Changed

- `topic_measurement::ReferenceTopicFit` now exposes the joint generalized-Gauss-Newton precision through the owner-issued fit aggregate. The detached `ReferenceTopicInput` builder is crate-private, so downstream consumers cannot rebind dimension-compatible input/model/config state while manufacturing owner-looking precision. Existing owner-internal malformed-geometry coverage remains on the arithmetic primitive. Caller-supplied topic UUIDs remain provisional fit-local coordinates and are not promoted to Evidence-authenticated or semantic topic identity.
