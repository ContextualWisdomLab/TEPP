# Scientific simulation

- Added a deterministic known-truth projection from latent event `TransitionsTo` relations to canonical generated document UUID pairs. This lets rolling-origin recovery consume the simulator's relational signal without inventing document edges outside the simulation owner.
- Event transitions select only canonical `Original` reports; revision, translation, and template/copy derivatives remain under their separate method/provenance semantics and cannot become state-transition representatives even when their UUID sorts first. A transition endpoint with no original report fails closed.
- The projection preserves the underlying event-level truth and is not production Evidence or relation-activation authority.
