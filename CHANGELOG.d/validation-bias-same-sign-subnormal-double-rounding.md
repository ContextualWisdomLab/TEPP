# Validation bias same-sign subnormal single rounding

- `validation_core::mean_bias` now rounds same-sign all-subnormal represented residuals once at the final binary64 subnormal grid when the scientific divisor is at least the surviving term count.
- The public contract fixes the three-residual case `(3 * 2^52 - 64) / 3`, which must round to subnormal units `2^52 - 21` rather than the adjacent `2^52 - 22` produced by normalize-then-rescale double rounding.
- Halfway cases use IEEE 754 round-to-nearest, ties-to-even on exact represented subnormal units; a companion fixture prevents the discarded direct-float repair from reintroducing a one-ULP halfway error.
