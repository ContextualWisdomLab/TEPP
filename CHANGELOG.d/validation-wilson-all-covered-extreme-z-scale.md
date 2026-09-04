### Fixed

- Preserved the positive all-covered Wilson lower endpoint for durable `u64` sample counts that are not exactly representable in binary64 when a large but finite standard-normal critical value makes `z² / n > 1`. The canonical count-based producer now uses the complementary-miss form only on the small-`z² / n` side and the algebraically equivalent direct reciprocal form on the large side, avoiding both false exact `1.0` and false exact `0.0` endpoints.
