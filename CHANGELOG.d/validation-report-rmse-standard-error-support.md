### Fixed

- Reject `ValidationReport` artifacts whose point `rmse_standard_error` exceeds the squared-residual delta-method support bound `RMSE / 2` (with binary64 admission tolerance). The canonical boundary case remains admissible, exact-zero RMSE still requires zero RMSE standard error, and positive RMSE with zero standard error remains valid for constant squared residuals.
