# Known prevalence EventTime basis

## Purpose

The known-topic simulator generates prevalence trajectories with `normalized_event_position(ordinal, event_count)`, while the reference estimator freezes a centered/scaled EventTime feature from each admitted training window. Those coefficient vectors are not directly comparable unless both time features are expressed against one physical origin.

`TruthManifest::prevalence_time_basis()` is the simulation-owned reconstruction of the generating feature. It uses only the manifest's digest-bound latent events and returns `(origin_event_time, center_seconds_from_origin, scale_seconds)` for

`x = (t_seconds_from_origin - center_seconds_from_origin) / scale_seconds`.

Generated events must have at least two rows, contiguous zero-based ordinals, and strictly increasing equally spaced EventTime values. Otherwise the projection fails closed with `ManifestInvariantViolation`; it does not guess an affine physical-time interpretation for ordinal truth that the event clocks do not support. `TruthManifest::prevalence_time_coordinate_at(...)` evaluates that owner basis and is contract-tested to reproduce the DGP's `-1..1` normalized event positions exactly at generated event times.

## Recovery composition

For a rolling-origin training window, the frozen `topic_measurement::PrevalenceDesignBasis` remains the estimator owner of training-feature origin, location, scale, and column ordering. The #680 acceptance harness must express its training center against the same physical origin returned by `TruthManifest::prevalence_time_basis()`. It may then use `validation_core::reexpress_linear_prevalence_time_basis(...)` to move simulator intercepts/slopes or fitted coefficients into one chosen feature basis before coefficient bias/RMSE.

This EventTime feature transformation is independent of topic-label/ALR alignment. Topic-content alignment and ALR reference changes remain owned by `align_topic_probability_rows(...)` and `realign_additive_log_ratio(...)`; covariance propagation remains separate. None of these validation projections authenticate production EventTime, availability, source provenance, or release identity.

## Evidence boundary

The owner contract is `crates/tepp_simulation/tests/prevalence_time_basis_truth_contract.rs`. It covers exact generated-coordinate reconstruction and refuses one-event, non-contiguous-ordinal, irregular-spacing, and non-increasing-clock manifests. The contract is prerequisite plumbing only. Repeated leakage-safe rolling-origin recovery, failure denominators, Monte Carlo uncertainty, independent review, protected-main merge, and immutable release remain required before a scientific recovery claim can be promoted.
