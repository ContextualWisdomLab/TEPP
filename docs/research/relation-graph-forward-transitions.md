# Typed relation graph and forward-only transitions

## Claim boundary

TEPP stores document, segment, event, entity, revision, translation, citation, support, contradiction, retrospective, and state-transition relations as first-class typed edges. Forward transition kinds never move backward in event time and never form cycles. Provenance kinds may point to the past without becoming reverse state transitions. Observed and inferred evidence status remain distinct.

## Authority

- Product and ontology: PRD v0.4; ADR 0003; ERD relation vocabulary.
- Temporal order: ADR 0002; Allen classification in `temporal_core`.
- Event intelligence promotion boundary: ADR 0016.

## Scientific and standards basis

Qualitative temporal reasoning over interval relations follows Allen (1983). Event and time-marking annotation practice separates surface references from ordered event structure (Pustejovsky et al., 2003; TimeML). Graph-based dependence and multilevel measurement require explicit edges rather than atomistic document independence (Snijders, 2011; Raudenbush & Bryk, 2002).

## References

Allen, J. F. (1983). Maintaining knowledge about temporal intervals. *Communications of the ACM, 26*(11), 832–843. https://doi.org/10.1145/182.358434

Pustejovsky, J., Castaño, J., Ingria, R., Saurí, R., Gaizauskas, R., Setzer, A., & Katz, G. (2003). TimeML: Robust specification of event and temporal expressions in text. In *New Directions in Question Answering* (pp. 28–34). AAAI Press.

Raudenbush, S. W., & Bryk, A. S. (2002). *Hierarchical linear models: Applications and data analysis methods* (2nd ed.). Sage.

Snijders, T. A. B. (2011). Statistical models for social networks. *Annual Review of Sociology, 37*, 131–153. https://doi.org/10.1146/annurev.soc.012809.102709
