### Fixed

- `validation_core::bias_standard_error` now preserves the exact three-observation identity `SE(mean) = |level_gap| / 3` when exactly two represented residual levels are equal after an exact translated-residual admission.
- This avoids an unnecessary square → second-moment → square-root projection that moves `[0, next_down(1), next_down(1)]` one ULP below the correctly rounded represented-input standard error.
- The repair is deliberately bounded to the proven three-observation two-level identity; the general translated second-moment path and its fail-closed fallbacks remain unchanged.
