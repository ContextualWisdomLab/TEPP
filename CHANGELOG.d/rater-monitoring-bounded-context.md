### Added

- Added a leakage-safe temporal rater-monitoring bounded context with distinct
  invocation-noise, gradual-drift, configuration-change, and
  measurement-invariance artifacts.
- Added `RaterMonitoringRun` and `RaterMonitoringArtifact` aggregate invariants,
  including `available_at <= knowledge_cutoff`, sealed source sets, unique
  parameter snapshots, and source-evidence containment.
- Kept monitoring artifacts immutable and explicitly prevented temporal analysis
  from rewriting source parameters, observations, scores, or product decisions.
