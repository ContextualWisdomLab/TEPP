# Posterior topic-context analysis-run composition

**Active slice:** ADR 0068 / `topic_context_posterior_v1`
**Protected-main status:** not implemented-main

`analysis_engine` already validates digest-bound posterior topic-context
artifacts through `TopicContextPosteriorArtifact`. This slice binds that
producer contract to a cutoff-safe analysis-run profile so an operator can
request a digest-bound terminal result.

Execution requires one authoritative snapshot manifest. It binds the source
snapshot digest and exact artifact digest, provides an availability instant for
every represented document, and independently binds the exact set of lineage,
relation, and membership `evidence_resource_id` values actually used by the
artifact. Missing, substituted, malformed, or post-cutoff document/support
availability fails closed. The manifest admits only the `trsl-tm-v1` producer
contract, and the terminal summary counts the logistic-normal coordinates
actually validated.

The support ledger is admission evidence, not a posterior rewrite mechanism.
The executor does not subtract late support from an already-fitted artifact,
infer topic importance, collapse missing draws, or invent birth/split/merge
events. Lineage events remain producer-supplied. It is not a Bayesian sampler
and not GPU execution.

ADR 0068 remains Proposed until the branch is validated and integrated into
protected main. Exact-head required workflows, resolved review threads, and the
qualifying current-head approval required by the live ruleset must be satisfied
before any implemented-main claim.
