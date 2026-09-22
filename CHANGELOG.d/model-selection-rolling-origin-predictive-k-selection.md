# Rolling-origin predictive candidate-K selection

`model_selection` now selects candidate `K` from owner-issued fixed-training prevalence-mean predictive scores on one admitted `RollingOriginPartition`. Candidate `K` is derived from each fitted topic dimension, duplicate dimensions fail closed, and every candidate is scored through the partition-bound predictive consumer introduced by #686.

The highest finite predictive score wins with deterministic smaller-`K` tie breaking, independent of candidate input order. The existing in-sample Schwarz selector remains separate. This is not STM document-completion likelihood and does not by itself establish multi-window or realistic true-`K` recovery acceptance.