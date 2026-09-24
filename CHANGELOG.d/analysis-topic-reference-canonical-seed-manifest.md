# Analysis: canonical reference seed manifests

## Fixed

- `ReferenceTopicModelConfig` now rejects zero and duplicate deterministic seeds while preserving caller order. This moves the RNG-alias and repeated-initialization invariant into the numerical owner used by `ReferenceTopicFit`, while the topic-lineage release parser keeps its existing nonzero/unique checks as defense in depth.
