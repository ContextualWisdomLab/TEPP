# Mention-confidence Brier score

## Scope

This note doctors the `event_core` calibration contract for fallible event mentions:

1. mention confidence is a probability on `[0, 1]`;
2. `mention_brier_score` is the mean squared error against binary truth;
3. empty or length-mismatched streams fail closed.

TDT/CHRONOS promotion remains on the event-intelligence active PR. No database migration is allocated.

## Authoritative sources

Brier, G. W. (1950). Verification of forecasts expressed in terms of probability. *Monthly Weather Review, 78*(1), 1–3. https://doi.org/10.1175/1520-0493(1950)078<0001:VOFEIT>2.0.CO;2

Gneiting, T., & Raftery, A. E. (2007). Strictly proper scoring rules, prediction, and estimation. *Journal of the American Statistical Association, 102*(477), 359–378. https://doi.org/10.1198/016214506000001437

## Application

Brier (1950) defines the mean squared error of a probability forecast. Gneiting and Raftery (2007) treat the Brier score as a strictly proper scoring rule, so a mention that is certain when true and impossible when false is uniquely optimal. TEPP therefore scores mention confidence against known binary outcomes rather than treating a high score as an event instance (Brier, 1950; Gneiting & Raftery, 2007).

## Verification

- forecasts `(0,1,0,1)` against outcomes `(false,true,false,true)` recover Brier `0`;
- constant `0.5` against mixed outcomes recovers `0.25` with computed residual RMSE;
- empty and mismatched streams return `InvalidWirePayload`.
