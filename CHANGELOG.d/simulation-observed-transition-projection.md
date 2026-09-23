### Scientific validation

- `tepp_simulation::TruthManifest` now exposes a separate observed-transition document projection for recovery fitting. Latent `TransitionsTo` truth remains available for diagnostics, while estimator-facing recovery input follows the noisy `observed_relations` channel so relation false negatives cannot be silently restored as oracle edges.
- Observed projection keeps #698's canonical-original representative rule, ignores non-transition reference noise, and deliberately does not inspect simulation-only `is_true_positive` metadata when deciding estimator visibility. Missing original endpoints fail closed. See #699.
