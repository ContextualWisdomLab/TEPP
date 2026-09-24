# Bind corpus snapshots to one knowledge cutoff

`CorpusSnapshot` now binds itself to the `KnowledgeCutoff` used by its first successful document admission and rejects later insertions that supply a different cutoff. The bound cutoff is exposed read-only so downstream scientific admission can prove the historical availability horizon represented by the snapshot.

This does not equate `EventTime` with `KnowledgeCutoff`: future event-valid times may be legitimate when the supporting evidence was available by the cutoff. Source/event-time authenticity remains an Evidence/Temporal Semantics provenance responsibility tracked by #671.
