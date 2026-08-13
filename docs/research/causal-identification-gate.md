# Causal identification versus association

## Scope

This note doctors the `relation_graph` gate that keeps TEPP from converting association, temporal precedence, or document links into causal language:

1. only `causes` and `intervenes_on` may be described as identified causal claims;
2. `leads_to`, `enables`, production, input/process, and all provenance kinds fail closed.

No database migration is allocated. A later identified design can widen the allowed set with an ADR.

## Authoritative sources

Pearl, J. (2009). *Causality: Models, reasoning, and inference* (2nd ed.). Cambridge University Press.

Holland, P. W. (1986). Statistics and causal inference. *Journal of the American Statistical Association, 81*(396), 945–960. https://doi.org/10.1080/01621459.1986.10478354

## Application

Holland (1986) and Pearl (2009) distinguish association and temporal order from an identified causal effect. TEPP therefore refuses to treat `references`, `leads_to`, or `enables` as `causes` without a later identification argument (Holland, 1986; Pearl, 2009).

## Verification

- `refuse_association_as_cause(Causes)` and `IntervenesOn` succeed;
- every other closed vocabulary kind returns `CausalClaimNotIdentified`.
