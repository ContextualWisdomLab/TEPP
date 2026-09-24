# Coverage calibration execution consumes the declared window cardinality

- Remove the analysis-local rolling-origin window-count literal from the prospective coverage-calibration executor.
- Read the required window count from `CoverageCalibrationDesign::tepp_nominal_95_v1().declared_rolling_origin_window_count()` for cutoff geometry, allocation, final successful-window validation, and the regression contract.
- Keep rolling-origin partition construction in `analysis_engine`; this changes dependency direction only and does not alter simulation seeds/configuration, equal-window aggregation, interval arithmetic, the 10,000-DGP denominator, or numerical-failure policy.
- Currentize the study-assembly rustdoc to schema-v9 evidence arithmetic.

Refs #751 #750 #639 #680.
