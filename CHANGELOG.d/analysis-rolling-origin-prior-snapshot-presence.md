### Scientific validation

- `corpus_split::admit_rolling_origin_partition` now rejects an evaluation identity that is already present in the prior training snapshot, even when the caller omits that UUID from the selected training-ID slice and presents a later `AvailableTime` in the evaluation snapshot. Prior-snapshot presence is treated as local proof that the row is not newly available; this closes a rolling-origin temporal-leakage path without claiming external source-provenance authentication. See #695.
