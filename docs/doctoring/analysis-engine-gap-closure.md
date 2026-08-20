# Analysis Engine v1 — Buyer Gap Closure

**Review date:** 2026-08-21
**Active slice:** PR #157 terminal-result contract → stacked analysis execution
engine for issue #166

The organization-wide buyer-gap register is maintained by a separate landing
vehicle. This document records only the analysis-engine slice so it can land
without competing with that register or requiring another PR to be present.

## Buyer-visible gap

An accepted analysis run previously had a durable receipt and a terminal-result
DTO, but no executable path that applied a historical availability cutoff and
returned a verifiable artifact. A buyer could submit work but could not yet
demonstrate that the result was complete, cutoff-safe, multiple-membership aware,
and unchanged after transport.

## Bounded closure

The stacked `analysis_engine` crate closes the first vertical slice with a
standalone Rust API. It accepts a bounded identity-free evidence snapshot,
rejects snapshot mismatches and duplicate opaque IDs, excludes future-available
evidence, preserves membership counts, and emits a digest-bound terminal result
or a redacted no-eligible-evidence failure.

This is readiness evidence, not a psychometric estimate. Latent-variable,
multilingual, GPU, and HTTP service gaps remain separately governed by their
own ADRs and must not be implied by this slice.

## Next leverage-ranked gaps

1. Add a streaming snapshot adapter with the same digest and cutoff semantics.
2. Bind the engine to a versioned standalone HTTP port without cross-service
   table access.
3. Add known-truth estimator execution with RMSE, bias, interval coverage,
   multilevel/multiple-membership recovery, and CPU/GPU parity.
4. Add buyer-facing visual analytics only after the interaction contract is
   stable; then create a Figma file and Storybook inventory and record its real
   File ID in a new UI ADR.

## Evidence boundary

The current implementation is active-PR evidence only. Exact-head checks,
independent review, protected merge, release evidence, and deployment controls
are required before a capability is promoted to implemented-main.
