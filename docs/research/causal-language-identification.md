# Association is not causal language (doctoring)

## Scope

`causal_language` keeps association, temporal precedence, and document
links distinct from identified causal claims. Recovery is the computed
share of recovered claim kinds that match known truth.

This slice does not persist the graph, estimate ESEM/DSEM, allocate
migration `0008`, or replace `relation_graph`.

## Authority

### Normative TEPP contract

- `docs/adr/0003-relational-event-multiple-membership.md` — do not make
  every relation a transition or causal edge.
- `docs/adr/0005-posterior-esem-dsem.md` — temporal precedence, document
  linkage, event tracking, or model prediction alone do not justify
  causal language.

### Supporting literature

Holland (1986) separates statistical association from causal inference.
A earlier-later order or a document link is not an identified design.

Holland, P. W. (1986). Statistics and causal inference. *Journal of the
American Statistical Association, 81*(396), 945–960.
https://doi.org/10.1080/01621459.1986.10478354
