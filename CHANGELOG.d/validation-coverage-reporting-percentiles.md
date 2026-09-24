# Prospective coverage reporting percentiles

- Bind the empirical coverage-reporting probabilities `0.025` and `0.975` to `CoverageCalibrationDesign::tepp_nominal_95_v1()` instead of accepting them from evidence callers.
- Include both probabilities in coverage-design fingerprint domain v4 so percentile-reporting configuration cannot change under the same prospective design fingerprint after outcomes are observed.
- Make schema-v6 coverage evidence and final shard assembly consume the validation-owner probabilities directly; this changes reporting provenance, not the practical coverage band, Monte Carlo precision criterion, DGP scenario, or numerical-failure policy.

The 10,000-DGP acceptance study remains unexecuted; this change does not promote #680 or define an acceptable numerical-failure rate. (#745)
