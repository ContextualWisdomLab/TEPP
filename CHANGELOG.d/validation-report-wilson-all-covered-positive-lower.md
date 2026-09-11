### Fixed

- `validation_core::ValidationReport` now rejects exact all-covered Wilson evidence whose stored lower endpoint is numeric zero. The canonical producer returns `n / (n + z²)` for `p = 1`; with a non-empty sample and finite represented `z²`, that lower endpoint is strictly positive even when positive `z` squares to zero. This closes a durable-evidence state that passed the degenerate eliminated-root identity but could not be emitted by the producer.
- Added a public regression contract covering an extreme finite `z`, positive canonical lower support, `+0.0`/`-0.0` rejection, JSON egress, human projection, and serde ingress.
