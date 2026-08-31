# Simulation method-effect analysis-run composition

**Active slice:** ADR 0057 / `method_effects_v1`
**Protected-main status:** not implemented-main

`tepp_simulation` already labels generated documents with
`DocumentMethodEffect` and admits them through
`refuse_unavailable_document`. This slice binds that census to a
cutoff-safe analysis-run profile so operators can request a digest-bound
identity artifact.

The artifact content digest covers exactly the documents admitted at the
knowledge cutoff. A conservative 1,000,000-row execution budget is checked
before allocating the generated corpus.

The artifact inference status is
`simulation_method_effect_labels_not_estimator_model`. This is not an
estimator-side method model, not GPU, not MCMC, and not topic
birth/split/merge.
