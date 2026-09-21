# Validation bias standard error avoids rounded-mean dispersion drift

- Evaluate larger-sample bias standard errors from exact anchor-relative residual deltas when those deltas are provably representable, even when the original pairwise residual subtractions were exact.
- Prevent an exactly represented residual vector such as `[1, 1-2^-52, 1]` from being turned into a different dispersion geometry by rounding its mean before centering.
- Keep the predecessor rounded-mean path when exact translated deltas cannot be established; this remains a bounded binary64 repair rather than a global correct-rounding claim.
