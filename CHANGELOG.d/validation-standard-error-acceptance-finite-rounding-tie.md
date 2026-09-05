### Fixed

- Preserve the represented-input inequality in `accept_within_standard_errors` when a finite residual and finite `k * SE` round to the same binary64 value: if subtraction is exact and FMA exposes a nonzero product residual, that residual now disambiguates the tie instead of defaulting to acceptance. This closes the concrete `(1 - 2^-27) * (1 + 2^-27) = 1 - 2^-54` false-accept boundary while leaving inexact-subtraction ties on the existing conservative rounded path.
