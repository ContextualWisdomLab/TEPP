# Multi-window rolling-origin predictive candidate-K selection

`model_selection` now aggregates owner-issued fixed-training prevalence-mean predictive log likelihood across a contiguous sequence of admitted rolling-origin windows. Every window must expose the same unique fitted candidate-K set, and each `(window, K)` diagnostic is recomputed through the partition-bound consumer before aggregation.

Disconnected windows, duplicate fitted dimensions, candidate-set substitution, partition/input mismatch, and non-finite aggregate scores fail closed. The largest finite aggregate wins with deterministic smaller-`K` tie breaking. This is not per-window voting, the in-sample Schwarz criterion, STM document-completion likelihood, or realistic replicated recovery acceptance.