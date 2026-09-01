# Posterior topic-context analysis-run composition

**Active slice:** ADR 0068 / `topic_context_posterior_v1`
**Protected-main status:** not implemented-main

`analysis_engine` already validates digest-bound posterior topic-context
artifacts through `TopicContextPosteriorArtifact`. This slice binds that
producer contract to a cutoff-safe analysis-run profile so an operator can
request a digest-bound terminal result.

The executor does not infer topic importance, does not collapse missing
draws, and does not invent birth/split/merge events. Lineage events remain
producer-supplied. It is not a Bayesian sampler and not GPU execution.

Exact-head Checks and two independent approvals are required before any
implemented-main claim.
