# CHRONOS occurrence-prediction calibration

## Scope

This note doctors the `event_core` contract for CHRONOS-style occurrence forecasts:

1. a forecast is hypothesized future or schema-completion evidence, not a promoted event instance;
2. `chronos_prediction_brier_score` is the mean squared error of occurrence probabilities against later-observed binary truth;
3. empty or length-mismatched streams fail closed.

No database migration is allocated. Mention-confidence scoring, TDT detection, schema-slot extraction, and temporal-consistency reasoning remain separate slices.

## Authoritative sources

Anagnostopoulos, E., Batsakis, S., & Petrakis, E. G. M. (2013). CHRONOS: A reasoning engine for qualitative temporal information in OWL. *Procedia Computer Science, 22*, 70–77. https://doi.org/10.1016/j.procs.2013.09.082

Brier, G. W. (1950). Verification of forecasts expressed in terms of probability. *Monthly Weather Review, 78*(1), 1–3. https://doi.org/10.1175/1520-0493(1950)078<0001:VOFEIT>2.0.CO;2

Gneiting, T., & Raftery, A. E. (2007). Strictly proper scoring rules, prediction, and estimation. *Journal of the American Statistical Association, 102*(477), 359–378. https://doi.org/10.1198/016214506000001437

## Application

CHRONOS-style reasoning may propose next-event or schema-completion candidates (Anagnostopoulos et al., 2013). ADR 0016 keeps those candidates hypothetical until later evidence supports them. Brier (1950) defines the mean squared error of a probability forecast, and Gneiting and Raftery (2007) treat that score as strictly proper, so a forecast that is certain when the event later occurs and impossible when it does not is uniquely optimal. TEPP therefore scores CHRONOS occurrence probabilities against later-observed truth and refuses to cast a prediction as an event instance (Brier, 1950; Gneiting & Raftery, 2007).

## Verification

- forecasts `(1,0,1)` against outcomes `(occurred, did_not_occur, occurred)` recover Brier `0`;
- calibrated probabilities beat an always-occur predictor on mixed later truth;
- empty and mismatched streams return `InvalidWirePayload`;
- `refuse_prediction_as_instance` always returns `PredictionIsNotEventInstance`.
