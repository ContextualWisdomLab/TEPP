### Fixed

- Restricted TRSL-TM reference-estimator relation admission to directly observed forward transition edges. `RelationEvidenceStatus::Inferred` edges remain available to the relation graph for provenance/review but no longer enter the relational objective, eta gradient, GGN precision/diagonal uncertainty, or emitted sequence-lineage counts until their owner explicitly promotes the evidence status. An inferred-only transition graph now fails numerical admission instead of silently treating model/heuristic proposals as documentary relation evidence. (#673)
