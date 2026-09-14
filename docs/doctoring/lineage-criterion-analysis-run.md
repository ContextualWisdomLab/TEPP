# Independent TDT link-criterion analysis-run composition

**Active slice:** ADR 0063 / `lineage_criterion_v1`  
**Protected-main status:** not implemented-main  
**Decision maturity:** Proposed

`analysis_engine::fit_lineage_criterion_posteriors` on protected main remains the scientific owner for independently observed TDT link-criterion Jeffreys posteriors. The active branch only composes that fitter into a request/receipt/cutoff-bound Analysis Run profile.

The composition keeps three temporal facts separate. Predecessor/successor event-time draws remain producer evidence about event occurrence. Pair-level `AvailableTime` says when the complete observation became usable by an analysis. `KnowledgeCutoff` says what the run was allowed to know. Same-snapshot observations unavailable after the cutoff are censored before duplicate, event-time, and fitter admission; cross-snapshot evidence fails closed. Equivalent RFC 3339 spellings are compared by parsed instant rather than text.

Visible event-time draws must parse as `EventTime`. The application boundary also limits raw pairs to `MAX_EVIDENCE_UNITS` and admitted pair × draw materialization to 1,000,000 values before invoking the fitter. These are resource/admission controls, not new psychometric arithmetic.

Successful artifacts keep the domain claim `independent_tdt_criterion_not_date_from_record_order`; terminal `AnalysisResultSummary.validation_status` remains the provider state `validated`. Raw posterior values and pair identities stay with the scientific fitter.

The branch carries replay tests showing that a same-snapshot future duplicate, even with malformed future event-time content, cannot change an earlier run. It separately tests cross-snapshot and provenance-alignment refusal, malformed visible event times, resource limits, artifact tampering, and digest binding. The 256 KiB inbound artifact cap remains fail closed; maximal-valid output is proven below that bound rather than guarded by an unreachable post-validation branch.

This profile is a fold child of the surviving Analysis Run consolidation vehicle, not an independent landing claim. Current-head workflow receipts and the repository ruleset's qualifying approval must apply to the exact surviving head; predecessor checks and COMMENTED reviews do not transfer.
