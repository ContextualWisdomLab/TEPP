### Added

- `topic_measurement` now exposes versioned fit-local document-coordinate evidence that pairs each admitted document's fitted ALR location with the CPU reference fit's existing positive diagonal-Laplace variance under the same numerator/reference topic indexes. The owner view is derived from the converged `ReferenceTopicModel` and admitted `ReferenceTopicInput`; Analysis does not recompute curvature.
- This does not yet change the unreleased `tepp.trsl_topic_lineage.v2` wire. Release projection remains blocked on #664 together with #663 fitted-topic basis binding and #658/#527 source/vocabulary provenance. The fit-local summary does not claim joint covariance, independent plausible values, interval coverage, calibration, or semantic topic authority.
