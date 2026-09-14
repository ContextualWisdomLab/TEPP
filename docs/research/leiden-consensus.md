# Leiden consensus partitions (GAP-009 remainder)

## Scope

This slice replaces the greedy union-find stand-in inside
`network_analysis::consensus` with Traag, Waltman, and van Eck (2019)
Leiden community detection on admitted positive topic–topic edges:

1. fast local moving of nodes under Newman–Girvan modularity with γ = 1;
2. a refinement step that starts from singletons inside each community
   and forbids internally disconnected communities;
3. aggregation of well-connected communities before the next local-move
   pass;
4. co-assignment consensus across independently perturbed replicates
   (Monti, Tamayo, Mesirov, & Golub, 2003; Hennig, 2007) with an explicit caller-supplied
   `edge_drop_probability`.

Isolated topics stay unclustered. Negative-effect edges never enter the
partition graph. This slice does not fit a graphical lasso, does not
claim that a topic community is a causal construct, and does not add an
export workflow.

## Authoritative sources

Traag, V. A., Waltman, L., & van Eck, N. J. (2019). From Louvain to
Leiden: Guaranteeing well-connected communities. *Scientific Reports, 9*,
Article 5233. https://doi.org/10.1038/s41598-019-41695-z

Monti, S., Tamayo, P., Mesirov, J., & Golub, T. (2003). Consensus clustering: A resampling-based method for
class discovery and visualization of gene expression microarray data.
*Machine Learning, 52*(1–2), 91–118.
https://doi.org/10.1023/A:1023949509487

Hennig, C. (2007). Cluster-wise assessment of cluster stability.
*Computational Statistics & Data Analysis, 52*(1), 258–271.
https://doi.org/10.1016/j.csda.2006.11.025

Newman, M. E. J., & Girvan, M. (2004). Finding and evaluating community
structure in networks. *Physical Review E, 69*(2), 026113.
https://doi.org/10.1103/PhysRevE.69.026113

## Formula notes

- Undirected edge weight `w_ij` is the admitted positive effect. Self-loops
  and non-positive effects are dropped. Duplicate pairs are summed.
- Strength `k_i = Σ_j w_ij`. Total weight `m = (Σ_i k_i) / 2`.
- Modularity gain of moving node `i` from community `c` to `d` at γ = 1:
  `ΔQ = (k_{i→d} − k_{i→c}) / m − k_i (Σ_d − Σ_c + k_i) / (2 m²)`,
  where `Σ_c` still includes `k_i`.
- Louvain is rejected: it can emit internally disconnected communities
  (Traag et al., 2019). Union-find is rejected: it merges every surviving
  edge into one component and makes no modularity claim.

## Verification

- empty and edgeless graphs stay singletons;
- two nodes with a positive edge share a community;
- two triangles joined by a weak bridge stay two communities (union-find
  would glue them) both at the Leiden partition and at the consensus
  wrapper with zero drop probability;
- out-of-range, self, and non-positive edges are ignored;
- identical seeds reproduce the partition;
- isolated topics keep a distinct unclustered assignment at consensus;
- aggregation sums parallel supernode edges and keeps distinct neighbours;
- the connectedness oracle rejects a disconnected community and accepts
  singletons;
- aggregation continues when nodes moved even if the aggregate did not
  shrink;
- out-of-range and self-loop endpoints stay unclustered at consensus;
- permuting equal edges does not change the consensus assignment.
