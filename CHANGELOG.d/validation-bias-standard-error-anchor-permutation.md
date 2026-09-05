### Fixed

- `validation_core::bias_standard_error` now searches exact translated-residual anchors in canonical represented `(high, low)` order instead of making the first observation authoritative. This preserves permutation invariance when one represented anchor admits exact deltas but another anchor would force the rounded-residual fallback.
- The public regression contract fixes a three-residual boundary whose low- and high-anchor permutations previously returned the adjacent lower binary64 standard error while the middle anchor returned the correctly rounded represented-input result. Sign-mirrored orderings are covered as well.
