# Shared reference topic dense-allocation admission

- Enforce checked 16 MiB dense design admission before feature expansion and
  design allocation in `ReferenceTopicInput`, including actual sparse
  covariate high indices and active membership contexts.
- Enforce the existing conservative 128 MiB simultaneous numeric payload
  envelope at shared fit entry before any seed, returning typed
  `ResourceLimitExceeded` on overbudget or overflow. Reuse the shared planners
  from CLI admission without changing its stricter operator count/work limits.
- Exercise dimension-only boundaries/overflow, direct sparse inputs/fits,
  executor refusal/no artifact, and actual candidate-K all-refused/mixed
  selection, retaining small positives and observed-only graph semantics.
- Supersede CLI-only resource authority in ADR 0012 and API/source-trace docs.
  Payload caps are not RSS/OOM guarantees; optional joint precision remains
  independently capped. No estimand, latent meaning, PRD target, or wire version
  changes; this remains active-PR offline CPU evidence, not release acceptance.
