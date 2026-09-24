# Model-selection coverage-calibration fit design

- Added `CoverageCalibrationFitDesign::truth_k_v1()` as the versioned owner of the coverage-calibration estimator seeds, convergence controls, explicit reference-model hyperparameters, and truth-`K` single-candidate strategy.
- Added a domain-separated SHA-256 fingerprint for each concrete truth-`K` numerical fit design so the eventual 10,000-DGP experiment can bind estimator configuration separately from validation-design and simulation-scenario identities.
- This change does not execute or accept the scientific run. `analysis_engine` consumption plus shard/final-evidence binding remain required before #680 scientific acceptance.
