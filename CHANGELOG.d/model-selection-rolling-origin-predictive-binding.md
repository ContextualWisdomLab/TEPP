# Rolling-origin predictive binding

`model_selection` now binds the fixed-training prevalence-mean predictive diagnostic to one admitted `RollingOriginPartition`. The training fit's exact document identities must equal the partition's training set, and evaluation identities must be duplicate-free and exactly equal the partition's evaluation set before any numerical score is accepted.

The consumer calls the owner-issued `ReferenceTopicTrainingFit` predictive scorer rather than reconstructing cutoff, prevalence, or topic coordinates. Identity substitution and non-finite partition aggregation fail closed. The result remains a prevalence-mean fixed-training predictive log likelihood, not STM document-completion likelihood, the in-sample Schwarz criterion, or a replacement for realistic rolling-origin recovery evidence.
