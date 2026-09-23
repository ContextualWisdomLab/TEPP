## Scientific recovery failure admission

- Add `selected_k_recovery_summary_from_results(...)` as the canonical repeated-recovery summary path. It applies the owner failure classifier before denominator-preserving selected-K statistics, so numerical fit/selection failures remain attempted failures while structural split, horizon, configuration, training-state, predictive-input, or authority errors abort the experiment instead of being converted to caller-authored `None` values.
- Keep `selected_k_recovery_summary(&[Option<u32>], ...)` as a lower-level compatibility primitive for already-admitted outcomes; #680-style scientific acceptance should use the typed result path.
