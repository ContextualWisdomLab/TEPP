### Fixed

- Preserve exactly translated two-level `bias_standard_error` rational-square geometry when the represented result is subnormal. Exact rational scales such as `3/44` now round once in minimum-subnormal units instead of normalizing a quotient and then crossing a second binary64 rounding boundary during power-of-two restoration; mathematically nonzero values below binary64 range continue to fail closed.
