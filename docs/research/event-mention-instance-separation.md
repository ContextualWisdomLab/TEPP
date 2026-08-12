# Event mention versus event instance separation

## Claim boundary

TEPP treats textual event mentions as fallible evidence-grounded observations and event instances as versioned domain objects created only by explicit promotion or authoritative assertion. Mentions never silently become instances.

## Authority

- Product and ontology authority: PRD v0.4; ADR 0003; ADR 0016 (event intelligence task separation).
- Temporal support: ADR 0002 six-clock semantics for event-time validity on instances.
- Privacy: ADR 0009 purpose-bound identifiers (opaque IDs only in this crate).

## Scientific and standards basis

Separating mentions from instances reduces measurement error and atomistic fallacy by preventing text-span detections from being treated as independent ground-truth events in multilevel models (Raudenbush & Bryk, 2002). Event extraction literature similarly distinguishes surface detections from resolved event structures (Doddington et al., 2004). TEPP encodes that boundary in the type system.

## References

Doddington, G., Mitchell, A., Przybocki, M., Ramshaw, L., Strassel, S., & Weischedel, R. (2004). The Automatic Content Extraction (ACE) program—Tasks, data, and evaluation. In *Proceedings of LREC 2004* (pp. 837–840). European Language Resources Association.

Raudenbush, S. W., & Bryk, A. S. (2002). *Hierarchical linear models: Applications and data analysis methods* (2nd ed.). Sage.

Pustejovsky, J., Castaño, J., Ingria, R., Saurí, R., Gaizauskas, R., Setzer, A., & Katz, G. (2003). TimeML: Robust specification of event and temporal expressions in text. In *New Directions in Question Answering* (pp. 28–34). AAAI Press.
