### Fixed

- `validation_core::bias_standard_error` now preserves an exactly translated two-level sample's count geometry when its standard-error factor reduces to a reciprocal power of two. For the 6/10 split of 16 observations, `SE(mean)` is exactly `|level_gap| / 8`; the represented `next_down(1.0)` gap therefore remains `0x3fbf_ffff_ffff_ffff` instead of being rounded up to `0.125` by the generic sum/square/square-root path.
- The dyadic shortcut is admitted only after exact residual translation and exact integer count verification. Other non-singleton two-level samples retain the existing translated second-moment path, and a mathematically nonzero dyadic result that falls below binary64 range still fails closed.
