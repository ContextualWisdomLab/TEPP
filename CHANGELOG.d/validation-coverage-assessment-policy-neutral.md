# Policy-neutral coverage assessment API

- Removed the public `CoverageCalibrationAssessment::supports_calibration_claim()` aggregate boolean while numerical-failure acceptability remains prospectively undefined.
- Kept practical-band and Monte Carlo precision decisions as separate public components alongside the unconditional attempted/success/failure denominator and failure rate.
- Added an owner-surface contract so the aggregate claim method cannot silently reappear without an explicit policy decision.

This change does not alter the coverage estimand, validation-design fingerprint, interval construction, simulation scenario or seeds, evidence schema v6, or any numerical acceptance arithmetic. It does not execute or promote the 10,000-DGP study.
