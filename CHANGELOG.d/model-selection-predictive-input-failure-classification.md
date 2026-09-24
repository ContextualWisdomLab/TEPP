### Fixed

- Kept structurally invalid rolling-origin evaluation payloads out of scientific recovery failure-rate denominators. `model_selection` now preserves `topic_measurement` input/geometry incompatibility as `PredictiveEvaluationInputInvalid`, while non-finite predictive arithmetic and unrepresentable fitted ALR means remain numerical `InvalidDiagnostic` failures eligible for denominator-preserving recovery accounting (#711).
