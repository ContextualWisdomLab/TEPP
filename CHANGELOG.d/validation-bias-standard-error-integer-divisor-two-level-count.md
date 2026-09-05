### Fixed

- `validation_core::bias_standard_error` now preserves exactly translated two-level sample geometries whose count factor reduces to the reciprocal square of an exactly representable integer, not only a reciprocal power of two. For a 3/6 split of nine observations, `SE(mean)^2 = gap^2 / 36`, so `SE(mean)` is exactly `|gap| / 6`; the represented gap `0x1.ffffffffffffdp-1` now rounds once to `0x3fc5_5555_5555_5553` instead of being moved one ULP upward by the generic sum/square/square-root reconstruction.
- The shortcut remains gated by exact residual translation and checked integer count algebra. Non-square count factors retain the existing translated second-moment path, and a mathematically nonzero reciprocal-integer result that falls below binary64 range still fails closed.
