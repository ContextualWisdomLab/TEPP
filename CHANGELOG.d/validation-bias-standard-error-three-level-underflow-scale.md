### Fixed

- Prevent `bias_standard_error` from accepting underflowed three-level square/cross-product intermediates as exact zero. Exact translated three-observation samples now retry the bounded proof on an exactly reversible power-of-two scale when nonzero products underflow, preserving representable nonzero dispersion and retaining the predecessor fallback when the normalized identity is not exactly provable.
