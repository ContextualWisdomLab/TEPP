### Fixed

- Preserve the exact four-observation pair-distance standard-error proof when the GCD-reduced `u128` numerator exceeds `2^53`. The binary64 numerator conversion is now only a candidate seed; exact dyadic-square and midpoint comparisons remain authoritative for the returned rounding, so representable exact ratios do not fall back solely because the seed integer is not exactly representable as binary64.
