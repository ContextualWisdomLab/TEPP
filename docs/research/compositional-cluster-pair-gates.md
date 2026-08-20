# Compositional cluster-pair gates (doctoring)

## Scope

`network_analysis` refuses to treat raw topic proportions as ordinary
Euclidean coordinates and scores recovered cluster assignments with
label-invariant pair precision and recall against known truth.

This slice does not fit a graphical lasso, run Leiden clustering, or claim
that a topic cluster is a causal construct.

## Authority

### Normative TEPP contract

- `docs/adr/0005-posterior-esem-dsem.md` and
  `docs/adr/0012-temporal-relational-shared-latent-topic-measurement.md` —
  topic proportions are compositional; downstream network analysis uses
  logistic-normal or valid log-ratio coordinates.

### Supporting literature

Aitchison (1982) motivates compositional geometry and log-ratio transformations;
raw simplex proportions must not be analyzed with ordinary Euclidean distances.
Traag, Waltman, and van Eck (2019) motivate later community-detection work;
they do **not** authorize Euclidean distances on raw topic proportions.

Aitchison, J. (1982). The statistical analysis of compositional data. *Journal
of the Royal Statistical Society: Series B (Methodological), 44*(2), 139–177.
https://doi.org/10.1111/j.2517-6161.1982.tb01195.x

Traag, V. A., Waltman, L., & van Eck, N. J. (2019). From Louvain to Leiden:
Guaranteeing well-connected communities. *Scientific Reports, 9*, Article 5233.
https://doi.org/10.1038/s41598-019-41695-z
