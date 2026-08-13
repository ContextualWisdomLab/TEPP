# Event-instance persistence (doctoring)

## Scope

`event_instance` already exists on the foundation schema with valid/system
windows and lifecycle status. This slice adds the fail-closed insert and
as-known-at lookup contract so an instance cannot be persisted with inverted
windows or hostile type/lifecycle labels. Mentions remain a separate identity
class (Hovy et al., 2006; Doddington et al., 2004).

This does not add a new migration number. Catalog CHECKs for interval order
already live in `0005`.

## Authority

Doddington, G., Mitchell, A., Przybocki, M., Ramshaw, L., Strassel, S., &
Weischedel, R. (2004). The Automatic Content Extraction (ACE) program—Tasks,
data, and evaluation. In *Proceedings of LREC 2004* (pp. 837–840). European
Language Resources Association.

Jensen, C. S., & Snodgrass, R. T. (1999). Temporal data management. *IEEE
Transactions on Knowledge and Data Engineering, 11*(1), 36–44.
https://doi.org/10.1109/69.755613

Pustejovsky, J., Castano, J., Ingria, R., Saurí, R., Gaizauskas, R., Setzer,
A., Katz, G., & Radev, D. (2003). TimeML: Robust specification of event and
temporal expressions in text. In *New Directions in Question Answering* (pp.
28–34). AAAI Press.

Instances are versioned domain objects with event-time validity. Collapsing
an inverted interval into a point, or treating a mention as the instance,
would destroy temporal and measurement identity (Jensen & Snodgrass, 1999;
Pustejovsky et al., 2003).

## Verification

- contract tests reject inverted valid/system windows and hostile labels;
- equal point bounds and open-ended `NULL` ends render;
- recording-session coverage for insert and as-known-at SQL;
- live PostgreSQL CI inserts a valid instance, looks it up as-known-at, and
  refuses inverted windows when `TEPP_LIVE_POSTGRES=1`.
