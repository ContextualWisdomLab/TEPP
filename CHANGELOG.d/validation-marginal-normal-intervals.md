### Validation

- Added covariance-bound marginal normal interval construction for scientific recovery. The validation owner now requires one finite location vector and a matching finite symmetric positive-semidefinite covariance before constructing `location ± z * sqrt(variance)` bounds, including zero-variance degenerate intervals. This is coordinate arithmetic only and does not claim empirical calibration; #680 still requires simulator-backed coverage over independent DGP replications without treating repeated rolling-origin document estimates as IID trials.
